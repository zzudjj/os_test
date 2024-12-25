//! Implementation of syscalls
//!
//! The single entry point to all system calls, [`syscall()`], is called
//! whenever userspace wishes to perform a system call using the `ecall`
//! instruction. In this case, the processor raises an 'Environment call from
//! U-mode' exception, which is handled as one of the cases in
//! [`crate::trap::trap_handler`].
//!
//! For clarity, each single syscall is implemented as its own function, named
//! `sys_` then the name of the syscall. You can find functions like this in
//! submodules, and you should also implement syscalls this way.
const SYSCALL_READ: usize = 63;
const SYSCALL_WRITE: usize = 64;
const SYSCALL_EXIT: usize = 93;
const SYSCALL_YIELD: usize = 124;
const SYSCALL_GET_TIME: usize = 169;
const SYSCALL_GETPID: usize = 172;
const SYSCALL_FORK: usize = 220;
const SYSCALL_EXEC: usize = 221;
const SYSCALL_WAITPID: usize = 260;
const SYSCALL_THREAD_CREATE: usize = 1000;
const SYSCALL_GETTID: usize = 1001;
const SYSCALL_WAITTID: usize = 1002;
const SYSCALL_SLEEP: usize = 101;
const SYSCALL_MUTEX_CREATE: usize = 501;
const SYSCALL_MUTEX_LOCK: usize = 502;
const SYSCALL_MUTEX_UNLOCK: usize = 503;
const SYSCALL_SEM_CREATE: usize = 504;
const SYSCALL_SEM_WAIT: usize = 506;
const SYSCALL_SEM_POST: usize = 507;
const SYSCALL_SEM_DESTROY: usize = 508;
const SYSCALL_MUTEX_DESTROY: usize = 509;
const SYSCALL_MONITOR_CREATE: usize = 510;
const SYSCALL_MONITOR_ENTER: usize = 511;
const SYSCALL_MONITOR_LEAVE: usize = 512;
const SYSCALL_MONITOR_CREATE_RES_SEM: usize = 513;
const SYSCALL_MONITOR_WAIT: usize = 514;
const SYSCALL_MONITOR_SIGNAL: usize = 515;
const SYSCALL_MONITOR_DESTROY: usize = 516;
const SYSCALL_MONITOR_CHECK: usize = 517;

mod fs;
mod process;
mod thread;
mod sync;

use fs::*;
use process::*;
use thread::*;
use sync::*;
/// handle syscall exception with `syscall_id` and other arguments
pub fn syscall(syscall_id: usize, args: [usize; 3]) -> isize {
    match syscall_id {
        SYSCALL_READ => sys_read(args[0], args[1] as *const u8, args[2]),
        SYSCALL_WRITE => sys_write(args[0], args[1] as *const u8, args[2]),
        SYSCALL_EXIT => sys_exit(args[0] as i32),
        SYSCALL_YIELD => sys_yield(),
        SYSCALL_GET_TIME => sys_get_time(),
        SYSCALL_GETPID => sys_getpid(),
        SYSCALL_FORK => sys_fork(),
        SYSCALL_EXEC => sys_exec(args[0] as *const u8),
        SYSCALL_WAITPID => sys_waitpid(args[0] as isize, args[1] as *mut i32),
        SYSCALL_THREAD_CREATE => sys_thread_create(args[0], args[1]),
        SYSCALL_GETTID => sys_gettid(),
        SYSCALL_WAITTID => sys_waittid(args[0]) as isize,
        SYSCALL_SLEEP => sys_sleep(args[0]),
        SYSCALL_MUTEX_CREATE => sys_mutex_create() as isize,
        SYSCALL_MUTEX_LOCK => sys_mutex_lock(args[0]),
        SYSCALL_MUTEX_UNLOCK => sys_mutex_unlock(args[0]),
        SYSCALL_SEM_CREATE => sys_sem_create(args[0] as isize) as isize,
        SYSCALL_SEM_WAIT => sys_sem_wait(args[0]),
        SYSCALL_SEM_POST => sys_sem_post(args[0]),
        SYSCALL_MUTEX_DESTROY => sys_mutex_destroy(args[0]),
        SYSCALL_SEM_DESTROY => sys_sem_destroy(args[0]),
        SYSCALL_MONITOR_CREATE => sys_monitor_create() as isize,
        SYSCALL_MONITOR_ENTER => sys_monitor_enter(args[0]),
        SYSCALL_MONITOR_LEAVE => sys_monitor_leave(args[0]),
        SYSCALL_MONITOR_CREATE_RES_SEM => sys_monitor_create_res_sem(args[0]) as isize,
        SYSCALL_MONITOR_WAIT => sys_monitor_wait(args[0], args[1]),
        SYSCALL_MONITOR_SIGNAL => sys_monitor_signal(args[0], args[1]),
        SYSCALL_MONITOR_CHECK => sys_monitor_check(args[0]),
        SYSCALL_MONITOR_DESTROY => sys_monitor_destroy(args[0]),
        _ => panic!("Unsupported syscall_id: {}", syscall_id),
    }
}
