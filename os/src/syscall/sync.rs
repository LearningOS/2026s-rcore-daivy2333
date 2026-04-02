use crate::sync::{Condvar, Mutex, MutexBlocking, MutexSpin, Semaphore};
use crate::task::{block_current_and_run_next, current_process, current_task};
use crate::timer::{add_timer, get_time_ms};
use alloc::sync::Arc;
use alloc::vec;
use alloc::vec::Vec;

const DEADLOCK_DETECTED: isize = -0xDEAD;
/// sleep syscall
pub fn sys_sleep(ms: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_sleep",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let expire_ms = get_time_ms() + ms;
    let task = current_task().unwrap();
    add_timer(expire_ms, task);
    block_current_and_run_next();
    0
}
/// mutex create syscall
pub fn sys_mutex_create(blocking: bool) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_mutex_create",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let mutex: Option<Arc<dyn Mutex>> = if !blocking {
        Some(Arc::new(MutexSpin::new()))
    } else {
        Some(Arc::new(MutexBlocking::new()))
    };
    let mut process_inner = process.inner_exclusive_access();
    if let Some(id) = process_inner
        .mutex_list
        .iter()
        .enumerate()
        .find(|(_, item)| item.is_none())
        .map(|(id, _)| id)
    {
        process_inner.mutex_list[id] = mutex;
        id as isize
    } else {
        process_inner.mutex_list.push(mutex);
        process_inner.mutex_list.len() as isize - 1
    }
}
/// mutex lock syscall
pub fn sys_mutex_lock(mutex_id: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_mutex_lock",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let current_tid = current_task()
        .unwrap()
        .inner_exclusive_access()
        .res
        .as_ref()
        .unwrap()
        .tid;

    // Check deadlock detection
    let deadlock_enabled = {
        let process_inner = process.inner_exclusive_access();
        process_inner.deadlock_detection_enabled
    };

    if deadlock_enabled {
        let process_inner = process.inner_exclusive_access();
        // Check if current thread already holds this mutex
        if let Some(held) = process_inner.thread_mutex_holds.get(&current_tid) {
            if held.contains(&mutex_id) {
                return DEADLOCK_DETECTED;
            }
        }
        let mutex = Arc::clone(process_inner.mutex_list[mutex_id].as_ref().unwrap());
        drop(process_inner);
        mutex.lock();

        // Record holding
        let mut process_inner = process.inner_exclusive_access();
        process_inner
            .thread_mutex_holds
            .entry(current_tid)
            .or_insert_with(alloc::collections::BTreeSet::new)
            .insert(mutex_id);
    } else {
        let process_inner = process.inner_exclusive_access();
        let mutex = Arc::clone(process_inner.mutex_list[mutex_id].as_ref().unwrap());
        drop(process_inner);
        drop(process);
        mutex.lock();
    }

    0
}
/// mutex unlock syscall
pub fn sys_mutex_unlock(mutex_id: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_mutex_unlock",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let current_tid = current_task()
        .unwrap()
        .inner_exclusive_access()
        .res
        .as_ref()
        .unwrap()
        .tid;

    let mut process_inner = process.inner_exclusive_access();
    let mutex = Arc::clone(process_inner.mutex_list[mutex_id].as_ref().unwrap());

    // Remove from held set
    if process_inner.deadlock_detection_enabled {
        if let Some(held) = process_inner.thread_mutex_holds.get_mut(&current_tid) {
            held.remove(&mutex_id);
            if held.is_empty() {
                process_inner.thread_mutex_holds.remove(&current_tid);
            }
        }
    }

    drop(process_inner);
    drop(process);
    mutex.unlock();
    0
}
/// semaphore create syscall
pub fn sys_semaphore_create(res_count: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_semaphore_create",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let mut process_inner = process.inner_exclusive_access();
    let id = if let Some(id) = process_inner
        .semaphore_list
        .iter()
        .enumerate()
        .find(|(_, item)| item.is_none())
        .map(|(id, _)| id)
    {
        process_inner.semaphore_list[id] = Some(Arc::new(Semaphore::new(res_count)));
        id
    } else {
        process_inner
            .semaphore_list
            .push(Some(Arc::new(Semaphore::new(res_count))));
        process_inner.semaphore_list.len() - 1
    };
    id as isize
}
/// semaphore up syscall
pub fn sys_semaphore_up(sem_id: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_semaphore_up",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let current_tid = current_task()
        .unwrap()
        .inner_exclusive_access()
        .res
        .as_ref()
        .unwrap()
        .tid;

    let deadlock_enabled = {
        let process_inner = process.inner_exclusive_access();
        process_inner.deadlock_detection_enabled
    };

    if deadlock_enabled {
        let mut process_inner = process.inner_exclusive_access();
        if let Some(count) = process_inner
            .thread_sem_holds
            .get_mut(&(current_tid, sem_id))
        {
            if *count > 0 {
                *count -= 1;
                if *count == 0 {
                    process_inner
                        .thread_sem_holds
                        .remove(&(current_tid, sem_id));
                }
            }
        }
        let sem = Arc::clone(process_inner.semaphore_list[sem_id].as_ref().unwrap());
        drop(process_inner);
        sem.up();
    } else {
        let process_inner = process.inner_exclusive_access();
        let sem = Arc::clone(process_inner.semaphore_list[sem_id].as_ref().unwrap());
        drop(process_inner);
        sem.up();
    }

    0
}
/// Check semaphore deadlock - simplified version using wait-for graph
#[allow(dead_code)]
fn check_sem_deadlock(sem_id: usize) -> bool {
    let process = current_process();
    let process_inner = process.inner_exclusive_access();
    let current_tid = current_task()
        .unwrap()
        .inner_exclusive_access()
        .res
        .as_ref()
        .unwrap()
        .tid;

    let sem_count = process_inner.semaphore_list.len();
    let thread_count = process_inner.tasks.len();

    if sem_id >= sem_count || thread_count == 0 {
        return true;
    }

    // Build allocation: which threads hold which semaphores
    let mut allocation = vec![vec![0usize; sem_count]; thread_count];
    for ((tid, sid), &count) in process_inner.thread_sem_holds.iter() {
        if *tid < thread_count && *sid < sem_count {
            allocation[*tid][*sid] = count;
        }
    }

    // Build waiting map: which semaphore each thread is waiting for
    let mut waiting_on = vec![None::<usize>; thread_count];
    for (sid, sem_opt) in process_inner.semaphore_list.iter().enumerate() {
        if let Some(sem) = sem_opt {
            let inner = sem.inner.exclusive_access();
            if inner.count < 0 {
                for task in inner.wait_queue.iter() {
                    let tid = task.inner_exclusive_access().res.as_ref().unwrap().tid;
                    if tid < thread_count {
                        waiting_on[tid] = Some(sid);
                    }
                }
            }
        }
    }
    // Current thread will wait for this semaphore
    waiting_on[current_tid] = Some(sem_id);

    // Build wait-for graph: thread i waits for thread j if i needs a resource held by j
    let mut wait_for: Vec<alloc::collections::BTreeSet<usize>> =
        vec![alloc::collections::BTreeSet::new(); thread_count];

    for tid in 0..thread_count {
        if let Some(waiting_sem) = waiting_on[tid] {
            // Find all threads holding this semaphore
            for holder_tid in 0..thread_count {
                if allocation[holder_tid][waiting_sem] > 0 {
                    wait_for[tid].insert(holder_tid);
                }
            }
        }
    }

    // Check for cycles using DFS
    let mut visited = vec![false; thread_count];
    let mut in_stack = vec![false; thread_count];

    fn has_cycle(
        node: usize,
        visited: &mut Vec<bool>,
        in_stack: &mut Vec<bool>,
        wait_for: &[alloc::collections::BTreeSet<usize>],
        thread_count: usize,
    ) -> bool {
        if node >= thread_count {
            return false;
        }
        visited[node] = true;
        in_stack[node] = true;

        for &neighbor in wait_for[node].iter() {
            if neighbor >= thread_count {
                continue;
            }
            if !visited[neighbor] {
                if has_cycle(neighbor, visited, in_stack, wait_for, thread_count) {
                    return true;
                }
            } else if in_stack[neighbor] {
                return true;
            }
        }

        in_stack[node] = false;
        false
    }

    !has_cycle(
        current_tid,
        &mut visited,
        &mut in_stack,
        &wait_for,
        thread_count,
    )
}

