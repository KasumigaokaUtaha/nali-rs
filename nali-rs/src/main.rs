mod util;

extern crate clap;
extern crate regex;
extern crate ipdb_parser;

use std::vec::Vec;
use clap::{App, Arg, ArgMatches};
use ipdb_parser::IPDatabase;

/// nali-rs IP-Addr... (store IP database in default location ~/.nali-rs/)
/// nali-rs update -p/--path path in which the IP database is stored
fn main() {
    let matches = init();

    match matches.values_of("IP-Addr") {
        Some(values) => {
            let values: Vec<&str> = values.collect();
            let mut database = IPDatabase::new();
            for value in values {
                match util::parse_into_ipv4(value.trim()) {
                    Some(ip) => database.search_ip_info(ip).display(),
                    None => println!("`{}` is not a valid ip address!", value),
                }
            }
        },
        None => match matches.subcommand() {
            Some(("update", update_matches)) => {
                match update_matches.value_of("Path") {
                    Some(dir_path) => IPDatabase::update(dir_path),
                    None => ()
                }
            },
            Some(("dig", _dig_matches)) => {},
            Some(("nslookup", _nslookup_matches)) => {},
            _ => (),
        },
    }
}

fn init() -> ArgMatches {
    App::new("nali-rs")      
        .about("A simple utility for querying geo info about ip address(es)")
        .arg(
            Arg::new("IP-Addr")
                .about("IP address(es) to be queried")
                // .required(true)
                .multiple(true)
                .index(1)
        )
        .subcommand(
            App::new("update")
                .about("Update ip database(s)")
                .arg(
                    Arg::new("Path")
                        .about("Directory path in which IP database will be stored")
                        .short('p')
                        .long("path")
                        .takes_value(true)
                )
        )
        .subcommand(
            App::new("dig")
                .about("to be implemented")
        )
        .subcommand(
            App::new("nslookup")
                .about("to be implemented")
        )
        .get_matches()
}
