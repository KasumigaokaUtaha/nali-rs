# nali-rs

A simple CLI utility for querying IP addresses.

## Build
Run `cargo build` in the project root to build the executable file.

## Usage
### Query IP address
`nali-rs <IP-Addr>...`: `nali-rs` accepts multiple IP addresses and it will then displays the query result for them.

For example:

```bash
$ nali-rs 114.114.114.114                        
114.114.114.114          [江苏省南京市 南京信风网络科技有限公司GreatbitDNS服务器]

$ nali-rs 1.1.1.1 2.2.2.2 3.3.3.3 4.4.4.4 5.5.5.5
1.1.1.1          [美国 APNIC&CloudFlare公共DNS服务器]
2.2.2.2          [法国 unknown area]
3.3.3.3          [美国 Amazon EC2服务器]
4.4.4.4          [美国 新泽西州纽瓦克市Level3Communications]
5.5.5.5          [德国 unknown area]
```

### Update IP database
Run `nali-rs update` to update the IP database. This sub command `update` will first remove previous downloaded database and then download and store new one in the default location (`~/.nali-rs/`), if no custom path specified.
One can also specify where to store the database by given the option `-p` or `--path` following the path to the directory.


## Reference
This project is inspired by the following projects:

> [Sukka - nali-cli](https://github.com/SukkaW/nali-cli)

> [Mikubill - nali-go](https://github.com/Mikubill/nali-go)

Special thanks to the IP database provider [CZ88 IP database](http://www.cz88.net/fox/ipdat.shtml) and the mirror site [qqwry-mirror](https://qqwry.mirror.noc.one/) .
