use std::io;
use std::env;
use std::thread;
use std::sync::{Arc, Mutex};

fn main() {
    // let args: Vec<String> = env::args().collect();
    // println!("{:?}", args);

    // let range_start = &args[1];
    // let range_end = &args[2];
    // let num_threads = &args[3];

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

    // Lag 2D array hvor hver ytre entry er kombo av start og slutt
    //Lag 1D array hvor hver entry er intervall lengende til den tråden. Index indikerer thread
    let mut threads = Vec::new();
    let mut all_primes = Vec::new();
    let num_threads = inputs[2];
    let mut intervals: Vec<i64> = Vec::new();
    find_intervals(&mut intervals, inputs);
    println!("Intervals: {:?}", intervals);


    let m = Arc::new(Mutex::new(intervals));

    for i in 0..inputs[2] {
        let m_clone = m.clone();
        threads.push(thread::spawn(move || {
            let intervals = m_clone.lock().unwrap();
            let mut start = 0;
            for index in 1..i+1 {
                start = start + intervals[index as usize - 1];
            }
            let end = start + intervals[i as usize];
            drop(intervals);
            find_primes(start, end)
        }));
    }

    let mut result_primes: Vec<i64>;
    for thread in threads {
        result_primes = thread.join().unwrap();
        all_primes.append(&mut result_primes);
    }

    println!("Result: {:?}", all_primes);
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
    println!("thread res: {:?}", primes);

    primes
}

fn find_intervals(intervals: &mut Vec<i64>, inputs: [i64;3]) {
    let num_threads = inputs[2];
    let total_range_length = inputs[1] - inputs[0];
    let base_interval_length = (total_range_length) / num_threads; //floor?

    let mut dividable_rest = total_range_length - (num_threads as i64 * base_interval_length);
    for _ in 0..num_threads as usize{ //for lengden av arrayet? De siste som ikke får rest får 
        dividable_rest = dividable_rest / 2;
        intervals.push(base_interval_length + dividable_rest);
    }
}
