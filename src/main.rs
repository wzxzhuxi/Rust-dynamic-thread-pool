// 引入线程池模块
// Import thread pool module
mod thread_pool; 

// 导入Arc（原子引用计数）和Mutex（互斥锁）
// Import Arc (atomic reference counting) and Mutex (mutual exclusion lock)
use std::sync::{Arc, Mutex}; 

// 导入线程模块
// Import thread module
use std::thread; 

// 导入Duration（时间段）
// Import Duration (time duration)
use std::time::Duration; 

// 从线程池模块导入ThreadPool
// Import ThreadPool from thread pool module
use thread_pool::ThreadPool; 

// 使用标准库的OnceLock创建全局互斥锁，替代lazy_static
// Use standard library's OnceLock to create global mutex, replacing lazy_static
// OnceLock是Rust 1.70+引入的标准库类型，用于创建只初始化一次的全局变量
// OnceLock is a standard library type introduced in Rust 1.70+, used to create global variables that are initialized only once
static COUT_MTX: std::sync::OnceLock<Mutex<()>> = std::sync::OnceLock::new();

// 获取控制台输出互斥锁的辅助函数
// Helper function to get console output mutex
fn get_cout_mutex() -> &'static Mutex<()> {
    // 如果未初始化则初始化，否则返回已有的值
    // Initialize if not already initialized, otherwise return existing value
    COUT_MTX.get_or_init(|| Mutex::new(())) 
}

// 定义任务函数
// Define task function
fn task(task_id: usize) {
    // 任务开始时，输出任务ID
    // Output task ID when task begins
    {
        // 获取控制台输出的锁，使用expect提供错误信息
        // Get console output lock, use expect for error information
        let _guard = get_cout_mutex().lock().expect("Failed to lock console output mutex"); 
        
        // 输出任务开始信息
        // Output task start information
        println!("task-{} begin!", task_id); 
    } // _guard在此处自动释放，解锁互斥锁
      // _guard automatically releases here, unlocking mutex

    // 模拟任务执行2秒
    // Simulate task execution for 2 seconds
    // 休眠2秒，模拟实际工作负载
    // Sleep for 2 seconds, simulating actual workload
    thread::sleep(Duration::from_secs(2)); 

    // 任务结束时，输出任务ID
    // Output task ID when task ends
    {
        // 再次获取控制台输出的锁
        // Get console output lock again
        let _guard = get_cout_mutex().lock().expect("Failed to lock console output mutex"); 
        
        // 输出任务结束信息
        // Output task end information
        println!("task-{} end!", task_id); 
    } // _guard在此处自动释放，解锁互斥锁
      // _guard automatically releases here, unlocking mutex
}

// 定义监控函数，用于实时监控线程池状态
// Define monitor function for real-time monitoring of thread pool status
fn monitor(pool: Arc<ThreadPool>, seconds: usize) {
    // 循环监控指定秒数，每100毫秒检查一次线程数
    // Loop monitoring for specified seconds, checking thread count every 100 milliseconds
    for i in 1..seconds * 10 { // 每秒钟循环10次，共循环秒数乘以10次
                               // Loop 10 times per second, total loops = seconds * 10
        {
            // 获取控制台输出的锁
            // Get console output lock
            let _guard = get_cout_mutex().lock().expect("Failed to lock console output mutex"); 
            
            println!("thread num: {} (monitoring: {}/{})", 
                pool.threads_num(), // 输出当前线程数
                                    // Output current thread count
                i, // 当前监控次数
                   // Current monitoring iteration
                seconds * 10 - 1 // 总监控次数
                                 // Total monitoring iterations
            );
        } // _guard在此处自动释放
          // _guard automatically releases here
        
        // 休眠100毫秒，控制监控频率
        // Sleep for 100 milliseconds, controlling monitoring frequency
        thread::sleep(Duration::from_millis(100)); 
    }
    
    // 监控结束提示
    // Monitoring end notification
    {
        let _guard = get_cout_mutex().lock().expect("Failed to lock console output mutex");
        // 输出监控结束和最终线程数
        // Output monitoring end and final thread count
        println!("监控结束，最终线程数: {}", pool.threads_num()); 
    }
}

