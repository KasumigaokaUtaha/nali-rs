extern crate attohttpc;
extern crate encoding;
extern crate flate2;
extern crate shellexpand;

use std::fs;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};
use std::net::{IpAddr, Ipv4Addr};
use std::os::unix;
use std::path::Path;
use std::vec::Vec;

use encoding::{DecoderTrap, Encoding};
use encoding::all::GBK;
use flate2::read::ZlibDecoder;

/// index item struct: Value(ip): 4 Bytes, Offset: 3 Bytes
///
/// entry item struct: Value(ip): 4 Bytes, country, area

const INDEX_VAL_LEN: u8 = 4;
const INDEX_ITEM_LEN: u8 = 7;
const ENTRY_HEADER_LEN: u8 = 4;
const ENTRY_MODE_LEN: u8 = 1;
const ENTRY_OFFSET_LEN: u8 = 3;
const REDIRECT_MODE_1: u8 = 1;
const REDIRECT_MODE_2: u8 = 2;
// const REDIRECT_OFFSET_LEN: u8 = 3;
const QQWRY_IP_DATABASE_URL: &str = "https://qqwry.mirror.noc.one/qqwry.rar";
const QQWRY_IP_DATABASE_KEY_URL: &str = "https://qqwry.mirror.noc.one/copywrite.rar";
const QQWRY_IP_DATABASE_NAME: &str = "qqwry.db";
const DEFAULT_DIR: &str = "~/.nali-rs";

enum Endian {
    Le,
    Be,
}

pub struct IPInfo {
    ip: IpAddr,
    country: String,
    area: String,
}

impl IPInfo {
    pub fn display(&self) {
        println!("{}", self.to_string());
    }

    pub fn to_string(&self) -> String {
        format!("{} \t \x1b[0;0;36m[{} {}]\x1b[0m", self.ip.to_string(), &self.country,
                if self.area == "" { "unknown area" } else { &self.area })
    }
}

pub struct IPDatabase {
    file: File,
    index_start_offset: u32,
    index_end_offset: u32,
}

impl IPDatabase {
    /// dir_path: path to the directory in which IP database is stored
    /// this directory will be created if not exist
    pub fn new() -> IPDatabase {
        let default_dir: &str = &shellexpand::tilde(DEFAULT_DIR).to_owned();
        if !IPDatabase::exists_database() { IPDatabase::update(Some(default_dir)); }
        let mut file = File::open(
            Path::new(default_dir).join(QQWRY_IP_DATABASE_NAME)
        ).unwrap();

        let mut index_start_offset = [0_u8; 4];
        let mut index_end_offset = [0_u8; 4];

        file.read_exact(&mut index_start_offset).unwrap();
        file.read_exact(&mut index_end_offset).unwrap();

        let index_start_offset = u32::from_le_bytes(index_start_offset);
        let index_end_offset = u32::from_le_bytes(index_end_offset);

        IPDatabase {
            file,
            index_start_offset,
            index_end_offset,
        }
    }

    fn read_exact_from(&mut self, buf: &mut [u8], offset: u64) {
        self.file.seek(SeekFrom::Start(offset)).unwrap();
        self.file.read_exact(buf).unwrap();
    }

    fn read_ipv4_from(&mut self, offset: u64, endian: Endian) -> Ipv4Addr {
        let mut buf = [0_u8; 4];
        self.read_exact_from(&mut buf, offset);

        match endian {
            Endian::Le => {
                Ipv4Addr::new(buf[3], buf[2], buf[1], buf[0])
            }
            Endian::Be => {
                Ipv4Addr::new(buf[0], buf[1], buf[2], buf[3])
            }
        }
    }

    /// read three bytes start from the given offset
    /// and convert it to u32 which is used as offset in the given IP database
    fn read_offset_to_u32(&mut self, offset: u64, endian: Endian) -> u32 {
        let mut buf = [0_u8; 3];
        self.read_exact_from(&mut buf, offset);

        match endian {
            Endian::Le => {
                let _buf = [buf[0], buf[1], buf[2], 0_u8];
                u32::from_le_bytes(_buf)
            }
            Endian::Be => {
                let _buf = [0_u8, buf[0], buf[1], buf[2]];
                u32::from_be_bytes(_buf)
            }
        }
    }

