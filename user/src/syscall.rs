use core::arch::asm;

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
const SYSCALL_LOCK: usize = 502;
const SYSCALL_UNLOCK: usize = 503;
const SYSCALL_SEM_CREATE: usize = 504;
const SYSCALL_SEM_INIT: usize = 505;
const SYSCALL_SEM_WAIT: usize = 506;
const SYSCALL_SEM_POST: usize = 507;
const SYSCALL_SEM_DESTROY: usize = 508;
const SYSCALL_MUTEX_DESTROY: usize = 509;

fn syscall(id: usize, args: [usize; 3]) -> isize {
    let mut ret: isize;
    unsafe {
        asm!(
            "ecall",
            inlateout("x10") args[0] => ret,
            in("x11") args[1],
            in("x12") args[2],
            in("x17") id
        );
    }
    ret
}

pub fn sys_read(fd: usize, buffer: &mut [u8]) -> isize {
    syscall(
        SYSCALL_READ,
        [fd, buffer.as_mut_ptr() as usize, buffer.len()],
    )
}

pub fn sys_write(fd: usize, buffer: &[u8]) -> isize {
    syscall(SYSCALL_WRITE, [fd, buffer.as_ptr() as usize, buffer.len()])
}

pub fn sys_exit(exit_code: i32) -> ! {
    syscall(SYSCALL_EXIT, [exit_code as usize, 0, 0]);
    panic!("sys_exit never returns!");
}

pub fn sys_yield() -> isize {
    syscall(SYSCALL_YIELD, [0, 0, 0])
}

pub fn sys_get_time() -> isize {
    syscall(SYSCALL_GET_TIME, [0, 0, 0])
}

pub fn sys_getpid() -> isize {
    syscall(SYSCALL_GETPID, [0, 0, 0])
}

pub fn sys_fork() -> isize {
    syscall(SYSCALL_FORK, [0, 0, 0])
}

pub fn sys_exec(path: &str) -> isize {
    syscall(SYSCALL_EXEC, [path.as_ptr() as usize, 0, 0])
}

pub fn sys_waitpid(pid: isize, exit_code: *mut i32) -> isize {
    syscall(SYSCALL_WAITPID, [pid as usize, exit_code as usize, 0])
}

pub fn sys_thread_create(entry: usize, arg: usize) -> isize {
    syscall(SYSCALL_THREAD_CREATE, [entry, arg, 0])
}

pub fn sys_gettid() -> isize {
    syscall(SYSCALL_GETTID, [0, 0, 0])
}

pub fn sys_waittid(tid: usize) -> isize {
    syscall(SYSCALL_WAITTID, [tid, 0, 0])
}

pub fn sys_sleep(ms: usize) -> isize {
    syscall(SYSCALL_SLEEP, [ms, 0, 0])
}

pub fn sys_mutex_create() -> usize {
    syscall(SYSCALL_MUTEX_CREATE, [0, 0, 0]) as usize
}

pub fn sys_lock(mutex_id: usize) -> isize {
    syscall(SYSCALL_LOCK, [mutex_id, 0, 0])
}

pub fn sys_unlock(mutex_id: usize) -> isize {
    syscall(SYSCALL_UNLOCK, [mutex_id, 0, 0])
}


pub fn sys_mutex_destroy(mutex_id: usize) -> isize {
    syscall(SYSCALL_MUTEX_DESTROY, [mutex_id, 0, 0])
}

pub fn sys_sem_create() -> usize {
    syscall(SYSCALL_SEM_CREATE, [0, 0, 0]) as usize
}

pub fn sys_sem_init(sem_id:usize, value: isize) -> isize {
    syscall(SYSCALL_SEM_INIT, [sem_id, value as usize, 0])
}

pub fn sys_sem_wait(sem_id: usize) -> isize {
    syscall(SYSCALL_SEM_WAIT, [sem_id, 0, 0])
}

pub fn sys_sem_post(sem_id: usize) -> isize {
    syscall(SYSCALL_SEM_POST, [sem_id, 0, 0])
}

pub fn sys_sem_destroy(sem_id: usize) -> isize {
    syscall(SYSCALL_SEM_DESTROY, [sem_id, 0, 0])
}