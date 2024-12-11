//! Task management implementation
//!
//! Everything about task management, like starting and switching tasks is
//! implemented here.
//!
//! A single global instance of [`TaskManager`] called `TASK_MANAGER` controls
//! all the tasks in the whole operating system.
//!
//! A single global instance of [`Processor`] called `PROCESSOR` monitors running
//! task(s) for each core.
//!
//! A single global instance of [`PidAllocator`] called `PID_ALLOCATOR` allocates
//! pid for user apps.
//!
//! Be careful when you see `__switch` ASM function in `switch.S`. Control flow around this function
//! might not be what you expect.
mod context;
mod manager;
mod id;
mod processor;
mod switch;
mod process;
mod thread;

use crate::loader::get_app_data_by_name;
use crate::sbi::shutdown;
use alloc::sync::Arc;
use lazy_static::*;
use manager::{remove_from_pid2process, remove_task};
pub use manager::{fetch_task, TaskManager};
use process::ProcessControlBlock;
use switch::__switch;
pub use thread::{ThreadControlBlock, TaskStatus};

pub use context::TaskContext;
pub use manager::{add_task, wakeup_task};
pub use id::{pid_alloc, KernelStack, RecycleAllocator, PidHandle};
pub use processor::{
    current_task, current_trap_cx, current_user_token, run_tasks, schedule, take_current_task,
    current_user_process, current_trap_cx_va,
    Processor,
};
/// Suspend the current 'Running' task and run the next task in task list.
pub fn suspend_current_and_run_next() {
    // There must be an application running.
    let task = take_current_task().unwrap();

    // ---- access current TCB exclusively
    let mut task_inner = task.inner_exclusive_access();
    let task_cx_ptr = &mut task_inner.task_cx as *mut TaskContext;
    // Change status to Ready
    task_inner.task_status = TaskStatus::Ready;
    drop(task_inner);
    // ---- release current PCB

    // push back to ready queue.
    add_task(task);
    // jump to scheduling cycle
    schedule(task_cx_ptr);
}

/// pid of usertests app in make run TEST=1
pub const IDLE_PID: usize = 0;

/// Exit the current 'Running' task and run the next task in task list.
pub fn exit_current_and_run_next(exit_code: i32) {
    // take from Processor
    let task = take_current_task().unwrap();

    let mut task_inner = task.inner_exclusive_access();
    let tid = task_inner.res.as_ref().unwrap().tid;
    let process = task.process.upgrade().unwrap();
    task_inner.exit_code = Some(exit_code);
    task_inner.res = None;
    drop(task_inner);
    drop(task);

    if tid == 0 {
        let pid = process.getpid();
        if pid == IDLE_PID {
            println!(
                "[kernel] Idle process exit with exit_code {} ...",
                exit_code
            );
            if exit_code != 0 {
                //crate::sbi::shutdown(255); //255 == -1 for err hint
                shutdown(true);
            } else {
                //crate::sbi::shutdown(0); //0 for success hint
                shutdown(false);
            }
        }
        remove_from_pid2process(pid);
        let mut process_inner = process.inner_exclusive_access();
        process_inner.is_zombie = true;
        process_inner.exit_code = exit_code;

        {
            let mut initproc_inner = INITPROC.inner_exclusive_access();
            for child in process_inner.children.iter() {
                child.inner_exclusive_access().parent = Some(Arc::downgrade(&INITPROC));
                initproc_inner.children.push(child.clone());
            }
        }

        for thread in process_inner.threads.iter().filter(|t| t.is_some()) {
            let thread = thread.as_ref().unwrap();
            remove_inactive_task(thread.clone());
        }

        process_inner.children.clear();
        while process_inner.threads.len() > 1 {
            process_inner.threads.pop();
        }
        process_inner.memory_set.recycle_data_pages();
    }
    drop(process);
    // we do not have to save task context
    let mut _unused = TaskContext::zero_init();
    schedule(&mut _unused as *mut _);
}

pub fn block_current_and_run_next() {
    let thread = take_current_task().unwrap();
    let mut thread_inner = thread.inner_exclusive_access();
    let task_cx_ptr = &mut thread_inner.task_cx as *mut TaskContext;
    thread_inner.task_status = TaskStatus::Blocked;
    drop(thread_inner);
    schedule(task_cx_ptr);
}

lazy_static! {
    ///Globle process that init user shell
    pub static ref INITPROC: Arc<ProcessControlBlock> =ProcessControlBlock::new(
        get_app_data_by_name("initproc").unwrap()
    );
}
///Add init process to the manager
pub fn add_initproc() {
    let _initproc = INITPROC.clone();
}

pub fn remove_inactive_task(task: Arc<ThreadControlBlock>) {
    remove_task(task.clone());
}
