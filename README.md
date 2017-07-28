# Thread Pool
I am learning Rust, and this seemed like a fun project. The first version did not use mutexes, and was a bit messier. When I came to the chapter in the Rust Book on creating a thread pool, mutexes were used, and I realized that they would be better than my current solution. Still, the rest is my own design.

## Usage
`ThreadPool::with(thread_number)` Creates a pool with the specified thread number. Panics if the number given is less than one.
`ThreadPool::new()` Creates a thread pool that uses up the rest of the computer's available CPUs.
`thread_pool.assign(some_function)` Give a thread pool a `FnBox` to compute.
`thread_pool.join_all()` Join all of a thread pool's threads with the current one. Returns an error if panic(s) have occurred.

Note: Nightly Rust is required because, as of now, `FnOnce` cannot be called from inside a `Box`, but the experimental `FnBox`, which is in every other way equivalent to `FnOnce`, can.
#### Example
```
extern crate thread_pool;

use thread_pool::ThreadPool;

use std::thread;
use std::time::Duration;

let mut pool = ThreadPool::with(4);

for i in 0..10 {
    pool.assign(move || {
        println!("Starting calculation {}...", i);
        thread::sleep(Duration::new(2, 0));
        println!("Done with {}!", i);
    });
}

pool.join_all().unwrap();
```
