# 使用 Rust 编写的动态线程池

[简体中文](README.md) | [English](README.en.md)

## 特性

1. **动态线程管理** - 根据任务数量动态调整线程数
2. **智能回收机制** - 支持空闲线程超时回收（默认10s）
3. **零外部依赖** - 完全基于 Rust 标准库实现
4. **高并发安全** - 解决了竞态条件和资源泄漏问题
5. **性能优化** - 原子操作优化，减少锁竞争
6. **详细中文注释** - 逐行注释，轻松上手学习

## 架构特点

### 核心组件
- **任务队列**: 使用 `VecDeque` 实现的高性能双端队列
- **线程管理**: 动态创建和回收工作线程
- **同步机制**: `Arc`、`Mutex`、`Condvar` 和原子操作
- **资源清理**: 自动清理退出线程的句柄，防止内存泄漏

### 并发安全
- 原子化的线程创建操作，避免竞态条件
- 线程安全的任务分发和状态管理
- 优雅的线程池关闭和资源清理

## 性能表现

- **低延迟**: 优化的锁策略减少线程阻塞
- **高吞吐**: 原子操作减少锁竞争
- **内存高效**: 及时清理退出线程，避免资源泄漏
- **适合短期使用**: 10秒超时机制适合批量任务处理

## 使用样例

```rust

use std::sync::Arc;
use std::thread;
use std::time::Duration;
use thread_pool::ThreadPool;

fn main() {
// 创建线程池，默认最大线程数为硬件支持的最大线程数
let pool = Arc::new(ThreadPool::new());

    // 监控线程数20秒
    let pool_clone = Arc::clone(&pool);
    pool.submit(move || monitor(pool_clone, 20));
    
    // 提交100个任务
    let total_tasks = 100;
    for task_id in 0..total_tasks {
        thread::sleep(Duration::from_millis(200));
        let pool_clone = Arc::clone(&pool);
        pool_clone.submit(move || task(task_id));
    }
    
    // 等待所有任务完成
    pool.wait_for_completion();
    }

fn task(task_id: usize) {
	println!("执行任务 {}", task_id);
	thread::sleep(Duration::from_secs(2));
	println!("任务 {} 完成", task_id);
}

fn monitor(pool: Arc<ThreadPool>, seconds: usize) {
	for i in 0..seconds * 10 {
		println!("当前线程数: {}", pool.threads_num());
			  			wqfthread::sleep(Duration::from_millis(100));
	}	
}

```

## 项目依赖

**无外部依赖！** 完全基于 Rust 标准库实现。

```toml

[package]
name = "rust-dynamic-thread-pool"
version = "0.2.0"
edition = "2021"

[dependencies]

# 无需外部依赖

```

## 致谢

本项目设计思路源于：[senlinzhan/dpool](https://github.com/senlinzhan/dpool)  
基础知识学习书籍《Rust 圣经》：[sunface/rust-course](https://github.com/sunface/rust-course)



# Dynamic Thread Pool Written in Rust

[简体中文](README.md) | [English](README.en.md)

## Features

1. **Dynamic Thread Management** - Automatically adjusts thread count based on workload
2. **Intelligent Recycling** - Idle thread timeout recycling (default 10s)
3. **Zero External Dependencies** - Built entirely on Rust standard library
4. **High Concurrency Safety** - Resolved race conditions and resource leaks
5. **Performance Optimized** - Atomic operations with reduced lock contention
6. **Extensive Documentation** - Detailed Chinese comments for easy learning


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
