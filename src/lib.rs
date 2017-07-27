#![feature(fnbox)]

extern crate num_cpus;

use std::boxed::FnBox;
use std::sync::mpsc;
use std::thread;

use std::time::Duration;
#[test]
fn works() {
    let mut pool = ThreadPool::new();

    for i in 0..10 {
        pool.assign(move || {
            println!("Starting calculation {}...", i);
            thread::sleep(Duration::new(2, 0));
            println!("Done with {}!", i);
        }).unwrap();
    }

    pool.join_all().unwrap();
}

pub struct ThreadPool {
    handles: Vec<PoolLink>,
}

impl ThreadPool {
    pub fn with(thread_number: usize) -> ThreadPool {
        assert!(thread_number > 0);
        let mut pool = ThreadPool {
            handles: Vec::with_capacity(thread_number),
        };

        for _ in 0..thread_number {
            let (tx, rx): (mpsc::Sender<Next>, mpsc::Receiver<Next>) = mpsc::channel();

            let (tx_to_parent, rx_from_child): (mpsc::Sender<JobDone>,
                mpsc::Receiver<JobDone>) = mpsc::channel();

            pool.handles.push(PoolLink {
                load: 0,
                sender: tx,
                receiver: rx_from_child,
                joiner: thread::spawn(move || {
                    for next in rx {
                        match next {
                            Next::Job(j) => (j)(),
                            Next::Stop   => return,
                        }

                        tx_to_parent.send(JobDone).unwrap();
                    }
                }),
            });
        }

        pool
    }

    pub fn new() -> ThreadPool { ThreadPool::with(num_cpus::get()) }

    pub fn assign<T>(&mut self, job: T)
        -> Result<(), mpsc::SendError<Next>>
        where T: FnBox() + Send + 'static
    {
        for link in self.handles.iter_mut() {
            while let Ok(_) = link.receiver.try_recv() {
                link.load -= 1;
            }
        }

        self.handles.sort();

        let mut link: &mut PoolLink = &mut self.handles[0];
        link.sender.send(Next::Job(Box::new(job)))?;
        link.load += 1;

        Ok(())
    }

    pub fn join_all(mut self) -> Result<(), JoinAllError<Next>> {
        for link in self.handles.drain(..) {
            if let Err(e) = link.sender.send(Next::Stop) {
                 return Err(JoinAllError::SendError(e));
            }
            if let Err(e) = link.joiner.join() {
                return Err(JoinAllError::JoinError(e));
            }
        }

        Ok(())
    }
}

struct JobDone;

#[derive(Debug)]
pub enum JoinAllError<T> {
    SendError(mpsc::SendError<T>),
    JoinError(Box<std::any::Any + Send + 'static>),
}

pub enum Next {
    Job(Box<FnBox() + Send + 'static>),
    Stop,
}

use std::fmt;

impl fmt::Debug for Next {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", match self {
            &Next::Job(_) => "Job",
            &Next::Stop   => "Stop",
        })?;

        Ok(())
    }
}

struct PoolLink {
    load: u32,
    sender: mpsc::Sender<Next>,
    receiver: mpsc::Receiver<JobDone>,
    joiner: thread::JoinHandle<()>,
}

use std::cmp;

impl cmp::Ord for PoolLink {
    fn cmp(&self, other: &PoolLink) -> cmp::Ordering {
        self.load.cmp(&other.load)
    }
}

impl cmp::PartialOrd for PoolLink {
    fn partial_cmp(&self, other: &PoolLink) -> Option<cmp::Ordering> {
        Some(self.load.cmp(&other.load))
    }
}

impl cmp::PartialEq for PoolLink {
    fn eq(&self, other: &PoolLink) -> bool {
        self.load == other.load
    }
}

impl Eq for PoolLink {}
