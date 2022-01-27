use std::sync::{Arc, Mutex, Condvar};
use std::sync::mpsc::{channel, Sender, Receiver};
use std::thread;

struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

struct WorkerPool {
    n : usize,
    workers : Vec<Worker>,
    //channels : Vec<Sender<Job>>
}

struct Job;

impl Worker {
    pub fn new(id: usize, r: Arc<Mutex<Receiver<Job>>>) -> Worker {
        let t = std::thread::spawn(|| { //move??
            //TODO
            r;
            //Wait for task coming into the receiver.
            //Send a result to the main_sender channel -> main threads get result. 
        });
        Worker {
            id, 
            thread: Some(t),
        }
    }
}

impl WorkerPool {
    fn start(&mut self) -> () {
        let (s, r): (Sender<Job>, Receiver<Job>) = channel();
        let r = Arc::new(Mutex::new(r));
        for i in 0..self.n {
            //self.channels[i] = s; //Clone()???
            self.workers.push(Worker::new(i, Arc::clone(&r)));
        }
        return
    }

    fn stop() -> bool {
        //TODO
        return true
    }

    fn post<T, Job>(&mut self, j: Job) 
    where   T: Send + 'static,
            Job: FnOnce() -> T + Send + 'static
    {
        //TODO
        //Send job to self.threadpool.channel
        return
    }

    /**
     * If the worker holds a thread, then we take it from the worker.
     * This is necessesary for join() since, it needs ownership of the thread
     * to consume it.
     * 
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

    let mut threads = Vec::new();
    //let (main_sender, main_receiver): (Sender<Job>, Receiver<Job>) = channel();

    for i in 0..4 {
        threads.push(thread::spawn(move || {
            
        }));
    }
    let mut worker_threads = WorkerPool{n:4, workers:Vec::new()};
    let mut event_loop = WorkerPool{n:1, workers:Vec::new()};

    worker_threads.start(); // Create 4 internal threads
    event_loop.start(); // Create 1 internal thread
    
    worker_threads.post(|| {} /*Task A*/);
    worker_threads.post(|| {
        // Task B
        // Might run in parallel with task A
    });
    
    event_loop.post(|| {
        // Task C
        // Might run in parallel with task A and B
    });
    event_loop.post(|| {
        // Task D
        // Will run after task C
        // Might run in parallel with task A and B
    });
    
    worker_threads.join(); // Calls join() on the worker threads
    event_loop.join(); // Calls join() on the event thread
}
