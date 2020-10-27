# nali-rs

A simple CLI utility for querying IP addresses.

## Build
Run the following command to install the nali-rs utility:

```bash
$ cd nali-rs
$ cargo install --path ./nali-rs
```

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

Apart from that, `nali-rs` also supports querying IP addresses from the output of other programs by simply connecting them using the pipeline operator `|`.

For example:

```bash
$ dig google.com | nali-rs

; <<>> DiG 9.10.6 <<>> google.com
;; global options: +cmd
;; Got answer:
;; ->>HEADER<<- opcode: QUERY, status: NOERROR, id: 50449
;; flags: qr rd ra; QUERY: 1, ANSWER: 1, AUTHORITY: 0, ADDITIONAL: 1

;; OPT PSEUDOSECTION:
; EDNS: version: 0, flags:; udp: 1232
;; QUESTION SECTION:
;google.com.			IN	A

;; ANSWER SECTION:
google.com.		262	IN	A	216.58.207.46 	 [美国 Google全球边缘网络]

;; Query time: 18 msec
;; SERVER: 1.1.1.1 	 [美国 APNIC&CloudFlare公共DNS服务器]#53(1.1.1.1 	 [美国 APNIC&CloudFlare公共DNS服务器])
;; WHEN: Tue Oct 27 19:34:50 CET 2020
;; MSG SIZE  rcvd: 55
```

### Update IP database
Run `nali-rs update` to update the IP database. This sub command `update` will first remove previous downloaded database and then download and store new one in the default location (`~/.nali-rs/`), if no custom path specified.
One can also specify where to store the database by given the option `-p` or `--path` following the path to the directory.


## ToDo

- [ ] Support of IPv6

## Reference
This project is inspired by the following projects:

> [Sukka - nali-cli](https://github.com/SukkaW/nali-cli)

> [Mikubill - nali-go](https://github.com/Mikubill/nali-go)

Special thanks to the IP database provider [CZ88 IP database](http://www.cz88.net/fox/ipdat.shtml) and the mirror site [qqwry-mirror](https://qqwry.mirror.noc.one/) .
