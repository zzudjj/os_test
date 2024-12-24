#![no_std]
#![feature(linkage)]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]

#[macro_use]
pub mod console;
mod lang_items;
mod syscall;

use buddy_system_allocator::LockedHeap;
use syscall::*;

const USER_HEAP_SIZE: usize = 16384;

static mut HEAP_SPACE: [u8; USER_HEAP_SIZE] = [0; USER_HEAP_SIZE];

#[global_allocator]
static HEAP: LockedHeap = LockedHeap::empty();

#[alloc_error_handler]
pub fn handle_alloc_error(layout: core::alloc::Layout) -> ! {
    panic!("Heap allocation error, layout = {:?}", layout);
}

#[no_mangle]
#[link_section = ".text.entry"]
pub extern "C" fn _start() -> ! {
    unsafe {
        HEAP.lock()
            .init(HEAP_SPACE.as_ptr() as usize, USER_HEAP_SIZE);
    }
    exit(main());
}

#[linkage = "weak"]
#[no_mangle]
fn main() -> i32 {
    panic!("Cannot find main!");
}

pub struct Mutex(usize);

impl Mutex {

    pub fn new() -> Self {
        Self(sys_mutex_create())
    }

    pub fn lock(&self) -> isize {
        sys_mutex_lock(self.0)
    }

    pub fn unlock(&self) -> isize {
        sys_mutex_unlock(self.0)
    }

    pub fn destroy(&self) -> isize {
        sys_mutex_destroy(self.0)
    }
}

pub struct Semaphore(usize);

impl Semaphore {
    
    pub fn new(value: isize) -> Self {
        Self(sys_sem_create(value))
    }

    pub fn wait(&self) -> isize {
        sys_sem_wait(self.0)
    }

    pub fn post(&self) -> isize {
        sys_sem_post(self.0)
    }

    pub fn destroy(&self) -> isize {
        sys_sem_destroy(self.0)
    }
}

pub struct HoareMonitor(usize);

impl HoareMonitor {

    pub fn new() -> Self {
        Self(sys_monitor_create())
    }

    pub fn create_res_sem(&self) -> usize {
        sys_monitor_create_res_sem(self.0)
    } 

    pub fn enter(&self) -> isize {
        sys_monitor_enter(self.0)
    }

    pub fn leave(&self) -> isize {
        sys_monitor_leave(self.0)
    }
 
    pub fn wait(&self, res_id: usize) -> isize {
        sys_monitor_wait(self.0, res_id)
    }

    pub fn signal(&self, res_id: usize) -> isize {
        sys_monitor_signal(self.0, res_id)
    }

    pub fn destroy(&self) -> isize {
        sys_monitor_destroy(self.0)
    }
}

pub fn read(fd: usize, buf: &mut [u8]) -> isize {
    sys_read(fd, buf)
}
pub fn write(fd: usize, buf: &[u8]) -> isize {
    sys_write(fd, buf)
}
pub fn exit(exit_code: i32) -> ! {
    sys_exit(exit_code);
}
pub fn yield_() -> isize {
    sys_yield()
}
pub fn get_time() -> isize {
    sys_get_time()
}
pub fn getpid() -> isize {
    sys_getpid()
}
pub fn fork() -> isize {
    sys_fork()
}
pub fn exec(path: &str) -> isize {
    sys_exec(path)
}
pub fn wait(exit_code: &mut i32) -> isize {
    loop {
        match sys_waitpid(-1, exit_code as *mut _) {
            -2 => {
                yield_();
            }
            // -1 or a real pid
            exit_pid => return exit_pid,
        }
    }
}

pub fn waitpid(pid: usize, exit_code: &mut i32) -> isize {
    loop {
        match sys_waitpid(pid as isize, exit_code as *mut _) {
            -2 => {
                yield_();
            }
            // -1 or a real pid
            exit_pid => return exit_pid,
        }
    }
}

pub fn sleep(sleep_ms: usize) {
    sys_sleep(sleep_ms);
}

pub fn thread_create(entry: usize, arg: usize) -> isize {
    sys_thread_create(entry, arg)
}

pub fn gettid() -> isize {
    sys_gettid()
}

pub fn waittid(tid: usize) -> isize {
    loop {
        match sys_waittid(tid) {
            -2 => {
                yield_();
            }
            exit_code => return exit_code,
        }
    }
}

// pub fn mutex_create() -> usize {
//     sys_mutex_create()
// }

// pub fn mutex_lock(mutex_id: usize) -> isize {
//     sys_mutex_lock(mutex_id)
// }

// pub fn mutex_unlock(mutex_id: usize) -> isize {
//     sys_mutex_unlock(mutex_id)
// }

// pub fn mutex_destroy(mutex_id: usize) -> isize {
//     sys_mutex_destroy(mutex_id)
// }

// pub fn sem_create() -> usize {
//     sys_sem_create()
// }

// pub fn sem_init(sem_id: usize, value: isize) -> isize {
//     sys_sem_init(sem_id, value)
// }

// pub fn sem_wait(sem_id: usize) -> isize {
//     sys_sem_wait(sem_id)
// }

// pub fn sem_post(sem_id: usize) -> isize {
//     sys_sem_post(sem_id)
// }

// pub fn sem_destroy(sem_id: usize) -> isize {
//     sys_sem_destroy(sem_id)
// }

// pub fn monitor_create() -> usize {
//     sys_monitor_create()
// }

// pub fn monitor_enter(monitor_id: usize) -> isize {
//     sys_monitor_enter(monitor_id)
// }

// pub fn monitor_leave(monitor_id: usize) -> isize {
//     sys_monitor_leave(monitor_id)
// }

// pub fn monitor_create_res_sem(monitor_id: usize) -> usize {
//     sys_monitor_create_res_sem(monitor_id)
// }

// pub fn monitor_wait(monitor_id: usize, res_id: usize) -> isize {
//     sys_monitor_wait(monitor_id, res_id)
// }

// pub fn monitor_signal(monitor_id: usize, res_id: usize) -> isize {
//     sys_monitor_signal(monitor_id, res_id)
// }

// pub fn monitor_destroy(monitor_id: usize) -> isize {
//     sys_monitor_destroy(monitor_id)
// }

