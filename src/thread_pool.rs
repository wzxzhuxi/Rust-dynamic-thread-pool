// 导入 VecDeque（双端队列，标准库实现）
// Import VecDeque (double-ended queue, standard library implementation)
use std::collections::VecDeque; 

// 导入 HashMap（哈希表）
// Import HashMap (hash table)
use std::collections::HashMap; 

// 导入 Arc（原子引用计数）、Mutex（互斥锁）和 Condvar（条件变量）
// Import Arc (atomic reference counting), Mutex (mutual exclusion lock), and Condvar (condition variable)
use std::sync::{Arc, Mutex, Condvar}; 

// 导入线程模块和 JoinHandle（用于线程句柄）
// Import thread module and JoinHandle (for thread handles)
use std::thread::{self, JoinHandle}; 

// 导入 AtomicBool（原子布尔）、AtomicUsize（原子无符号整数）和 Ordering（内存排序操作）
// Import AtomicBool (atomic boolean), AtomicUsize (atomic unsigned integer), and Ordering (memory ordering operations)
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering}; 

// 导入 Duration（时间段）
// Import Duration (time duration)
use std::time::Duration; 

// Atomic 相关概念
// Atomic concepts explanation
// Atomic 类型允许在多个线程之间安全地共享和修改数据，避免数据竞争
// Atomic types allow safe sharing and modification of data between multiple threads, preventing data races
// AtomicBool: 一个布尔值类型的原子变量，适用于需要在线程之间共享和修改布尔值的场景
// AtomicBool: An atomic variable of boolean type, suitable for scenarios where boolean values need to be shared and modified between threads
// AtomicUsize: 一个无符号整数类型的原子变量，适用于需要在线程之间共享和修改计数器的场景
// AtomicUsize: An atomic variable of unsigned integer type, suitable for scenarios where counters need to be shared and modified between threads
// Ordering: 用于控制原子操作的内存可见性和顺序，包括 SeqCst（顺序一致性）、Acquire（获取操作）、Release（释放操作）等
// Ordering: Used to control memory visibility and order of atomic operations, including SeqCst (sequential consistency), Acquire (acquire operation), Release (release operation), etc.

// 定义任务类型为无返回值的闭包，必须是线程安全和可发送的
// Define task type as a closure with no return value, must be thread-safe and sendable
type Task = Box<dyn FnOnce() + Send + 'static>; 

// 获取CPU核心数的函数，使用标准库替代num_cpus
// Function to get CPU core count, using standard library to replace num_cpus
fn get_cpu_count() -> usize {
    // 使用标准库的available_parallelism()获取可用并行度
    // Use standard library's available_parallelism() to get available parallelism
    // 如果获取失败，默认使用4个线程
    // If retrieval fails, default to 4 threads
    std::thread::available_parallelism()
        .map(|n| n.get()) // 获取NonZeroUsize的值
                          // Get the value of NonZeroUsize
        .unwrap_or(4) // 如果获取失败，默认使用4个线程
                      // If retrieval fails, default to 4 threads
}

// 线程池结构体定义
// Thread pool struct definition
pub struct ThreadPool {
    // 任务队列，使用 Arc 包装的 Mutex 和 Condvar，改用VecDeque提高性能
    // Task queue, using Arc-wrapped Mutex and Condvar, changed to VecDeque for better performance
    tasks: Arc<(Mutex<VecDeque<Task>>, Condvar)>, 
    
    // 线程集合，使用 Arc 包装的 Mutex 和 HashMap
    // Thread collection, using Arc-wrapped Mutex and HashMap
    threads: Arc<Mutex<HashMap<usize, JoinHandle<()>>>>, 
    
    // 最大线程数
    // Maximum number of threads
    max_threads: usize, 
    
    // 下一个线程 ID，原子无符号整数
    // Next thread ID, atomic unsigned integer
    next_thread_id: AtomicUsize, 
    
    // 退出标志，原子布尔
    // Exit flag, atomic boolean
    quit: Arc<AtomicBool>, 
    
    // 当前线程数，原子无符号整数
    // Current thread count, atomic unsigned integer
    current_threads: Arc<AtomicUsize>, 
    
