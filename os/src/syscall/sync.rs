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
    //获取当前运行进程的引用
    let process = current_user_process();
    let mut process_inner = process.inner_exclusive_access();
    //创建新互斥锁
    let new_mutex = Mutex::new();
    //将新互斥锁加入互斥锁队列中
    process_inner.mutex_list.push(Some(Arc::new(new_mutex)));
    let mutex_id = process_inner.mutex_list.len() - 1;
    drop(process_inner);
    drop(process);
    //返回该互斥锁在队列中的位置，即互斥锁标识号
    mutex_id
}
///申请锁系统调用
pub fn sys_mutex_lock(mutex_id: usize) -> isize {
    //获取当前进程的引用
    let process = current_user_process();
    let process_inner = process.inner_exclusive_access();
    //从进程的互斥锁资源队列中根据mutex_id获取互斥锁mutex
    let mutex = process_inner.mutex_list[mutex_id].as_ref().unwrap().clone();
    drop(process_inner);
    drop(process);
    //申请锁
    mutex.lock();
    0
}
///释放锁系统调用
pub fn sys_mutex_unlock(mutex_id: usize) -> isize {
    let process = current_user_process();
    let process_inner = process.inner_exclusive_access();
    let mutex = process_inner.mutex_list[mutex_id].as_ref().unwrap().clone();
    drop(process_inner);
    drop(process);
    mutex.unlock();
    0
}
///销毁锁系统调用
pub fn sys_mutex_destroy(mutex_id: usize) -> isize {
    let process = current_user_process();
    let mut process_inner = process.inner_exclusive_access();
    //消除进程互斥锁资源队列中的指定互斥锁
    process_inner.mutex_list[mutex_id] = None;
    drop(process_inner);
    drop(process);
    0
}
///信号量资源创建系统调用
pub fn sys_sem_create(value: isize) -> usize {
     //获取当前运行进程的引用
    let process = current_user_process();
    let mut process_inner = process.inner_exclusive_access();
    let sem_list = &mut process_inner.sem_list;
     //创建新信号量
    let new_sem = Arc::new(Semaphore::new(value));
     //将新信号量加入进程的信号量资源队列中
    sem_list.push(Some(new_sem));
    let sem_id = sem_list.len() - 1;
    drop(process_inner);
    drop(process);
    //返回该信号量在队列中的位置，即信号量标识号
    sem_id
}
///P操作系统调用
pub fn sys_sem_wait(sem_id: usize) -> isize {
    //获取当前进程的引用
    let process = current_user_process();
    let process_inner = process.inner_exclusive_access();
    //从进程的信号量资源队列中根据sem_id获取信号量sem
    let sem = process_inner.sem_list[sem_id].as_ref().unwrap().clone();
    drop(process_inner);
    drop(process);
    //执行P操作
    sem.sem_wait();
    0
}
///V操作系统调用
pub fn sys_sem_post(sem_id: usize) -> isize {
    let process = current_user_process();
    let process_inner = process.inner_exclusive_access();
    let sem = process_inner.sem_list[sem_id].as_ref().unwrap().clone();
    drop(process_inner);
    drop(process);
    //执行V操作
    sem.sem_post();
    0
}
///信号量资源注销系统调用
pub fn sys_sem_destroy(sem_id: usize) -> isize {
    let process = current_user_process();
    let mut process_inner = process.inner_exclusive_access();
    //消除当前进程信号量资源队列中的指定信号量
    process_inner.sem_list[sem_id] = None;
    drop(process_inner);
    drop(process);
    0
}
///管程资源创建的系统调用
pub fn sys_monitor_create() -> usize {
    let process = current_user_process();
    let mut process_inner = process.inner_exclusive_access();
    let monitor_list = &mut process_inner.monitor_list;
    let new_monitor = Arc::new(HoareMonitor::new());
    //将新的管程资源加入到管程资源管理队列中
    monitor_list.push(Some(new_monitor));
    let monitor_id = monitor_list.len() - 1;
    drop(process_inner);
    drop(process);
    //返回新管程资源的标识符
    monitor_id
}
///进入指定管程系统调用
pub fn sys_monitor_enter(monitor_id: usize) -> isize {
    let process = current_user_process();
    let process_inner = process.inner_exclusive_access();
    //从进程的管程资源管理队列中获取指定的HoareMonitor实例
    let monitor = process_inner.monitor_list[monitor_id].as_ref().unwrap().clone();
    drop(process_inner);
    drop(process);
    //调用管程内部方法，进入管程
    monitor.enter();
    0
}
///离开管程系统调用
pub fn sys_monitor_leave(monitor_id: usize) -> isize {
    let process = current_user_process();
    let process_inner = process.inner_exclusive_access();
    let monitor = process_inner.monitor_list[monitor_id].as_ref().unwrap().clone();
    drop(process_inner);
    drop(process);
    //调用管程内部方法，离开管程
    monitor.leave();
    0
}
///在管程中创建条件变量的系统调用
pub fn sys_monitor_create_res_sem(monitor_id: usize) -> usize {
    let process = current_user_process();
    let process_inner = process.inner_exclusive_access();
    let monitor = process_inner.monitor_list[monitor_id].as_ref().unwrap().clone();
    drop(process_inner);
    drop(process);
    //调用管程内部方法，在管程中创建条件变量
    monitor.create_res_sem()
}
///对指定管程的指定条件变量执行wait操作的系统调用
pub fn sys_monitor_wait(monitor_id: usize, res_id: usize) -> isize {
    let process = current_user_process();
    let process_inner = process.inner_exclusive_access();
    let monitor = process_inner.monitor_list[monitor_id].as_ref().unwrap().clone();
    drop(process_inner);
    drop(process);
    //调用管程内部方法，对指定管程的指定条件变量执行wait操作
    monitor.wait(res_id);
    0
}
///对指定管程的指定条件变量执行signal操作的系统调用
pub fn sys_monitor_signal(monitor_id: usize, res_id: usize) -> isize {
    let process = current_user_process();
    let process_inner = process.inner_exclusive_access();
    let monitor = process_inner.monitor_list[monitor_id].as_ref().unwrap().clone();
    drop(process_inner);
    drop(process);
    //调用管程内部方法，对指定管程的指定条件变量执行signal操作
    monitor.signal(res_id);
    0
}
///对指定管程进行饥饿或死锁检测的系统调用
pub fn sys_monitor_check(monitor_id: usize) -> isize {
    let process = current_user_process();
    let process_inner = process.inner_exclusive_access();
    let monitor = process_inner.monitor_list[monitor_id].as_ref().unwrap().clone();
    drop(process_inner);
    drop(process);
    //调用管程内部方法，对指定管程进行检测
    monitor.check_self();
    0
}
///销毁指定管程资源系统调用
pub fn sys_monitor_destroy(monitor_id: usize) -> isize {
    let process = current_user_process();
    let mut process_inner = process.inner_exclusive_access();
    //在进程的管程资源管理队列中销毁指定管程
    process_inner.monitor_list[monitor_id] = None;
    drop(process_inner);
    drop(process);
    0
}