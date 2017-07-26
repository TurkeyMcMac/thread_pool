#![feature(fnbox)]

use std::sync::mpsc;
use std::thread;
use std::boxed::FnBox;

pub struct ThreadPool {
    handles: Vec<PoolLink>,
}

impl ThreadPool {
    pub fn new(thread_number: usize) -> ThreadPool {
        assert!(thread_number > 0);
        let mut pool = ThreadPool {
            handles: Vec::with_capacity(thread_number),
        };

        for _ in 0..thread_number {
            let (tx, rx): (mpsc::Sender<Next>, mpsc::Receiver<Next>) = mpsc::channel();

            let (tx_to_parent, rx_from_child): (mpsc::Sender<()>, mpsc::Receiver<()>) =
                mpsc::channel();

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

                        tx_to_parent.send(()).unwrap();
                    }
                }),
            });
        }

        pool
    }

    pub fn assign(&mut self, job: Box<FnBox() + Send + 'static>) -> Result<(), mpsc::SendError<Next>> {
        for link in self.handles.iter_mut() {
            while let Ok(_) = link.receiver.try_recv() {
                link.load -= 1;
            }
        }

        self.handles.sort();

        let mut link: &mut PoolLink = &mut self.handles[0];
        link.sender.send(Next::Job(job))?;
        link.load += 1;

        Ok(())
    }

    pub fn join_all(&mut self) -> Result<(), Box<std::error::Error>> {
        for link in self.handles.drain(..) {
            link.sender.send(Next::Stop)?;
            link.joiner.join().unwrap();
        }

        Ok(())
    }
}

pub enum Next {
    Job(Box<FnBox() + Send + 'static>),
    Stop,
}

struct PoolLink {
    load: u32,
    sender: mpsc::Sender<Next>,
    receiver: mpsc::Receiver<()>,
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