    // 空闲线程数，原子无符号整数
    // Idle thread count, atomic unsigned integer
    idle_threads: Arc<AtomicUsize>, 
    
    // 活跃任务数，原子无符号整数
    // Active task count, atomic unsigned integer
    active_tasks: Arc<AtomicUsize>, 
    
    // 已提交任务数，原子无符号整数（重命名，更准确地表示含义）
    // Submitted task count, atomic unsigned integer (renamed for more accurate representation)
    submitted_tasks: Arc<AtomicUsize>, 
    
    // 已完成任务数，改为原子类型避免锁竞争，提高性能
    // Completed task count, changed to atomic type to avoid lock contention and improve performance
    completed_tasks: Arc<AtomicUsize>, 
}

impl ThreadPool {
    // 创建新的线程池实例
    // Create a new thread pool instance
    pub fn new(max_threads: Option<usize>) -> Self {
        let thread_count = match max_threads {
            // 如果提供了参数，使用指定的线程数
            // If parameter is provided, use specified thread count
            Some(threads) => {
                // 验证线程数必须大于0
                // Validate that thread count must be greater than 0
                assert!(threads > 0, "Thread pool size must be greater than 0");
                threads
            }
            // 如果没有提供参数，使用CPU核心数
            // If no parameter provided, use CPU core count
            None => get_cpu_count()
        };
        
        ThreadPool::with_max_threads(thread_count)
    }

    // 使用指定最大线程数创建线程池
    // Create thread pool with specified maximum thread count
    pub fn with_max_threads(max_threads: usize) -> Self {
        ThreadPool {
            // 初始化任务队列，使用VecDeque替代自定义LinkedList
            // Initialize task queue, using VecDeque instead of custom LinkedList
            tasks: Arc::new((Mutex::new(VecDeque::new()), Condvar::new())), 
            
            // 初始化线程集合
            // Initialize thread collection
            threads: Arc::new(Mutex::new(HashMap::new())), 
            
            // 设置最大线程数
            // Set maximum thread count
            max_threads, 
            
            // 初始化下一个线程 ID
            // Initialize next thread ID
            next_thread_id: AtomicUsize::new(0), 
            
            // 初始化退出标志
            // Initialize exit flag
            quit: Arc::new(AtomicBool::new(false)), 
            
            // 初始化当前线程数
            // Initialize current thread count
            current_threads: Arc::new(AtomicUsize::new(0)), 
            
            // 初始化空闲线程数
            // Initialize idle thread count
            idle_threads: Arc::new(AtomicUsize::new(0)), 
            
            // 初始化活跃任务数
            // Initialize active task count
            active_tasks: Arc::new(AtomicUsize::new(0)), 
            
            // 初始化已提交任务数
            // Initialize submitted task count
            submitted_tasks: Arc::new(AtomicUsize::new(0)), 
            
            // 初始化已完成任务数，改为原子类型
            // Initialize completed task count, changed to atomic type
            completed_tasks: Arc::new(AtomicUsize::new(0)), 
        }
    }

    // 原子化的线程创建函数，解决竞态条件问题
    // Atomic thread creation function to solve race condition problems
    fn try_spawn_thread(&self) -> bool {
        // 无限循环，直到成功创建线程或确定不需要创建
        // Infinite loop until successfully creating a thread or determining no need to create one
        loop { 
            // 获取当前线程数
            // Get current thread count
            let current = self.current_threads.load(Ordering::SeqCst); 
            
            // 如果当前线程数已达到最大值
            // If current thread count has reached maximum
            if current >= self.max_threads { 
                // 返回false，表示不需要创建新线程
                // Return false, indicating no need to create new thread
                return false; 
            }
            
            // 使用原子的compare_exchange_weak操作，确保检查-创建的原子性
            // Use atomic compare_exchange_weak operation to ensure atomicity of check-create operation
            // compare_exchange_weak: 比较当前值与期望值，如果相等则更新为新值
            // compare_exchange_weak: Compare current value with expected value, update to new value if equal
            // 参数: 期望值、新值、成功时的内存序、失败时的内存序
            // Parameters: expected value, new value, memory order on success, memory order on failure
            match self.current_threads.compare_exchange_weak(
                current, // 期望的当前值
                         // Expected current value
                current + 1, // 要设置的新值（当前值+1）
                             // New value to set (current value + 1)
                Ordering::SeqCst, // 成功时的内存排序（顺序一致性）
                                  // Memory ordering on success (sequential consistency)
                Ordering::Relaxed, // 失败时的内存排序（宽松排序）
                                   // Memory ordering on failure (relaxed ordering)
            ) {
                // 如果CAS操作成功
                // If CAS operation succeeds
                Ok(_) => { 
                    // 创建新线程
                    // Create new thread
                    self.spawn_thread(); 
                    
                    // 返回true，表示成功创建线程
                    // Return true, indicating successful thread creation
                    return true; 
                }
                // 如果CAS操作失败，重试循环
                // If CAS operation fails, retry the loop
                Err(_) => continue, 
            }
        }
    }

