#![feature(fnbox)]

extern crate num_cpus;

use std::boxed::FnBox;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::JoinHandle;

use std::time::Duration;
#[test]
fn works() {
    let mut pool = ThreadPool::new();

    for i in 0..10 {
        pool.assign(move || {
            println!("Starting calculation {}...", i);
            thread::sleep(Duration::new(2, 0));
            println!("Done with {}!", i);
        });
    }

    pool.join_all().unwrap();
}

pub struct ThreadPool {
    task_queue: Arc<Mutex<VecDeque<Task>>>,
    join_handles: Vec<JoinHandle<()>>
}

impl ThreadPool {
    pub fn with(thread_number: usize) -> ThreadPool {
        assert!(thread_number > 0, "ThreadPools must have at least one thread");
        let mut pool = ThreadPool {
            task_queue: Arc::new(Mutex::new(VecDeque::new())),
            join_handles: Vec::with_capacity(thread_number),
        };

        for _ in 0..thread_number {
            pool.join_handles.push(spawn_worker(pool.task_queue.clone()));
        }

        pool
    }

    pub fn new() -> ThreadPool { ThreadPool::with(num_cpus::get()) }

    pub fn assign<T>(&mut self, job: T)
        where T: FnBox() + Send + 'static
    {
        self.task_queue.lock().unwrap().push_back(Task::Job(Box::new(job)));
    }

    pub fn join_all(mut self) -> thread::Result<()> {
        for _ in 0..self.join_handles.len() {
            match self.task_queue.lock() {
                Ok(v)  => v,
                Err(_) => break,
            }.push_back(Task::Terminate);
        }

        for join_handle in self.join_handles.drain(..) {
            join_handle.join()?;
        }

        Ok(())
    }
}

enum Task {
    Job(Box<FnBox() + Send + 'static>),
    Terminate,
}

fn spawn_worker(task_queue: Arc<Mutex<VecDeque<Task>>>) -> JoinHandle<()> {
    thread::spawn(move || {
        loop {
            match {
                let mut queue = task_queue.lock().unwrap();
                queue.pop_front()
            } {
                Some(Task::Job(j)) => (j)(),
                Some(Task::Terminate) => return,
                None => continue,
            };
        }
    })
}
