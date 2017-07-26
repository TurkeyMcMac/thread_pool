extern crate thread_pool;

use thread_pool::ThreadPool;

use std::thread;
use std::time::Duration;


fn main() {
    fn long_calculation() {
        println!("Starting calculation...");
        thread::sleep(Duration::new(2, 0));
        println!("Done!");
    }

    let mut pool = ThreadPool::new(4);

    for _ in 0..15 {
        pool.give(Box::new(long_calculation)).unwrap();
    }

    thread::sleep(Duration::new(3, 0));

    for _ in 0..5 {
        pool.give(Box::new(long_calculation)).unwrap();
    }

    pool.join_all().unwrap();
}
