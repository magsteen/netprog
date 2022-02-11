use core::panic;
use std::f32::consts::E;
use std::io;
use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;
use std::os::unix::thread;
use std::thread::JoinHandle;

const SERVER_ADDRESS: &str = "localhost:8080";

fn main() {
    // Listen for incoming TCP connections on localhost port 7878
    let mut handles = Vec::new();

    handles.push(std::thread::spawn(|| start_math_client()));
    handles.push(std::thread::spawn(|| start_web_client()));
    handles.push(std::thread::spawn(|| start_math_server()));
    handles.push(std::thread::spawn(|| start_web_server()));

    for handle in handles {
        handle.join().unwrap();
    }
}

fn start_math_client() {
    println!("Starting client...");
    loop {
        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(_) => {
                input.pop().unwrap();
                //println!("Got input: {}, with length: {:?}", &input, input.len())
                send_math_request(input);
            }
            Err(error) => {
                println!("Error: {:?}", error)
            }
        };
    }
}

fn start_web_client() {}

fn start_math_server() {}

fn start_web_server() {
    let listener = TcpListener::bind(SERVER_ADDRESS).unwrap();
    // Block forever, handling each request that arrives at this IP address
    for stream in listener.incoming() {
        let stream = stream.unwrap();

        handle_web_connection(stream);
    }
}

fn handle_web_connection(mut stream: TcpStream) {
    // Read the first 1024 bytes of data from the stream
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();

    let get = b"GET / HTTP/1.1\r\n";

    // Respond with greetings or a 404,
    // depending on the data in the request
    let (status_line, filename) = if buffer.starts_with(get) {
        ("HTTP/1.1 200 OK\r\n\r\n", "greeting.html")
    } else {
        ("HTTP/1.1 404 NOT FOUND\r\n\r\n", "404.html")
    };
    let test = "<!DOCTYPE html>
<html lang='no'>
  <head>
    <meta charset='utf-8'>
    <title>Calcumulator</title>
  </head>
  <body>
    <h1>Hei og velkommen til min web-tjener</h1>
    <div>Header fra klient gav følgende: "
        .to_owned()
        + std::str::from_utf8(&buffer).unwrap()
        + "
    </div>
  </body>
</html>
";
    println!("{}", std::str::from_utf8(&buffer).unwrap());
    let response = format!("{status_line}{test}");
    stream.write_all(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}

fn send_math_request(input: String) {
    match TcpStream::connect(SERVER_ADDRESS) {
        Ok(mut stream) => {
            println!("Connected to server: {:?}", stream.peer_addr().unwrap());
            println!("Got input: {}, with length: {:?}", &input, input.len());
        }
        Err(error) => {
            println!("Error: {:?}", error);
        }
    }
}
