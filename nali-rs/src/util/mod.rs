use regex::Regex;

pub fn parse_into_ipv4(val: &str) -> Option<[u8; 4]> {
    let mut re_str = String::from(r"^(25[0-5]|2[0-4]\d|1\d\d|[1-9]\d|\d)");
    for _ in 0..3 {
        re_str.push_str(r"(?:\.(25[0-5]|2[0-4]\d|1\d\d|[1-9]\d|\d))");
    }
    re_str.push('$');
    let re = Regex::new(&re_str).unwrap();
    let mut ip: Option<[u8; 4]> = None;

    if re.is_match(val) {
        let caps = re.captures(val).unwrap();
        if caps.len() - 1 == 4 {
            let mut res = [0_u8; 4];

            for index in 1..caps.len() {
                res[index - 1] = caps.get(index).unwrap().as_str().parse::<u8>().unwrap();
            }
            ip = Some(res);
        }
    }

    ip
}