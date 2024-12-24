//!Implementation of [`Processor`] and Intersection of control flow
use super::__switch;
use super::process::ProcessControlBlock;
use super::thread::ThreadControlBlock;
use super::{fetch_task, TaskStatus};
use super::TaskContext;
use crate::sync::UPSafeCell;
use crate::trap::TrapContext;
use alloc::sync::Arc;
use lazy_static::*;
///Processor management structure
pub struct Processor {
    ///The task currently executing on the current processor
    current: Option<Arc<ThreadControlBlock>>,
    ///The basic control flow of each core, helping to select and switch process
    idle_task_cx: TaskContext,
}

impl Processor {
    ///Create an empty Processor
    pub fn new() -> Self {
        Self {
            current: None,
            idle_task_cx: TaskContext::zero_init(),
        }
    }
    ///Get mutable reference to `idle_task_cx`
    fn get_idle_task_cx_ptr(&mut self) -> *mut TaskContext {
        &mut self.idle_task_cx as *mut _
    }
    ///Get current task in moving semanteme
    pub fn take_current(&mut self) -> Option<Arc<ThreadControlBlock>> {
        self.current.take()
    }
    ///Get current task in cloning semanteme
    pub fn current(&self) -> Option<Arc<ThreadControlBlock>> {
        self.current.as_ref().map(Arc::clone)
    }
}

lazy_static! {
    pub static ref PROCESSOR: UPSafeCell<Processor> = unsafe { UPSafeCell::new(Processor::new()) };
}
///The main part of process execution and scheduling
///Loop `fetch_task` to get the process that needs to run, and switch the process through `__switch`
pub fn run_tasks() {
    loop {
        let mut processor = PROCESSOR.exclusive_access();
        if let Some(task) = fetch_task() {
            let idle_task_cx_ptr = processor.get_idle_task_cx_ptr();
            // access coming task TCB exclusively
            let mut task_inner = task.inner_exclusive_access();
            let next_task_cx_ptr = &task_inner.task_cx as *const TaskContext;
            task_inner.task_status = TaskStatus::Running;
            drop(task_inner);
            // release coming task TCB manually
            processor.current = Some(task);
            // release processor manually
            drop(processor);
            unsafe {
                __switch(idle_task_cx_ptr, next_task_cx_ptr);
            }
        }
    }
}
///将当前运行线程的线程控制块从处理器管理结构中取出
pub fn take_current_task() -> Option<Arc<ThreadControlBlock>> {
    PROCESSOR.exclusive_access().take_current()
}
///克隆一份当前运行线程的线程控制块返回
pub fn current_task() -> Option<Arc<ThreadControlBlock>> {
    PROCESSOR.exclusive_access().current()
}
///Get token of the address space of current task
pub fn current_user_token() -> usize {
    let task = current_task().unwrap();
    let token = task.get_user_token();
    token
}
///获取当前运行的进程的进程控制块的引用
pub fn current_user_process() -> Arc<ProcessControlBlock> {
    current_task().unwrap().process.upgrade().unwrap()
}
///Get the mutable reference to trap context of current task
pub fn current_trap_cx() -> &'static mut TrapContext {
    current_task()
        .unwrap()
        .inner_exclusive_access()
        .get_trap_cx()
}
///获取当前运行线程的Trap上下文的虚拟地址
pub fn current_trap_cx_va() -> usize {
    current_task()
    .unwrap()
    .inner_exclusive_access()
    .res
    .as_ref()
    .unwrap()
    .trap_cx_user_va()
} 
#[allow(unused)]
pub fn current_kstack_top() -> usize {
    current_task()
    .unwrap()
    .kernel_stack
    .get_top()
}
///Return to idle control flow for new scheduling
pub fn schedule(switched_task_cx_ptr: *mut TaskContext) {
    let mut processor = PROCESSOR.exclusive_access();
    let idle_task_cx_ptr = processor.get_idle_task_cx_ptr();
    drop(processor);
    unsafe {
        __switch(switched_task_cx_ptr, idle_task_cx_ptr);
    }
}