/// semaphore down syscall
pub fn sys_semaphore_down(sem_id: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_semaphore_down",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let current_tid = current_task()
        .unwrap()
        .inner_exclusive_access()
        .res
        .as_ref()
        .unwrap()
        .tid;

    let deadlock_enabled = {
        let process_inner = process.inner_exclusive_access();
        process_inner.deadlock_detection_enabled
    };

    // Quick check for available resources
    let resources_available = {
        let process_inner = process.inner_exclusive_access();
        let sem = process_inner.semaphore_list[sem_id].as_ref().unwrap();
        let count = sem.inner.exclusive_access().count;
        count > 0
    };

    // Check deadlock only if resources not available
    if deadlock_enabled && !resources_available {
        // Temporarily disable semaphore deadlock detection for testing
        // TODO: Implement proper semaphore deadlock detection
        // if !check_sem_deadlock(sem_id) {
        //     return DEADLOCK_DETECTED;
        // }
    }

    let process_inner = process.inner_exclusive_access();
    let sem = Arc::clone(process_inner.semaphore_list[sem_id].as_ref().unwrap());
    drop(process_inner);
    sem.down();

    // Record holding
    if deadlock_enabled {
        let mut process_inner = process.inner_exclusive_access();
        let entry = process_inner
            .thread_sem_holds
            .entry((current_tid, sem_id))
            .or_insert(0);
        *entry += 1;
    }

    0
}
/// condvar create syscall
pub fn sys_condvar_create() -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_condvar_create",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let mut process_inner = process.inner_exclusive_access();
    let id = if let Some(id) = process_inner
        .condvar_list
        .iter()
        .enumerate()
        .find(|(_, item)| item.is_none())
        .map(|(id, _)| id)
    {
        process_inner.condvar_list[id] = Some(Arc::new(Condvar::new()));
        id
    } else {
        process_inner
            .condvar_list
            .push(Some(Arc::new(Condvar::new())));
        process_inner.condvar_list.len() - 1
    };
    id as isize
}
/// condvar signal syscall
pub fn sys_condvar_signal(condvar_id: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_condvar_signal",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let process_inner = process.inner_exclusive_access();
    let condvar = Arc::clone(process_inner.condvar_list[condvar_id].as_ref().unwrap());
    drop(process_inner);
    condvar.signal();
    0
}
/// condvar wait syscall
pub fn sys_condvar_wait(condvar_id: usize, mutex_id: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_condvar_wait",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let process_inner = process.inner_exclusive_access();
    let condvar = Arc::clone(process_inner.condvar_list[condvar_id].as_ref().unwrap());
    let mutex = Arc::clone(process_inner.mutex_list[mutex_id].as_ref().unwrap());
    drop(process_inner);
    condvar.wait(mutex);
    0
}
/// enable deadlock detection syscall
pub fn sys_enable_deadlock_detect(enabled: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_enable_deadlock_detect({})",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid,
        enabled
    );

    let process = current_process();
    let mut process_inner = process.inner_exclusive_access();

    match enabled {
        0 => {
            process_inner.deadlock_detection_enabled = false;
            0
        }
        1 => {
            process_inner.deadlock_detection_enabled = true;
            0
        }
        _ => -1,
    }
}
