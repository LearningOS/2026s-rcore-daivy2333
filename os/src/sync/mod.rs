//! Synchronization and interior mutability primitives

mod condvar;
mod deadlock;
mod mutex;
mod semaphore;
mod up;

pub use condvar::Condvar;
pub use deadlock::{add_mutex_hold, check_deadlock, enable_deadlock_detect, remove_mutex_hold};
pub use mutex::{Mutex, MutexBlocking, MutexSpin};
pub use semaphore::Semaphore;
pub use up::UPSafeCell;
