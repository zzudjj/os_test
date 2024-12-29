use core::cell::RefMut;

use alloc::{sync::Arc, vec::Vec};

use super::{Semaphore, UPSafeCell};

///霍尔管程
pub struct HoareMonitor {
    inner: UPSafeCell<HoareMonitorInner>,
}
///霍尔管程内部可变量
pub struct HoareMonitorInner {
    pub res_sem_list: Vec<Arc<Semaphore>>, //保存信号量x_sem的队列
    pub res_count_list: Vec<usize>, //记录对应x_sem的x_count
    pub mutex: Arc<Semaphore>, //管理入口等待队列的信号量
    pub next_count: usize, //紧急等待队列中的线程数
    pub next: Arc<Semaphore>, //管理紧急等待队列的信号量
    pub thread_count: isize, //管程中以及管程入口等待队列中的线程数目
}

impl HoareMonitor {
    ///创建一个Hoare管程
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
    ///获得inner的可变引用
    pub fn inner_exclusive_access(&self) -> RefMut<'_, HoareMonitorInner> {
        self.inner.exclusive_access()
    }
    ///创建一个信号量x_sem
    pub fn create_res_sem(&self) -> usize {
        let mut inner = self.inner_exclusive_access();
        let sem = Arc::new(Semaphore::new(0));
        inner.res_sem_list.push(sem);
        inner.res_count_list.push(0);
        let res_id = inner.res_sem_list.len() - 1;
        drop(inner);
        res_id
    }
    ///进入管程
    pub fn enter(&self) {
        let inner = self.inner_exclusive_access();
        let mutex = inner.mutex.clone();
        drop(inner);
        //thread_count加1
        self.add_thread_count(1);
        //申请进入管程的锁
        mutex.sem_wait();
    }
    ///离开管程
    pub fn leave(&self) {
        let inner = self.inner_exclusive_access();
        if inner.next_count > 0 {
            //如果紧急等待队列中存在线程，优先唤醒其中的线程
            let next = inner.next.clone();
            drop(inner);
            next.sem_post();
        } else {
            //紧急等待队列中不存在等待线程，则唤醒管程入口等待队列中的线程
            let mutex = inner.mutex.clone();
            drop(inner);
            mutex.sem_post();
        }
        //thread_count减一
        self.add_thread_count(-1);
    }
    ///Hoare的wait操作
    pub fn wait(&self, res_id: usize) {
        let mut inner = self.inner_exclusive_access();
        let x_count = &mut inner.res_count_list[res_id];
        //等待资源的线程数加一
        *x_count += 1;
        if inner.next_count > 0 {
            //如果紧急等待队列中存在线程，优先唤醒其中的线程
            let next = inner.next.clone();
            drop(inner);
            next.sem_post();
        } else {
              //紧急等待队列中不存在等待线程，则唤醒管程入口等待队列中的线程
            let mutex = inner.mutex.clone();
            drop(inner);
            mutex.sem_post();
        }
        let inner = self.inner_exclusive_access();
        //获取指定的x_sem
        let x_sem = inner.res_sem_list[res_id].clone();
        drop(inner);
        //阻塞当前调用线程并加入到x_sem管理的资源等待线程中
        x_sem.sem_wait();
        let mut inner = self.inner_exclusive_access();
        let x_count = &mut inner.res_count_list[res_id];
        //线程苏醒后，将等待线程数减一
        *x_count -= 1;
    }
    ///Hoare的signal操作
    pub fn signal(&self, res_id: usize) {
        let mut inner = self.inner_exclusive_access();
        let x_count = inner.res_count_list[res_id];
        //当x_sem管理的资源队列中没有等待线程跳过if代码块
        if x_count > 0 {
            //紧急等待队列中的线程数加一
            inner.next_count += 1;
            let x_sem = inner.res_sem_list[res_id].clone();
            let next = inner.next.clone();
            drop(inner);
            //唤醒x_sem管理的资源等待队列中的一个线程
            x_sem.sem_post();
            //将当前线程阻塞并加入到紧急等待队列中
            next.sem_wait();
            let mut inner = self.inner_exclusive_access();
            //线程从紧急队列中苏醒后，紧急等待队列中的线程数减一
            inner.next_count -= 1;
        }
    }
    ///检测管程内部的线程集合是否可能出现了死锁或饥饿情况
    #[allow(unused)]
    pub fn check_self(&self) -> isize{ 
        let mut waited_thread_count: isize = 0;
        let mut inner = self.inner_exclusive_access();
        //如果管程中本来就没有线程，一定不会发生这些情况
        if inner.thread_count == 0 {
            return 0;
        }
        //获得各x_sem管理的资源等待队列中的实时线程数
        for sem in inner.res_sem_list.iter() {
            let sem_count = sem.inner_exclusive_access().waited_queue.len();
            waited_thread_count += sem_count as isize;
        }
        //紧急等待队列中的实时线程数
        waited_thread_count += inner.next.inner_exclusive_access().waited_queue.len() as isize;
        //管程入口等待队列中的实时线程数
        waited_thread_count += inner.mutex.inner_exclusive_access().waited_queue.len() as isize;
        if waited_thread_count == inner.thread_count {
            //管程中的所有线程均阻塞，认定为出现饥饿或死锁
            println!("[kernal] monitor_checker warning! threads will be killed");
            //杀死各x_sem所管理的资源等待队列中的所有线程
            for sem in inner.res_sem_list.iter() {
                let sem_waited_queue = &mut sem.inner_exclusive_access().waited_queue;
                while sem_waited_queue.len() > 0 {
                    let thread = sem_waited_queue.pop_front().unwrap();
                    let mut thread_inner = thread.inner_exclusive_access();
                    println!("[kernal] thread{} is killed",thread_inner.res.as_ref().unwrap().tid);
                    thread_inner.exit_code = Some(-1);
                    thread_inner.res = None;
                    drop(thread_inner);
                    drop(thread);
                }
            } 
            let mutex = inner.mutex.clone();
            let mutex_waited_queue = &mut mutex.inner_exclusive_access().waited_queue;
            //杀死管程入口等待队列中的所有线程
            while mutex_waited_queue.len() > 0 {
                let thread = mutex_waited_queue.pop_front().unwrap();
                let mut thread_inner = thread.inner_exclusive_access();
                println!("[kernal] thread{} is killed",thread_inner.res.as_ref().unwrap().tid);
                thread_inner.exit_code = Some(-1);
                thread_inner.res = None;
                drop(thread_inner);
                drop(thread);
            }
            let next = inner.next.clone();
            let next_waited_queue = &mut next.inner_exclusive_access().waited_queue;
            //杀死紧急等待队列中的所有线程
            while next_waited_queue.len() > 0 {
                let thread = next_waited_queue.pop_front().unwrap();
                let mut thread_inner = thread.inner_exclusive_access();
                println!("[kernal] thread{} is killed",thread_inner.res.as_ref().unwrap().tid);
                thread_inner.exit_code = Some(-1);
                thread_inner.res = None;
                drop(thread_inner);
                drop(thread);
            }
            //管程中的线程数归0
            inner.thread_count = 0;
            1
        } else {
            //认定当前不存在饥饿与死锁问题
            0
        }
    }
    ///设置thread_count的增量
    fn add_thread_count(&self, num: isize) {
        let mut inner =  self.inner_exclusive_access();
        inner.thread_count += num
    }
}