    fn read_mode(&mut self, offset: u64) -> u8 {
        let mut buf = [0_u8; 1];
        self.read_exact_from(&mut buf, offset);

        buf[0]
    }

    fn read_c_string_from(&mut self, offset: u64) -> (String, u64) {
        let mut buf: Vec<u8> = Vec::new();
        let mut val = [0_u8; 1];
        let mut _offset = offset;
        self.file.seek(SeekFrom::Start(offset)).unwrap();

        loop {
            self.file.read_exact(&mut val).unwrap();
            if val[0] != 0 {
                buf.push(val[0]);
                _offset += 1;
            } else {
                break;
            }
        }

        let string = GBK.decode(&buf, DecoderTrap::Strict).unwrap();

        (string, _offset)
    }

    // ip: u8 array in big-endian
    pub fn search_ip_info(&mut self, ip: IpAddr) -> IPInfo {
        match ip {
            IpAddr::V4(v4_addr) => { self.search_ipv4_info(v4_addr) }
            IpAddr::V6(_) => panic!("Ipv6 is not implemented yet"),
        }
    }

    fn search_ipv4_info(&mut self, ip: Ipv4Addr) -> IPInfo {
        let entry_data_offset = self.search_entry_offset(ip.clone()) as u64
            + ENTRY_HEADER_LEN as u64;
        let mut mode = self.read_mode(entry_data_offset);
        let country: String;
        let mut area: String;
        let mut redirect_offset: u64;
        let area_offset: u64;

        if mode == REDIRECT_MODE_1 {
            redirect_offset = self.read_offset_to_u32(
                entry_data_offset + ENTRY_MODE_LEN as u64, Endian::Le) as u64;
            mode = self.read_mode(redirect_offset);

            if mode == REDIRECT_MODE_2 {
                area_offset = redirect_offset + (ENTRY_MODE_LEN + ENTRY_OFFSET_LEN) as u64;
                redirect_offset = self.read_offset_to_u32(
                    redirect_offset + ENTRY_MODE_LEN as u64, Endian::Le) as u64;
                let (_country, _) = self.read_c_string_from(redirect_offset);
                country = _country;
                area = self.get_area(area_offset);
            } else {
                let res = self.read_c_string_from(redirect_offset);
                country = res.0;
                area_offset = res.1;
                area = self.get_area(area_offset);
            }
        } else if mode == REDIRECT_MODE_2 {
            redirect_offset = self.read_offset_to_u32(
                entry_data_offset + ENTRY_MODE_LEN as u64, Endian::Le) as u64;
            let (_country, _) = self.read_c_string_from(redirect_offset);
            country = _country;
            area_offset = entry_data_offset + (ENTRY_MODE_LEN + ENTRY_OFFSET_LEN) as u64;
            area = self.get_area(area_offset);
        } else {
            let res = self.read_c_string_from(entry_data_offset);
            country = res.0;
            area_offset = res.1;
            area = self.get_area(area_offset);
        }

        // remove unrelated area info
        if area.contains("CZ88.NET") {
            area = "".to_string();
        }

        return IPInfo {
            ip: IpAddr::V4(ip),
            country,
            area,
        };
    }

    // fn search_ipv6_info(&mut self, ip: Ipv6Addr) -> IPInfo {
    //
    // }

    /// offset: the start position storing area information
    fn get_area(&mut self, mut offset: u64) -> String {
        let mode = self.read_mode(offset);
        let area: String;

        if mode == REDIRECT_MODE_1 || mode == REDIRECT_MODE_2 {
            offset = self.read_offset_to_u32(
                offset + ENTRY_MODE_LEN as u64, Endian::Le) as u64;
        }

        if offset != 0 {
            area = self.read_c_string_from(offset).0;
        } else {
            area = "unknown area".to_string();
        }

        area
    }

