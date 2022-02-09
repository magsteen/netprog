use std::{net, os::unix::thread};

const LISTENING_ADDR: &str = "localhost:8080";

fn socket_serve() {
    //initalise new thread to handle datagram (calculation) and send datagram response
    let mut buf = [0; 2024];
}

fn main() {
    println!("Starting server...");

    let socket = net::UdpSocket::bind(LISTENING_ADDR).expect("Socket binding expect");
    let mut serve = true;
    let mut buf = []
    while serve {
        let (num_b, addr) = net::UdpSocket::peek_from()
        if socket.peek(buf)
    }
}
