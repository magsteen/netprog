use core::panic;
use std::error::Error;
use std::f32::consts::E;
use std::io;
use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;
use std::net::ToSocketAddrs;
use std::os::unix::thread;
use std::ptr::NonNull;
use std::thread::JoinHandle;

const MATH_SERVER_ADDRESS: &str = "localhost:8080";
const HTTP_SERVER_ADDRESS: &str = "localahost:8081";

struct Expression {
    operand_left: i32,
    operand_right: i32,
    operator: char,
}

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
                send_tcp_request(input, MATH_SERVER_ADDRESS);
            }
            Err(error) => {
                println!("Error: {}", error)
            }
        };
    }
}

fn start_web_client() {}

fn start_math_server() {
    let listener = TcpListener::bind(MATH_SERVER_ADDRESS).unwrap();
    // Block forever, handling each request that arrives at this IP address
    for stream in listener.incoming() {
        let stream = stream.unwrap();

        let result: i32;
        match handle_math_connection(stream) {
            Ok(v) => result = v,
            Err(e) => println!("Calculation failed: {:?}", e),
        };
        //write response with result
    }
}

fn start_web_server() {}

fn handle_math_connection(mut stream: TcpStream) -> Result<i32, &'static str> {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();

    let input = match std::str::from_utf8(&buffer) {
        Ok(res) => res,
        Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
    };

    println!("Got input: {}, with length: {:?}", input, input.len());
    let expr = translate_to_expression(input.to_string());
    match expr {
        Ok(v) => {
            return calculate(v);
        }
        Err(e) => return Err("input couldn 't be translated to an expression: {e}"),
    }
}

fn handle_web_connection(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();

    let get = b"GET / HTTP/1.1\r\n";

    let (status_line, html) = if buffer.starts_with(get) {
        (
            "HTTP/1.1 200 OK\r\n\r\n",
            "<!DOCTYPE html>
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
        ",
        )
    } else {
        ("HTTP/1.1 404 NOT FOUND\r\n\r\n", "".to_owned())
    };
    println!("{}", std::str::from_utf8(&buffer).unwrap());
    let response = format!("{status_line}{html}");
    stream.write_all(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}

fn send_tcp_request<A>(input: String, addr: A)
where
    A: ToSocketAddrs,
{
    match TcpStream::connect(addr) {
        Ok(mut stream) => {
            println!("Connected to server: {}", stream.peer_addr().unwrap());
            match stream.write(input.as_bytes()) {
                Ok(n) => {
                    println!("Wrote {} bytes", n)
                }
                Err(e) => {
                    println!("Error: {}", e);
                }
            }
        }
        Err(error) => {
            println!("Error: {}", error);
        }
    }
}

fn translate_to_expression(expr: String) -> Result<Expression, &'static str> {
    let (operand_left, operator, operand_right): (i32, char, i32);
    let op_index: usize;
    match get_op_index(expr.clone()) {
        Ok(v) => op_index = v,
        Err(e) => return Err(e),
    }
    let res_left = expr[..op_index].parse::<i32>();
    match res_left {
        Ok(v) => operand_left = v,
        Err(_) => return Err("left side is not a number"),
    }
    operator = expr[op_index..op_index + 1].parse::<char>().unwrap();
    let res_right = expr[op_index + 1..].parse::<i32>();
    match res_right {
        Ok(v) => operand_right = v,
        Err(_) => return Err("right side is not a number"),
    }
    return Ok(Expression {
        operand_left,
        operand_right,
        operator,
    });
}

fn calculate(expr: Expression) -> Result<i32, &'static str> {
    match expr.operator {
        '+' => return Ok(expr.operand_left + expr.operand_right),
        '-' => return Ok(expr.operand_left - expr.operand_right),
        _ => return Err("unknown operator used: {_}"),
    }
}

fn get_op_index(expr: String) -> Result<usize, &'static str> {
    let operators = ['+', '-'];
    for op in operators {
        match expr.find(op) {
            Some(val) => return Ok(val),
            None => continue,
        }
    }
    return Err("Operator not found in expression!"); //panic!("Operator not found in expression!");
}
