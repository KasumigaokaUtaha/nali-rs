extern crate clap;
extern crate ipdb_parser;
extern crate regex;

use std::io::{self, BufReader};
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
                ("update", update_matches) => match update_matches.value_of("Path") {
                    Some(dir_path) => IPDatabase::update(Some(dir_path)),
                    None => IPDatabase::update(None),
                },
                (name, _) => println!("Undefined subcommand `{}`", name),
            },
            None => {
                let mut buf_stdin = BufReader::new(io::stdin());
                loop {
                    let output = util::parse_and_search_ip(&mut buf_stdin);
                    match output {
                        Some(value) => print!("{}", value),
                        None => break,
                    }
                }
            }
        },
    }
}
