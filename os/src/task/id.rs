//!Implementation of [`RecycleAllocator`]
use super::process::ProcessControlBlock;
use crate::config::{KERNEL_STACK_SIZE, PAGE_SIZE, TRAMPOLINE, TRAP_CONTEXT, USER_STACK_SIZE};
use crate::mm::{MapPermission, PhysPageNum, VirtAddr, KERNEL_SPACE};
use crate::sync::UPSafeCell;
use alloc::vec::Vec;
use alloc::sync::{Arc, Weak};
use lazy_static::*;

///通用资源分配器
pub struct RecycleAllocator {
    current: usize, //表示当前可分配的最大标识符
    recycled: Vec<usize>, //保存了已回收的标识符方便再分配
}

impl RecycleAllocator {
    ///创建一个资源分配器
    pub fn new() -> Self {
        RecycleAllocator {
            current: 0,
            recycled: Vec::new(),
        }
    }
    ///分配一个资源
    pub fn alloc(&mut self) -> usize {
        if let Some(id) = self.recycled.pop() {
            id
        } else {
            self.current += 1;
            self.current - 1
        }
    }
    ///回收一个资源
    pub fn dealloc(&mut self, id: usize) {
        assert!(id < self.current);
        assert!(
            !self.recycled.iter().any(|i| *i == id),
            "pid {} has been deallocated!",
            id
        );
        self.recycled.push(id);
    }
}

lazy_static! {
    pub static ref PID_ALLOCATOR: UPSafeCell<RecycleAllocator> =
        unsafe { UPSafeCell::new(RecycleAllocator::new()) };
    pub static ref KSTACK_ALLOCATOR: UPSafeCell<RecycleAllocator> =
        unsafe{ UPSafeCell::new(RecycleAllocator::new()) };
}
///Bind pid lifetime to `PidHandle`
pub struct PidHandle(pub usize);

impl Drop for PidHandle {
    fn drop(&mut self) {
        //println!("drop pid {}", self.0);
        PID_ALLOCATOR.exclusive_access().dealloc(self.0);
    }
}
///Allocate a pid from PID_ALLOCATOR
pub fn pid_alloc() -> PidHandle {
    PidHandle(PID_ALLOCATOR.exclusive_access().alloc())
}

/// Return (bottom, top) of a kernel stack in kernel space.
pub fn kernel_stack_position(app_id: usize) -> (usize, usize) {
    let top = TRAMPOLINE - app_id * (KERNEL_STACK_SIZE + PAGE_SIZE);
    let bottom = top - KERNEL_STACK_SIZE;
    (bottom, top)
}
///Kernelstack for app
pub struct KernelStack {
    kstack_id: usize,
}

impl KernelStack {
    ///Create a kernelstack from pid
    pub fn new() -> Self {
        let kstack_id = KSTACK_ALLOCATOR.exclusive_access().alloc();
        let (kernel_stack_bottom, kernel_stack_top) = kernel_stack_position(kstack_id);
        KERNEL_SPACE.exclusive_access().insert_framed_area(
            kernel_stack_bottom.into(),
            kernel_stack_top.into(),
            MapPermission::R | MapPermission::W,
        );
        KernelStack { kstack_id }
    }
    ///Get the value on the top of kernelstack
    pub fn get_top(&self) -> usize {
        let (_, kernel_stack_top) = kernel_stack_position(self.kstack_id);
        kernel_stack_top
    }
}

pub fn kstack_alloc() -> KernelStack {
    KernelStack::new()
}

impl Drop for KernelStack {
    fn drop(&mut self) {
        let (kernel_stack_bottom, _) = kernel_stack_position(self.kstack_id);
        let kernel_stack_bottom_va: VirtAddr = kernel_stack_bottom.into();
        KERNEL_SPACE
            .exclusive_access()
            .remove_area_with_start_vpn(kernel_stack_bottom_va.into());
        KSTACK_ALLOCATOR.exclusive_access().dealloc(self.kstack_id);
    }
}

