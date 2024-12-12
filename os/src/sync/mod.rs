//! Synchronization and interior mutability primitives
mod up;
mod mutex;
mod semaphore;

pub use up::UPSafeCell;
pub use mutex::Mutex;
pub use semaphore::Semaphore;