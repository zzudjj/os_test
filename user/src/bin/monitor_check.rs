#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;
extern crate alloc;

use alloc::{format, string::String, vec::Vec};
use lazy_static::*;
use user_lib::{gettid, monitor_check, monitor_create, monitor_create_res_sem, monitor_destroy, monitor_enter, monitor_leave, monitor_signal, monitor_wait, sleep, thread_create, waittid, UPSafeCell};

struct CycleBuf {
    read: usize,
    write: usize,
    buf: [i32; 6],
}

struct Monitor {
    monitor_id: usize,
    full_res_id: usize,
    empty_res_id: usize,
    test_res_id: usize,
    full_count: i32,
    history: Vec<String>,
    cyc_buf: CycleBuf,
}

impl Monitor {
    
    pub fn new() -> Self {
        let monitor_id = monitor_create();
        let full_res_id = monitor_create_res_sem(monitor_id);
        let empty_res_id = monitor_create_res_sem(monitor_id);
        let test_res_id = monitor_create_res_sem(monitor_id);
        Self {
            monitor_id: monitor_create(),
            full_res_id: full_res_id,
            empty_res_id: empty_res_id,
            test_res_id: test_res_id,
            full_count: 0,
            history: Vec::new(),
            cyc_buf: CycleBuf {
                read: 0,
                write: 0,
                buf: [0; 6],
            }
        }
    }

    pub fn process(&mut self, value: i32) {
        monitor_enter(self.monitor_id);
        for _ in 0..5 {
            if self.full_count == 6 {
                monitor_wait(self.monitor_id, self.empty_res_id);
            }
            self.cyc_buf.buf[self.cyc_buf.write] = value;
            sleep(5);
            self.cyc_buf.write = (self.cyc_buf.write + 1) % 6;
            self.full_count += 1;
            let last_write_ptr = (self.cyc_buf.write + 6 - 1) % 6;
            let history= format!("processor{} wrote the value {} in buf{}", gettid(), value, last_write_ptr);
            self.history.push(history);

            monitor_signal(self.monitor_id, self.full_res_id);
        }
        monitor_leave(self.monitor_id);
    } 

    pub fn consume(&mut self) {
        monitor_enter(self.monitor_id);
        for _ in 0..5 {
            if self.full_count == 0 {
                monitor_wait(self.monitor_id, self.full_res_id);
            }
            let value = self.cyc_buf.buf[self.cyc_buf.read];
            sleep(5);
            self.cyc_buf.buf[self.cyc_buf.read] = 0;
            self.cyc_buf.read = (self.cyc_buf.read + 1) % 6;
            self.full_count -= 1;
            let last_read_ptr = (self.cyc_buf.read + 6 - 1) % 6;
            let history= format!("consumer{} read the value {} from buf{}", gettid(), value, last_read_ptr);
            self.history.push(history);
            monitor_wait(self.monitor_id, self.test_res_id);
            monitor_signal(self.monitor_id, self.empty_res_id);
        }
        monitor_leave(self.monitor_id);
    } 

    pub fn print_history(&self) {
        for his in self.history.iter() {
            println!("{}",his.as_str());
        }
    }

    pub fn print_cyc_buf(&self) {
        for value in self.cyc_buf.buf.iter() {
            print!("{} ",value);
        }
        println!("");
    }

    pub fn check_self(&self) {
        monitor_check(self.monitor_id);
    }

    pub fn destroy(&self) {
        monitor_destroy(self.monitor_id);
    }
}

lazy_static! {
    static ref monitor: UPSafeCell<Monitor> = unsafe {
        UPSafeCell::new(Monitor::new())
    };
}

pub fn processor(v: *const i32) {
    let value = unsafe { &*v };
    monitor.exclusive_access().process(*value);
}

pub fn consumer() {
   monitor.exclusive_access().consume();
}

pub fn checker() {
    monitor.exclusive_access().check_self();
}

#[no_mangle]
pub fn main() -> isize {
    let mut consumers = Vec::new();
    let mut processors = Vec::new();
    let values = [1,2,3,4];
    for i in 0..4 {
        processors.push(
            thread_create(processor as usize, &values[i] as *const _ as usize)
        );
    }
    for _ in 0..2 {
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
    let monitor_inner = monitor.exclusive_access();
    monitor_inner.print_history();
    monitor_inner.print_cyc_buf();
    monitor_inner.destroy();
    drop(monitor_inner);
    0
}