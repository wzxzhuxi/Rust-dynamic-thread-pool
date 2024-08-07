mod thread_pool; // 引入线程池模块
mod link_list; // 引入链表模块

use std::sync::{Arc, Mutex}; // 导入Arc（原子引用计数）和Mutex（互斥锁）
use std::thread; // 导入线程模块
use std::time::Duration; // 导入Duration（时间段）

use thread_pool::ThreadPool; // 从线程池模块导入ThreadPool

// 定义一个全局的互斥锁，用于保护控制台输出
lazy_static::lazy_static! {
    static ref COUT_MTX: Mutex<()> = Mutex::new(()); // 使用Mutex保护控制台输出
}

// 定义任务函数
fn task(task_id: usize) {
    // 任务开始时，输出任务ID
    {
        let _guard = COUT_MTX.lock().unwrap(); // 获取控制台输出的锁
        println!("task-{} begin!", task_id); // 输出任务开始信息
    }

    // 模拟任务执行2秒
    thread::sleep(Duration::from_secs(2)); // 休眠2秒

    // 任务结束时，输出任务ID
    {
        let _guard = COUT_MTX.lock().unwrap(); // 获取控制台输出的锁
        println!("task-{} end!", task_id); // 输出任务结束信息
    }
}

// 定义监控函数
fn monitor(pool: Arc<ThreadPool>, seconds: usize) {
    for _ in 1..seconds * 10 { // 每秒钟循环10次，共循环秒数乘以10次
        {
            let _guard = COUT_MTX.lock().unwrap(); // 获取控制台输出的锁
            println!("thread num: {}", pool.threads_num()); // 输出当前线程数
        }
        thread::sleep(Duration::from_millis(100)); // 休眠100毫秒
    }
}

fn main() {
    // 创建线程池，最大线程数为100
    let pool = Arc::new(ThreadPool::new(100)); // 创建线程池并使用Arc进行引用计数

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
