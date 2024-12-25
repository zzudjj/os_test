use core::cell::RefMut;

use alloc::{collections::vec_deque::VecDeque, sync::Arc};

use crate::task::{block_current_and_run_next, current_task, wakeup_task, ThreadControlBlock};

use super::UPSafeCell;

///信号量
pub struct Semaphore {
    inner: UPSafeCell<SemaphoreInner>,
}

///信号量结构体中的可变量集合
pub struct SemaphoreInner {
    //value>0时，表示当前可用的资源数
    //value<0时，其绝对值表示信号量队列中等待的线程数
    pub value: isize, 
    //信号量等待队列
    pub waited_queue: VecDeque<Arc<ThreadControlBlock>>,
} 

impl Semaphore {
    ///创建一个信号量资源
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
    ///获取可变量inner的可变引用
    pub fn inner_exclusive_access(&self) -> RefMut<'_, SemaphoreInner> {
        self.inner.exclusive_access()
    }
    ///P操作
    pub fn sem_wait(&self) {
        let mut inner = self.inner_exclusive_access();
        //消耗一个资源
        inner.value -= 1;
        //资源耗尽，当前申请资源的线程加入信号量等待队列
        if inner.value < 0 {
            let thread = current_task().unwrap();
            inner.waited_queue.push_back(thread.clone());
            drop(inner);
            block_current_and_run_next();
        }
    }
    ///V操作
    pub fn sem_post(&self) {
        let mut inner = self.inner_exclusive_access();
        //释放一个空闲资源
        inner.value += 1;
        //当信号量队列中还存在等待线程时，唤醒第一个线程使之得到该资源
        if inner.value <= 0 {
            let thread = inner.waited_queue.pop_front().unwrap();
            drop(inner);
            wakeup_task(thread);
        }
    }
}