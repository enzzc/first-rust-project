use std::io::prelude::*;
use std::thread;
use std::boxed::Box;
use std::sync::{Arc, Mutex, Condvar};
use std::collections::VecDeque;
use std::net::TcpStream;


type SharedQueue = Mutex<VecDeque<TcpStream>>;


#[derive(Debug)]
pub struct WorkerQueue {
    queue: SharedQueue,
    cv: Condvar,
}


#[derive(Debug)]
pub struct ThreadPool {
    workers: Vec<Worker>,
}

#[derive(Debug)]
pub struct Worker {
    id: usize,
    thread: thread::JoinHandle<()>,
    queue: Arc<WorkerQueue>,
}




impl Worker {
    pub fn new(id: usize) -> Worker {
        let shared_queue: SharedQueue = Mutex::new(VecDeque::new());
        let cv = Condvar::new();
        let mtx_queue = Arc::new(WorkerQueue { queue: shared_queue, cv });
        let worker_queue = Arc::clone(&mtx_queue);

        let thread = thread::spawn(move || {
            let _id = id.clone();
            let recv_queue = Arc::clone(&mtx_queue);

            let mut buf = Box::new([0; 65535]);
            static RESP: &[u8] = "HTTP/1.0 200 OK\r\nContent-Length: 5\r\n\r\nHELLO".as_bytes();

            loop {
                let mut shared_data = recv_queue.queue.lock().unwrap();
                match shared_data.pop_back() {
                    Some(mut stream) => {
                        stream.read(&mut *buf).unwrap();
                        stream.write(&RESP).unwrap();
                        stream.flush().unwrap();
                    },
                    None => {
                        shared_data = recv_queue.cv.wait(shared_data).unwrap();
                    }
                }
            }
        });

        Worker{ id, thread, queue: worker_queue }
    }
}



impl ThreadPool {
    pub fn new(size: usize) -> ThreadPool {
        let mut workers = Vec::with_capacity(size);
        for id in 0..size {
            let w = Worker::new(id);
            workers.push(w);
        }
        ThreadPool { workers }
    }

    pub fn submit(&self, stream: TcpStream, n: usize) {
        let w = &self.workers.get(n).unwrap();
        let mut shared_data = w.queue.queue.lock().unwrap();
        shared_data.push_front(stream);
        w.queue.cv.notify_all();
    }

}
