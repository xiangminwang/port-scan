#[macro_use]
extern crate clap;

use clap::{App, Arg};
use std::thread;
use std::sync::mpsc::{Sender, channel};
use std::time::Duration;
use std::net::{TcpStream, IpAddr, Ipv4Addr, SocketAddr};

fn scan(sender: Sender<u16>, range: Vec<u16>, ip_address: Ipv4Addr) {
    for port_number in range {
        let socket = SocketAddr::new(IpAddr::from(ip_address), port_number);

        if TcpStream::connect_timeout(&socket, Duration::new(5, 0)).is_ok() {
            println!("{:?} is open..", port_number);
            sender.send(port_number).unwrap();
        }
    }
}

fn main() {
    let matches = App::new("PortScanner")
        .version("1.0")
        .author("Xiangmin Wang <wang@xiangmin.net>")
        .about("Do a quickly multi-threaded port scan in Rust.")
        .arg(
            Arg::with_name("ip")
                .short("i")
                .long("ip")
                .takes_value(true)
                .help("The target IP Address to scan")
        )
        .arg(
            Arg::with_name("threads")
                .short("t")
                .long("threads")
                .takes_value(true)
                .help("Threads you want to perform")
        )
        .get_matches();

    // User inputs
    let ip = matches.value_of("ip").unwrap_or("127.0.0.1");
    let thread_count = value_t!(matches, "threads", usize).unwrap_or(10);
    let ip_address = ip.parse::<Ipv4Addr>().expect("Cannot parse your input into Ipv4Addr!");

    // Logic uses
    let (sender, receiver) = channel::<u16>();
    let mut open_ports: Vec<u16> = vec![];
    let socket_ports: Vec<u16> = (1..=65535).collect();

    if thread_count > 65535 {
        panic!("Threads count must not larger than 65535");
    } else if thread_count < 1 {
        panic!("Threads count must larger than 0");
    }

    let chunk_count = 65535 / thread_count;

    let mut dispatched_threads = 0;

    for chunk in socket_ports.chunks(chunk_count) {
        let chunk = chunk.to_owned();
        let sender = sender.clone();

        dispatched_threads += 1;

        thread::spawn(move || {
            scan(sender, chunk, ip_address);
        });
    }

    println!("Start scanning {} with {} threads(avg chunk size {}) by 5s timeout...", ip, dispatched_threads, chunk_count);

    drop(sender);

    for port in receiver {
        open_ports.push(port);
    }

    println!("Completed, opened TCP ports are: {:?}", &open_ports);
}