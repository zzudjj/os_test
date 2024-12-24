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

///线程控制块
pub struct ThreadControlBlock {
    // 不变量
    pub process: Weak<ProcessControlBlock>, //所属进程的弱引用
    pub kernel_stack: KernelStack, //线程的内核栈
    // 可变量
    inner: UPSafeCell<ThreadControlBlockInner>, 
}

///线程控制块中的可变量集合
pub struct ThreadControlBlockInner {
    pub res: Option<TaskUserRes>, //线程资源集合
    pub trap_cx_ppn: PhysPageNum, //Trap上下文
    pub task_cx: TaskContext, //任务上下文
    pub task_status: TaskStatus, //线程状态
    pub exit_code: Option<i32>, //退出码
}

impl ThreadControlBlock {
    ///创建一个线程控制块
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
    ///获取线程控制块inner成员变量的可变引用
    pub fn inner_exclusive_access(&self) -> RefMut<'_, ThreadControlBlockInner> {
        self.inner.exclusive_access()
    }
    ///获取父进程地址空间的token
    pub fn get_user_token(&self) -> usize {
        let process = self.process.upgrade().unwrap();
        let process_innner = process.inner_exclusive_access();
        process_innner.get_user_token()
    }
}

impl ThreadControlBlockInner {
    ///获取线程的Trap上下文
    pub fn get_trap_cx(&self) -> &'static mut TrapContext {
        self.trap_cx_ppn.get_mut()
    }
}

///线程状态
#[derive(Copy, Clone, PartialEq)]
pub enum TaskStatus {
    Ready, //就绪态
    Running, //运行态
    Blocked, //阻塞态
}
