// use std::{
//     sync::{mpsc, Arc},
//     thread::{spawn, JoinHandle},
// };

// pub struct Job;
// struct Worker {
//     id: usize,
//     thread: JoinHandle<()>,
// }
// pub struct ThreadPool {
//     workers: Vec<Worker>,
//     sender: mpsc::Sender<Job>,
// }

// impl ThreadPool {
//     /// Create a new threadpool
//     ///
//     /// thread_ct must be greater than 0
//     fn new(thread_ct: usize) -> ThreadPool {
//         assert!(thread_ct > 0);

//         let (sender, receiver) = mpsc::channel();
//         let mut workers = Vec::with_capacity(thread_ct);

//         for id in 0..thread_ct {
//             // workers.push(Worker::new(id, Arc::<mpsc::Receiver<Job>>(receiver)))
//         }

//         ThreadPool { workers, sender }
//     }
//     // fn spawn<T, F>(fxn: F) -> JoinHandle<T>
//     // where
//     //     F: FnOnce() -> T,
//     //     F: Send + 'static,
//     //     T: Send + 'static,
//     // {
//     // }
//     fn execute<F>(&self, fxn: F)
//     where
//         F: FnOnce() + Send + 'static,
//     {
//     }
// }

// impl Worker {
//     fn new(id: usize, receiver: mpsc::Receiver<Job>) -> Worker {
//         let thread = spawn(|| {
//             receiver;
//         });
//         Worker { id, thread }
//     }
// }
