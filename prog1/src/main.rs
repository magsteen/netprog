use std::env;
use std::thread;
use std::sync::{Arc, Mutex};

//Other strategy: Each threads send found prime numbers to the same channel. 
//One threads continuesly pulls data from channel and adds it to a min heap.
//Hepa is sorted when pulled. 

fn main() {
    let args: Vec<String> = env::args().collect();
    println!("{:?}", args);

    if args[1] == "-h" {
        println!("HELP: run command (where arguments are number inputs): <binary> 'start' 'end 'num_ofthreads'");
        return
    }

    let range_start = args[1].parse::<i64>().unwrap();
    let range_end = args[2].parse::<i64>().unwrap();
    let num_threads = args[3].parse::<i64>().unwrap();

    println!("Welcom to primenumber finder with extra steps!");

    let mut threads = Vec::new();
    let mut all_primes = Vec::new();
    let mut intervals: Vec<i64> = Vec::new();
    find_intervals(&mut intervals, range_start, range_end, num_threads);
    println!("Intervals: {:?}", intervals);

    let m = Arc::new(Mutex::new(intervals));

    for i in 0..num_threads {
        let m_clone = m.clone();
        threads.push(thread::spawn(move || {
            let intervals = m_clone.lock().unwrap();
            let mut start = range_start;
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
            if num % i == 0 {
                is_prime = false;
                break
            }
        }
        if is_prime {primes.push(num)}
    }
    //println!("thread res: {:?}", primes);

    primes
}

fn find_intervals(intervals: &mut Vec<i64>, range_start:i64, range_end:i64, num_threads:i64) {
    let total_range_length = range_end - range_start;
    let base_interval_length = total_range_length / num_threads;

    let mut dividable_rest = total_range_length % num_threads as i64;
    println!("rest: {}", dividable_rest);
    for _ in 0..num_threads as usize{
        dividable_rest = dividable_rest / 2;
        intervals.push(base_interval_length + dividable_rest);
    }
}