    // 提交新任务到线程池
    // Submit new task to thread pool
    pub fn submit<F>(&self, task: F)
    where
        F: FnOnce() + Send + 'static, // 任务为无返回值闭包，线程安全且可发送
                                       // Task is a closure with no return value, thread-safe and sendable
    {
        // 将任务封装为 Box
        // Wrap task in Box
        let task = Box::new(task); 
        
        // 减少锁持有时间，优化性能
        // Reduce lock holding time to optimize performance
        {
            // 获取任务队列的锁，使用expect提供更好的错误信息
            // Get task queue lock, use expect for better error information
            let mut tasks = self.tasks.0.lock().expect("Failed to lock tasks mutex"); 
            
            // 将任务加入队列尾部
            // Add task to the end of queue
            tasks.push_back(task); 
        } // 作用域结束，自动释放锁
          // Scope ends, automatically release lock
        
        // 原子性地增加已提交任务数
        // Atomically increment submitted task count
        self.submitted_tasks.fetch_add(1, Ordering::SeqCst); 
        
        // 通知一个等待的线程
        // Notify one waiting thread
        self.tasks.1.notify_one(); 
        // notify_one 方法用于通知一个等待在该条件变量上的线程，使其从等待状态唤醒
        // notify_one method is used to notify one thread waiting on this condition variable, waking it from waiting state

        // 只在需要时尝试创建线程，避免不必要的检查
        // Only try to create thread when needed, avoid unnecessary checks
        // 使用Relaxed排序提高性能，因为这里只是一个提示性检查
        // Use Relaxed ordering for better performance, as this is just a hint check
        if self.idle_threads.load(Ordering::Relaxed) == 0 { // 如果没有空闲线程
                                                             // If no idle threads
            // 尝试创建新线程（原子化操作）
            // Try to create new thread (atomic operation)
            self.try_spawn_thread(); 
        }
    }

