extern crate encoding;

use std::vec::Vec;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};

use encoding::all::GBK;
use encoding::{Encoding, DecoderTrap};

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

enum Endian {
    Le,
    Be,
}

#[derive(Debug)]
pub struct IPInfo {
    ip: [u8; 4],
    country: String,
    area: String,
}

impl IPInfo {
    pub fn display(&self) {
        println!("{}.{}.{}.{} \t \x1b[0;0;36m[{} {}]\x1b[0m",
                 self.ip[0], self.ip[1], self.ip[2], self.ip[3], &self.country,
                 if self.area == "" { "unknown area" } else { &self.area });
    }
}

pub struct IPDatabase {
    file: File,
    index_start_offset: u32,
    index_end_offset: u32,
}

impl IPDatabase {
    pub fn new(path: &str) -> IPDatabase {
        let mut file = File::open(path).unwrap();
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

    /// read four bytes start from the given offset
    /// and convert it to u32 which represents the integer value of an IP address
    fn read_ip_to_u32(&mut self, offset: u64, endian: Endian) -> u32 {
        let mut buf = [0_u8; 4];
        self.read_exact_from(&mut buf, offset);

        match endian {
            Endian::Le => u32::from_le_bytes(buf),
            Endian::Be => u32::from_be_bytes(buf),
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
            },
            Endian::Be => {
                let _buf = [0_u8, buf[0], buf[1], buf[2]];
                u32::from_be_bytes(_buf)
            },
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
    pub fn search_ip_info(&mut self, ip: [u8; 4]) -> IPInfo {
        let entry_data_offset = self.search_entry_offset(
            ip.clone()) as u64 + ENTRY_HEADER_LEN as u64;
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
            ip: ip.clone(),
            country,
            area,
        };
    }

    /// offset: the start position storing area information
    fn get_area(&mut self, mut offset: u64) -> String {
        let mode = self.read_mode(offset);
        let mut area: String;

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

    /// ip: u8 array in big-endian
    fn search_entry_offset(&mut self, ip: [u8; 4]) -> u32 {
        let mut left = self.index_start_offset as u64;
        let mut right = self.index_end_offset as u64;
        let dst_ip = u32::from_be_bytes(ip);
        let res: u32;

        loop {
            // compute mid_offset
            // left = a_1, right = a_n = a_1 + n * d
            // right - left = n * d
            let mut mid = ((right - left) / INDEX_ITEM_LEN as u64) / 2;
            mid = left + mid * INDEX_ITEM_LEN as u64;
            // get u32 ip integer from mid_offset
            let mid_ip= self.read_ip_to_u32(mid, Endian::Le);
            // end case
            if right - left == INDEX_ITEM_LEN as u64 {
                if dst_ip < self.read_ip_to_u32(right, Endian::Le) {
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
                res =  self.read_offset_to_u32(mid + INDEX_VAL_LEN as u64, Endian::Le);
                break;
            }
        }

        res
    }
}