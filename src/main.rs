use std::fs;
use std::vec::Vec;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};

// index item struct: Value(ip): 4 Bytes, Offset: 3 Bytes
// entry item struct: Value(ip): 4 Bytes, country, area

const INDEX_VAL_LEN: u8 = 4;
const INDEX_ITEM_LEN: u8 = 7;
const ENTRY_HEADER_LEN: u8 = 4;
const REDIRECT_MODE_1: u8 = 1;
const REDIRECT_MODE_2: u8 = 2;
const REDIRECT_OFFSET_LEN: u8 = 3;

fn main() {
    println!("Hello, world!");

    // let ip_database = fs::read("ipv4.dat").unwrap();
    // let mut start: [u8; 4] = [0; 4];
    // let mut end: [u8; 4] = [0; 4];
    //
    // for (index, val) in ip_database[0..8].into_iter().enumerate() {
    //     if index < 4 {
    //         start[index] = *val;
    //     } else {
    //         end[index % 4] = *val;
    //     }
    // }
    //
    // let start_index = u32::from_le_bytes(start) as usize;
    // let end_index = u32::from_le_bytes(end) as usize;
    //
    // // let first_entry = IndexEntry::new(&ip_database[start_index..start_index + 7]);
    // // let second_entry = IndexEntry::new(&ip_database[start_index + 7..start_index + 14]);
    // //
    // // println!("first_entry: {:?}", first_entry);
    // // println!("second_entry: {:?}", second_entry);
    //
    // let entry_len: usize = 7;
    // let mut count = 0;
    // for i in (start_index..start_index + entry_len * 20 + 1).into_iter().step_by(7) {
    //     if i + 7 < ip_database.len() {
    //         println!("{}-th entry: {:?}", count, IndexEntry::new(&ip_database[i..i+7]));
    //     }
    //     count += 1;
    // }

    test_seek_and_read("ipv4.dat");
}

#[derive(Debug)]
struct IndexEntry {
    ip: [u8; 4],
    offset: u32,
}

impl IndexEntry {
    fn new(data: &[u8]) -> IndexEntry {
        if data.len() != 7 {
            panic!("Incorrect length of data");
        }

        let mut ip_bytes: [u8; 4] = [0; 4];
        let mut offset_bytes: [u8; 4] = [0; 4];
        for (index, val) in data.into_iter().enumerate() {
            if index < 4 {
                ip_bytes[index] = *val;
            } else {
                offset_bytes[index % 4 + 1] = *val;
            }
        }
        let ip: [u8; 4] = u32::to_be_bytes(u32::from_le_bytes(ip_bytes));
        let offset: u32 = u32::from_le_bytes(offset_bytes);

        IndexEntry {
            ip,
            offset,
        }
    }
}

fn test_seek_and_read(path: &str) {
    let mut file = File::open(path).unwrap();
    let mut buffer = [0_u8; 8];
    file.read_exact(&mut buffer).unwrap();
    println!("First 8 Bytes: {:?}", buffer);
    file.read_exact(&mut buffer).unwrap();
    println!("Second 8 Bytes: {:?}", buffer);
    let pos = file.seek(SeekFrom::Start(0)).unwrap();
    println!("Seek to the position: {}", pos);
    file.read_exact(&mut buffer).unwrap();
    println!("=========\nFirst 8 Bytes: {:?}", buffer);
}

enum Endian {
    Le,
    Be,
}

struct IPInfo {
    ip: [u8; 4],
    country: String,
    area: String,
}

struct IPDatabase {
    file: File,
    index_start_offset: u32,
    index_end_offset: u32,
}

impl IPDatabase {
    fn new(path: &str) -> IPDatabase {
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

    fn read_exact_from(&mut self, mut buf: &mut [u8], offset: u64) {
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
            Endian::Le => u32::from_le_bytes(buf),
            Endian::Be => u32::from_be_bytes(buf),
        }
    }

    fn read_c_string_from(&mut self, offset: u64) -> (String, u32) {

    }

    // ip: u8 array in big-endian
    fn search_ip(&mut self, ip: [u8; 4]) -> IPInfo {
        let entry_data_offset = (self.get_entry_offset_for_ip(ip.clone()) + ENTRY_HEADER_LEN) as u64;
        let mut mode = [0_u8; 1];
        self.read_exact_from(&mut mode, entry_data_offset);
        let mut country: String = "".to_string();
        let mut area: String = "".to_string();
        let mut area_offset: u32 = 0;

        if mode[0] == REDIRECT_MODE_1 {
            let mut redirect_offset = [0_u8; 3];
            self.read_exact_from(&mut redirect_offset, entry_data_offset + 1);
            let redirect_offset= u32::from_le_bytes(redirect_offset) as u64;

            self.read_exact_from(&mut mode, redirect_offset);
            if mode[0] == REDIRECT_MODE_2 {
                let mut sec_redirect_offset = [0_u8; 3];
                self.read_exact_from(&mut sec_redirect_offset, redirect_offset + 1);
                let sec_redirect_offset = u32::from_le_bytes(sec_redirect_offset) as u64;
            }

            (country, area_offset) = self.read_c_string_from(redirect_offset);
            area = self.get_area(area_offset as u64);
        } else if mode[0] == REDIRECT_MODE_2 {

        } else {
            (country, area_offset) = self.read_c_string_from(entry_data_offset);
            area = self.get_area(area_offset as u64);
        }

        return IPInfo {
            ip: ip.clone(),
            country: country,
            area,
        };
    }

    fn get_area(&mut self, offset: u64) -> String {

    }

    /// ip: u8 array in big-endian
    fn get_entry_offset_for_ip(&mut self, ip: [u8; 4]) -> u32 {
        let mut left = self.index_start_offset as u64;
        let mut right = self.index_end_offset as u64;
        let dst_ip = u32::from_be_bytes(ip);
        let res: u32;

        loop {
            // compute mid_offset
            // left = a_1, right = a_n = a_1 + n * d
            // right - left = n * d
            let mut mid = left + (right - left) / 2 ;
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