    fn search_entry_offset(&mut self, dst_ip: Ipv4Addr) -> u32 {
        let mut left = self.index_start_offset as u64;
        let mut right = self.index_end_offset as u64;
        let res: u32;

        loop {
            // compute mid_offset
            // left = a_1, right = a_n = a_1 + n * d
            // right - left = n * d
            let mut mid = ((right - left) / INDEX_ITEM_LEN as u64) / 2;
            mid = left + mid * INDEX_ITEM_LEN as u64;
            // get u32 ip integer from mid_offset
            let mid_ip = self.read_ipv4_from(mid, Endian::Le);
            // end case
            if right - left == INDEX_ITEM_LEN as u64 {
                if dst_ip < self.read_ipv4_from(right, Endian::Le) {
                    res = self.read_offset_to_u32(left + INDEX_VAL_LEN as u64, Endian::Le);
                } else {
                    res = self.read_offset_to_u32(right + INDEX_VAL_LEN as u64, Endian::Le)
                }
                break;
            }
            // compare it with the target ip integer
            if dst_ip < mid_ip {
                right = mid;
            } else if dst_ip > mid_ip {
                left = mid
            } else {
                res = self.read_offset_to_u32(mid + INDEX_VAL_LEN as u64, Endian::Le);
                break;
            }
        }

        res
    }

    fn exists_database() -> bool {
        let default_dir: &str = &shellexpand::tilde(DEFAULT_DIR).to_owned();
        let default_dir: &Path = Path::new(default_dir);
        if default_dir.exists() {
            default_dir.join(QQWRY_IP_DATABASE_NAME).exists()
        } else {
            false
        }
    }

    /// Update the existing IP database
    /// or create it if not exist
    pub fn update(dir: Option<&str>) {
        let dir_path = fs::canonicalize(
            dir.unwrap_or(&shellexpand::tilde(DEFAULT_DIR).to_owned())
        ).unwrap();
        IPDatabase::update_or_create(dir_path.as_path());
    }

    /// dir: the canonical path of a given dir
    fn update_or_create(dir: &Path) {
        IPDatabase::remove_database();
        IPDatabase::create_database(dir);
    }

    fn remove_database() {
        let default_dir: &str = &shellexpand::tilde(DEFAULT_DIR).to_owned();
        let default_dir: &Path = Path::new(default_dir);
        if default_dir.exists() {
            let default_db = default_dir.join(QQWRY_IP_DATABASE_NAME);
            let default_db_meta = default_db.symlink_metadata().unwrap();
            let is_file = default_db_meta.file_type().is_file();
            let is_symlink = default_db_meta.file_type().is_symlink();
            if is_file || is_symlink {
                if is_symlink {
                    fs::remove_file(default_db.canonicalize().unwrap()).unwrap();
                    println!("Successfully removed previous IP database");
                    fs::remove_file(default_db).unwrap();
                    println!("Successfully removed symbolic link to previous IP database");
                } else {
                    fs::remove_file(default_db).unwrap();
                    println!("Successfully removed previous IP database");
                }
            }
        }
    }

    fn create_database(dir: &Path) {
        let default_dir: &str = &shellexpand::tilde(DEFAULT_DIR).to_owned();
        let default_dir: &Path = Path::new(default_dir);
        if !default_dir.exists() {
            fs::create_dir(default_dir).unwrap();
        }

        let resp = attohttpc::get(QQWRY_IP_DATABASE_URL).send().unwrap();
        let raw_db = resp.bytes().unwrap();
        let key = IPDatabase::get_key(QQWRY_IP_DATABASE_KEY_URL);
        let db = IPDatabase::decrypt_database(raw_db, key);

        let mut f = File::create(dir.join(QQWRY_IP_DATABASE_NAME)).unwrap();
        f.write_all(&db).unwrap();

        println!("Successfully updated/created IP database");

        if default_dir != dir {
            unix::fs::symlink(
                dir.join(QQWRY_IP_DATABASE_NAME),
                default_dir.join(QQWRY_IP_DATABASE_NAME),
            ).unwrap();
        }
    }

    fn decrypt_database(mut raw: Vec<u8>, mut key: u32) -> Vec<u8> {
        for i in 0..0x200 {
            key = key * 0x805;
            key += 1;
            key = key & 0xff;

            raw[i] = raw[i] ^ key as u8;
        }

        let mut decoder = ZlibDecoder::new(&raw[..]);
        let mut buf: Vec<u8> = Vec::new();
        decoder.read_to_end(&mut buf).unwrap();

        buf
    }

    fn get_key(key_url: &str) -> u32 {
        let resp = attohttpc::get(key_url).send().unwrap();
        let body = resp.bytes().unwrap();

        u32::from_le_bytes([body[20], body[21], body[22], body[23]])
    }
}