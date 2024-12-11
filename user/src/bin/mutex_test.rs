#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

extern crate alloc;

use alloc::vec::Vec;

use user_lib::{thread_create, waittid, mutex_create, lock, unlock};


static mut NUM: i32 = 30;
static mut MUTEX: usize = 0;

pub fn thread() {
    for _ in 0..10 {
        
        unsafe {
            lock(MUTEX);
            NUM =  NUM  - 1;
            unlock(MUTEX);
        }
    }
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
    0
}
