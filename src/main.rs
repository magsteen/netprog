use std::io;
use std::thread;

fn main() {
    println!("Welcom to primenumber finder with extra steps!");

    let stdin = io::stdin();
    let mut input = String::new();

    println!("Please provide number inputs in this order and seperated by a space: 'start_of_interval' 'end_of_interval' 'num_of_threads'");

    stdin
        .read_line(&mut input)
        .expect("Falied to read line");

    println!("Got: {}", input);

    let mut inputs:[i64; 3] = [0, 0, 0];
    let splits = input.split_whitespace();
    for (i, split) in splits.enumerate() {
        inputs[i] = split
            .parse()
            .expect("Failed to parse split to int");
    }
    println!("array: {:?}", inputs);

    let thread_interval_length = (inputs[1] - inputs[0]) / inputs[2]; //TODO: handle potential floats

    let mut threads = Vec::new();

    for i in 0..inputs[2] {
        threads.push(thread::spawn(move || {
            let start = thread_interval_length * i;
            let end = start + thread_interval_length;
            let primes = find_primes(start, end);// Function call to find primes between start and end. 
            println!("output from thread {:?}", primes);
        }));
    }

    for thread in threads {
        let _ = thread.join();
    }
}

fn find_primes(start: i64, end: i64) -> Vec<i64> {
    let mut primes = Vec::new();
    for num in start..end {
        if num < 2 {continue}
        if num == 2 {
            primes.push(num);
            continue;
        }
        let num_f = num as f64;
        let num_square = num_f.sqrt().ceil() as i64 + 1;
        let mut is_prime: bool = true;
        for i in 2..num_square {
            //println!("start: {}, num {}, i {}", start, num,i);
            if num % i == 0 {
                is_prime = false;
                break
            }
        }
        if is_prime {primes.push(num)}
    }

    primes
}