    // 创建新线程的具体实现
    // Specific implementation of creating new thread
    fn spawn_thread(&self) {
        // 获取并原子性地增加下一个线程 ID
        // Get and atomically increment next thread ID
        let thread_id = self.next_thread_id.fetch_add(1, Ordering::SeqCst); 
        
        // 克隆任务队列的 Arc，用于在线程间共享
        // Clone Arc of task queue for sharing between threads
        let tasks = Arc::clone(&self.tasks); 
        
        // 克隆退出标志的 Arc
        // Clone Arc of exit flag
        let quit = Arc::clone(&self.quit); 
        
        // 克隆当前线程数的 Arc
        // Clone Arc of current thread count
        let current_threads = Arc::clone(&self.current_threads); 
        
        // 克隆空闲线程数的 Arc
        // Clone Arc of idle thread count
        let idle_threads = Arc::clone(&self.idle_threads); 
        
        // 克隆活跃任务数的 Arc
        // Clone Arc of active task count
        let active_tasks = Arc::clone(&self.active_tasks); 
        
        // 克隆已完成任务数的 Arc
        // Clone Arc of completed task count
        let completed_tasks = Arc::clone(&self.completed_tasks); 
        
        // 克隆线程集合的 Arc，用于线程自清理
        // Clone Arc of thread collection for thread self-cleanup
        let threads = Arc::clone(&self.threads); 

        // 创建并启动新线程
        // Create and start new thread
        let handle = thread::spawn(move || { // move 关键字将所有克隆的Arc移动到线程闭包中
                                             // move keyword moves all cloned Arcs into thread closure
            // 线程的主循环
            // Main loop of thread
            loop { 
                // 声明任务变量
                // Declare task variable
                let task: Task; 
                {
                    // 解引用Arc，获取任务队列的锁和条件变量
                    // Dereference Arc to get lock and condition variable of task queue
                    let (lock, cvar) = &*tasks; 
                    
                    // 加锁获取任务队列的可变引用
                    // Lock to get mutable reference of task queue
                    let mut task_queue = lock.lock().expect("Failed to lock tasks mutex"); 
                    
                    // 原子性地增加空闲线程数
                    // Atomically increment idle thread count
                    idle_threads.fetch_add(1, Ordering::SeqCst); 
                    
                    // 当任务队列为空且未设置退出标志时，线程等待
                    // Wait when task queue is empty and exit flag is not set
                    while task_queue.is_empty() && !quit.load(Ordering::SeqCst) {
                        // wait_timeout: 在条件变量上等待，最多等待指定时间
                        // wait_timeout: Wait on condition variable for at most specified time
                        // 参数: MutexGuard、超时时长
                        // Parameters: MutexGuard, timeout duration
                        // 返回: (MutexGuard, WaitTimeoutResult)
                        // Returns: (MutexGuard, WaitTimeoutResult)
                        let result = cvar
                            .wait_timeout(task_queue, Duration::from_secs(10)) // 等待条件变量，最多等待10秒
                                                                               // Wait on condition variable for at most 10 seconds
                            .expect("Condvar wait failed"); // 如果等待失败则panic，提供错误信息
                                                             // If wait fails, panic with error message
                        
                        // 更新任务队列的MutexGuard
                        // Update MutexGuard of task queue
                        task_queue = result.0; 
                        
                        // 如果等待超时
                        // If wait times out
                        if result.1.timed_out() { 
                            // 原子性地减少空闲线程数
                            // Atomically decrement idle thread count
                            idle_threads.fetch_sub(1, Ordering::SeqCst); 
                            
                            // 原子性地减少当前线程数
                            // Atomically decrement current thread count
                            current_threads.fetch_sub(1, Ordering::SeqCst); 
                            
                            // 清理线程句柄，避免资源泄漏
                            // Clean up thread handle to avoid resource leak
                            // try_lock: 尝试获取锁，如果失败不阻塞
                            // try_lock: Try to acquire lock, don't block if failed
                            if let Ok(mut threads) = threads.try_lock() {
                                // 从HashMap中移除当前线程的句柄
                                // Remove current thread's handle from HashMap
                                threads.remove(&thread_id); 
                            }
                            // 退出线程
                            // Exit thread
                            return; 
                        }
                    }
                    
                    // 原子性地减少空闲线程数（线程即将执行任务）
                    // Atomically decrement idle thread count (thread is about to execute task)
                    idle_threads.fetch_sub(1, Ordering::SeqCst); 

                    // 如果设置了退出标志且任务队列为空，则退出线程
                    // Exit thread if exit flag is set and task queue is empty
                    if quit.load(Ordering::SeqCst) && task_queue.is_empty() {
                        // 原子性地减少当前线程数
                        // Atomically decrement current thread count
                        current_threads.fetch_sub(1, Ordering::SeqCst); 
                        
                        // 清理线程句柄，避免资源泄漏
                        // Clean up thread handle to avoid resource leak
                        if let Ok(mut threads) = threads.try_lock() {
                            // 从HashMap中移除当前线程的句柄
                            // Remove current thread's handle from HashMap
                            threads.remove(&thread_id); 
                        }
                        // 退出线程
                        // Exit thread
                        return; 
                    }

                    // 从队列前端获取任务
                    // Get task from front of queue
                    task = task_queue.pop_front().expect("Task queue empty when expected task");
                } // 锁的作用域结束，自动释放锁
                  // Lock scope ends, automatically release lock
                
                // 原子性地增加活跃任务数
                // Atomically increment active task count
                active_tasks.fetch_add(1, Ordering::SeqCst); 
                
                // 执行任务
                // Execute task
                task(); 
                
                // 原子性地减少活跃任务数
                // Atomically decrement active task count
                active_tasks.fetch_sub(1, Ordering::SeqCst); 
                
                // 原子性地增加已完成任务数
                // Atomically increment completed task count
                completed_tasks.fetch_add(1, Ordering::SeqCst); 
            }
        });

        // 将新线程的句柄插入线程集合
        // Insert new thread's handle into thread collection
        self.threads
            .lock()
            .expect("Failed to lock threads mutex") // 获取线程集合的锁
                                                     // Get lock of thread collection
            .insert(thread_id, handle); // 插入线程ID和对应的JoinHandle
                                        // Insert thread ID and corresponding JoinHandle
    }

