use crate::task::{block_current_and_run_next, current_task};
use crate::timer::{add_timer, get_time_ms};


pub fn sys_sleep(ms: usize) -> isize {
    let expire_ms = get_time_ms() + ms;
    let thread = current_task().unwrap();
    add_timer(expire_ms, thread);
    block_current_and_run_next();
    0
}