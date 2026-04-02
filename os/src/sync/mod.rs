//! Synchronization and interior mutability primitives

mod condvar;
mod deadlock;
mod mutex;
mod semaphore;
mod up;

pub use condvar::Condvar;
pub use deadlock::{
    add_mutex_holder, check_mutex_deadlock, check_semaphore_deadlock, enable_deadlock_detect,
    is_deadlock_enabled, remove_mutex_holder,
};
pub use mutex::{Mutex, MutexBlocking, MutexSpin};
pub use semaphore::Semaphore;
pub use up::UPSafeCell;
