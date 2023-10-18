use std::sync::{
    mpsc::{self, Receiver, Sender},
    Arc, Mutex,
};

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<Sender<Job>>,
}

impl ThreadPool {
    pub fn new(n: usize) -> Self {
        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));
        let workers = (0..n).map(|_| Worker::new(receiver.clone())).collect();
        Self { sender: Some(sender), workers }
    }
    pub fn execute<F: FnOnce() + Send + 'static>(&self, f: F) {
        self.sender.as_ref().unwrap().send(Box::new(f)).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        for worker in &mut self.workers {
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

struct Worker {
    thread: Option<std::thread::JoinHandle<()>>,
}

impl Worker {
    fn new(receiver: Arc<Mutex<Receiver<Job>>>) -> Worker {
        let thread = Some(std::thread::spawn(move || while let Ok(job) =  receiver.lock().unwrap().recv() {
            job();
        }));
        Worker { thread}
    }
}

type Job = Box<dyn FnOnce() + Send + 'static>;
