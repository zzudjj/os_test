//!Implementation of [`TaskControlBlock`]
use super::id::{kstack_alloc, TaskUserRes};
use super::process::ProcessControlBlock;
use super::TaskContext;
use super::KernelStack;
use crate::mm::PhysPageNum;
use crate::sync::UPSafeCell;
use crate::trap::TrapContext;
use alloc::sync::{Arc, Weak};
use core::cell::RefMut;

pub struct ThreadControlBlock {
    // immutable
    pub process: Weak<ProcessControlBlock>,
    pub kernel_stack: KernelStack,
    // mutable
    inner: UPSafeCell<ThreadControlBlockInner>,
}

pub struct ThreadControlBlockInner {
    pub res: Option<TaskUserRes>,
    pub trap_cx_ppn: PhysPageNum,
    pub task_cx: TaskContext,
    pub task_status: TaskStatus,
    pub exit_code: Option<i32>,
}

impl ThreadControlBlock {
    pub fn new(
        process: Arc<ProcessControlBlock>,
        ustack_base: usize,
        alloc_user_res: bool        
        ) -> Self {
            let task_user_res = TaskUserRes::new(ustack_base, process.clone(), alloc_user_res);
            let trap_cx_ppn = task_user_res.trap_cx_ppn();
            let kernel_stack = kstack_alloc();
            let task_cx = TaskContext::goto_trap_return(kernel_stack.get_top());
            let thread_inner = ThreadControlBlockInner {
                res: Some(task_user_res),
                trap_cx_ppn: trap_cx_ppn,
                task_cx: task_cx,
                task_status: TaskStatus::Ready,
                exit_code: None
            };
            let thread = ThreadControlBlock {
                process: Arc::downgrade(&process),
                kernel_stack: kernel_stack,
                inner: unsafe { UPSafeCell::new(thread_inner) }
            };
            thread
        }

    pub fn inner_exclusive_access(&self) -> RefMut<'_, ThreadControlBlockInner> {
        self.inner.exclusive_access()
    }
    
    pub fn get_user_token(&self) -> usize {
        let process = self.process.upgrade().unwrap();
        let process_innner = process.inner_exclusive_access();
        process_innner.get_user_token()
    }
}

impl ThreadControlBlockInner {
    pub fn get_trap_cx(&self) -> &'static mut TrapContext {
        self.trap_cx_ppn.get_mut()
    }
}

#[derive(Copy, Clone, PartialEq)]
pub enum TaskStatus {
    Ready,
    Running,
    Blocked,
}
