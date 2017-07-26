use std::sync::mpsc;
use std::thread;

pub struct ThreadPool<T>
    where T: FnOnce() + Send + 'static
{
    handles: Vec<PoolLink<T>>,
}

impl <T> ThreadPool<T>
    where T: FnOnce() + Send + 'static
{
    pub fn new(thread_number: usize) -> ThreadPool<T> {
        assert!(thread_number > 0);
        let mut pool = ThreadPool {
            handles: Vec::with_capacity(thread_number),
        };

        for _ in 0..thread_number {
            let (tx, rx): (mpsc::Sender<Next<T>>, mpsc::Receiver<Next<T>>) = mpsc::channel();

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

    pub fn give(&mut self, job: Box<T>) -> Result<(), mpsc::SendError<Next<T>>> {
        for link in self.handles.iter_mut() {
            while let Ok(_) = link.receiver.try_recv() {
                link.load -= 1;
            }
        }

        self.handles.sort();

        let mut link: &mut PoolLink<T> = &mut self.handles[0];
        link.sender.send(Next::Job(*job))?;
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

pub enum Next<T>
    where T: FnOnce() + Send + 'static
{
    Job(T),
    Stop,
}

struct PoolLink<T: FnOnce() + Send + 'static>{
    load: u32,
    sender: mpsc::Sender<Next<T>>,
    receiver: mpsc::Receiver<()>,
    joiner: thread::JoinHandle<()>,
}

use std::cmp;

impl <T> cmp::Ord for PoolLink<T>
    where T: FnOnce() + Send + 'static
{
    fn cmp(&self, other: &PoolLink<T>) -> cmp::Ordering {
        self.load.cmp(&other.load)
    }
}

impl <T> cmp::PartialOrd for PoolLink<T>
    where T: FnOnce() + Send + 'static
{
    fn partial_cmp(&self, other: &PoolLink<T>) -> Option<cmp::Ordering> {
        Some(self.load.cmp(&other.load))
    }
}

impl <T> cmp::PartialEq for PoolLink<T>
    where T: FnOnce() + Send + 'static
{
    fn eq(&self, other: &PoolLink<T>) -> bool {
        self.load == other.load
    }
}

impl <T> Eq for PoolLink<T>
    where T: FnOnce() + Send + 'static
{}
