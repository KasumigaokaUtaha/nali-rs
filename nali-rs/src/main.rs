mod util;

extern crate clap;
extern crate regex;
extern crate ipdb_parser;

use std::vec::Vec;
use clap::{App, Arg, ArgMatches};
use ipdb_parser::IPDatabase;

fn main() {
    let matches = init();

    match matches.values_of("IP-Addr") {
        Some(values) => {
            let values: Vec<&str> = values.collect();
            let mut database = IPDatabase::new("ipv4.dat"); // TODO make database file path optional
            for value in values {
                match util::parse_into_ipv4(value.trim()) {
                    Some(ip) => database.search_ip_info(ip).display(),
                    None => println!("`{}` is not a valid ip address!", value), // TODO consider replace panic with better one
                }
            }
        },
        None => match matches.subcommand() {
            Some(("update", _update_matches)) => {},
            Some(("dig", _dig_matches)) => {},
            Some(("nslookup", _nslookup_matches)) => {},
            _ => (),
        },
    }
}

fn init() -> ArgMatches {
    App::new("nali-rs")
        .version("0.1")
        .author("Silver Crow <kasumigaokautahasaki@gmail.com>")
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