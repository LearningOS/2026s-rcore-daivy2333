//! Deadlock detection

use crate::sync::UPSafeCell;
use crate::task::{current_process, current_task};
use alloc::collections::BTreeMap;
use alloc::vec;
use alloc::vec::Vec;
use lazy_static::*;

lazy_static! {
    static ref DEADLOCK_ENABLE: UPSafeCell<bool> = unsafe { UPSafeCell::new(false) };
    static ref MUTEX_HOLDERS: UPSafeCell<BTreeMap<usize, usize>> =
        unsafe { UPSafeCell::new(BTreeMap::new()) };
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

/// Check for mutex deadlock
pub fn check_mutex_deadlock(mutex_id: usize, _mutex_locked: bool) -> bool {
    if !is_deadlock_enabled() {
        return false;
    }

    let task = current_task().unwrap();
    let tid = task.inner_exclusive_access().res.as_ref().unwrap().tid;

    let holders = MUTEX_HOLDERS.exclusive_access();
    if let Some(&holder_tid) = holders.get(&mutex_id) {
        if holder_tid == tid {
            return true;
        }
    }

    false
}

/// Add mutex holder record
pub fn add_mutex_holder(mutex_id: usize) {
    if !is_deadlock_enabled() {
        return;
    }

    let task = current_task().unwrap();
    let tid = task.inner_exclusive_access().res.as_ref().unwrap().tid;

    let mut holders = MUTEX_HOLDERS.exclusive_access();
    holders.insert(mutex_id, tid);
}

/// Remove mutex holder record
pub fn remove_mutex_holder(mutex_id: usize) {
    if !is_deadlock_enabled() {
        return;
    }

    let mut holders = MUTEX_HOLDERS.exclusive_access();
    holders.remove(&mutex_id);
}

/// Check for semaphore deadlock using banker's algorithm
pub fn check_semaphore_deadlock(sem_id: usize) -> bool {
    if !is_deadlock_enabled() {
        return false;
    }

    let process = current_process();
    let process_inner = process.inner_exclusive_access();

    let sem_count = process_inner.semaphore_list.len();
    if sem_id >= sem_count {
        return false;
    }

    let task_count = process_inner.tasks.len();
    let current_task = current_task().unwrap();
    let tid = current_task
        .inner_exclusive_access()
        .res
        .as_ref()
        .unwrap()
        .tid;

    let mut available: Vec<isize> = Vec::with_capacity(sem_count);
    for sem in &process_inner.semaphore_list {
        if let Some(sem) = sem {
            let count = sem.inner.exclusive_access().count;
            available.push(count);
        } else {
            available.push(0);
        }
    }

    let mut allocation: Vec<Vec<isize>> = Vec::with_capacity(task_count);
    for task_opt in &process_inner.tasks {
        if let Some(task) = task_opt {
            let task_inner = task.inner_exclusive_access();
            let mut alloc = task_inner.sem_allocation.clone();
            if alloc.len() < sem_count {
                alloc.resize(sem_count, 0);
            }
            allocation.push(alloc);
        } else {
            allocation.push(vec![0; sem_count]);
        }
    }

    if tid >= allocation.len() || allocation[tid].len() <= sem_id {
        return false;
    }

    available[sem_id] -= 1;
    allocation[tid][sem_id] += 1;

    let mut work = available;
    let mut finish: Vec<bool> = vec![false; task_count];

    for i in 0..task_count {
        if allocation[i].iter().all(|&x| x == 0) {
            finish[i] = true;
        }
    }

    loop {
        let mut found = false;
        for i in 0..task_count {
            if !finish[i] {
                let can_finish = allocation[i]
                    .iter()
                    .zip(work.iter())
                    .all(|(&alloc, &w)| alloc <= w);
                if can_finish {
                    for j in 0..sem_count {
                        work[j] += allocation[i][j];
                    }
                    finish[i] = true;
                    found = true;
                }
            }
        }
        if !found {
            break;
        }
    }

    !finish[tid]
}
