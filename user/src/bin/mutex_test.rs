#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

extern crate alloc;

use alloc::vec::Vec;

use user_lib::{exit, lock, mutex_create, thread_create, unlock, waittid, sleep};


static mut NUM: i32 = 30;
static mut MUTEX: usize = 0;

pub fn thread() -> ! {
    for _ in 0..10 {
        lock(unsafe { MUTEX });
        let n =  unsafe { NUM }  - 1;
        sleep(5);
        unsafe { NUM = n };
        unlock(unsafe { MUTEX });
    }
    exit(0);
}

#[no_mangle]
pub fn main() -> i32 {
    let mut threads: Vec<isize> = Vec::new();
    unsafe { MUTEX = mutex_create() };
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
