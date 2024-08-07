use crate::link_list::LinkedList;// 导入 LinkedList（双端队列）
use std::collections::HashMap; // 导入 HashMap（哈希表）
// use std::collections::VecDeque; // 导入 VecDeque（双端队列）
use std::sync::{Arc, Mutex, Condvar}; // 导入 Arc（原子引用计数）、Mutex（互斥锁）和 Condvar（条件变量）
use std::thread::{self, JoinHandle}; // 导入线程模块和 JoinHandle（用于线程句柄）
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering}; // 导入 AtomicBool（原子布尔）、AtomicUsize（原子无符号整数）和 Ordering（内存排序操作）
use std::time::Duration; // 导入 Duration（时间段）

// Atomic 相关概念
// Atomic 类型允许在多个线程之间安全地共享和修改数据，避免数据竞争
// AtomicBool: 一个布尔值类型的原子变量，适用于需要在线程之间共享和修改布尔值的场景
// AtomicUsize: 一个无符号整数类型的原子变量，适用于需要在线程之间共享和修改计数器的场景
// Ordering: 用于控制原子操作的内存可见性和顺序，包括 SeqCst（顺序一致性）、Acquire（获取操作）、Release（释放操作）等

// 定义任务类型为无返回值的闭包，必须是线程安全和可发送的
type Task = Box<dyn FnOnce() + Send + 'static>; 

// // 线程池结构体定义
pub struct ThreadPool {
    tasks: Arc<(Mutex<LinkedList<Task>>, Condvar)>, // 任务队列，使用 Arc 包装的 Mutex 和 Condvar
    threads: Arc<Mutex<HashMap<usize, JoinHandle<()>>>>, // 线程集合，使用 Arc 包装的 Mutex 和 HashMap
    max_threads: usize, // 最大线程数
    next_thread_id: AtomicUsize, // 下一个线程 ID，原子无符号整数
    quit: Arc<AtomicBool>, // 退出标志，原子布尔
    current_threads: Arc<AtomicUsize>, // 当前线程数，原子无符号整数
    idle_threads: Arc<AtomicUsize>, // 空闲线程数，原子无符号整数
    active_tasks: Arc<AtomicUsize>, // 活跃任务数，原子无符号整数
    total_tasks: Arc<AtomicUsize>, // 总任务数，原子无符号整数
    completed_tasks: Arc<(Mutex<usize>, Condvar)>, // 已完成任务计数，使用 Arc 包装的 Mutex 和 Condvar
}

impl ThreadPool {
    // 创建新的线程池实例
    pub fn new() -> Self {
        let max_threads = num_cpus::get(); // 获取 CPU 核心数
        ThreadPool::with_max_threads(max_threads) // 使用 CPU 核心数创建线程池
    }

    pub fn with_max_threads(max_threads: usize) -> Self {
        ThreadPool {
            tasks: Arc::new((Mutex::new(LinkedList::new()), Condvar::new())), // 初始化任务队列
            threads: Arc::new(Mutex::new(HashMap::new())), // 初始化线程集合
            max_threads, // 设置最大线程数
            next_thread_id: AtomicUsize::new(0), // 初始化下一个线程 ID
            quit: Arc::new(AtomicBool::new(false)), // 初始化退出标志
            current_threads: Arc::new(AtomicUsize::new(0)), // 初始化当前线程数
            idle_threads: Arc::new(AtomicUsize::new(0)), // 初始化空闲线程数
            active_tasks: Arc::new(AtomicUsize::new(0)), // 初始化活跃任务数
            total_tasks: Arc::new(AtomicUsize::new(0)), // 初始化总任务数
            completed_tasks: Arc::new((Mutex::new(0), Condvar::new())), // 初始化已完成任务计数
        }
    }

    // 提交新任务到线程池
    pub fn submit<F>(&self, task: F) 
    where
        F: FnOnce() + Send + 'static, // 任务为无返回值闭包，线程安全且可发送
    {
        let task = Box::new(task); // 将任务封装为 Box
        let mut tasks = self.tasks.0.lock().unwrap(); // 获取任务队列的锁
        tasks.push_back(task); // 将任务加入队列
        self.total_tasks.fetch_add(1, Ordering::SeqCst); // 增加总任务数
        self.tasks.1.notify_one(); // 通知一个等待的线程
        // notify_one 方法用于通知一个等待在该条件变量上的线程，使其从等待状态唤醒

        // 如果没有空闲线程且当前线程数小于最大线程数，则创建新线程
        if self.idle_threads.load(Ordering::SeqCst) == 0 && self.current_threads.load(Ordering::SeqCst) < self.max_threads {
            self.spawn_thread();
        }
    }

