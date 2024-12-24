use alloc::{collections::vec_deque::VecDeque, sync::Arc};

use crate::task::{block_current_and_run_next, current_task, wakeup_task, ThreadControlBlock};

use super::UPSafeCell;
use core::cell::RefMut;

///互斥锁
pub struct Mutex {
    inner: UPSafeCell<MutexInner>,
}

///互斥锁中的可变量
pub struct MutexInner {
    pub is_locked: bool, //互斥锁状态,当存在线程拥有锁时,值为true,否则为false
    pub waited_queue: VecDeque<Arc<ThreadControlBlock>>, //互斥锁队列
}

impl Mutex {
    ///新建一个互斥锁
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

    ///返回互斥锁中的可变量的可变引用
    pub fn inner_exclusive_access(&self) -> RefMut<'_, MutexInner> {
        self.inner.exclusive_access()
    }

    ///申请锁
    pub fn lock(&self) {
        let mut is_locked = self.is_locked();
        //当有线程占有锁时，进入循环，当前线程阻塞
        while is_locked {
            //将线程加入互斥锁队列，并阻塞该线程
            let thread = current_task().unwrap();
            let mut inner = self.inner_exclusive_access();
            inner.waited_queue.push_back(thread.clone()); 
            drop(inner);
            block_current_and_run_next();
            //线程被唤醒后，需重复检查当前是否符合等待条件
            is_locked = self.is_locked();
        }
        //当没有线程拥有锁时，当前线程占有锁
        let mut inner = self.inner_exclusive_access();
        inner.is_locked = true;
    }

    ///释放锁
    pub fn unlock(&self) {
        //当前线程修改锁状态，释放锁
        let mut inner = self.inner_exclusive_access();
        inner.is_locked = false;
        //当互斥锁队列中还存在等待线程时，唤醒第一个线程
        if let Some(waited_thread) = inner.waited_queue.pop_front() {
            wakeup_task(waited_thread);
        }
    }

    ///获取锁当前的状态
    pub fn is_locked(&self) -> bool {
        self.inner_exclusive_access().is_locked
    }
}
