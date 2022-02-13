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
    operand_left: f32,
    operand_right: f32,
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
    println!("Starting MATH client...");
    loop {
        println!("Please provide an expression to calculate (e.g: <number><operator><number>): ");
        let mut input = String::new();
        let output: f32;
        match io::stdin().read_line(&mut input) {
            Ok(_) => {
                input.pop().unwrap();
                let mut stream: TcpStream;
                match TcpStream::connect(MATH_SERVER_ADDRESS) {
                    Ok(val) => {
                        println!("Connected to server: {}", val.peer_addr().unwrap());
                        stream = val;
                    }
                    Err(e) => {
                        println!("Error: {}", e);
                        return;
                    }
                };
                stream = send_tcp_request(stream, input);
                output = recieve_response(stream);
                println!("Response/Answer: {}", output);
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
    for stream in listener.incoming() {
        handle_math_connection(stream.unwrap());
    }
}

fn start_web_server() {}

fn handle_math_connection(mut stream: TcpStream) {
    let mut buf = [0; 1024];
    let n = stream.read(&mut buf).unwrap();
    let byte_data = &buf[..n];
    let input = match std::str::from_utf8(&byte_data) {
        Ok(res) => res,
        Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
    };

    println!("Got input: {}, with length: {:?}", input, input.len());
    let expr = translate_to_expression(input.to_string());
    match expr {
        Ok(v) => {
            match calculate(v) {
                Ok(answer) => {
                    //write response with result
                    stream.write(&answer.to_be_bytes()).unwrap();
                    println!("Answer: {:?}", answer);
                }
                Err(e) => println!("Calculation failed: {:?}", e),
            };
        }
        Err(e) => println!("Error: {}", e), //"input couldn 't be translated to an expression: {e}"
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
    //println!("{}", std::str::from_utf8(&buffer).unwrap());
    let response = format!("{status_line}{html}");
    stream.write_all(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}

fn send_tcp_request(mut stream: TcpStream, input: String) -> TcpStream {
    match stream.write(input.as_bytes()) {
        Ok(n) => {
            println!("Wrote {} bytes", n)
        }
        Err(e) => {
            println!("Error: {}", e);
        }
    }
    return stream;
}

fn recieve_response(mut stream: TcpStream) -> f32 {
    let mut buf = [0; 4];
    let mut counter = 0;
    loop {
        counter += 1;
        println!("Count: {}", counter);
        match stream.read(&mut buf) {
            Ok(v) => {
                let n = v;
                println!("Read {} bytes", n);
                break;
            }
            Err(e) => {
                println!("Error: {}", e);
            }
        };
    }

    return f32::from_be_bytes(buf);
}

fn translate_to_expression(expr: String) -> Result<Expression, &'static str> {
    let (operand_left, operator, operand_right): (f32, char, f32);
    let op_index: usize;
    match get_op_index(expr.clone()) {
        Ok(v) => op_index = v,
        Err(e) => return Err(e),
    }
    let res_left = expr[..op_index].parse::<f32>();
    match res_left {
        Ok(v) => operand_left = v,
        Err(_) => return Err("left side is not a number"),
    }
    operator = expr[op_index..op_index + 1].parse::<char>().unwrap();
    let res_right = expr[op_index + 1..].parse::<f32>();
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

fn calculate(expr: Expression) -> Result<f32, &'static str> {
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
