# Dynamic Thread Pool Written in Rust

[简体中文](README.md) | [English](README.en.md)

## Features

1. **Dynamic Thread Management** - Automatically adjusts thread count based on workload
2. **Intelligent Recycling** - Idle thread timeout recycling (default 10s)
3. **Zero External Dependencies** - Built entirely on Rust standard library
4. **High Concurrency Safety** - Resolved race conditions and resource leaks
5. **Performance Optimized** - Atomic operations with reduced lock contention
6. **Extensive Documentation** - Detailed English comments for easy learning


## Architecture

### Core Components
- **Task Queue**: High-performance `VecDeque` implementation
- **Thread Management**: Dynamic worker thread creation and recycling
- **Synchronization**: `Arc`, `Mutex`, `Condvar`, and atomic operations
- **Resource Cleanup**: Automatic cleanup of exited thread handles

### Concurrency Safety
- Atomic thread creation operations prevent race conditions
- Thread-safe task distribution and state management
- Graceful thread pool shutdown and resource cleanup

## Performance

- **Low Latency**: Optimized locking strategy reduces thread blocking
- **High Throughput**: Atomic operations reduce lock contention
- **Memory Efficient**: Timely cleanup prevents resource leaks
- **Short-term Optimized**: 10-second timeout suitable for batch processing

## Example Usage

```rust

use std::sync::Arc;
use std::thread;
use std::time::Duration;
use thread_pool::ThreadPool;

fn main() {
// Create thread pool with default max threads (CPU cores)
let pool = Arc::new(ThreadPool::new());

    // Monitor thread count for 20 seconds
    let pool_clone = Arc::clone(&pool);
    pool.submit(move || monitor(pool_clone, 20));
    
    // Submit 100 tasks
    let total_tasks = 100;
    for task_id in 0..total_tasks {
        thread::sleep(Duration::from_millis(200));
        let pool_clone = Arc::clone(&pool);
        pool_clone.submit(move || task(task_id));
    }
    
    // Wait for all tasks to complete
    pool.wait_for_completion();
    }

fn task(task_id: usize) {
	println!("Executing task {}", task_id);
	thread::sleep(Duration::from_secs(2));
	println!("Task {} completed", task_id);
}

fn monitor(pool: Arc<ThreadPool>, seconds: usize) {
	for i in 0..seconds * 10 {
		println!("Current threads: {}", pool.threads_num());
		thread::sleep(Duration::from_millis(100));
	}
}

```

## Project Dependencies

**No external dependencies!** Built entirely on Rust standard library.

```toml

[package]
name = "rust-dynamic-thread-pool"
version = "0.2.0"
edition = "2021"

[dependencies]

# No external dependencies required

```


## Acknowledgments

Design inspired by: [senlinzhan/dpool](https://github.com/senlinzhan/dpool)
Foundational learning book "The Rust Bible": [sunface/rust-course](https://github.com/sunface/rust-course)
