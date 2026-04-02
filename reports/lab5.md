# Lab5 实验报告

## 一、功能实现总结

本实验实现了进程内的死锁检测机制。主要包括：

1. **Mutex 死锁检测**：
   - 使用全局的 `MUTEX_HOLDERS` BTreeMap 跟踪每个 mutex 的持有者线程 ID
   - 检测线程重入锁情况：当线程尝试锁定自己已持有的 mutex 时返回 `-0xDEAD` 错误码
   - **重要**：只对 blocking mutex 进行死锁检测和持有者跟踪，spin mutex 不进行检测
   - 在 `sys_mutex_lock` 中调用 `check_mutex_deadlock` 检测死锁
   - 在锁定成功后调用 `add_mutex_holder` 记录持有者
   - 在 `sys_mutex_unlock` 中调用 `remove_mutex_holder` 清除记录

2. **Semaphore 死锁检测**：
   - 使用 Banker's 算法（银行家算法）检测死锁，而非资源分配图算法
   - 在 `TaskControlBlockInner` 中添加 `sem_allocation: Vec<isize>` 字段跟踪每个线程的资源分配情况
   - 在 `sys_semaphore_create` 时初始化所有线程的 allocation 向量
   - 在 `sys_semaphore_down` 时：
     - 首先扩展当前线程的 sem_allocation 到正确的长度
     - 调用 `check_semaphore_deadlock` 进行死锁检测
     - 更新 allocation 记录后执行 down 操作
   - 在 `sys_semaphore_up` 时更新 allocation 记录并执行 up 操作

3. **Banker's 算法实现**：
   - 构建 Available 向量：当前可用的各类资源数量
   - 构建 Allocation 矩阵：每个线程已分配的资源数量
   - 模拟当前线程请求资源后的状态
   - 通过安全性检查算法判断是否存在安全序列
   - 如果不存在安全序列则返回 `-0xDEAD`

4. **系统调用**：
   - 实现 `sys_enable_deadlock_detect` 系统调用，允许进程动态启用/禁用死锁检测功能
   - 使用全局的 `DEADLOCK_ENABLE` 标志控制检测开关

5. **Mutex trait 扩展**：
   - 为 `Mutex` trait 添加 `as_any()` 方法支持类型转换
   - 为 `MutexBlocking` 添加 `inner_exclusive_access()` 方法公开内部状态访问
   - 用于区分 blocking mutex 和 spin mutex，实现差异化处理

当检测到死锁时，`mutex_lock` 和 `semaphore_down` 系统调用返回 `-0xDEAD`（十进制 -57005）错误码，拒绝资源分配请求。

完成本次实验用时约 8 小时。

## 二、关键代码修改

### 1. 死锁检测模块 (`os/src/sync/deadlock.rs`)

```rust
lazy_static! {
    static ref DEADLOCK_ENABLE: UPSafeCell<bool> = unsafe { UPSafeCell::new(false) };
    static ref MUTEX_HOLDERS: UPSafeCell<BTreeMap<usize, usize>> =
        unsafe { UPSafeCell::new(BTreeMap::new()) };
}

pub fn check_mutex_deadlock(mutex_id: usize, mutex_locked: bool) -> bool {
    if !is_deadlock_enabled() {
        return false;
    }
    let task = current_task().unwrap();
    let tid = task.inner_exclusive_access().res.as_ref().unwrap().tid;
    let holders = MUTEX_HOLDERS.exclusive_access();
    if let Some(&holder_tid) = holders.get(&mutex_id) {
        if holder_tid == tid {
            return true;  // 当前线程已持有该 mutex
        }
    }
    false
}

pub fn check_semaphore_deadlock(sem_id: usize) -> bool {
    // Banker's 算法实现
    // 构建 Available 和 Allocation
    // 进行安全性检查
    // 返回是否存在死锁
}
```

### 2. 任务控制块扩展 (`os/src/task/task.rs`)

