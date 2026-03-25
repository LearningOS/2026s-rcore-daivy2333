# Chapter 5 实现计划：spawn 系统调用与 stride 调度算法

## 概述

本章需要实现两个主要功能：
1. **spawn 系统调用**：创建新进程并执行目标程序（替代 fork + exec 组合）
2. **stride 调度算法**：带优先级的进程调度算法

## 一、spawn 系统调用实现

### 1.1 系统调用定义

```rust
// syscall ID: 400
fn sys_spawn(path: *const u8) -> isize
```

- 功能：新建子进程，使其执行目标程序
- 返回值：成功返回子进程 ID，失败返回 -1
- 错误情况：无效的文件名

### 1.2 实现位置

**文件：[`os/src/syscall/process.rs`](os/src/syscall/process.rs:183-191)**

需要修改 `sys_spawn` 函数，当前是空实现：

```rust
// 当前代码（需要替换）
pub fn sys_spawn(_path: *const u8) -> isize {
    trace!(
        "kernel:pid[{}] sys_spawn NOT IMPLEMENTED",
        current_task().unwrap().pid.0
    );
    -1
}
```

### 1.3 实现步骤

**步骤 1：在 [`os/src/task/task.rs`](os/src/task/task.rs) 中添加 spawn 方法**

在 `TaskControlBlock` impl 块中添加新方法（约第 214 行之后）：

```rust
/// spawn a new process from elf_data
pub fn spawn(self: &Arc<Self>, elf_data: &[u8]) -> Arc<Self> {
    // 创建新的地址空间（不复制父进程）
    let (memory_set, user_sp, entry_point) = MemorySet::from_elf(elf_data);
    let trap_cx_ppn = memory_set
        .translate(VirtAddr::from(TRAP_CONTEXT_BASE).into())
        .unwrap()
        .ppn();
    
    // 分配 pid 和内核栈
    let pid_handle = pid_alloc();
    let kernel_stack = kstack_alloc();
    let kernel_stack_top = kernel_stack.get_top();
    
    // 创建新的 TCB
    let task_control_block = Arc::new(TaskControlBlock {
        pid: pid_handle,
        kernel_stack,
        inner: unsafe {
            UPSafeCell::new(TaskControlBlockInner {
                trap_cx_ppn,
                base_size: user_sp,
                task_cx: TaskContext::goto_trap_return(kernel_stack_top),
                task_status: TaskStatus::Ready,
                memory_set,
                parent: Some(Arc::downgrade(self)),  // 设置父进程
                children: Vec::new(),
                exit_code: 0,
                heap_bottom: user_sp,
                program_brk: user_sp,
                syscall_count: [0; MAX_SYSCALL_NUM],
                // 新增字段（stride 调度）
                priority: 16,      // 默认优先级
                stride: 0,         // 初始 stride
            })
        },
    });
    
    // 将子进程添加到父进程的 children 列表
    self.inner_exclusive_access().children.push(task_control_block.clone());
    
    // 初始化 trap context
    let trap_cx = task_control_block.inner_exclusive_access().get_trap_cx();
    *trap_cx = TrapContext::app_init_context(
        entry_point,
        user_sp,
        KERNEL_SPACE.exclusive_access().token(),
        kernel_stack_top,
        trap_handler as usize,
    );
    
    task_control_block
}
```

**步骤 2：修改 [`os/src/syscall/process.rs`](os/src/syscall/process.rs:183-191) 中的 sys_spawn**

```rust
/// spawn: create a new process and execute the program
pub fn sys_spawn(path: *const u8) -> isize {
    trace!("kernel:pid[{}] sys_spawn", current_task().unwrap().pid.0);
    let token = current_user_token();
    let path = translated_str(token, path);
    
    if let Some(data) = get_app_data_by_name(path.as_str()) {
        let current_task = current_task().unwrap();
        let new_task = current_task.spawn(data);
        let new_pid = new_task.pid.0;
        // 将新任务加入调度队列
        add_task(new_task);
        new_pid as isize
    } else {
        -1  // 无效的文件名
    }
}
```

---

## 二、stride 调度算法实现

### 2.1 算法原理

- 每个进程有一个 `stride`（当前步长）和 `pass`（步进值）
- `pass = BigStride / priority`
- 每次调度选择 `stride` 最小的进程
- 被调度的进程：`stride += pass`
- 优先级越高，pass 越小，被调度越频繁

### 2.2 常量定义

**文件：[`os/src/task/task.rs`](os/src/task/task.rs)**

在文件开头添加常量：

```rust
/// BigStride constant for stride scheduling
pub const BIG_STRIDE: usize = 100000;
/// Default priority for new processes
pub const DEFAULT_PRIORITY: usize = 16;
```

### 2.3 修改 TaskControlBlockInner

**文件：[`os/src/task/task.rs`](os/src/task/task.rs:40-75)**

在 `TaskControlBlockInner` 结构体中添加两个字段：

```rust
pub struct TaskControlBlockInner {
    // ... 现有字段 ...
    
    /// Process priority for stride scheduling (>= 2)
    pub priority: usize,
    
    /// Current stride for stride scheduling
    pub stride: usize,
}
```

