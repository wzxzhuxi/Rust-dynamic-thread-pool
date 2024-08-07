# Dynamic Thread Pool Written in Rust

1. Dynamically adjust the number of threads based on the number of tasks
2. Support idle thread timeout recycling (default 10s)
3. Extensive Chinese comments added for easy understanding

## Example Usage:

```rust
fn main() {
    // Create a thread pool with the default maximum number of threads supported by the hardware
    let pool = Arc::new(ThreadPool::new()); // Create a thread pool and use Arc for reference counting

    // Monitor the number of threads for 13 seconds
    let pool_clone = Arc::clone(&pool); // Clone the Arc of the thread pool
    pool.submit(move || monitor(pool_clone, 20)); // Submit the monitoring task

    // Submit 100 tasks
    let total_tasks = 100; // Total number of tasks is 100
    for task_id in 0..total_tasks { // Loop to submit tasks
        thread::sleep(Duration::from_millis(200)); // Sleep for 200 milliseconds after submitting each task
        let pool_clone = Arc::clone(&pool); // Clone the Arc of the thread pool
        pool_clone.submit(move || task(task_id)); // Submit the task
    }

    // Wait for all tasks to complete
    pool.wait_for_completion(); // Wait for all tasks to complete
}
```

## Project Dependencies:

```toml
[dependencies]
lazy_static = "1.4"
num_cpus = "1.13"
```

Design inspired by: [senlinzhan/dpool](https://github.com/senlinzhan/dpool)  
LinkList implementation inspired by the Rust Bible from UST: [sunface/rust-course](https://github.com/sunface/rust-course)
