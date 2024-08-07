
# 使用 Rust 编写的动态线程池

[简体中文](README.md) | [English](README_en.md)

1. 根据任务数量动态调整线程数
2. 支持空闲线程超时回收（默认10s）
3. 添加大量中文注释，轻松上手

## 使用样例：

```rust
fn main() {
    // 创建线程池，默认最大线程数为硬件支持的最大线程数
    let pool = Arc::new(ThreadPool::new()); // 创建线程池并使用Arc进行引用计数

    // 监控线程数13秒
    let pool_clone = Arc::clone(&pool); // 克隆线程池的Arc
    pool.submit(move || monitor(pool_clone, 20)); // 提交监控任务

    // 提交100个任务
    let total_tasks = 100; // 总任务数为100
    for task_id in 0..total_tasks { // 循环提交任务
        thread::sleep(Duration::from_millis(200)); // 每次提交任务后休眠200毫秒
        let pool_clone = Arc::clone(&pool); // 克隆线程池的Arc
        pool_clone.submit(move || task(task_id)); // 提交任务
    }

    // 等待所有任务完成
    pool.wait_for_completion(); // 等待所有任务完成
}
```

## 本项目库依赖：

```toml
[dependencies]
lazy_static = "1.4"
num_cpus = "1.13"
```

本项目设计思路源于：[senlinzhan/dpool](https://github.com/senlinzhan/dpool)  
本项目使用的 LinkList 源于 ust 圣经：[sunface/rust-course](https://github.com/sunface/rust-course)
