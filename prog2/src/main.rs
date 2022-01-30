use std::fmt::Debug;
use std::sync::{Arc, Mutex, Condvar};
use std::sync::mpsc::{channel, Sender, Receiver, TryRecvError};
use std::{thread, time};

#[derive(Debug)]
struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}
#[derive(Debug)]
struct WorkerPool {
    n : usize,
    workers : Vec<Worker>,
    s : Sender<Job>,
    lock_pair : Arc<(Mutex<Receiver<Box<dyn FnBox + Send>>>, Condvar)>,
}
trait FnBox {
    fn call_box(self: Box<Self>);
}

impl<F: FnOnce()> FnBox for F {
    fn call_box(self: Box<F>) {
        (*self)()
    }
}
type Job = Box<dyn FnBox + Send + 'static>;

impl Worker {
    pub fn new(id: usize, lock_pair: Arc<(Mutex<Receiver<Job>>, Condvar)>) -> Worker {
        //println!("New thread w. id: {:?}", id);
        let t = std::thread::spawn(move|| loop {
            let (lock, cvar) = &*lock_pair;
            let r_lock = lock.lock().unwrap();
            //print!("Locked, id: {:?}\n", id);

            match r_lock.try_recv() {
                Ok(job) => {job.call_box()}
                Err(TryRecvError::Disconnected) => {/* Handle disconnection */ print!("Channel disconnected\n")}
                Err(TryRecvError::Empty) => {
                    //print!("Channel is empty. Wait id: {:?}\n", id);
                    let _ = cvar.wait(r_lock).unwrap();
                    //println!("--Thread woken up: {:?}", id);
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
        let lock_pair = Arc::new((Mutex::new(r), Condvar::new()));

        WorkerPool { 
            n, 
            workers, 
            s,
            lock_pair,
        }
    }

    fn start(&mut self) -> () {
        for i in 0..self.n {
            self.workers.push(Worker::new(i, Arc::clone(&self.lock_pair)));
        }
        return
    }

    fn stop() -> bool {
        //TODO
        return true
    }

    /**
     * Function sends the specified lambda job to a shared channel nad notifies the
     * condition variable of the state change. 
     */
    fn post<F>(&mut self, f: F) 
    where   F: FnOnce() + Send + 'static
    {
        let job = Box::new(f);
        self.s.send(job).unwrap();
        let (_, cvar) = &*self.lock_pair;
        cvar.notify_one();
        //print!("Sent job.\n");
        return
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
                let _ = t.join(); //Add error handling
            }
        }
        return
    }

    fn post_timeout() -> usize {
        //TODO

        //EXTRA: Add epoll for linux
        return 0
    }
}


fn main() {
    //let (main_sender, main_receiver): (Sender<Job>, Receiver<Job>) = channel();

    let mut worker_threads = WorkerPool::new(4, Vec::new());
    let mut event_loop = WorkerPool::new(1, Vec::new());

    worker_threads.start(); // Create 4 internal threads
    event_loop.start(); // Create 1 internal thread
    
    worker_threads.post(|| {print!("Job A posted\n")} /*Task A*/);
    worker_threads.post(|| {
        print!("Job B posted\n")
        // Task B
        // Might run in parallel with task A
    });
    
    event_loop.post(|| {
        print!("Job C posted\n")
        // Task C
        // Might run in parallel with task A and B
    });

    //thread::sleep(time::Duration::from_secs(2));
    event_loop.post(|| {
        print!("Job D posted\n")
        // Task D
        // Will run after task C
        // Might run in parallel with task A and B
    });
    
    worker_threads.join(); // Calls join() on the worker threads
    event_loop.join(); // Calls join() on the event thread
}
