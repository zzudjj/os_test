use alloc::sync::Arc;

use crate::mm::KERNEL_SPACE;
use crate::task::{add_task, current_task, current_user_process, ThreadControlBlock};
use crate::trap::{trap_handler, TrapContext};


pub fn sys_thread_create(entry: usize, arg:usize) -> isize{
    let process = current_user_process();
    let ustack_base = current_task().unwrap().inner_exclusive_access().res.as_ref().unwrap().ustack_base();
    let new_thread = Arc::new(ThreadControlBlock::new(process.clone(), ustack_base, true));
    let new_thread_inner = new_thread.inner_exclusive_access();
    let new_thread_res = new_thread_inner.res.as_ref().unwrap();
    let new_thread_id = new_thread_res.tid;
    let mut process_inner = process.inner_exclusive_access();
    let threads = &mut process_inner.threads;
    while threads.len() < new_thread_id + 1 {
       threads.push(None);
    }
    threads[new_thread_id] = Some(Arc::clone(&new_thread));
    let new_thread_trap_cx = new_thread_inner.get_trap_cx();
    *new_thread_trap_cx = TrapContext::app_init_context(
        entry,
        new_thread_res.ustack_top(),
        KERNEL_SPACE.exclusive_access().token(),
        new_thread.kernel_stack.get_top(),
        trap_handler as usize,
    );
    (*new_thread_trap_cx).x[10] = arg;
    add_task(new_thread.clone());
    new_thread_id as isize
}

pub fn sys_gettid() -> isize {
    current_task()
    .unwrap()
    .inner_exclusive_access()
    .res
    .as_ref()
    .unwrap()
    .tid as isize
}

pub fn sys_waittid(tid: usize) -> i32 {
    let thread = current_task().unwrap();
    let thread_inner = thread.inner_exclusive_access();
    let process = current_user_process();
    let mut process_inner = process.inner_exclusive_access();
    if tid == thread_inner.res.as_ref().unwrap().tid {
        return -1;
    }
    let mut exit_code = None;
    let waited_thread = process_inner.threads[tid].as_ref();
    if let Some(waited_thread) = waited_thread {
        if let Some(waited_exit_code) = waited_thread.inner_exclusive_access().exit_code {
            exit_code = Some(waited_exit_code);
        }
    } else {
        return -1;
    }
    if let Some(exit_code) = exit_code {
        process_inner.threads[tid] = None;
        exit_code
    } else {
        -2 
    }
}