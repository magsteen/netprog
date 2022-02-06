use std::fmt::Debug;
use std::sync::mpsc::{channel, Receiver, Sender, TryRecvError};
use std::sync::{Arc, Condvar, Mutex};
use std::{result, thread, time};

#[derive(Debug)]
struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}
#[derive(Debug)]
struct JobList {
    r: Receiver<Job>,
    running: bool,
}
#[derive(Debug)]
struct WorkerPool {
    n: usize,
    workers: Vec<Worker>,
    s: Sender<Job>,
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
    pub fn new(id: usize, lock_pair: Arc<(Mutex<JobList>, Condvar)>) -> Worker {
        //println!("New thread w. id: {:?}", id);
        let running = true;
        let t = std::thread::spawn(move || loop {
            let (lock, cvar) = &*lock_pair;
            let r_lock = lock.lock().unwrap();
            //print!("Locked, id: {:?}\n", id);

            match (*r_lock).r.try_recv() {
                Ok(job) => job.call_box(),
                Err(TryRecvError::Disconnected) => {
                    /* Handle disconnection */
                    print!("Channel disconnected\n")
                }
                Err(TryRecvError::Empty) => {
                    print!("Channel is empty. Wait id: {:?}\n", id);
                    let r_lock = cvar.wait(r_lock).unwrap();
                    //println!("--Thread woken up: {:?}", id);
                    if !(*r_lock).running {
                        println!("Loop broken");
                    }
                }
            }
        });
        Worker {
            id,
            thread: Some(t),
        }
    }
}

impl WorkerPool {
    pub fn new(n: usize, workers: Vec<Worker>) -> WorkerPool {
        let (s, r): (Sender<Job>, Receiver<Job>) = channel();
        let jl = JobList { r, running: true };
        let lock_pair = Arc::new((Mutex::new(jl), Condvar::new()));

        WorkerPool {
            n,
            workers,
            s,
            lock_pair,
        }
    }

    fn start(&mut self) -> () {
        for i in 0..self.n {
            self.workers
                .push(Worker::new(i, Arc::clone(&self.lock_pair)));
        }
        return;
    }

    fn stop(&mut self) -> bool {
        let (lock, cvar) = &*self.lock_pair;
        let mut r_lock = lock.lock().unwrap();
        (*r_lock).running = false;
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
        self.s.send(job).unwrap();
        let (_, cvar) = &*self.lock_pair;
        cvar.notify_one();
        //print!("Sent job.\n");
        return;
    }

    /**
     * If the worker holds a thread, then we take it from the worker.
     * This is necessesary for join() since, it needs ownership of the thread
     * to consume it.
     */
    fn join(&mut self) {
        //TODO
        for worker in &mut self.workers {
            if let Some(t) = worker.thread.take() {
                match t.join() {
                    Ok(data) => {
                        println!("Data: {:?}", data)
                    }
                    Err(e) => {
                        println!("Error: {:?}", e)
                    }
                }
                // t.join().expect("Couldn't join on the associated thread");
                println!("Join called");
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
    //let (main_sender, main_receiver): (Sender<Job>, Receiver<Job>) = channel();

    let mut worker_threads = WorkerPool::new(4, Vec::new());
    //let mut event_loop = WorkerPool::new(1, Vec::new());

    worker_threads.start(); // Create 4 internal threads
                            //event_loop.start(); // Create 1 internal thread

    worker_threads.post(|| print!("Job A posted\n") /*Task A*/);
    worker_threads.post(|| {
        print!("Job B posted\n")
        // Task B
        // Might run in parallel with task A
    });

    // event_loop.post(|| {
    //     print!("Job C posted\n")
    //     // Task C
    //     // Might run in parallel with task A and B
    // });

    // //thread::sleep(time::Duration::from_secs(2));
    // event_loop.post(|| {
    //     print!("Job D posted\n")
    //     // Task D
    //     // Will run after task C
    //     // Might run in parallel with task A and B
    // });
    worker_threads.stop();
    //event_loop.stop();

    worker_threads.join(); // Calls join() on the worker threads
                           //event_loop.join(); // Calls join() on the event thread
    return;
}
