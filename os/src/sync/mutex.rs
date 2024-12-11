use alloc::{collections::vec_deque::VecDeque, sync::Arc};

use crate::task::{block_current_and_run_next, take_current_task, wakeup_task, ThreadControlBlock};

use super::UPSafeCell;
use core::cell::RefMut;

pub struct Mutex {
    inner: UPSafeCell<MutexInner>,
}

pub struct MutexInner {
    pub is_locked: bool,
    pub waited_queue: VecDeque<Arc<ThreadControlBlock>>,
}

impl Mutex {
    pub fn new() -> Self {
        Self {
            inner: unsafe {
                UPSafeCell::new(
                    MutexInner {
                        is_locked: false,
                        waited_queue: VecDeque::new(),
                    }
                )
            }
        }
    }

    pub fn inner_exclusive_access(&self) -> RefMut<'_, MutexInner> {
        self.inner.exclusive_access()
    }

    pub fn lock(&self) {
        let mut inner = self.inner_exclusive_access();
        if inner.is_locked {
            let thread = take_current_task().unwrap();
            inner.waited_queue.push_back(thread);
            block_current_and_run_next();
        }
        inner.is_locked = true;
    }

    pub fn unlock(&self) {
        let mut inner = self.inner_exclusive_access();
        inner.is_locked = false;
        if let Some(waited_thread) = inner.waited_queue.pop_front() {
            wakeup_task(waited_thread);
        }
    }
}