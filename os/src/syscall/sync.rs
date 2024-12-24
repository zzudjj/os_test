use alloc::sync::Arc;
use crate::sync::{HoareMonitor, Mutex, Semaphore};
use crate::task::{block_current_and_run_next, current_task, current_user_process};
use crate::timer::{add_timer, get_time_ms};


///线程睡眠系统调用
pub fn sys_sleep(ms: usize) -> isize {
    let expire_ms = get_time_ms() + ms;
    let thread = current_task().unwrap();
    add_timer(expire_ms, thread);
    block_current_and_run_next();
    0
}

///互斥锁创建系统调用
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

///互斥锁
pub fn sys_mutex_lock(mutex_id: usize) -> isize {
    let process = current_user_process();
    let process_inner = process.inner_exclusive_access();
    let mutex = process_inner.mutex_list[mutex_id].as_ref().unwrap().clone();
    drop(process_inner);
    drop(process);
    mutex.lock();
    0
}

pub fn sys_mutex_unlock(mutex_id: usize) -> isize {
    let process = current_user_process();
    let process_inner = process.inner_exclusive_access();
    let mutex = process_inner.mutex_list[mutex_id].as_ref().unwrap().clone();
    drop(process_inner);
    drop(process);
    mutex.unlock();
    0
}

pub fn sys_mutex_destroy(mutex_id: usize) -> isize {
    let process = current_user_process();
    let mut process_inner = process.inner_exclusive_access();
    process_inner.mutex_list[mutex_id] = None;
    drop(process_inner);
    drop(process);
    0
}

pub fn sys_sem_create(value: isize) -> usize {
    let process = current_user_process();
    let mut process_inner = process.inner_exclusive_access();
    let sem_list = &mut process_inner.sem_list;
    let new_sem = Arc::new(Semaphore::new(value));
    sem_list.push(Some(new_sem));
    let sem_id = sem_list.len() - 1;
    drop(process_inner);
    drop(process);
    sem_id
}

pub fn sys_sem_wait(sem_id: usize) -> isize {
    let process = current_user_process();
    let process_inner = process.inner_exclusive_access();
    let sem = process_inner.sem_list[sem_id].as_ref().unwrap().clone();
    drop(process_inner);
    drop( process);
    sem.sem_wait();
    0
}

pub fn sys_sem_post(sem_id: usize) -> isize {
    let process = current_user_process();
    let process_inner = process.inner_exclusive_access();
    let sem = process_inner.sem_list[sem_id].as_ref().unwrap().clone();
    drop(process_inner);
    drop(process);
    sem.sem_post();
    0
}

pub fn sys_sem_destroy(sem_id: usize) -> isize {
    let process = current_user_process();
    let mut process_inner = process.inner_exclusive_access();
    process_inner.sem_list[sem_id] = None;
    drop(process_inner);
    drop(process);
    0
}

pub fn sys_monitor_create() -> usize {
    let process = current_user_process();
    let mut process_inner = process.inner_exclusive_access();
    let monitor_list = &mut process_inner.monitor_list;
    let new_monitor = Arc::new(HoareMonitor::new());
    monitor_list.push(Some(new_monitor));
    let monitor_id = monitor_list.len() - 1;
    drop(process_inner);
    drop(process);
    monitor_id
}

pub fn sys_monitor_enter(monitor_id: usize) -> isize {
    let process = current_user_process();
    let process_inner = process.inner_exclusive_access();
    let monitor = process_inner.monitor_list[monitor_id].as_ref().unwrap().clone();
    drop(process_inner);
    drop(process);
    monitor.enter();
    0
}

pub fn sys_monitor_leave(monitor_id: usize) -> isize {
    let process = current_user_process();
    let process_inner = process.inner_exclusive_access();
    let monitor = process_inner.monitor_list[monitor_id].as_ref().unwrap().clone();
    drop(process_inner);
    drop(process);
    monitor.leave();
    0
}

pub fn sys_monitor_create_res_sem(monitor_id: usize) -> usize {
    let process = current_user_process();
    let process_inner = process.inner_exclusive_access();
    let monitor = process_inner.monitor_list[monitor_id].as_ref().unwrap().clone();
    drop(process_inner);
    drop(process);
    monitor.create_res_sem()
}

pub fn sys_monitor_wait(monitor_id: usize, res_id: usize) -> isize {
    let process = current_user_process();
    let process_inner = process.inner_exclusive_access();
    let monitor = process_inner.monitor_list[monitor_id].as_ref().unwrap().clone();
    drop(process_inner);
    drop(process);
    monitor.wait(res_id);
    0
}

pub fn sys_monitor_signal(monitor_id: usize, res_id: usize) -> isize {
    let process = current_user_process();
    let process_inner = process.inner_exclusive_access();
    let monitor = process_inner.monitor_list[monitor_id].as_ref().unwrap().clone();
    drop(process_inner);
    drop(process);
    monitor.signal(res_id);
    0
}

pub fn sys_monitor_destroy(monitor_id: usize) -> isize {
    let process = current_user_process();
    let mut process_inner = process.inner_exclusive_access();
    process_inner.monitor_list[monitor_id] = None;
    drop(process_inner);
    drop(process);
    0
}