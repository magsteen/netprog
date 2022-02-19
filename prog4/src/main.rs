use async_std::{io, net::UdpSocket, task};
use std::env;

const MATH_SERVER_ADDRESS: &str = "localhost:8080";
struct Expression {
    operand_left: f64,
    operand_right: f64,
    operator: char,
}

fn main() {
    let args: Vec<String> = env::args().collect();
    match args[1].as_str() {
        "-c" => start_math_client(args[2].to_string()),
        "-s" => start_servers(),
        _ => panic!("A client/server argument must be specified (-c or -s)"),
    };
}

fn start_servers() {
    let mut handles = Vec::new();

    handles.push(std::thread::spawn(|| start_math_server()));

    for handle in handles {
        handle.join().unwrap();
    }
}

fn start_math_server() {
    task::block_on(async_math_server()).unwrap();
}

fn start_math_client(addr: String) {
    task::block_on(async_math_client(addr)).unwrap();
}

async fn async_math_client(addr: String) -> io::Result<()> {
    println!("Starting MATH client...");
    let socket = UdpSocket::bind(addr).await?;
    socket.connect(MATH_SERVER_ADDRESS).await?;

    loop {
        let mut input = String::new();
        let mut output = [0; 8];

        println!("Please provide an expression to calculate (e.g: <number><operator><number>): ");
        io::stdin().read_line(&mut input).await?;

        socket.send(input.as_bytes()).await?;
        if socket.recv(&mut output).await? == 0 {
            break;
        };
        let result = f64::from_be_bytes(output);
        println!("Answer: {}", result)
    }
    Ok(())
}

async fn async_math_server() -> io::Result<()> {
    let socket = UdpSocket::bind(MATH_SERVER_ADDRESS).await?;

    loop {
        let mut data = [0; 1024];
        let (bytes, addr) = socket.recv_from(&mut data).await?;
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
                        socket.send_to(&answer.to_be_bytes(), addr).await?;
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