///线程资源集合
pub struct TaskUserRes {
    pub tid: usize, //线程标识符
    pub ustack_base: usize, //用户栈基址
    pub process: Weak<ProcessControlBlock>, //所属进程的弱引用
}

impl TaskUserRes {
    ///新建一个线程的资源集合，参数中的布尔值防止重复分配
    pub fn new(ustack_base: usize, process: Arc<ProcessControlBlock>, alloc_user_res: bool) -> Self {
        let tid = process.inner_exclusive_access().alloc_tid();
        let task_user_res = Self {
            tid: tid,
            ustack_base: ustack_base,
            process: Arc::downgrade(&process),
        };
        if alloc_user_res {
            task_user_res.alloc_user_res();
        }
        task_user_res
    }
    ///分配Trap上下文以及用户栈资源
    pub fn alloc_user_res(&self) {
        let process = self.process.upgrade().unwrap();
        let mut process_inner = process.inner_exclusive_access();
        let ustack_bottom = ustack_bottom_from_tid(self.ustack_base, self.tid);
        let ustack_top = ustack_bottom + USER_STACK_SIZE;
        process_inner.memory_set.insert_framed_area(
            ustack_bottom.into(), 
            ustack_top.into(),
            MapPermission::R | MapPermission::U | MapPermission::W,
        );
        let trap_cx_bottom = trap_cx_bottom_from_tid(self.tid);
        let trap_cx_top = trap_cx_bottom + PAGE_SIZE;
        process_inner.memory_set.insert_framed_area(
            trap_cx_bottom.into(),
             trap_cx_top.into(),
             MapPermission::R | MapPermission::W,
            );
    }
    ///回收Trap上下文以及用户栈资源
    pub fn dealloc_user_res(&self) {
        let process = self.process.upgrade().unwrap();
        let mut process_inner = process.inner_exclusive_access();
        let ustack_bottom = ustack_bottom_from_tid(self.ustack_base, self.tid);
        process_inner.memory_set.remove_area_with_start_vpn(ustack_bottom.into());
        let trap_cx_bottom = trap_cx_bottom_from_tid(self.tid);
        process_inner.memory_set.remove_area_with_start_vpn(trap_cx_bottom.into());
    }
    ///回收线程标识符
    pub fn dealloc_tid(&self) {
        let process = self.process.upgrade().unwrap();
        let mut process_inner = process.inner_exclusive_access();
        process_inner.dealloc_tid(self.tid);
    }
    ///线程Trap上下文地址
    pub fn trap_cx_user_va(&self) -> usize {
        trap_cx_bottom_from_tid(self.tid)
    }
    ///线程Trap上下文物理页号
    pub fn trap_cx_ppn(&self) -> PhysPageNum {
        let process = self.process.upgrade().unwrap();
        let process_inner = process.inner_exclusive_access();
        let trap_cx_user_va: VirtAddr = trap_cx_bottom_from_tid(self.tid).into();
        process_inner
            .memory_set
            .translate(trap_cx_user_va.into())
            .unwrap()
            .ppn()
    } 
    ///进程用户栈基址
    pub fn ustack_base(&self) -> usize {
        self.ustack_base
    }
    ///线程用户栈顶地址
    pub fn ustack_top(&self) -> usize {
        ustack_bottom_from_tid(self.ustack_base, self.tid) + USER_STACK_SIZE
    }

}

impl Drop for TaskUserRes {
    fn drop(&mut self) {
        self.dealloc_tid();
        self.dealloc_user_res();    
    }
}

///获取线程的Trap上下文地址
fn trap_cx_bottom_from_tid(tid: usize) -> usize {
    TRAP_CONTEXT - PAGE_SIZE * tid
}

///获取线程的用户栈的栈底地址
fn ustack_bottom_from_tid(ustack_base: usize, tid: usize) -> usize {
    ustack_base + (USER_STACK_SIZE + PAGE_SIZE) * tid
}