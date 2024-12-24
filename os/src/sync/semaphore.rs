use core::cell::RefMut;

use alloc::{collections::vec_deque::VecDeque, sync::Arc};

use crate::task::{block_current_and_run_next, current_task, wakeup_task, ThreadControlBlock};

use super::UPSafeCell;

pub struct Semaphore {
    inner: UPSafeCell<SemaphoreInner>,
}

pub struct SemaphoreInner {
    pub value: isize,
    pub waited_queue: VecDeque<Arc<ThreadControlBlock>>,
} 

impl Semaphore {
    pub fn new(value: isize) -> Self {
        Self {
            inner : unsafe {
                UPSafeCell::new(
                    SemaphoreInner {
                        value: value,
                        waited_queue: VecDeque::new(),
                    }
                )
            }
        }
    }

    pub fn inner_exclusive_access(&self) -> RefMut<'_, SemaphoreInner> {
        self.inner.exclusive_access()
    }

    pub fn sem_wait(&self) {
        let mut inner = self.inner_exclusive_access();
        inner.value -= 1;
        if inner.value < 0 {
            let thread = current_task().unwrap();
            inner.waited_queue.push_back(thread.clone());
            drop(inner);
            block_current_and_run_next();
        }
    }

    pub fn sem_post(&self) {
        let mut inner = self.inner_exclusive_access();
        inner.value += 1;
        if inner.value <= 0 {
            let thread = inner.waited_queue.pop_front().unwrap();
            drop(inner);
            wakeup_task(thread);
        }
    }
}