### 2.4 修改所有创建 TaskControlBlockInner 的地方

需要修改以下位置：

1. **[`os/src/task/task.rs`](os/src/task/task.rs:98-139) - `TaskControlBlock::new` 方法**

在 `TaskControlBlockInner` 初始化中添加：
```rust
priority: DEFAULT_PRIORITY,  // 或 16
stride: 0,
```

2. **[`os/src/task/task.rs`](os/src/task/task.rs:170-213) - `TaskControlBlock::fork` 方法**

在 `TaskControlBlockInner` 初始化中添加：
```rust
priority: parent_inner.priority,  // 继承父进程优先级
stride: 0,  // 新进程 stride 从 0 开始
```

### 2.5 实现 sys_set_priority 系统调用

**文件：[`os/src/syscall/process.rs`](os/src/syscall/process.rs:193-200)**

```rust
/// Set task priority for stride scheduling
/// syscall ID: 140
/// Returns prio if successful, -1 if invalid (prio < 2)
pub fn sys_set_priority(prio: isize) -> isize {
    trace!("kernel:pid[{}] sys_set_priority({})", current_task().unwrap().pid.0, prio);
    if prio < 2 {
        return -1;
    }
    let task = current_task().unwrap();
    let mut inner = task.inner_exclusive_access();
    inner.priority = prio as usize;
    prio
}
```

### 2.6 修改调度器实现 stride 算法

**文件：[`os/src/task/manager.rs`](os/src/task/manager.rs)**

需要修改 `TaskManager` 的 `fetch` 方法，从 FIFO 改为 stride 调度：

```rust
impl TaskManager {
    // ... add 方法保持不变 ...
    
    /// Take the process with minimum stride (stride scheduling)
    pub fn fetch(&mut self) -> Option<Arc<TaskControlBlock>> {
        if self.ready_queue.is_empty() {
            return None;
        }
        
        // 找到 stride 最小的任务
        let mut min_idx = 0;
        let mut min_stride = usize::MAX;
        
        for (idx, task) in self.ready_queue.iter().enumerate() {
            let inner = task.inner_exclusive_access();
            if inner.stride < min_stride {
                min_stride = inner.stride;
                min_idx = idx;
            }
        }
        
        // 取出并返回该任务
        Some(self.ready_queue.remove(min_idx).unwrap())
    }
}
```

### 2.7 更新 stride 值

**文件：[`os/src/task/processor.rs`](os/src/task/processor.rs)**

在 `run_tasks` 函数中，任务被调度后需要更新其 stride。需要找到切换到任务执行的位置，在调度前更新 stride。

或者更简单的方式：在 `suspend_current_and_run_next` 函数中更新。

**文件：[`os/src/task/mod.rs`](os/src/task/mod.rs:50-67)**

修改 `suspend_current_and_run_next` 函数：

```rust
pub fn suspend_current_and_run_next() {
    let task = take_current_task().unwrap();
    
    // ---- access current TCB exclusively
    let mut task_inner = task.inner_exclusive_access();
    let task_cx_ptr = &mut task_inner.task_cx as *mut TaskContext;
    // Change status to Ready
    task_inner.task_status = TaskStatus::Ready;
    
    // 更新 stride: stride += BIG_STRIDE / priority
    task_inner.stride += BIG_STRIDE / task_inner.priority;
    
    drop(task_inner);
    // ---- release current PCB
    
    // push back to ready queue
    add_task(task);
    // jump to scheduling cycle
    schedule(task_cx_ptr);
}
```

---

## 三、需要修改的文件汇总

| 文件路径 | 修改内容 |
|---------|---------|
| [`os/src/task/task.rs`](os/src/task/task.rs) | 添加 priority/stride 字段、spawn 方法、常量定义 |
| [`os/src/task/mod.rs`](os/src/task/mod.rs) | 修改 suspend_current_and_run_next 更新 stride |
| [`os/src/task/manager.rs`](os/src/task/manager.rs) | 修改 fetch 方法实现 stride 调度 |
| [`os/src/syscall/process.rs`](os/src/syscall/process.rs) | 实现 sys_spawn 和 sys_set_priority |

---

## 四、测试验证

运行测试命令：
```bash
make run BASE=2
```

在终端中输入 `ch5_usertest` 运行所有测试。

需要通过的测试：
- ch5_spawn0: spawn 基本功能
- ch5_spawn1: spawn + wait/waitpid
- ch5_setprio: set_priority 系统调用
- ch5_stride: stride 调度公平性测试

---

## 五、注意事项

1. **spawn 与 fork 的区别**：
   - fork 复制父进程地址空间
   - spawn 创建全新地址空间，加载指定程序

2. **stride 溢出处理**：
   - 使用 usize 类型，BIG_STRIDE 选择适中值（如 100000）
   - 可选：实现溢出后的重置逻辑

3. **优先级验证**：
   - sys_set_priority 必须验证 prio >= 2
   - 默认优先级为 16

4. **前向兼容**：
   - 确保之前章节的测试用例仍然通过