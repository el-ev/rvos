use crate::{Mutex, utils::pool::UsizePool};

static PID_POOL: Mutex<UsizePool> = Mutex::new(UsizePool::new());

#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd)]
pub struct Pid(pub usize);

impl PartialEq<usize> for Pid {
    fn eq(&self, other: &usize) -> bool {
        self.0 == *other
    }
}

pub struct PidHandle(Pid);

impl PidHandle {
    pub fn pid(&self) -> Pid {
        self.0
    }
}

impl Drop for PidHandle {
    fn drop(&mut self) {
        PID_POOL.lock().dealloc(self.0.0);
    }
}

pub fn alloc_pid() -> PidHandle {
    PidHandle(Pid(PID_POOL.lock().alloc()))
}
