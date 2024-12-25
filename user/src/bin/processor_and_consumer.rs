#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;
extern crate alloc;

use alloc::{format, string::String, vec::Vec};
use user_lib::{exit, gettid, sleep, thread_create, waittid};

struct CycleBuf {
    read: usize,
    write: usize,
    buf: [i32; 6],
}


static mut HISTORY: Vec<String> = Vec::new();
static mut CYC_BUF: CycleBuf = CycleBuf {
    read: 0,
    write: 0,
    buf: [0; 6],
};

pub fn write_i32(value: i32) {
    unsafe {
        CYC_BUF.buf[CYC_BUF.write] = value;
        sleep(5);
        CYC_BUF.write = (CYC_BUF.write + 1) % 6;
    }
}

pub fn read_i32() -> i32 {
    let value: i32;
    unsafe {
        value = CYC_BUF.buf[CYC_BUF.read];
        sleep(5);
        CYC_BUF.buf[CYC_BUF.read] = 0;
        CYC_BUF.read = (CYC_BUF.read + 1) % 6;
    }
    value
}

pub fn processor(v: *const i32) {
    unsafe {
        for _ in 0..5 {
            let value = &*v;
            write_i32(*value);
            let last_write_ptr = (CYC_BUF.write + 6 - 1) % 6;
            let history= format!("processor{} wrote the value {} in buf{}", gettid(), *value, last_write_ptr);
            HISTORY.push(history);
        }
    }
    exit(0);
}

pub fn consumer() {
    unsafe {
        for _ in 0..10 {
            let value = read_i32();
            let last_read_ptr = (CYC_BUF.read + 6 - 1) % 6;
            let history= format!("consumer{} read the value {} from buf{}", gettid(), value, last_read_ptr);
            HISTORY.push(history);
        }
    }
    exit(0);
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
    for tid in processors.iter() {
        waittid(*tid as usize);
        println!("processor{}:exited", tid);
    }
    for tid in consumers.iter() {
        waittid(*tid as usize);
        println!("consumer{}:exited", tid);
    }
    unsafe {
        for history in HISTORY.iter() {
            println!("{}",history.as_str());
        }
        for value in CYC_BUF.buf.iter() {
            print!("{} ",value);
        }
        println!("");
    }
    0
}