#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;
extern crate alloc;

use alloc::vec::Vec;
use lazy_static::*;
use user_lib::{exit, Mutex, thread_create, waittid, sleep};


static mut NUM: i32 = 30;
lazy_static! {
    static ref MUTEX: Mutex = Mutex::new();
}


pub fn thread() -> ! {
    for _ in 0..10 {
        MUTEX.lock();
        let n =  unsafe { NUM }  - 1;
        sleep(5);
        unsafe { NUM = n };
        MUTEX.unlock();
    }
    exit(0);
}

#[no_mangle]
pub fn main() -> i32 {
    let mut threads: Vec<isize> = Vec::new();
    for _ in 0..3 {
        threads.push(
            thread_create(thread as usize, 0 as usize)
        );
    }
    for t in threads.iter() {
        let exit_code = waittid(*t as usize);
        println!("thread#{} exited with code {}", t, exit_code);
    }
    println!("NUM:{}",unsafe{NUM});
    0
}
