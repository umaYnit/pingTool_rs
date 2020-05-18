use std::io::Read;
use std::net::UdpSocket;
use std::time::Instant;

use futures::{executor, future};
use pnet::datalink;
use pnet::ipnetwork::IpNetwork;
use winping::{AsyncPinger, Buffer};
use std::process::Command;
use encoding_rs::GBK;
use regex::{Regex, Captures};
use std::collections::HashMap;


fn main() {
    let pinger = AsyncPinger::new();


    let network = match get() {
        Some(net) => net,
        _ => { return; }
    };

    let mut list_of_delay = vec![];
    for ip in network.iter() {
        list_of_delay.push(Box::pin(pinger.send(ip, Buffer::new())));
    }


    let now = Instant::now();
    executor::block_on(future::join_all(list_of_delay));

    println!("time elapsed: {:?}", now.elapsed());


    let output = Command::new("arp")
        .arg("-a")
        .output()
        .expect("Failed to execute command");



    let buf = GBK.decode(output.stdout.as_slice()).0;

    let mac_patten = Regex::new(r"(.+\s+)(?:30-09-)(([a-f0-9]{2}:)|([a-f0-9]{2}-)){3}[a-f0-9]{2}").unwrap();
    let search_text = Regex::new(r"[.:]").unwrap();

    let mut result = HashMap::new();

    for caps in mac_patten.captures_iter(&buf) {
        let mut x = caps[0].trim().split_whitespace();
        let ele = x.next().unwrap().to_owned() + "\t" + &x.next().unwrap().to_owned();
        let show = String::from(&ele);
        let show = search_text.replace_all(&show, "");
        result.insert(ele,show.to_owned());
    }

    println!("{:?}", result);





}


pub fn get() -> Option<IpNetwork> {
    let socket = match UdpSocket::bind("0.0.0.0:0") {
        Ok(s) => s,
        Err(_) => return None,
    };

    match socket.connect("8.8.8.8:80") {
        Ok(()) => (),
        Err(_) => return None,
    };

    let ip = match socket.local_addr() {
        Ok(addr) => addr.ip(),
        Err(_) => return None,
    };

    for iface in datalink::interfaces() {
        for &network in iface.ips.iter() {
            if network.ip() == ip {
                return Some(network);
            }
        }
    }
    None
}