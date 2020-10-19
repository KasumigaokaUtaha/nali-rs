extern crate clap;
extern crate ipdb_parser;

use ipdb_parser::IPDatabase;

fn main() {
    let mut database = IPDatabase::new("ipv4.dat");
    let ip_info = database.search_ip_info([114, 114, 114, 88]);
    println!("{:?}", ip_info);
}