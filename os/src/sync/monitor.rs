use core::cell::RefMut;

use alloc::{sync::Arc, vec::Vec};

use super::{Semaphore, UPSafeCell};

pub struct HoareMonitor {
    inner: UPSafeCell<HoareMonitorInner>,
}

pub struct HoareMonitorInner {
    pub res_sem_list: Vec<Arc<Semaphore>>, //局部于管程的资源队列（x_sem）
    pub res_count_list: Vec<usize>, //记录对应资源的等待线程数（x_count）
    pub mutex: Arc<Semaphore>, //管理入口等待队列的信号量
    pub next_count: usize, //紧急等待队列
    pub next: Arc<Semaphore>, //管理紧急等待队列的信号量
    pub thread_count: isize, //记录当前在管程中的线程数目
}

impl HoareMonitor {
    pub fn new() -> Self {
        Self {
            inner: unsafe {
                UPSafeCell::new(
                    HoareMonitorInner {
                        res_sem_list: Vec::new(),
                        res_count_list: Vec::new(),
                        mutex: Arc::new(Semaphore::new(1)),
                        next_count: 0,
                        next: Arc::new(Semaphore::new(0)),
                        thread_count: 0,
                    }
                )
            }
        }
    }

    pub fn inner_exclusive_access(&self) -> RefMut<'_, HoareMonitorInner> {
        self.inner.exclusive_access()
    }

    pub fn create_res_sem(&self) -> usize {
        let mut inner = self.inner_exclusive_access();
        let sem = Arc::new(Semaphore::new(0));
        inner.res_sem_list.push(sem);
        inner.res_count_list.push(0);
        let res_id = inner.res_sem_list.len() - 1;
        drop(inner);
        res_id
    }

    pub fn enter(&self) {
        let inner = self.inner_exclusive_access();
        let mutex = inner.mutex.clone();
        drop(inner);
        mutex.sem_wait();
        self.add_thread_count(1);
    }

    pub fn leave(&self) {
        let inner = self.inner_exclusive_access();
        if inner.next_count > 0 {
            let next = inner.next.clone();
            drop(inner);
            next.sem_post();
        } else {
            let mutex = inner.mutex.clone();
            drop(inner);
            mutex.sem_post();
            self.add_thread_count(-1);
        }
    }

    pub fn wait(&self, res_id: usize) {
        let mut inner = self.inner_exclusive_access();
        let x_count = &mut inner.res_count_list[res_id];
        *x_count += 1;
        if inner.next_count > 0 {
            let next = inner.next.clone();
            drop(inner);
            next.sem_post();
        } else {
            let mutex = inner.mutex.clone();
            drop(inner);
            mutex.sem_post();
        }
        let inner = self.inner_exclusive_access();
        let x_sem = inner.res_sem_list[res_id].clone();
        drop(inner);
        x_sem.sem_wait();
        let mut inner = self.inner_exclusive_access();
        let x_count = &mut inner.res_count_list[res_id];
        *x_count -= 1;
    }

    pub fn signal(&self, res_id: usize) {
        let mut inner = self.inner_exclusive_access();
        let x_count = inner.res_count_list[res_id];
        if x_count > 0 {
            inner.next_count += 1;
            let x_sem = inner.res_sem_list[res_id].clone();
            let next = inner.next.clone();
            drop(inner);
            x_sem.sem_post();
            next.sem_wait();
            let mut inner = self.inner_exclusive_access();
            inner.next_count -= 1;
        }
    }
    ///检测管程内部的线程集合是否可能出现了死锁或饥饿情况
    #[allow(unused)]
    pub fn check_self(&self) -> isize{ 
        let mut waited_thread_count: isize = 0;
        let inner = self.inner_exclusive_access();
        for sem_count in inner.res_count_list.iter() {
            waited_thread_count += *sem_count as isize;
        }
        waited_thread_count += inner.next_count as isize;
        if waited_thread_count == inner.thread_count {
            for sem in inner.res_sem_list.iter() {
                println!("[kernel] 管程内部的线程集合可能出现了死锁或饥饿情况，将杀死管程内部的所有线程");
                let sem_waited_queue = &mut sem.inner_exclusive_access().waited_queue;
                while sem_waited_queue.len() > 0 {
                    let thread = sem_waited_queue.pop_front().unwrap();
                    let mut thread_inner = thread.inner_exclusive_access();
                    thread_inner.exit_code = Some(-1);
                    thread_inner.res = None;
                    drop(thread_inner);
                    drop(thread);
                }
            } 
            1
        } else {
            0
        }
    }

    fn add_thread_count(&self, num: isize) {
        let mut inner =  self.inner_exclusive_access();
        inner.thread_count += num
    }
}