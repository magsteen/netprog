use std::env;
use std::fmt;
use std::result;

use async_std::io::prelude::BufReadExt;
use async_std::io::{BufReader, ReadExt, WriteExt};
use async_std::prelude::*;
use async_std::{io, net::TcpListener, net::TcpStream, task};

const MATH_SERVER_ADDRESS: &str = "localhost:8080";
const HTTP_SERVER_ADDRESS: &str = "localahost:8081";

struct Expression {
    operand_left: f32,
    operand_right: f32,
    operator: char,
}

fn main() {
    let args: Vec<String> = env::args().collect();
    match args[1].as_str() {
        "-s" => start_servers(),
        "-c" => start_math_client(),
        _ => panic!("You must specify a client/server argument (-c or -s)"),
    };
}

fn start_servers() {
    let mut handles = Vec::new();

    handles.push(std::thread::spawn(|| start_math_server()));
    handles.push(std::thread::spawn(|| start_web_server()));

    for handle in handles {
        handle.join().unwrap();
    }
}

fn start_math_client() {
    task::block_on(async_math_client()).unwrap();
}

fn start_math_server() {
    task::block_on(async_math_server()).unwrap();
}

async fn async_math_client() -> io::Result<()> {
    println!("Starting MATH client...");
    let mut stream = TcpStream::connect(MATH_SERVER_ADDRESS).await?;

    loop {
        let mut input = String::new();
        let mut output = [0; 4];

        println!("Please provide an expression to calculate (e.g: <number><operator><number>): ");
        io::stdin().read_line(&mut input).await?;

        stream.write(input.as_bytes()).await?;
        if stream.read(&mut output).await? == 0 {
            break;
        };
        let result = f32::from_be_bytes([output[0], output[1], output[2], output[3]]);
        println!("Answer: {}", result)
    }
    Ok(())
}

async fn async_math_server() -> io::Result<()> {
    let listener = TcpListener::bind(MATH_SERVER_ADDRESS).await?;
    let mut incoming = listener.incoming();
    while let Some(stream) = incoming.next().await {
        let stream = stream?;
        task::spawn(handle_math_connection(stream));
    }

    Ok(())
}

async fn handle_math_connection(mut stream: TcpStream) -> io::Result<()> {
    loop {
        let mut data = [0; 1024];
        let bytes = stream.read(&mut data).await?;
        //println!("Receive");
        let byte_data = &data[..bytes - 1]; //Drop newline char at end of message
        let input = match std::str::from_utf8(&byte_data) {
            Ok(res) => res,
            Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
        };

        println!("Got input: {:?}, with length: {:?}", input, input.len());
        if input.starts_with("exit") {
            break;
        }
        let expr = translate_to_expression(input.to_string());
        match expr {
            Ok(v) => {
                match calculate(v) {
                    Ok(answer) => {
                        //println!("Send");
                        stream.write(&answer.to_be_bytes()).await?;
                    }
                    Err(e) => println!("Server says: Calculation failed: {:?}", e),
                };
            }
            Err(e) => println!("Server says: Error: {}", e), //"input couldn 't be translated to an expression: {e}"
        }
    }

    Ok(())
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

fn start_web_client() {}

fn start_web_server() {}

// fn handle_web_connection(mut stream: TcpStream) {
//     let mut buffer = [0; 1024];
//     stream.read(&mut buffer).unwrap();

//     let get = b"GET / HTTP/1.1\r\n";

//     let (status_line, html) = if buffer.starts_with(get) {
//         (
//             "HTTP/1.1 200 OK\r\n\r\n",
//             "<!DOCTYPE html>
//         <html lang='no'>
//           <head>
//             <meta charset='utf-8'>
//             <title>Calcumulator</title>
//           </head>
//           <body>
//             <h1>Hei og velkommen til min web-tjener</h1>
//             <div>Header fra klient gav følgende: "
//                 .to_owned()
//                 + std::str::from_utf8(&buffer).unwrap()
//                 + "
//             </div>
//           </body>
//         </html>
//         ",
//         )
//     } else {
//         ("HTTP/1.1 404 NOT FOUND\r\n\r\n", "".to_owned())
//     };
//     //println!("{}", std::str::from_utf8(&buffer).unwrap());
//     let response = format!("{status_line}{html}");
//     stream.write_all(response.as_bytes()).unwrap();
//     stream.flush().unwrap();
// }
