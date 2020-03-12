use std::sync::{mpsc, Arc, Mutex};
use std::thread;

type Job = Box<dyn FnOnce() + Send + 'static>;

enum Message {
    NewJob(Job),
    Terminate,
}

struct Worker {
    _id: usize,
    _thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Message>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let message: Message = receiver.lock().unwrap().recv().unwrap();

            println!("Worker {} got job, executing", id);

            match message {
                Message::NewJob(job) => {
                    job();
                }
                Message::Terminate => {
                    println!("Worker {} got terminate signal", id);
                    break;
                }
            }
        });

        Worker {
            _id: id,
            _thread: Some(thread),
        }
    }
}

pub struct ThreadPool {
    _workers: Vec<Worker>,
    _sender: mpsc::Sender<Message>,
}

impl ThreadPool {
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        let (sender, receiver) = mpsc::channel::<Message>();
        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)))
        }

        ThreadPool {
            _sender: sender,
            _workers: workers,
        }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job: Job = Box::new(f);

        self._sender.send(Message::NewJob(job)).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        for _ in &mut self._workers {
            self._sender.send(Message::Terminate).unwrap();
        }

        for worker in &mut self._workers {
            if let Some(thread) = worker._thread.take() {
                thread.join().unwrap();
            };
        }
    }
}