```rust
pub struct TaskControlBlockInner {
    pub res: Option<TaskUserRes>,
    pub trap_cx_ppn: PhysPageNum,
    pub task_cx: TaskContext,
    pub task_status: TaskStatus,
    pub exit_code: Option<i32>,
    pub sem_allocation: Vec<isize>,  // 新增：信号量资源分配记录
}
```

### 3. 系统调用集成 (`os/src/syscall/sync.rs`)

```rust
pub fn sys_mutex_lock(mutex_id: usize) -> isize {
    let process = current_process();
    let process_inner = process.inner_exclusive_access();
    let mutex = Arc::clone(process_inner.mutex_list[mutex_id].as_ref().unwrap());
    
    let is_blocking = mutex.as_any().downcast_ref::<MutexBlocking>().is_some();
    
    if is_blocking {
        // 只对 blocking mutex 进行死锁检测
        if check_mutex_deadlock(mutex_id, mutex_locked) {
            return DEADLOCK_ERR;
        }
    }
    
    mutex.lock();
    
    if is_blocking {
        add_mutex_holder(mutex_id);
    }
    
    0
}

pub fn sys_semaphore_down(sem_id: usize) -> isize {
    // 扩展 sem_allocation
    // 检测死锁
    // 更新 allocation
    // 执行 down
}
```

### 4. Mutex trait 扩展 (`os/src/sync/mutex.rs`)

```rust
pub trait Mutex: Sync + Send {
    fn lock(&self);
    fn unlock(&self);
    fn as_any(&self) -> &dyn Any;  // 新增：用于类型转换
}

impl MutexBlocking {
    pub fn inner_exclusive_access(&self) -> core::cell::RefMut<'_, MutexBlockingInner> {
        self.inner.exclusive_access()
    }
}
```

## 三、问答题

### 问题1：主线程退出时的资源回收

**需要回收的资源有哪些？**

当主线程（0号线程）退出时，需要回收以下资源：

1. **线程控制块（TaskControlBlock）**：包括线程的内核栈、用户栈、trap上下文等
2. **内存资源**：线程的用户栈空间、堆内存、代码段、数据段等
3. **文件描述符**：该进程打开的所有文件
4. **同步原语资源**：
   - Mutex：已创建的所有 mutex
   - Semaphore：已创建的所有 semaphore 及其等待队列
   - Condvar：已创建的所有条件变量
5. **进程控制块（ProcessControlBlock）**：包括页表、内存集等
6. **PID 资源**：回收进程 ID

**其他线程的 TaskControlBlock 可能在哪些位置被引用，分别是否需要回收，为什么？**

其他线程的 TaskControlBlock 可能被引用的位置：

1. **进程的任务列表（process_inner.tasks）**：
   - 需要回收
   - 这是 TaskControlBlock 的主要存储位置，进程退出时必须清理

2. **调度器的就绪队列（Processor::ready_queue）**：
   - 需要回收
   - 线程可能在就绪队列中等待调度，进程退出时需要从队列中移除

3. **同步原语的等待队列**：
   - Mutex 的 wait_queue：需要回收，线程可能在等待获取锁
   - Semaphore 的 wait_queue：需要回收，线程可能在等待信号量
   - Condvar 的 wait_queue：需要回收，线程可能在等待条件变量
   - 如果不回收，这些线程将永远无法被唤醒，造成资源泄漏

4. **定时器队列（Timer）**：
   - 如果线程调用了 sleep，其 TaskControlBlock 会在定时器队列中
   - 需要回收，否则定时器到期时会尝试唤醒已不存在的线程

5. **当前运行线程指针（Processor::current）**：
   - 如果某个线程正在运行，其 TaskControlBlock 会被 current 引用
   - 需要处理，通常通过强制切换到其他进程来解决

**回收原因**：这些引用如果不清理，会导致：
- 内存泄漏：TaskControlBlock 及其关联资源无法释放
- 悬空指针：其他模块可能访问已释放的资源
- 系统崩溃：调度器或同步原语操作无效的 TaskControlBlock

### 问题2：两种 Mutex 实现的区别

**两种实现的区别：**