fn main() {
    // 输出程序开始信息
    // Output program start information
    println!("=== 线程池测试程序开始 ===");
    println!("=== Thread Pool Test Program Started ===");
    
    // 创建线程池，使用默认的CPU核心数作为最大线程数
    // Create thread pool using default CPU core count as maximum thread count
    // 创建线程池并使用Arc进行引用计数，允许多线程共享
    // Create thread pool and use Arc for reference counting, allowing multi-thread sharing
    let pool = Arc::new(ThreadPool::new(None)); 
    
    // 输出系统信息
    // Output system information
    // 使用线程池的方法获取最大线程数
    // Use thread pool method to get maximum thread count
    println!("CPU核心数: {}", pool.get_max_threads()); 
    println!("CPU cores: {}", pool.get_max_threads()); 
    
    // 输出线程池创建完成信息
    // Output thread pool creation completion information
    println!("线程池创建完成，最大线程数: {}", pool.get_max_threads()); 
    println!("Thread pool created, max threads: {}", pool.get_max_threads()); 

    // 启动监控线程，监控20秒的线程池状态
    // Start monitoring thread to monitor thread pool status for 20 seconds
    // 克隆线程池的Arc，用于在监控线程中使用
    // Clone Arc of thread pool for use in monitoring thread
    let pool_clone = Arc::clone(&pool); 
    
    // 提交监控任务到线程池
    // Submit monitoring task to thread pool
    pool.submit(move || monitor(pool_clone, 20)); 
    
    // 输出监控任务提交确认
    // Output monitoring task submission confirmation
    println!("监控任务已提交"); 
    println!("Monitoring task submitted"); 

    // 提交100个工作任务
    // Submit 100 work tasks
    // 总任务数为100
    // Total task count is 100
    let total_tasks = 100; 
    
    // 输出任务提交开始信息
    // Output task submission start information
    println!("开始提交 {} 个任务...", total_tasks); 
    println!("Starting to submit {} tasks...", total_tasks); 
    
    // 循环提交任务，任务ID从0到99
    // Loop to submit tasks, task IDs from 0 to 99
    for task_id in 0..total_tasks { 
        // 每隔200毫秒提交一个任务，模拟任务逐渐到达的场景
        // Submit a task every 200 milliseconds, simulating gradually arriving tasks scenario
        // 每次提交任务后休眠200毫秒，控制任务提交速率
        // Sleep for 200 milliseconds after each task submission, controlling task submission rate
        thread::sleep(Duration::from_millis(200)); 
        
        // 克隆线程池的Arc，用于在任务闭包中使用
        // Clone Arc of thread pool for use in task closure
        let pool_clone = Arc::clone(&pool); 
        
        // 提交工作任务到线程池
        // Submit work task to thread pool
        pool_clone.submit(move || task(task_id)); 
        
        // 每提交10个任务输出一次进度
        // Output progress every 10 submitted tasks
        if (task_id + 1) % 10 == 0 {
            // 输出任务提交进度
            // Output task submission progress
            println!("已提交 {}/{} 个任务", task_id + 1, total_tasks); 
            println!("Submitted {}/{} tasks", task_id + 1, total_tasks); 
        }
    }
    
    // 输出任务提交完成，开始等待执行
    // Output task submission completed, start waiting for execution
    println!("所有任务已提交完成，等待执行完毕..."); 
    println!("All tasks submitted, waiting for execution to complete..."); 

    // 等待所有任务完成
    // Wait for all tasks to complete
    // 记录开始等待的时间
    // Record start time of waiting
    let start_time = std::time::Instant::now(); 
    
    // 等待所有任务完成（包括监控任务和工作任务）
    // Wait for all tasks to complete (including monitoring and work tasks)
    pool.wait_for_completion(); 
    
    // 计算等待时间
    // Calculate waiting time
    let elapsed = start_time.elapsed(); 
    
    // 输出完成信息
    // Output completion information
    println!("=== 所有任务执行完成 ===");
    println!("=== All Tasks Execution Completed ===");
    
    // 输出总执行时间
    // Output total execution time
    println!("总耗时: {:?}", elapsed); 
    println!("Total time: {:?}", elapsed); 
    
    // 输出最终线程数
    // Output final thread count
    println!("最终线程数: {}", pool.threads_num()); 
    println!("Final thread count: {}", pool.threads_num()); 
    
    // 输出程序退出提示和资源清理说明
    // Output program exit notification and resource cleanup explanation
    println!("程序即将退出，线程池将自动清理资源..."); 
    println!("Program about to exit, thread pool will automatically clean up resources..."); 
} // pool在此处被drop，触发线程池的优雅关闭
  // pool is dropped here, triggering graceful shutdown of thread pool

