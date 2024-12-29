#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;
extern crate alloc;

use core::cell::RefMut;

use alloc::{format, string::String, vec::Vec};
use lazy_static::*;
use user_lib::{exit, gettid, monitor_check, monitor_create, monitor_create_res_sem, monitor_enter, monitor_leave, monitor_signal, monitor_wait, sleep, thread_create, waittid, UPSafeCell};

pub struct CycleBuf {
    read: usize,
    write: usize,
    buf: [i32; 6],
}

pub struct Monitor {
    monitor_id: usize,
    full_res_id: usize,
    empty_res_id: usize,
    inner: UPSafeCell<MonitorInner>
}

pub struct MonitorInner {
    full_count: i32,
    history: Vec<String>,
    cyc_buf: CycleBuf,
}

impl Monitor {
    
    pub fn new() -> Self {
        let monitor_id = monitor_create();
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

    pub fn inner_exclusive_access(&self) -> RefMut<'_, MonitorInner> {
        self.inner.exclusive_access()
    }

    pub fn process(&self, value: i32) {
        monitor_enter(self.monitor_id);
        for _ in 0..5 {
            let inner = self.inner_exclusive_access();
            if inner.full_count == 6 {
                drop(inner);
                monitor_wait(self.monitor_id, self.empty_res_id);
            } else {
                drop(inner);
            }
            let mut inner = self.inner_exclusive_access();
            let last_write_ptr = inner.cyc_buf.write;
            inner.cyc_buf.buf[last_write_ptr] = value;
            sleep(5);
            inner.cyc_buf.write = (last_write_ptr + 1) % 6;
            inner.full_count += 1;
            let history= format!("processor{} wrote the value {} in buf{}", gettid(), value, last_write_ptr);
            inner.history.push(history);
            drop(inner);
            monitor_signal(self.monitor_id, self.full_res_id);
        }
        monitor_leave(self.monitor_id);
    } 

    pub fn consume(&self) {
        monitor_enter(self.monitor_id);
        for _ in 0..10 {
            let inner = self.inner_exclusive_access();
            if inner.full_count == 0 {
                drop(inner);
                monitor_wait(self.monitor_id, self.full_res_id);
            } else {
                drop(inner);
            }
            let mut inner = self.inner_exclusive_access();
            let last_read_ptr = inner.cyc_buf.read;
            let value = inner.cyc_buf.buf[last_read_ptr];
            sleep(5);
            inner.cyc_buf.buf[last_read_ptr] = 0;
            inner.cyc_buf.read = (last_read_ptr + 1) % 6;
            inner.full_count -= 1;
            let history= format!("consumer{} read the value {} from buf{}", gettid(), value, last_read_ptr);
            inner.history.push(history);
            drop(inner);
            monitor_signal(self.monitor_id, self.empty_res_id);
        }
        monitor_leave(self.monitor_id);
    } 

    pub fn print_history(&self) {
        let inner = self.inner_exclusive_access();
        println!("-------------------HISTORY-----------------");
        for his in inner.history.iter() {
            println!("{}",his.as_str());
        }
    }

    pub fn print_cyc_buf(&self) {
        let inner = self.inner_exclusive_access();
        println!("-------------------CYC_BUF-----------------");
        for value in inner.cyc_buf.buf.iter() {
            print!("{} ",value);
        }
        println!("");
    }

    pub fn check_self(&self) -> isize{
        monitor_check(self.monitor_id)
    }
}

lazy_static! {
    static ref monitor: Monitor = Monitor::new();
}

pub fn processor(v: *const i32) {
    let value = unsafe { &*v };
    monitor.process(*value);
    exit(0);
}

pub fn consumer() {
   monitor.consume();
    exit(0);
}

pub fn checker() {
    loop {
        if monitor.check_self() == 1 {
            break;
        }
    }
    exit(0);
}

#[no_mangle]
pub fn main() -> isize {
    let mut consumers = Vec::new();
    let mut processors = Vec::new();
    let values = [1,2,3,4];
    //生产者将会生产20次
    for i in 0..4 {
        processors.push(
            thread_create(processor as usize, &values[i] as *const _ as usize)
        );
    }
    //消费者将会消费40次
    //这时会出现饥饿的情况
    for _ in 0..4 {
        consumers.push(
            thread_create(consumer as usize, 0)
        )
    }

    thread_create(checker as usize, 0);

    for tid in processors.iter() {
        waittid(*tid as usize);
        println!("processor{}:exited", tid);
    }
    for tid in consumers.iter() {
        waittid(*tid as usize);
        println!("consumer{}:exited", tid);
    }
    
    monitor.print_history();
    monitor.print_cyc_buf();
    0
}