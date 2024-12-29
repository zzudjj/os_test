#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;
extern crate alloc;

use core::cell::RefMut;

use alloc::{format, string::String, vec::Vec};
use lazy_static::*;
use user_lib::{exit, gettid, monitor_check, monitor_create, monitor_create_res_sem, monitor_destroy, monitor_enter, monitor_leave, monitor_signal, monitor_wait, sleep, thread_create, waittid, UPSafeCell};

///环形缓冲池数据结构
pub struct CycleBuf {
    read: usize,
    write: usize,
    buf: [i32; 6],
}
///管程数据结构
pub struct Monitor {
    //不变量
    monitor_id: usize, //管程标识符
    full_res_id: usize, //条件变量标识符
    empty_res_id: usize, //条件变量标识符
    //可变量
    inner: UPSafeCell<MonitorInner>,
}

pub struct MonitorInner {
    full_count: i32, //满缓冲区个数
    history: Vec<String>, //记录缓冲区历史
    cyc_buf: CycleBuf, //环形缓冲池
}

impl Monitor {
    ///创建一个管程实例
    pub fn new() -> Self {
        //通过系统调用创建一个Hoare管程并获取其标识符
        let monitor_id = monitor_create();
        //创建条件变量
        let full_res_id = monitor_create_res_sem(monitor_id);
        let empty_res_id = monitor_create_res_sem(monitor_id);
        Self {
            monitor_id: monitor_id,
            full_res_id: full_res_id,
            empty_res_id: empty_res_id,
            inner: unsafe {
                UPSafeCell::new(
                    MonitorInner {
                        full_count: 0,
                        history: Vec::new(),
                        cyc_buf: CycleBuf {
                            read: 0,
                            write: 0,
                            buf: [0; 6],
                        }
                    }
                )
            }
        }
    }
    //获取可变量inner的可变引用
    pub fn inner_exclusive_access(&self) -> RefMut<'_, MonitorInner> {
        self.inner.exclusive_access()
    }
    //生产函数
    pub fn process(&self, value: i32) {
        monitor_enter(self.monitor_id); //进入管程
        for _ in 0..5 {
            let inner = self.inner_exclusive_access();
            if inner.full_count == 6 {
                //缓冲区已满，在empty等待队列中等待空白缓冲区
                drop(inner);
                monitor_wait(self.monitor_id, self.empty_res_id);
            } else {
                drop(inner);
            }
            //写一个缓冲区
            let mut inner = self.inner_exclusive_access();
            let last_write_ptr = inner.cyc_buf.write;
            inner.cyc_buf.buf[last_write_ptr] = value;
            sleep(5);
            inner.cyc_buf.write = (last_write_ptr + 1) % 6;
            //增加一个满缓冲区
            inner.full_count += 1;
            let history= format!("processor{} wrote the value {} in buf{}", gettid(), value, last_write_ptr);
            inner.history.push(history);
            drop(inner);
            //唤醒full等待队列中的消费者线程，自己进入紧急等待队列
            monitor_signal(self.monitor_id, self.full_res_id);
        }
        monitor_leave(self.monitor_id); //离开管程
    } 

    pub fn consume(&self) {
        monitor_enter(self.monitor_id); //进入管程
        for _ in 0..10 {
            let inner = self.inner_exclusive_access();
            if inner.full_count == 0 {
                //空缓冲池，在full等待队列中等待满缓冲区
                drop(inner);
                monitor_wait(self.monitor_id, self.full_res_id);
            } else {
                drop(inner);
            }
            //读缓冲区
            let mut inner = self.inner_exclusive_access();
            let last_read_ptr = inner.cyc_buf.read;
            let value = inner.cyc_buf.buf[last_read_ptr];
            sleep(5);
            inner.cyc_buf.buf[last_read_ptr] = 0;
            inner.cyc_buf.read = (last_read_ptr + 1) % 6;
            //减少一个满缓冲区
            inner.full_count -= 1;
            let history= format!("consumer{} read the value {} from buf{}", gettid(), value, last_read_ptr);
            inner.history.push(history);
            drop(inner);
            //唤醒empty等待队列中的生产者线程，自己进入紧急等待队列
            monitor_signal(self.monitor_id, self.empty_res_id);
        }
        monitor_leave(self.monitor_id); //离开管程
    } 
    ///打印缓冲池操作历史
    pub fn print_history(&self) {
        let inner = self.inner_exclusive_access();
        println!("-------------------HISTORY-----------------");
        for his in inner.history.iter() {
            println!("{}",his.as_str());
        }
    }
    ///打印缓冲池
    pub fn print_cyc_buf(&self) {
        let inner = self.inner_exclusive_access();
        println!("-------------------CYC_BUF-----------------");
        for value in inner.cyc_buf.buf.iter() {
            print!("{} ",value);
        }
        println!("");
    }
    ///检测管程内部是否出现死锁或者饥饿情况
    pub fn check_self(&self) -> isize{
        monitor_check(self.monitor_id)
    }
    ///销毁管程
    pub fn destroy(&self) {
        monitor_destroy(self.monitor_id);
    }
}

lazy_static! {
    //创建管程的静态全局实例
    static ref monitor: Monitor = Monitor::new();
}
///生产者线程
pub fn processor(v: *const i32) {
    let value = unsafe { &*v };
    monitor.process(*value);
    exit(0);
}
///消费者线程
pub fn consumer() {
   monitor.consume();
   exit(0);
}
///管程守护者线程
pub fn checker() {
    loop {
        if monitor.check_self() == 1 {
            //管程内的所有线程均被杀死，守护线程已经没有继续下去的必要了
            break;
        }
    }
    exit(0);
}

#[no_mangle]
pub fn main() -> isize {
    let mut consumers = Vec::new(); //记录消费者线程标识符
    let mut processors = Vec::new(); //记录生产者线程标识符
    let values = [1,2,3,4];
    //创建生产者线程
    for i in 0..4 {
        processors.push(
            thread_create(processor as usize, &values[i] as *const _ as usize)
        );
    }
    //创建消费者线程
    for _ in 0..2 {
        consumers.push(
            thread_create(consumer as usize, 0)
        )
    }
    //创建守护者线程
    //注意守护者线程创建有两个条件：
    //一是守护者线程必须创建在所有需要进入管程的线程之后
    //二是主线程不能进入管程
    thread_create(checker as usize, 0);
    //等待线程结束
    for tid in processors.iter() {
        let exit_code = waittid(*tid as usize);
        println!("processor{}:exited {}", tid, exit_code);
    }
    for tid in consumers.iter() {
        waittid(*tid as usize);
        println!("consumer{}:exited", tid);
    }
    //打印缓冲池操作历史
    monitor.print_history();
    //打印缓冲池
    monitor.print_cyc_buf();
    //销毁管程
    monitor.destroy();
    0
}