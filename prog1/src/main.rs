use std::env;
use std::thread;

fn main() {
    let args: Vec<String> = env::args().collect();
    println!("{:?}", args);

    if args[1] == "-h" {
        println!("HELP: run command: <executable> 'start' 'end 'num_of_threads'");
        return
    }
    else if args.len() != 4 {
        println!("Insufficient number of datapoints provided. Use '-h' for help." );
        return
    }

    //Input verification
    let range_start;
    let input1 = args[1].parse::<u32>();
    match input1 {
        Ok(input1) => {range_start = input1}
        Err(_input1) => {println!("Datapoint 1 is not an integer."); return}
    }
    let range_end;
    let input2 = args[2].parse::<u32>();
    match input2 {
        Ok(input2) => {range_end = input2}
        Err(_input2) => {println!("Datapoint 2 is not an integer."); return}
    }
    let num_threads;
    let input3 = args[3].parse::<u32>();
    match input3 {
        Ok(input3) => {num_threads = input3}
        Err(_input3) => {println!("Datapoint 3 is not an integer."); return}
    }

    //Main program
    let mut threads = Vec::new();
    let mut all_primes = Vec::new();

    for i in 0..num_threads {
        threads.push(thread::spawn(move || {
            let data = find_thread_input(range_start + i, range_end, num_threads as usize);
            find_primes(data)
        }));
    }
    let mut result_primes: Vec<u32>;
    for thread in threads {
        result_primes = thread.join().unwrap();
        println!("Thread res: {:?}", result_primes);

        all_primes.append(&mut result_primes);
    }

    all_primes.sort();
    println!("Final sorted result: {:?}", all_primes);
}

//Finds and returns the primes in the given dataset.
fn find_primes(data: Vec<u32>) -> Vec<u32> {
    let mut primes = Vec::new();
    for num in data {
        if num < 2 {continue}
        if num == 2 {
            primes.push(num);
            continue;
        }
        let num_f = num as f64;
        let num_square = num_f.sqrt().ceil() as u32 + 1;
        let mut is_prime: bool = true;
        for i in 2..num_square {
            if num % i == 0 {
                is_prime = false;
                break
            }
        }
        if is_prime {primes.push(num)}
    }

    primes
}

//Returns all data points between given start and end, with a give stepsize
fn find_thread_input(start:u32, end:u32, step:usize) -> Vec<u32> {
    (start..=end)
        .step_by(step)
        .collect()
    }
