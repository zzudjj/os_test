//!Implementation of [`TaskManager`]
use super::process::ProcessControlBlock;
use super::{TaskStatus, ThreadControlBlock};
use crate::sync::UPSafeCell;
use crate::timer::remove_timer;
use alloc::collections::btree_map::BTreeMap;
use alloc::collections::VecDeque;
use alloc::sync::Arc;
use lazy_static::*;
///A array of `TaskControlBlock` that is thread-safe
pub struct TaskManager {
    ready_queue: VecDeque<Arc<ThreadControlBlock>>,
}

/// A simple FIFO scheduler.
impl TaskManager {
    ///Creat an empty TaskManager
    pub fn new() -> Self {
        Self {
            ready_queue: VecDeque::new(),
        }
    }
    ///Add a task to `TaskManager`
    pub fn add(&mut self, task: Arc<ThreadControlBlock>) {
        self.ready_queue.push_back(task);
    }
    ///Remove the first task and return it,or `None` if `TaskManager` is empty
    pub fn fetch(&mut self) -> Option<Arc<ThreadControlBlock>> {
        self.ready_queue.pop_front()
    }

    pub fn remove(&mut self, task: Arc<ThreadControlBlock>) {
        if let Some((id, _)) = self
            .ready_queue
            .iter()
            .enumerate()
            .find(|(_, t)| Arc::as_ptr(t) == Arc::as_ptr(&task))
        {
            self.ready_queue.remove(id);
        }
    }
}

lazy_static! {
    pub static ref TASK_MANAGER: UPSafeCell<TaskManager> =
        unsafe { UPSafeCell::new(TaskManager::new()) };
    pub static ref PID2PCB: UPSafeCell<BTreeMap<usize, Arc<ProcessControlBlock>>> = 
        unsafe { UPSafeCell::new(BTreeMap::new()) };
}
///Interface offered to add task
pub fn add_task(task: Arc<ThreadControlBlock>) {
    TASK_MANAGER.exclusive_access().add(task);
}
///Interface offered to pop the first task
pub fn fetch_task() -> Option<Arc<ThreadControlBlock>> {
    TASK_MANAGER.exclusive_access().fetch()
}

pub fn remove_task(task: Arc<ThreadControlBlock>) {
    TASK_MANAGER.exclusive_access().remove(task.clone());
    remove_timer(task.clone());
}

#[allow(unused)]
pub fn pid2process(pid: usize) -> Option<Arc<ProcessControlBlock>> {
    PID2PCB
    .exclusive_access()
    .get(&pid)
    .map(Arc::clone)
}

pub fn insert_into_pid2process(pid: usize, process: Arc<ProcessControlBlock>) {
    PID2PCB
    .exclusive_access()
    .insert(pid, process);
}

pub fn remove_from_pid2process(pid: usize) {
    if PID2PCB
    .exclusive_access()
    .remove(&pid)
    .is_none() {
        panic!("cannot find pid {} in pid2task!", pid);
    }
}

pub fn wakeup_task(task: Arc<ThreadControlBlock>) {
    let mut task_inner = task.inner_exclusive_access();
    task_inner.task_status = TaskStatus::Ready;
    drop(task_inner);
    add_task(task);
}