use std::env;

use async_std::io::BufReader;
use async_std::prelude::*;
use async_std::{io, net::TcpListener, net::TcpStream, task};

const MATH_SERVER_ADDRESS: &str = "localhost:8080";
const HTTP_SERVER_ADDRESS: &str = "localhost:8081";

struct Expression {
    operand_left: f64,
    operand_right: f64,
    operator: char,
}

fn main() {
    let args: Vec<String> = env::args().collect();
    match args[1].as_str() {
        "-c" => start_math_client(),
        "-s" => start_servers(),
        _ => panic!("A client/server argument must be specified (-c or -s)"),
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

fn start_web_server() {
    task::block_on(async_web_server()).unwrap();
}

async fn async_web_server() -> io::Result<()> {
    let listener = TcpListener::bind(HTTP_SERVER_ADDRESS).await?;
    let mut incoming = listener.incoming();
    while let Some(stream) = incoming.next().await {
        let stream = stream?;
        task::spawn(handle_web_connection(stream));
    }

    Ok(())
}

fn start_math_server() {
    task::block_on(async_math_server()).unwrap();
}

fn start_math_client() {
    task::block_on(async_math_client()).unwrap();
}

async fn async_math_client() -> io::Result<()> {
    println!("Starting MATH client...");
    let mut stream = TcpStream::connect(MATH_SERVER_ADDRESS).await?;

    loop {
        let mut input = String::new();
        let mut output = [0; 8];

        println!("Please provide an expression to calculate (e.g: <number><operator><number>): ");
        io::stdin().read_line(&mut input).await?;

        stream.write(input.as_bytes()).await?;
        if stream.read(&mut output).await? == 0 {
            break;
        };
        let result = f64::from_be_bytes(output);
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
                        stream.write(&answer.to_be_bytes()).await?;
                    }
                    Err(e) => println!("Server says: Calculation failed: {:?}", e),
                };
            }
            Err(e) => println!("Server says: Error: {}", e),
        }
    }

    Ok(())
}

fn translate_to_expression(expr: String) -> Result<Expression, &'static str> {
    let (operand_left, operator, operand_right): (f64, char, f64);
    let op_index: usize;
    match get_op_index(expr.clone()) {
        Ok(v) => op_index = v,
        Err(e) => return Err(e),
    }
    let res_left = expr[..op_index].parse::<f64>();
    match res_left {
        Ok(v) => operand_left = v,
        Err(_) => return Err("left side is not a number"),
    }
    operator = expr[op_index..op_index + 1].parse::<char>().unwrap();
    let res_right = expr[op_index + 1..].parse::<f64>();
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

fn calculate(expr: Expression) -> Result<f64, &'static str> {
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
    return Err("Operator not found in expression!");
}

async fn handle_web_connection(mut stream: TcpStream) -> io::Result<()> {
    let mut buffer = [0; 1024];
    let mut string_data = String::new();
    let mut reader = BufReader::new(stream.clone());

    let bytes = reader.read_line(&mut string_data).await?;
    if bytes == 0 {
        return Ok(());
    };

    reader.read(&mut buffer).await?;

    let (status_line, html) = if string_data.eq("GET / HTTP/1.1\r\n") {
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
            <p>Header fra klient gav følgende: </p>
                <ul>"
                .to_owned()
                + &format_header_item_list(&buffer)
                + "
                </ul>
          </body>
        </html>
        ",
        )
    } else {
        ("HTTP/1.1 404 NOT FOUND\r\n\r\n", "".to_owned())
    };
    println!("{}", std::str::from_utf8(&buffer).unwrap());
    let response = format!("{status_line}{html}");
    stream.write_all(response.as_bytes()).await?;
    stream.flush().await?;
    Ok(())
}

fn format_header_item_list(buf: &[u8; 1024]) -> String {
    let mut result = "".to_owned();
    std::str::from_utf8(buf)
        .unwrap()
        .split("\r\n")
        .filter(|line| line.to_owned().trim().len() != 0)
        .for_each(|line| result.push_str(format!("<li>{}</li>", line).as_str()));
    return result;
}
