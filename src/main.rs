extern crate thread_pool;

use thread_pool::ThreadPool;

use std::thread;
use std::time::Duration;
use std::sync::{Arc, Mutex};

fn main() {
    fn long_calculation() {
        println!("Starting calculation...");
        thread::sleep(Duration::new(2, 0));
        println!("Done!");
    }

    let mut pool = ThreadPool::new(4);

    for _ in 0..15 {
        pool.assign(Box::new(long_calculation)).unwrap();
    }

    thread::sleep(Duration::new(3, 0));


    let data = Arc::new(Mutex::new(16));
    let cdata = data.clone();
    pool.assign(Box::new(move || {
        let mut mdata = cdata.lock().unwrap();
        *mdata += 1;
    })).unwrap();

    pool.join_all().unwrap();
    println!("data = {:?}", data);
}