    // 创建新线程
    fn spawn_thread(&self) {
        let thread_id = self.next_thread_id.fetch_add(1, Ordering::SeqCst); // 获取并增加下一个线程 ID
        let tasks = Arc::clone(&self.tasks); // 克隆任务队列的 Arc
        let quit = Arc::clone(&self.quit); // 克隆退出标志的 Arc
        let current_threads = Arc::clone(&self.current_threads); // 克隆当前线程数的 Arc
        let idle_threads = Arc::clone(&self.idle_threads); // 克隆空闲线程数的 Arc
        let active_tasks = Arc::clone(&self.active_tasks); // 克隆活跃任务数的 Arc
        let completed_tasks = Arc::clone(&self.completed_tasks); // 克隆已完成任务计数的 Arc

        // 创建并启动新线程
        let handle = thread::spawn(move || {
            loop {
                let task: Task;
                {
                    let (lock, cvar) = &*tasks; // 获取任务队列的锁和条件变量
                    let mut tasks = lock.lock().unwrap(); // 加锁
                    idle_threads.fetch_add(1, Ordering::SeqCst); // 增加空闲线程数
                    while tasks.is_empty() && !quit.load(Ordering::SeqCst) { // 如果任务队列为空且未设置退出标志，则等待
                        let result = cvar.wait_timeout(tasks, Duration::from_secs(10)).unwrap(); // 等待条件变量，最多等待10秒
                        tasks = result.0; // 更新任务队列
                        if result.1.timed_out() { // 如果等待超时，则退出线程
                            idle_threads.fetch_sub(1, Ordering::SeqCst); // 减少空闲线程数
                            current_threads.fetch_sub(1, Ordering::SeqCst); // 减少当前线程数
                            return;
                        }
                    }
                    idle_threads.fetch_sub(1, Ordering::SeqCst); // 减少空闲线程数

                    if quit.load(Ordering::SeqCst) && tasks.is_empty() { // 如果设置了退出标志且任务队列为空，则退出循环
                        current_threads.fetch_sub(1, Ordering::SeqCst); // 减少当前线程数
                        return;
                    }

                    task = tasks.pop_front().unwrap(); // 获取并移除队列中的第一个任务
                }
                active_tasks.fetch_add(1, Ordering::SeqCst); // 增加活跃任务数
                task(); // 执行任务
                active_tasks.fetch_sub(1, Ordering::SeqCst); // 减少活跃任务数

                let (completed_tasks_lock, completed_tasks_cvar) = &*completed_tasks; // 获取已完成任务计数的锁和条件变量
                let mut completed_count = completed_tasks_lock.lock().unwrap(); // 加锁
                *completed_count += 1; // 增加已完成任务计数
                completed_tasks_cvar.notify_all(); // 通知所有等待的线程
                // notify_all 方法用于通知所有等待在该条件变量上的线程，使它们从等待状态唤醒
            }
        });

        self.threads.lock().unwrap().insert(thread_id, handle); // 将新线程的句柄插入线程集合
        self.current_threads.fetch_add(1, Ordering::SeqCst); // 增加当前线程数
    }

    // 返回当前线程数
    pub fn threads_num(&self) -> usize {
        self.current_threads.load(Ordering::SeqCst) // 加载当前线程数
    }

    // 等待所有任务完成
    pub fn wait_for_completion(&self) {
        let (completed_tasks_lock, completed_tasks_cvar) = &*self.completed_tasks; // 获取已完成任务计数的锁和条件变量
        let mut completed_count = completed_tasks_lock.lock().unwrap(); // 加锁
        loop { // 无限循环，直到所有任务完成
            let total_tasks = self.total_tasks.load(Ordering::SeqCst); // 获取总任务数
            if *completed_count >= total_tasks { // 如果已完成任务数大于等于总任务数，跳出循环
                break;
            }
            completed_count = completed_tasks_cvar.wait(completed_count).unwrap(); // 等待条件变量
        }
    }
}

// 线程池的析构函数，确保在销毁前完成所有任务
impl Drop for ThreadPool {
    fn drop(&mut self) {
        self.quit.store(true, Ordering::SeqCst); // 设置退出标志
        self.tasks.1.notify_all(); // 通知所有等待的线程

        let mut threads = self.threads.lock().unwrap(); // 获取线程集合的锁
        for handle in threads.drain().map(|(_, handle)| handle) { // 获取所有线程句柄
            handle.thread().unpark(); // 解除线程阻塞
            // unpark 方法用于解除线程的阻塞状态，使其可以继续执行
            handle.join().unwrap(); // 等待线程结束
            // join 方法用于阻塞当前线程，直到被调用的线程执行结束
        }

        while self.active_tasks.load(Ordering::SeqCst) > 0 { // 当有活跃任务时，主动让出线程以等待完成
            std::thread::yield_now(); // 让出线程
            // yield_now 方法用于主动让出当前线程的执行权，使其他线程有机会运行
        }
    }
}
