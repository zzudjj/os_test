//!Implementation of [`ProcessControlBlock`]
use super::manager::insert_into_pid2process;
use super::{add_task, RecycleAllocator};
use super::{pid_alloc, PidHandle};
use crate::mm::{MemorySet, KERNEL_SPACE};
use crate::sync::{HoareMonitor, Mutex, Semaphore, UPSafeCell};
use crate::trap::{trap_handler, TrapContext};
use crate::task::ThreadControlBlock;
use alloc::sync::{Arc, Weak};
use alloc::vec::Vec;
use core::cell::RefMut;

pub struct ProcessControlBlock {
    // immutable
    pub pid: PidHandle,
    // mutable
    inner: UPSafeCell<ProcessControlBlockInner>,
}

pub struct ProcessControlBlockInner {
    pub is_zombie: bool,
    pub memory_set: MemorySet,
    pub parent: Option<Weak<ProcessControlBlock>>,
    pub children: Vec<Arc<ProcessControlBlock>>,
    pub exit_code: i32,
    pub threads: Vec<Option<Arc<ThreadControlBlock>>>,
    pub mutex_list: Vec<Option<Arc<Mutex>>>,
    pub sem_list: Vec<Option<Arc<Semaphore>>>,
    pub monitor_list: Vec<Option<Arc<HoareMonitor>>>,
    pub thread_res_allocator: RecycleAllocator,
}

impl ProcessControlBlockInner {
    /*
    pub fn get_task_cx_ptr2(&self) -> *const usize {
        &self.task_cx_ptr as *const usize
    }
    */
    pub fn get_user_token(&self) -> usize {
        self.memory_set.token()
    }

    pub fn alloc_tid(&mut self) -> usize {
        self.thread_res_allocator.alloc()
    } 

    pub fn dealloc_tid(&mut self, tid: usize) {
        self.thread_res_allocator.dealloc(tid);
    }

    pub fn thread_count(&self) -> usize {
        self.threads.len()
    }

    pub fn get_task(&self, tid:usize) -> Arc<ThreadControlBlock> {
        self.threads[tid].as_ref().unwrap().clone()
    } 
}

impl ProcessControlBlock {
    pub fn inner_exclusive_access(&self) -> RefMut<'_, ProcessControlBlockInner> {
        self.inner.exclusive_access()
    }
    pub fn new(elf_data: &[u8]) -> Arc<Self> {
        // memory_set with elf program headers/trampoline/trap context/user stack
        let (memory_set, ustack_base, entry_point) = MemorySet::from_elf(elf_data);
        // alloc a pid and a kernel stack in kernel space
        let pid_handle = pid_alloc();
        // push a task context which goes to trap_return to the top of kernel stack
        let process = Arc::new(Self {
            pid: pid_handle,
            inner: unsafe {
                UPSafeCell::new(ProcessControlBlockInner {
                    is_zombie: false,
                    memory_set: memory_set,
                    parent: None,
                    children: Vec::new(),
                    exit_code: 0,
                    threads: Vec::new(),
                    mutex_list: Vec::new(),
                    sem_list: Vec::new(),
                    monitor_list: Vec::new(),
                    thread_res_allocator: RecycleAllocator::new(),
                })
            },
        });
        let main_thread = Arc::new(
            ThreadControlBlock::new(
            Arc::clone(&process), 
            ustack_base, 
            true
        ));
        let main_thread_inner = main_thread.inner_exclusive_access();
        let trap_cx = main_thread_inner.get_trap_cx();
        let ustack_top = main_thread_inner.res.as_ref().unwrap().ustack_top();
        let kstack_top = main_thread.kernel_stack.get_top();
        drop(main_thread_inner);
        *trap_cx = TrapContext::app_init_context(
            entry_point, 
            ustack_top,
            KERNEL_SPACE.exclusive_access().token(),
            kstack_top, 
            trap_handler as usize,
        );
        let mut process_inner = process.inner_exclusive_access();
        process_inner.threads.push(Some(Arc::clone(&main_thread)));
        drop(process_inner);
        add_task(main_thread);
        insert_into_pid2process(process.getpid(), process.clone());
        process
    }

    pub fn exec(&self, elf_data: &[u8]) {
        assert_eq!(self.inner_exclusive_access().thread_count(), 1);
        // memory_set with elf program headers/trampoline/trap context/user stack
        let (memory_set, ustack_base, entry_point) = MemorySet::from_elf(elf_data);
        self.inner_exclusive_access().memory_set = memory_set;
        let task = self.inner_exclusive_access().get_task(0);
        let mut task_inner = task.inner_exclusive_access();
        task_inner.res.as_mut().unwrap().ustack_base = ustack_base;
        task_inner.res.as_mut().unwrap().alloc_user_res();
        task_inner.trap_cx_ppn = task_inner.res.as_mut().unwrap().trap_cx_ppn();
        let user_sp = task_inner.res.as_mut().unwrap().ustack_top();
        // initialize trap_cx
        let trap_cx = TrapContext::app_init_context(
            entry_point,
            user_sp,
            KERNEL_SPACE.exclusive_access().token(),
            task.kernel_stack.get_top(),
            trap_handler as usize,
        );
        *task_inner.get_trap_cx() = trap_cx;
        // **** release inner automatically
    }
    
    pub fn fork(self: &Arc<Self>) -> Arc<Self> {
        // ---- access parent PCB exclusively
        let mut parent_inner = self.inner_exclusive_access();
        assert_eq!(parent_inner.thread_count(), 1);
        // copy user space(include trap context)
        let memory_set = MemorySet::from_existed_user(&parent_inner.memory_set);
        // alloc a pid and a kernel stack in kernel space
        let pid_handle = pid_alloc();
        let child_process = Arc::new(ProcessControlBlock {
            pid: pid_handle,
            inner: unsafe {
                UPSafeCell::new(ProcessControlBlockInner {
                    is_zombie: false,
                    memory_set: memory_set,
                    parent: Some(Arc::downgrade(self)),
                    children: Vec::new(),
                    threads: Vec::new(),
                    mutex_list: Vec::new(),
                    sem_list: Vec::new(),
                    monitor_list: Vec::new(),
                    thread_res_allocator: RecycleAllocator::new(),
                    exit_code: 0,
                })
            },
        });
        // add child
        parent_inner.children.push(child_process.clone());
        let child_main_thread = Arc::new(
            ThreadControlBlock::new(
            child_process.clone(),
            parent_inner
            .get_task(0)
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .ustack_base(),
            false
        ));
        let mut child_process_inner = child_process.inner_exclusive_access();
        child_process_inner.threads.push(Some(Arc::clone(&child_main_thread)));
        drop(child_process_inner);
        let child_main_thread_inner = child_main_thread.inner_exclusive_access();
        let trap_cx = child_main_thread_inner.get_trap_cx();
        trap_cx.kernel_sp = child_main_thread.kernel_stack.get_top();
        drop(child_main_thread_inner);
        add_task(child_main_thread);
        insert_into_pid2process(child_process.getpid(), child_process.clone());
        child_process
    }

    pub fn getpid(&self) -> usize {
        self.pid.0
    }
}