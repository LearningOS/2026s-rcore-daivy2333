//! Process management syscalls
//!
use alloc::sync::Arc;

use crate::{
    config::PAGE_SIZE,
    fs::{open_file, OpenFlags},
    mm::{check_permission, translated_refmut, translated_str, VirtAddr},
    task::{
        add_task, current_task, current_user_token, exit_current_and_run_next,
        suspend_current_and_run_next, TaskControlBlock,
    },
    timer::get_time_us,
};

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

pub fn sys_exit(exit_code: i32) -> ! {
    trace!("kernel:pid[{}] sys_exit", current_task().unwrap().pid.0);
    exit_current_and_run_next(exit_code);
    panic!("Unreachable in sys_exit!");
}

pub fn sys_yield() -> isize {
    //trace!("kernel: sys_yield");
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
    if let Some(app_inode) = open_file(path.as_str(), OpenFlags::RDONLY) {
        let all_data = app_inode.read_all();
        let task = current_task().unwrap();
        task.exec(all_data.as_slice());
        0
    } else {
        -1
    }
}

/// If there is not a child process whose pid is same as given, return -1.
/// Else if there is a child process but it is still running, return -2.
pub fn sys_waitpid(pid: isize, exit_code_ptr: *mut i32) -> isize {
    //trace!("kernel: sys_waitpid");
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
        *translated_refmut(inner.memory_set.token(), exit_code_ptr) = exit_code;
        found_pid as isize
    } else {
        -2
    }
    // ---- release current PCB automatically
}

/// YOUR JOB: get time with second and microsecond
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TimeVal`] is splitted by two pages ?
pub fn sys_get_time(ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel:pid[{}] sys_get_time", current_task().unwrap().pid.0);
    let us = get_time_us();
    let token = current_user_token();
    
    let ts_ref = translated_refmut(token, ts);
    ts_ref.sec = us / 1_000_000;
    ts_ref.usec = us % 1_000_000;
    0
}

/// YOUR JOB: Implement mmap.
pub fn sys_mmap(start: usize, len: usize, port: usize) -> isize {
    trace!("kernel:pid[{}] sys_mmap", current_task().unwrap().pid.0);
    
    // 参数校验：页对齐
    if start % PAGE_SIZE != 0 {
        return -1;
    }
    
    // 参数校验：port 有效
    if port & !0x7 != 0 {
        return -1;  // 其他位必须为 0
    }
    if port & 0x7 == 0 {
        return -1;  // 无意义权限
    }
    
    // 长度为 0 视为成功
    if len == 0 {
        return 0;
    }
    
    let task = current_task().unwrap();
    let mut inner = task.inner_exclusive_access();
    
    // 检查是否与现有映射冲突
    let start_va = VirtAddr::from(start);
    let end_va = VirtAddr::from(start + len);
    let start_vpn = start_va.floor();
    let end_vpn = end_va.ceil();
    
    for vpn in crate::mm::VPNRange::new(start_vpn, end_vpn) {
        if let Some(pte) = inner.memory_set.page_table.translate(vpn) {
            if pte.is_valid() {
                return -1;  // 已映射，冲突
            }
        }
    }
    
    // 转换权限：port 格式 -> MapPermission 格式
    use crate::mm::MapPermission;
    let mut perm = MapPermission::U;  // 必须有 U 标志
    if port & 0x1 != 0 { perm |= MapPermission::R; }
    if port & 0x2 != 0 { perm |= MapPermission::W; }
    if port & 0x4 != 0 { perm |= MapPermission::X; }
    
    // 创建新的映射区域
    inner.memory_set.insert_framed_area(start_va, end_va, perm);
    0
}

/// YOUR JOB: Implement munmap.
pub fn sys_munmap(start: usize, len: usize) -> isize {
    trace!("kernel:pid[{}] sys_munmap", current_task().unwrap().pid.0);
    
    // 参数校验：页对齐
    if start % PAGE_SIZE != 0 {
        return -1;
    }
    
    // 长度为 0 视为成功
    if len == 0 {
        return 0;
    }
    
    let task = current_task().unwrap();
    let mut inner = task.inner_exclusive_access();
    
    let start_va = VirtAddr::from(start);
    let end_va = VirtAddr::from(start + len);
    let start_vpn = start_va.floor();
    let end_vpn = end_va.ceil();
    
    // 检查所有页面是否都已映射
    for vpn in crate::mm::VPNRange::new(start_vpn, end_vpn) {
        if let Some(pte) = inner.memory_set.page_table.translate(vpn) {
            if !pte.is_valid() {
                return -1;  // 未映射
            }
        } else {
            return -1;
        }
    }
    
    // 取消映射
    for vpn in crate::mm::VPNRange::new(start_vpn, end_vpn) {
        inner.memory_set.page_table.unmap(vpn);
    }
    
    0
}

/// trace syscall: read or write a byte in user space
pub fn sys_trace(trace_request: usize, id: usize, data: usize) -> isize {
    trace!("kernel:pid[{}] sys_trace", current_task().unwrap().pid.0);
    let token = current_user_token();
    
    match trace_request {
        0 => {
            // 读取 - 检查用户可见且可读
            let va = VirtAddr::from(id);
            if !check_permission(token, va, true, false) {
                return -1;
            }
            let byte_ref = translated_refmut::<u8>(token, id as *mut u8);
            *byte_ref as isize
        }
        1 => {
            // 写入 - 检查用户可见且可写
            let va = VirtAddr::from(id);
            if !check_permission(token, va, false, true) {
                return -1;
            }
            let byte_ref = translated_refmut::<u8>(token, id as *mut u8);
            *byte_ref = data as u8;
            0
        }
        _ => -1,
    }
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

/// YOUR JOB: Implement spawn.
/// HINT: fork + exec =/= spawn
pub fn sys_spawn(path: *const u8) -> isize {
    trace!("kernel:pid[{}] sys_spawn", current_task().unwrap().pid.0);
    
    let token = current_user_token();
    let path_str = translated_str(token, path);
    
    // Open the file
    if let Some(app_inode) = open_file(path_str.as_str(), OpenFlags::RDONLY) {
        let all_data = app_inode.read_all();
        
        // Create a new task (similar to new, but with parent relationship)
        let current_task = current_task().unwrap();
        let new_task = TaskControlBlock::spawn(all_data.as_slice(), current_task.clone());
        let new_pid = new_task.pid.0;
        
        // Add new task to scheduler
        add_task(new_task);
        
        new_pid as isize
    } else {
        -1
    }
}

// YOUR JOB: Set task priority.
pub fn sys_set_priority(_prio: isize) -> isize {
    trace!(
        "kernel:pid[{}] sys_set_priority NOT IMPLEMENTED",
        current_task().unwrap().pid.0
    );
    -1
}
