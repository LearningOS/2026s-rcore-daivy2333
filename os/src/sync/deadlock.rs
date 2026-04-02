//! Deadlock detection

use crate::sync::UPSafeCell;
use crate::task::current_task;
use alloc::vec::Vec;
use lazy_static::*;

lazy_static! {
    static ref DEADLOCK_ENABLE: UPSafeCell<bool> = unsafe { UPSafeCell::new(false) };
    static ref RESOURCE_GRAPH: UPSafeCell<Vec<Vec<usize>>> = unsafe { UPSafeCell::new(Vec::new()) };
    static ref THREAD_MUTEX_MAP: UPSafeCell<Vec<Vec<usize>>> =
        unsafe { UPSafeCell::new(Vec::new()) };
}

/// Enable deadlock detection
pub fn enable_deadlock_detect(enabled: bool) -> isize {
    let mut deadlock_enable = DEADLOCK_ENABLE.exclusive_access();
    *deadlock_enable = enabled;
    0
}

/// Check if deadlock detection is enabled
pub fn is_deadlock_enabled() -> bool {
    *DEADLOCK_ENABLE.exclusive_access()
}

/// Check for deadlock
pub fn check_deadlock(mutex_id: usize) -> bool {
    if !is_deadlock_enabled() {
        return false;
    }

    let task = current_task().unwrap();
    let tid = task.inner_exclusive_access().res.as_ref().unwrap().tid;

    let mut graph = RESOURCE_GRAPH.exclusive_access();
    let mut thread_mutex = THREAD_MUTEX_MAP.exclusive_access();

    if tid >= thread_mutex.len() {
        thread_mutex.resize(tid + 1, Vec::new());
    }
    if mutex_id >= graph.len() {
        graph.resize(mutex_id + 1, Vec::new());
    }

    for held_mutex in &thread_mutex[tid] {
        if *held_mutex == mutex_id {
            return true;
        }
    }

    false
}

/// Add mutex hold record
pub fn add_mutex_hold(mutex_id: usize) {
    if !is_deadlock_enabled() {
        return;
    }

    let task = current_task().unwrap();
    let tid = task.inner_exclusive_access().res.as_ref().unwrap().tid;

    let mut thread_mutex = THREAD_MUTEX_MAP.exclusive_access();
    if tid >= thread_mutex.len() {
        thread_mutex.resize(tid + 1, Vec::new());
    }
    thread_mutex[tid].push(mutex_id);
}

/// Remove mutex hold record
pub fn remove_mutex_hold(mutex_id: usize) {
    if !is_deadlock_enabled() {
        return;
    }

    let task = current_task().unwrap();
    let tid = task.inner_exclusive_access().res.as_ref().unwrap().tid;

    let mut thread_mutex = THREAD_MUTEX_MAP.exclusive_access();
    if tid < thread_mutex.len() {
        thread_mutex[tid].retain(|m| *m != mutex_id);
    }
}
