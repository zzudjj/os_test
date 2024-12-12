use alloc::sync::Arc;

use crate::sync::Mutex;
use crate::task::{block_current_and_run_next, current_task, current_user_process};
use crate::timer::{add_timer, get_time_ms};


pub fn sys_sleep(ms: usize) -> isize {
    let expire_ms = get_time_ms() + ms;
    let thread = current_task().unwrap();
    add_timer(expire_ms, thread);
    block_current_and_run_next();
    0
}

pub fn sys_mutex_create() -> usize {
    let process = current_user_process();
    let mut process_inner = process.inner_exclusive_access();
    let new_mutex = Mutex::new();
    process_inner.mutex_list.push(Some(Arc::new(new_mutex)));
    let mutex_id = process_inner.mutex_list.len() - 1;
    drop(process_inner);
    drop(process);
    mutex_id
}

pub fn sys_lock(mutex_id: usize) -> isize {
    let process = current_user_process();
    let process_inner = process.inner_exclusive_access();
    let mutex = process_inner.mutex_list[mutex_id].as_ref().unwrap().clone();
    drop(process_inner);
    drop(process);
    mutex.lock();
    0
}

pub fn sys_unlock(mutex_id: usize) -> isize {
    let process = current_user_process();
    let process_inner = process.inner_exclusive_access();
    let mutex = process_inner.mutex_list[mutex_id].as_ref().unwrap().clone();
    drop(process_inner);
    drop(process);
    mutex.unlock();
    0
}