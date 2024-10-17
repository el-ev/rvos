use core::{arch::asm, sync::atomic::{AtomicBool, AtomicUsize, Ordering}};

use alloc::{boxed::Box, rc::Weak, string::String, sync::Arc, vec::Vec};
use log::{debug, trace};

use crate::{
    Mutex,
    mm::{addr::PhysPageNum, paging::page_table::PageTable},
    trap::context::UserContext,
};

use super::{
    pid::{Pid, PidHandle, alloc_pid},
    user_space::UserSpace,
};

unsafe impl Send for TaskControlBlock {}
unsafe impl Sync for TaskControlBlock {}

pub struct TaskControlBlock {
    pid: PidHandle,
    parent: Option<Weak<TaskControlBlock>>,
    context: Mutex<Box<UserContext>>,
    children: Mutex<Vec<Arc<TaskControlBlock>>>,
    memory: Mutex<UserSpace>,
    status: Mutex<TaskStatus>,
    is_exited: AtomicBool,
    exit_code: AtomicUsize,
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

    pub fn get_context(&self) -> &'static UserContext {
        unsafe { &*(&**self.context.lock() as *const UserContext) }
    }

    pub fn get_context_ptr(&self) -> *mut UserContext {
        &mut **self.context.lock() as *mut UserContext
    }

    pub fn exit(&self) {
        if self.is_exited.compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed).is_ok() {
            // self.set_status(TaskStatus::Zombie);
            self.exit_code.store(self.get_context().uregs[10], Ordering::Relaxed);
        }
    }

    pub fn is_exited(&self) -> bool {
        self.is_exited.load(Ordering::Acquire)
    }

    pub fn exit_code(&self) -> usize {
        self.exit_code.load(Ordering::Relaxed)
    }

    pub fn page_table(&self) -> PhysPageNum {
        self.memory.lock().page_table.ppn()
    }
}

impl TaskControlBlock {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            pid: alloc_pid(),
            parent: None,
            context: Mutex::new(Box::new(UserContext::default())),
            children: Mutex::new(Vec::new()),
            memory: Mutex::new(UserSpace::new()),
            status: Mutex::new(TaskStatus::Uninit),
            is_exited: AtomicBool::new(false),
            exit_code: AtomicUsize::new(0),
        })
    }

    pub fn init(self: Arc<Self>, elf: &[u8], args: Vec<String>) {
        let mut memory = self.memory.lock();
        let entry = memory.map_elf(elf);
        // TODO: arguments
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
        self.set_status(TaskStatus::Ready);
    }

    pub fn do_exit(&self) {
        // TODO cleanup
        trace!("Task {:?} exited with code {}", self.pid(), self.exit_code());
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum TaskStatus {
    Uninit,
    Ready,
    Running,
    Exited,
}
