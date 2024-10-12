use core::arch::asm;

use alloc::{boxed::Box, rc::Weak, string::String, sync::Arc, vec::Vec};

use crate::{Mutex, trap::context::UserContext};

use super::{
    pid::{alloc_pid, Pid, PidHandle},
    user_space::UserSpace,
};

unsafe impl Send for TaskControlBlock {}
unsafe impl Sync for TaskControlBlock {}

pub struct TaskControlBlock {
    pid: PidHandle,
    parent: Option<Weak<TaskControlBlock>>,
    context: Mutex<UserContext>,
    children: Mutex<Vec<Arc<TaskControlBlock>>>,
    memory: Mutex<UserSpace>,
    status: Mutex<TaskStatus>,
    exit_code: i32,
}

impl TaskControlBlock {
    pub fn pid(&self) -> Pid {
        self.pid.pid()
    }
    pub fn status(&self) -> TaskStatus {
        *self.status.lock()
    }
    pub fn set_status(&self, status: TaskStatus) {
        *self.status.lock() = status
    }
}

impl TaskControlBlock {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            pid: alloc_pid(),
            parent: None,
            context: Mutex::new(UserContext::default()),
            children: Mutex::new(Vec::new()),
            memory: Mutex::new(UserSpace::new()),
            status: Mutex::new(TaskStatus::Uninit),
            exit_code: 0,
        })
    }

    pub fn init(self: Arc<Self>, elf: &[u8], args: Vec<String>) {
        let mut memory = self.memory.lock();
        let entry = memory.map_elf(elf);
        let sp = memory.init_stack(args);
        memory.init_heap(1);
        let mut context = self.context.lock();
        context.sepc = entry;
        context.uregs[2] = sp;
        let sstatus: usize;
        unsafe {
            asm!("csrr {0}, sstatus", out(reg) sstatus);
        }
        context.usstatus = sstatus;
        // TODO: arguments
        self.set_status(TaskStatus::Ready);
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum TaskStatus {
    Uninit,
    Ready,
    Running,
    Sleeping,
    Zombie,
}