    // 返回当前线程数
    // Return current thread count
    pub fn threads_num(&self) -> usize {
        // 原子性地加载当前线程数
        // Atomically load current thread count
        self.current_threads.load(Ordering::SeqCst) 
    }

    // 等待所有任务完成（修复了逻辑缺陷）
    // Wait for all tasks to complete (fixed logic flaws)
    pub fn wait_for_completion(&self) {
        // 快照机制：获取当前已提交的任务数作为目标，避免不一致性问题
        // Snapshot mechanism: Get current submitted task count as target to avoid inconsistency issues
        let target = self.submitted_tasks.load(Ordering::SeqCst); 
        
        // 使用自旋等待，避免条件变量的复杂性，适合短期使用场景
        // Use spin waiting to avoid complexity of condition variables, suitable for short-term usage scenarios
        while self.completed_tasks.load(Ordering::SeqCst) < target { // 当已完成任务数小于目标数时循环
                                                                       // Loop while completed task count is less than target
            // 主动让出线程执行权，给其他线程运行机会
            // Voluntarily yield thread execution, giving other threads a chance to run
            std::thread::yield_now(); 
            // yield_now 方法用于主动让出当前线程的执行权，使其他线程有机会运行
            // yield_now method is used to voluntarily yield current thread's execution, allowing other threads to run
        }
    }

    // 获取最大线程数（新增方法，用于外部查询）
    // Get maximum thread count (new method for external queries)
    pub fn get_max_threads(&self) -> usize {
        // 返回最大线程数
        // Return maximum thread count
        self.max_threads 
    }
}

// 线程池的析构函数，确保在销毁前完成所有任务
// Destructor for thread pool, ensures all tasks are completed before destruction
impl Drop for ThreadPool {
    fn drop(&mut self) {
        // 原子性地设置退出标志为true
        // Atomically set exit flag to true
        self.quit.store(true, Ordering::SeqCst); 
        
        // 通知所有等待在条件变量上的线程
        // Notify all threads waiting on condition variable
        self.tasks.1.notify_all(); 
        // notify_all 方法用于通知所有等待在该条件变量上的线程，使它们从等待状态唤醒
        // notify_all method is used to notify all threads waiting on this condition variable, waking them from waiting state

        // 获取线程集合的锁
        // Get lock of thread collection
        let mut threads = self.threads.lock().expect("Failed to lock threads mutex"); 
        
        // drain(): 移除HashMap中的所有元素并返回迭代器
        // drain(): Remove all elements from HashMap and return iterator
        // map(): 将每个(key, value)对映射为value（即JoinHandle）
        // map(): Map each (key, value) pair to value (i.e., JoinHandle)
        for handle in threads.drain().map(|(_, handle)| handle) { // 获取所有线程句柄
                                                                   // Get all thread handles
            // 等待线程结束，如果线程panic则传播panic
            // Wait for thread to finish, propagate panic if thread panics
            handle.join().expect("Thread join failed"); 
            // join 方法用于阻塞当前线程，直到被调用的线程执行结束
            // join method is used to block current thread until the called thread finishes execution
        }

        // 等待所有活跃任务完成
        // Wait for all active tasks to complete
        while self.active_tasks.load(Ordering::SeqCst) > 0 { // 当还有活跃任务时循环等待
                                                              // Loop while there are active tasks
            // 主动让出线程执行权以等待任务完成
            // Voluntarily yield thread execution to wait for task completion
            std::thread::yield_now(); 
            // yield_now 方法用于主动让出当前线程的执行权，使其他线程有机会运行
            // yield_now method is used to voluntarily yield current thread's execution, allowing other threads to run
        }
    }
}

