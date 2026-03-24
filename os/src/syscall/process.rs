//! Process management syscalls
use alloc::sync::Arc;

use crate::{
    config::PAGE_SIZE,
    loader::get_app_data_by_name,
    mm::{translated_refmut, translated_str, MapPermission},
    task::{
        add_task, current_mmap, current_munmap, current_task, current_user_token,
        exit_current_and_run_next, suspend_current_and_run_next,
    },
    timer::get_time_us,
};

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

/// task exits and submit an exit code
pub fn sys_exit(exit_code: i32) -> ! {
    trace!("kernel:pid[{}] sys_exit", current_task().unwrap().pid.0);
    exit_current_and_run_next(exit_code);
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    trace!("kernel:pid[{}] sys_yield", current_task().unwrap().pid.0);
    suspend_current_and_run_next();
    0
}

pub fn sys_getpid() -> isize {
    trace!("kernel: sys_getpid pid:{}", current_task().unwrap().pid.0);
    current_task().unwrap().pid.0 as isize
}

pub fn sys_fork() -> isize {
    trace!("kernel:pid[{}] sys_fork", current_task().unwrap().pid.0);
    let current_task = current_task().unwrap();
    let new_task = current_task.fork();
    let new_pid = new_task.pid.0;
    // modify trap context of new_task, because it returns immediately after switching
    let trap_cx = new_task.inner_exclusive_access().get_trap_cx();
    // we do not have to move to next instruction since we have done it before
    // for child process, fork returns 0
    trap_cx.x[10] = 0;
    // add new task to scheduler
    add_task(new_task);
    new_pid as isize
}

pub fn sys_exec(path: *const u8) -> isize {
    trace!("kernel:pid[{}] sys_exec", current_task().unwrap().pid.0);
    let token = current_user_token();
    let path = translated_str(token, path);
    if let Some(data) = get_app_data_by_name(path.as_str()) {
        let task = current_task().unwrap();
        task.exec(data);
        0
    } else {
        -1
    }
}

/// If there is not a child process whose pid is same as given, return -1.
/// Else if there is a child process but it is still running, return -2.
pub fn sys_waitpid(pid: isize, exit_code_ptr: *mut i32) -> isize {
    trace!("kernel::pid[{}] sys_waitpid [{}]", current_task().unwrap().pid.0, pid);
    let task = current_task().unwrap();
    // find a child process

    // ---- access current PCB exclusively
    let mut inner = task.inner_exclusive_access();
    if !inner
        .children
        .iter()
        .any(|p| pid == -1 || pid as usize == p.getpid())
    {
        return -1;
        // ---- release current PCB
    }
    let pair = inner.children.iter().enumerate().find(|(_, p)| {
        // ++++ temporarily access child PCB exclusively
        p.inner_exclusive_access().is_zombie() && (pid == -1 || pid as usize == p.getpid())
        // ++++ release child PCB
    });
    if let Some((idx, _)) = pair {
        let child = inner.children.remove(idx);
        // confirm that child will be deallocated after being removed from children list
        assert_eq!(Arc::strong_count(&child), 1);
        let found_pid = child.getpid();
        // ++++ temporarily access child PCB exclusively
        let exit_code = child.inner_exclusive_access().exit_code;
        // ++++ release child PCB
        if let Some(exit_code_ref) = translated_refmut(inner.memory_set.token(), exit_code_ptr) {
            *exit_code_ref = exit_code;
        }
        found_pid as isize
    } else {
        -2
    }
    // ---- release current PCB automatically
}

/// get time with second and microsecond
pub fn sys_get_time(ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");
    let us = get_time_us();
    let token = current_user_token();

    if let Some(ts_ref) = translated_refmut(token, ts) {
        ts_ref.sec = us / 1_000_000;
        ts_ref.usec = us % 1_000_000;
        0
    } else {
        -1
    }
}

/// translated_refmut 只检查 U 标志，不检查 R/W 标志。如果需要严格检查权限，需要额外实现权限检查函数。
/// mmap: map memory region
pub fn sys_mmap(start: usize, len: usize, prot: usize) -> isize {
    trace!("kernel: sys_mmap");
    
    // 参数校验：页对齐
    if start % PAGE_SIZE != 0 {
        return -1;
    }
    
    // 参数校验：prot 有效
    if prot & !0x7 != 0 {
        return -1;  // 其他位必须为 0
    }
    if prot & 0x7 == 0 {
        return -1;  // 无意义权限
    }
    
    // 长度为 0 视为成功
    if len == 0 {
        return 0;
    }
    
    // 转换权限：prot 格式 -> MapPermission 格式
    let mut perm = MapPermission::U;  // 必须有 U 标志
    if prot & 0x1 != 0 { perm |= MapPermission::R; }
    if prot & 0x2 != 0 { perm |= MapPermission::W; }
    if prot & 0x4 != 0 { perm |= MapPermission::X; }
    
    current_mmap(start, len, perm)
}

/// munmap: unmap memory region
pub fn sys_munmap(start: usize, len: usize) -> isize {
    trace!("kernel: sys_munmap");
    
    // 参数校验：页对齐
    if start % PAGE_SIZE != 0 {
        return -1;
    }
    
    // 长度为 0 视为成功
    if len == 0 {
        return 0;
    }
    
    current_munmap(start, len)
}

/// change data segment size
pub fn sys_sbrk(size: i32) -> isize {
    trace!("kernel:pid[{}] sys_sbrk", current_task().unwrap().pid.0);
    if let Some(old_brk) = current_task().unwrap().change_program_brk(size) {
        old_brk as isize
    } else {
        -1
    }
}

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
/*

fn prot_to_perm(prot: usize) -> MapPermission {
    let mut perm = MapPermission::U;  // 用户态可访问
    if prot & 0x1 != 0 { perm |= MapPermission::R; }
    if prot & 0x2 != 0 { perm |= MapPermission::W; }
    if prot & 0x4 != 0 { perm |= MapPermission::X; }
    perm
}

*/
