use core::{arch::asm, sync::atomic::{AtomicBool, AtomicUsize, Ordering}};

use alloc::{boxed::Box, rc::Weak, string::String, sync::Arc, vec::Vec};
use log::{debug, trace};

use crate::{
    mm::{addr::{PhysPageNum, VirtAddr}, address_space::U_STACK_END, paging::page_table::PageTable}, trap::context::UserContext, Mutex
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
    exception_entry: Mutex<VirtAddr>,
    context: Mutex<Box<UserContext>>,
    ipc_info: Mutex<IpcInfo>,
    children: Mutex<Vec<Arc<TaskControlBlock>>>,
    memory: Mutex<UserSpace>,
    status: Mutex<TaskStatus>,
    is_exited: AtomicBool,
    exit_code: AtomicUsize,
    runs: AtomicUsize,
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

    pub fn get_context_mut(&self) -> &'static mut UserContext {
        unsafe { &mut *(&mut **self.context.lock() as *mut UserContext) }
    }

    pub fn get_context_ptr(&self) -> *mut UserContext {
        &mut **self.context.lock() as *mut UserContext
    }

    pub fn syscall_no(&self) -> usize {
        self.get_context().uregs[17]
    }

    pub fn syscall_args(&self) -> [usize; 6] {
        self.get_context().uregs[10..16].try_into().unwrap()
    }

    pub fn set_user_exception_entry(&self, entry: usize) {
        *self.exception_entry.lock() = VirtAddr(entry);
    }

    pub fn exit(&self) {
        if self.is_exited.compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed).is_ok() {
            // self.set_status(TaskStatus::Zombie);
            self.exit_code.store(self.get_context().uregs[10], Ordering::Relaxed);
        }
    }

    pub fn runs(&self) -> usize {
        self.runs.load(Ordering::Relaxed)
    }

    pub fn inc_runs(&self) {
        self.runs.fetch_add(1, Ordering::Relaxed);
    }

    pub fn is_exited(&self) -> bool {
        self.is_exited.load(Ordering::Acquire)
    }

    pub fn exit_code(&self) -> usize {
        self.exit_code.load(Ordering::Relaxed)
    }

    pub fn page_table(&self) -> PageTable {
        unsafe { PageTable::from_ppn(self.memory.lock().page_table.ppn()) }
    }

    pub fn memory(&self) -> &Mutex<UserSpace> {
        &self.memory
    }

    pub fn get_task(self: Arc<TaskControlBlock>, pid: Pid) -> Option<Arc<TaskControlBlock>> {
        if pid == Pid(0) {
            return Some(self.clone());
        }
        let children = self.children.lock();
        for child in children.iter() {
            if child.pid() == pid {
                if child.is_exited() || child.status() == TaskStatus::Uninit {
                    return None;
                }
                return Some(child.clone());
            }
        }
        None
    }
}

impl TaskControlBlock {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            pid: alloc_pid(),
            parent: None,
            exception_entry: Mutex::new(VirtAddr(0)),
            context: Mutex::new(Box::new(UserContext::default())),
            ipc_info: Mutex::new(IpcInfo::new()),
            children: Mutex::new(Vec::new()),
            memory: Mutex::new(UserSpace::new()),
            status: Mutex::new(TaskStatus::Uninit),
            is_exited: AtomicBool::new(false),
            exit_code: AtomicUsize::new(0),
            runs: AtomicUsize::new(0),
        })
    }

    pub fn init(self: Arc<Self>, elf: &[u8], args: Vec<String>) {
        let mut memory = self.memory.lock();
        let entry = memory.map_elf(elf);
        memory.init_heap(1);
        let mut context = self.context.lock();
        context.sepc = entry;
        context.uregs[2] = U_STACK_END;
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
    Sleeping,
    Exited,
}

#[repr(usize)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum IpcStatus {
    NotReceiving = 0,
    Receiving = 1,
}

#[repr(C)]
#[derive(Debug)]
pub struct IpcInfo {
    pub value: usize,
    pub from: usize,
    pub recving: IpcStatus,
    pub dstva: VirtAddr,
    pub perm: usize,
}

impl Default for IpcInfo {
    fn default() -> Self {
        Self::new()
    }
}
impl IpcInfo {
    pub const fn new() -> Self {
        Self {
            value: 0,
            from: 0,
            recving: IpcStatus::NotReceiving,
            dstva: VirtAddr(0),
            perm: 0,
        }
    }
}
