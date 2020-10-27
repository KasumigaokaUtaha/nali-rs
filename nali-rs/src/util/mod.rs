use std::collections::HashMap;
use std::io::BufRead;
use std::net::{AddrParseError, IpAddr, Ipv4Addr};
use std::str::FromStr;

use clap::{crate_version, App, Arg, ArgMatches};
use regex::Regex;

use ipdb_parser::IPDatabase;

pub fn parse_into_ipv4(val: &str) -> Option<Result<Ipv4Addr, AddrParseError>> {
    let ipv4_pattern =
        r"^((?:25[0-5]|2[0-4]\d|1\d\d|[1-9]\d|\d)(?:\.(?:25[0-5]|2[0-4]\d|1\d\d|[1-9]\d|\d)){3})$";
    let re = Regex::new(ipv4_pattern).unwrap();

    if re.is_match(val) {
        Some(Ipv4Addr::from_str(val))
    } else {
        None
    }
}

pub fn parse_into_ipv4s(val: &str) -> Option<Vec<Result<Ipv4Addr, AddrParseError>>> {
    let ipv4_pattern =
        r"((?:25[0-5]|2[0-4]\d|1\d\d|[1-9]\d|\d)(?:\.(?:25[0-5]|2[0-4]\d|1\d\d|[1-9]\d|\d)){3})";
    let re = Regex::new(ipv4_pattern).unwrap();

    if re.is_match(val) {
        let mut res: Vec<Result<Ipv4Addr, AddrParseError>> = Vec::new();
        for cap in re.captures_iter(val) {
            if let Some(matched_value) = cap.get(0) {
                res.push(Ipv4Addr::from_str(matched_value.as_str()));
            }
        }

        Some(res)
    } else {
        None
    }
}

pub fn parse_and_search_ip<R: BufRead>(reader: &mut R) -> Option<String> {
    let mut map: HashMap<Ipv4Addr, String> = HashMap::new();
    let mut database = IPDatabase::new();
    let mut buf = String::new();
    let read_amount = reader.read_line(&mut buf).unwrap();

    if read_amount == 0 {
        return None;
    }

    match parse_into_ipv4s(&buf) {
        Some(values) => {
            for value in values {
                match value {
                    Ok(ip) => {
                        map.insert(ip, database.search_ip_info(IpAddr::V4(ip)).to_string());
                    }
                    Err(err) => println!("Error: {}", err),
                }
            }
        }
        None => {}
    }

    for (ip, ip_info) in map {
        buf = buf.replace(&ip.to_string(), &ip_info);
    }

    Some(buf)
}

pub fn init() -> ArgMatches {
    App::new("nali-rs")
        .about("A simple utility for querying geo info about ip address(es)")
        .arg(
            Arg::new("IP-Addr")
                .about("IP address(es) to be queried")
                // .required(true)
                .multiple(true)
                .index(1),
        )
        .subcommand(
            App::new("update").about("Update ip database(s)").arg(
                Arg::new("Path")
                    .about("Directory path in which IP database will be stored")
                    .short('p')
                    .long("path")
                    .takes_value(true),
            ),
        )
        .version(crate_version!())
        .get_matches()
}
