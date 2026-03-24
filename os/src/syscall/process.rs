//! Process management syscalls
use crate::task::{change_program_brk, exit_current_and_run_next, suspend_current_and_run_next, current_user_token, current_mmap, current_munmap, get_syscall_count};
use crate::mm::{translated_refmut, MapPermission, VirtAddr, check_permission};
use crate::config::PAGE_SIZE;
use crate::timer::get_time_us;
#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

/// task exits and submit an exit code
pub fn sys_exit(_exit_code: i32) -> ! {
    trace!("kernel: sys_exit");
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    trace!("kernel: sys_yield");
    suspend_current_and_run_next();
    0
}

/// YOUR JOB: get time with second and microsecond
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TimeVal`] is splitted by two pages ?
/// get time with second and microsecond
pub fn sys_get_time(ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");
    let us = get_time_us();
    let token = current_user_token();
    
    match translated_refmut(token, ts) {
        Some(ts_ref) => {
            ts_ref.sec = us / 1_000_000;
            ts_ref.usec = us % 1_000_000;
            0
        }
        None => -1,
    }
}

/// TODO: Finish sys_trace to pass testcases
/// HINT: You might reimplement it with virtual memory management.
/// trace syscall: read or write a byte in user space, or get syscall count
pub fn sys_trace(trace_request: usize, id: usize, data: usize) -> isize {
    trace!("kernel: sys_trace");
    
    match trace_request {
        0 => {
            // Read: 读取用户空间地址 id 处的一个字节
            // 需要检查地址有效、U标志和R权限
            let token = current_user_token();
            let va = VirtAddr::from(id);
            if !check_permission(token, va, true, false) {
                return -1;
            }
            match translated_refmut::<u8>(token, id as *mut u8) {
                Some(byte_ref) => *byte_ref as isize,
                None => -1,
            }
        }
        1 => {
            // Write: 写入字节 data 到用户空间地址 id
            // 需要检查地址有效、U标志和W权限
            let token = current_user_token();
            let va = VirtAddr::from(id);
            if !check_permission(token, va, false, true) {
                return -1;
            }
            match translated_refmut::<u8>(token, id as *mut u8) {
                Some(byte_ref) => {
                    *byte_ref = data as u8;
                    0
                }
                None => -1,
            }
        }
        2 => {
            // Syscall: 返回系统调用 id 被调用的次数
            get_syscall_count(id)
        }
        _ => -1,
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
    trace!("kernel: sys_sbrk");
    if let Some(old_brk) = change_program_brk(size) {
        old_brk as isize
    } else {
        -1
    }
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