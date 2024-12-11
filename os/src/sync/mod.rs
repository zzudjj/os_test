//! Synchronization and interior mutability primitives
mod up;
mod mutex;

pub use up::UPSafeCell;
pub use mutex::Mutex;