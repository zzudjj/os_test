#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;
extern crate alloc;
use alloc::{format, string::String, vec::Vec};
use lazy_static::*;
use user_lib::{exit, gettid, Semaphore, sleep, thread_create, waittid};

///环形缓冲池
struct CycleBuf {
    read: usize,
    write: usize,
    buf: [i32; 6],
}

lazy_static! {
    //互斥信号量MUTEX，实现对缓冲池的互斥使用
    static ref MUTEX: Semaphore = Semaphore::new(1);
    //资源信号量FULL，代表满缓冲区资源
    static ref FULL: Semaphore = Semaphore::new(0);
    //资源信号量EMPTY，代表空缓冲区资源
    static ref EMPTY: Semaphore = Semaphore::new(6);
}
//HISTORY记录缓冲池操作历史
static mut HISTORY: Vec<String> = Vec::new();
//环形缓冲池
static mut CYC_BUF: CycleBuf = CycleBuf {
    read: 0,
    write: 0,
    buf: [0; 6],
};
///对缓冲池的写操作
pub fn write_i32(value: i32) {
    unsafe {
        CYC_BUF.buf[CYC_BUF.write] = value;
        sleep(5);
        CYC_BUF.write = (CYC_BUF.write + 1) % 6;
    }
}
///对缓冲池的读操作
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
///生产者线程
pub fn processor(v: *const i32) {
    unsafe {
        for _ in 0..5 {
            let value = &*v;
            EMPTY.wait(); //申请空缓冲区
            MUTEX.wait(); //申请对缓冲池写操作权限
            write_i32(*value);
            let last_write_ptr = (CYC_BUF.write + 6 - 1) % 6;
            let history= format!("processor{} wrote the value {} in buf{}", gettid(), *value, last_write_ptr);
            HISTORY.push(history);
            MUTEX.post(); //释放对缓冲池的锁
            FULL.post(); //唤醒正在等待的消费者线程
        }
    }
    exit(0);
}

pub fn consumer() {
    unsafe {
        for _ in 0..10 {
            FULL.wait();  //申请满缓冲区
            MUTEX.wait(); //申请对缓冲池读操作权限
            let value = read_i32();
            let last_read_ptr = (CYC_BUF.read + 6 - 1) % 6;
            let history= format!("consumer{} read the value {} from buf{}", gettid(), value, last_read_ptr);
            HISTORY.push(history);
            MUTEX.post(); //释放对缓冲池的锁
            EMPTY.post();  //唤醒正在等待的生产者线程
        }
    }
    exit(0);
}


#[no_mangle]
pub fn main() -> isize {
    let mut consumers = Vec::new(); //记录消费者线程标识号
    let mut processors = Vec::new(); //记录生产者线程标识号
    let values = [1,2,3,4]; //对应生产者进行写操作的值
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
    //等待生产者线程结束
    for tid in processors.iter() {
        waittid(*tid as usize);
        println!("processor{}:exited", tid);
    }
    //等待消费者线程结束
    for tid in consumers.iter() {
        waittid(*tid as usize);
        println!("consumer{}:exited", tid);
    }
    //销毁所有的信号量资源
    MUTEX.destroy();
    EMPTY.destroy();
    FULL.destroy();
    //打印缓冲池操作历史以及缓冲池
    unsafe {
        println!("-------------------HISTORY-----------------");
        for history in HISTORY.iter() {
            println!("{}",history.as_str());
        }
        println!("-------------------CYC_BUF-----------------");
        for value in CYC_BUF.buf.iter() {
            print!("{} ",value);
        }
        println!("");
    }
    0
}