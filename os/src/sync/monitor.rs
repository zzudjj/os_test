use core::cell::RefMut;

use alloc::{sync::Arc, vec::Vec};

use super::{Mutex, Semaphore, UPSafeCell};

pub struct HoareMonitor {
    inner: UPSafeCell<HoareMonitorInner>,
}

pub struct HoareMonitorInner {
    pub res_sem_list: Vec<Arc<Semaphore>>,
    pub res_count_list: Vec<usize>,
    pub mutex: Arc<Mutex>,
    pub next_count: usize,
    pub next: Arc<Semaphore>,
}

impl HoareMonitor {
    pub fn new() -> Self {
        Self {
            inner: unsafe {
                UPSafeCell::new(
                    HoareMonitorInner {
                        res_sem_list: Vec::new(),
                        res_count_list: Vec::new(),
                        mutex: Arc::new(Mutex::new()),
                        next_count: 0,
                        next: Arc::new(Semaphore::new()),
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
        let sem = Arc::new(Semaphore::new());
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
        mutex.lock();
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
            mutex.unlock();
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
            mutex.unlock();
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

}