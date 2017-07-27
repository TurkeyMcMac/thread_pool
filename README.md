# Thread Pool
I am learning Rust, and this seemed like a fun project. Nightly Rust is required
because, as of now, `FnOnce` cannot be called from inside a `Box`, but the
experimental `FnBox`, which is in every other way equivalent to `FnOnce`, can.

## Usage
To instantiate a pool of threads, use `ThreadPool::new(number_of_threads)`. To
give it something to compute, use `assign(Box::new(move some_function))`. The
pool will automatically distribute work between all available threads.
`join_all()` joins all  of a pool's threads after waiting for them to finish
their current jobs, consuming the pool in the process.

#### Example
```
extern crate thread_pool;

use thread_pool::ThreadPool;

use std::thread;
use std::time::Duration;

let pool = ThreadPool::with(4);

for i in 0..10 {
    pool.assign(move || {
        println!("Starting calculation {}...", i);
        thread::sleep(Duration::new(2, 0));
        println!("Done with {}!", i);
    }).unwrap();
}

pool.join_all().unwrap();
```
