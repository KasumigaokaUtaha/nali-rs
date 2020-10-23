extern crate clap;
extern crate ipdb_parser;
extern crate regex;

use std::net::IpAddr;
use std::vec::Vec;

use ipdb_parser::IPDatabase;

mod util;

/// nali-rs IP-Addr... (store IP database in default location ~/.nali-rs/)
/// nali-rs update -p/--path path in which the IP database is stored
fn main() {
    let matches = util::init();

    match matches.values_of("IP-Addr") {
        Some(values) => {
            let values: Vec<&str> = values.collect();
            let mut database = IPDatabase::new();
            for value in values {
                let value = value.trim();
                match util::parse_into_ipv4(value) {
                    Some(res) => match res {
                        Ok(ip) => database.search_ip_info(IpAddr::V4(ip)).display(),
                        Err(err) => println!("Error: {}", err),
                    },
                    None => {}
                }
            }
        }
        None => match matches.subcommand() {
            Some(subcommand) => match subcommand {
                ("update", update_matches) => {
                    match update_matches.value_of("Path") {
                        Some(dir_path) => IPDatabase::update(Some(dir_path)),
                        None => IPDatabase::update(None),
                    }
                }
                ("dig", _dig_matches) => {}
                ("nslookup", _nslookup_matches) => {}
                (name, _) => { println!("Undefined subcommand `{}`", name) }
            },
            None => {
                print!("{}", util::parse_and_search_ip_in(std::io::stdin()));
            }
        },
    }
}