**Mutex1 的特点：**
- `lock()` 中使用 `loop` 循环
- 当 `locked == false` 时，设置 `locked = true` 后直接 `break` 退出循环
- `unlock()` 中总是先设置 `locked = false`，然后再唤醒等待线程
- 唤醒的线程需要重新竞争锁（因为 `locked` 已经是 `false`）

**Mutex2 的特点：**
- `lock()` 中没有 `loop` 循环
- 当 `locked == false` 时，设置 `locked = true` 后直接返回
- `unlock()` 中优先唤醒等待线程，只有在等待队列为空时才设置 `locked = false`
- 唤醒的线程直接获得锁（因为 `locked` 仍然是 `true`，锁被传递给唤醒的线程）

**主要区别：**

1. **锁的传递方式不同**：
   - Mutex1：释放锁时先解锁，唤醒的线程需要重新竞争
   - Mutex2：释放锁时直接传递锁给等待线程，不需要重新竞争

2. **lock() 的结构不同**：
   - Mutex1 有循环，可能在 `locked == false` 时已经持有锁但还在循环中
   - Mutex2 没有循环，`locked == false` 时直接获得锁并返回

**可能导致的问题：**

**Mutex1 的问题：**

1. **竞态条件**：
   - 当 `unlock()` 设置 `locked = false` 后，唤醒的线程和可能新到达的线程都会尝试获取锁
   - 这会导致唤醒的线程可能无法立即获得锁，违背了"先来先服务"的公平性

2. **性能问题**：
   - 唤醒的线程可能需要多次循环才能获得锁，增加了上下文切换的开销

3. **饿死问题**：
   - 如果有新线程持续到来，唤醒的线程可能永远无法获得锁

**Mutex2 的问题：**

1. **潜在的死锁风险**：
   - 如果 `unlock()` 中 `add_task(waking_task)` 失败（虽然不太可能），锁会一直保持 `locked = true` 状态
   - 没有设置 `locked = false`，后续的线程无法获得锁

2. **锁状态不一致**：
   - 当 `lock()` 中 `locked == false` 时没有 `return` 语句（代码不完整）
   - 可能导致获得锁后继续执行下面的代码，造成逻辑错误

3. **优先级反转**：
   - 直接传递锁可能导致高优先级线程被低优先级线程阻塞更长时间

**建议的正确实现：**

Mutex1 的方式更加安全和标准，但应该在唤醒线程后立即调度，让唤醒的线程有机会先获得锁。Mutex2 的直接传递方式虽然有性能优势，但需要更谨慎地处理边界情况，确保锁状态的一致性。

推荐使用类似以下的方式（Mutex1 的改进版）：
- 在 `unlock()` 中，先唤醒线程，再解锁，减少竞态窗口
- 或者使用原子操作确保状态转换的原子性

---

## 荣誉准则

在完成本次实验的过程（含此前学习的过程）中，我曾分别与 以下各位 就（与本次实验相关的）以下方面做过交流，还在代码中对应的位置以注释形式记录了具体的交流对象及内容：

    gpt
    真有人能从零基础二月知道训练营三月开练，四月通关rcore的天才么
    我太废物了，我曾三度入门rust，四次报名训练营，呜呼哀哉

此外，我也参考了 以下资料 ，还在代码中对应的位置以注释形式记录了具体的参考来源及内容：

    rCore-Tutorial-Book v3
    rCore-Tutorial-Code
    RISC-V Privileged Architecture

3. 我独立完成了本次实验除以上方面之外的所有工作，包括代码与文档。我清楚地知道，从以上方面获得的信息在一定程度上降低了实验难度，可能会影响起评分。

4. 我从未使用过他人的代码，不管是原封不动地复制，还是经过了某些等价转换。我未曾也不会向他人（含此后各届同学）复制或公开我的实验代码，我有义务妥善保管好它们。我提交至本实验的评测系统的代码，均无意于破坏或妨碍任何计算机系统的正常运转。我清楚地知道，以上情况均为本课程纪律所禁止，若违反，对应的实验成绩将按"-100"分计。

---