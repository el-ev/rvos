use crate::{utils::pool::UsizePool, Mutex};

static PID_POOL: Mutex<UsizePool> = Mutex::new(UsizePool::new());

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pid(pub usize);

impl PartialEq<usize> for Pid {
    fn eq(&self, other: &usize) -> bool {
        self.0 == *other
    }
}

pub struct PidWrapper(Pid);

impl PidWrapper {
    pub fn pid(&self) -> Pid {
        self.0
    }
}

impl Drop for PidWrapper {
    fn drop(&mut self) {
        PID_POOL.lock().dealloc(self.0 .0);
    }
}

pub fn alloc_pid() -> PidWrapper {
    PidWrapper(Pid(PID_POOL.lock().alloc()))
}