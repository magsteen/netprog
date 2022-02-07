use std::sync::mpsc::{channel, Receiver, Sender, TryRecvError};
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;
use std::{result, thread, time};

#[derive(Debug)]
struct Worker {
    thread: Option<thread::JoinHandle<()>>,
}
struct JobList {
    jobs: Vec<Job>,
    stop: bool,
}
struct WorkerPool {
    n: usize,
    workers: Vec<Worker>,
    lock_pair: Arc<(Mutex<JobList>, Condvar)>,
}
trait FnBox {
    fn call_box(self: Box<Self>);
}

impl<F: FnOnce()> FnBox for F {
    fn call_box(self: Box<F>) {
        (*self)()
    }
}
type Job = Box<dyn FnBox + Send>;

impl Worker {
    pub fn new(lock_pair: Arc<(Mutex<JobList>, Condvar)>) -> Worker {
        let t = std::thread::spawn(move || loop {
            let (lock, cvar) = &*lock_pair;
            let mut r_lock = lock.lock().unwrap();

            while r_lock.jobs.is_empty() && !r_lock.stop {
                let result = cvar.wait_timeout(r_lock, Duration::from_millis(1)).unwrap();
                r_lock = result.0;
            }

            if r_lock.jobs.is_empty() && r_lock.stop {
                break;
            };

            let rem_jobs: Vec<Job> = r_lock.jobs.drain(1..).collect();
            let job = r_lock.jobs.pop().unwrap();
            r_lock.jobs = rem_jobs;
            drop(r_lock);

            job.call_box();
        });
        Worker { thread: Some(t) }
    }
}

impl WorkerPool {
    pub fn new(n: usize, workers: Vec<Worker>) -> WorkerPool {
        let jl = JobList {
            jobs: Vec::new(),
            stop: false,
        };
        let lock_pair = Arc::new((Mutex::new(jl), Condvar::new()));

        WorkerPool {
            n,
            workers,
            lock_pair,
        }
    }

    fn start(&mut self) -> () {
        for _ in 0..self.n {
            self.workers.push(Worker::new(Arc::clone(&self.lock_pair)));
        }
        return;
    }

    fn stop(&mut self) -> bool {
        let (lock, cvar) = &*self.lock_pair;
        let mut r_lock = lock.lock().unwrap();
        r_lock.stop = true;
        cvar.notify_all();
        return true;
    }

    /**
     * Function sends the specified lambda job to a shared channel nad notifies the
     * condition variable of the state change.
     */
    fn post<F>(&mut self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        let (lock, cvar) = &*self.lock_pair;
        let mut r_lock = lock.lock().unwrap();
        r_lock.jobs.push(job);
        cvar.notify_one();
        return;
    }

    /**
     * If the worker holds a thread, then we take it from the worker.
     * This is necessesary for join() since, it needs ownership of the thread
     * to consume it.
     */
    fn join(&mut self) {
        for worker in &mut self.workers {
            if let Some(t) = worker.thread.take() {
                match t.join() {
                    Ok(_) => {
                        //println!("Data: {:?}", data)
                    }
                    Err(e) => {
                        println!("Error: {:?}", e)
                    }
                }
            }
        }
        return;
    }

    fn post_timeout() -> usize {
        //TODO

        //EXTRA: Add epoll for linux
        return 0;
    }
}

fn main() {
    let mut worker_threads = WorkerPool::new(4, Vec::new());
    let mut event_loop = WorkerPool::new(1, Vec::new());

    worker_threads.start(); // Create 4 internal threads
    event_loop.start(); // Create 1 internal thread

    worker_threads.post(|| print!("Job A posted\n"));
    worker_threads.post(|| print!("Job B posted\n"));

    event_loop.post(|| print!("Job C posted\n"));
    event_loop.post(|| print!("Job D posted\n"));

    worker_threads.stop();
    event_loop.stop();

    worker_threads.join(); // Calls join() on the worker threads
    event_loop.join(); // Calls join() on the event thread
    return;
}
