use core::sync::atomic::{AtomicUsize, Ordering};

use alloc::{collections::btree_map::BTreeMap, sync::Arc};
use arch::SIEGuard;
use log::{debug, trace, warn};
use riscv::interrupt::{
    Trap,
    supervisor::{Exception, Interrupt},
};
use sync::Lazy;

use crate::{
    Mutex, config,
    error::OsError,
    get_hart_count,
    mm::paging::switch_page_table,
    syscall,
    task::user_space::UserPageFaultType,
    timer,
    trap::{context::UserContext, set_kernel_trap, set_user_trap},
    utils::ring_buffer::RingBuffer,
};

use super::{
    hart::{get_current_task, set_current_task, wake_hart},
    pid::Pid,
    taskdef::{TaskControlBlock, TaskStatus},
};

pub static SCHEDULER: Lazy<Scheduler> = Lazy::new(Scheduler::new);

pub struct Scheduler {
    tasks: Mutex<BTreeMap<Pid, Arc<TaskControlBlock>>>,
    queue: RingBuffer<Arc<TaskControlBlock>, { config::MAX_TASKS }>,
    alive_task_count: AtomicUsize,
}

impl Scheduler {
    pub fn new() -> Self {
        Self {
            tasks: Mutex::new(BTreeMap::new()),
            queue: RingBuffer::new(),
            alive_task_count: AtomicUsize::new(0),
        }
    }

    pub fn submit_task(&self, task: Arc<TaskControlBlock>) -> Result<(), OsError> {
        if self.queue.push(task.clone()).is_ok() {
            self.tasks.lock().insert(task.pid(), task.clone());
            self.alive_task_count.fetch_add(1, Ordering::Release);
            Ok(())
        } else {
            Err(OsError::NoFreeTask)
        }
    }

    fn try_get_task(&self) -> Option<Arc<TaskControlBlock>> {
        let task = self.queue.pop();
        if let Some(task) = task {
            match task.status() {
                TaskStatus::Ready => Some(task),
                TaskStatus::Sleeping => {
                    // The object which makes the task sleep take the responsibility to wake it up (and return it to the scheduler)
                    self.alive_task_count.fetch_sub(1, Ordering::Release);
                    None
                }
                TaskStatus::Running => {
                    panic!(
                        "Task {:?} is running and in queue, should not happen",
                        task.pid()
                    );
                }
                TaskStatus::Exited => {
                    self.tasks.lock().remove(&task.pid());
                    self.alive_task_count.fetch_sub(1, Ordering::Release);
                    None
                }
                TaskStatus::Uninit => {
                    panic!("Task {:?} is uninitialized, should not happen", task.pid());
                }
            }
        } else {
            None
        }
    }

    fn return_task(&self, task: Arc<TaskControlBlock>) {
        match self.queue.push(task) {
            Ok(head) => {
                let target_hart = head % get_hart_count();
                wake_hart(target_hart); // TODO: bad
            }
            Err(_) => {
                panic!("Task queue is full, should not happen");
            }
        }
    }

    pub fn hart_loop(&self) -> ! {
        loop {
            // sbi::legacy::sbi_clear_ipi();
            unsafe { riscv::register::sip::clear_ssoft() };
            if self.queue.is_empty() {
                if self.alive_task_count.load(Ordering::Acquire) == 0 {
                    panic!("No task to run");
                }
                riscv::asm::wfi();
                continue;
            }

            match self.try_get_task() {
                Some(task) => 'taskloop: loop {
                    let priority = task.get_priority();
                    for _ in 0..priority {
                        self.execute(task.clone());
                        if task.is_exited() {
                            debug!("Task {:?} exited, runs: {}", task.pid(), task.runs());
                            task.do_exit();
                            self.tasks.lock().remove(&task.pid());
                            self.alive_task_count.fetch_sub(1, Ordering::Release);
                            break 'taskloop;
                        }
                        if task.get_yield_flag() {
                            break;
                        }
                        if task.status() == TaskStatus::Sleeping {
                            break 'taskloop;
                        }
                    }
                    if task.get_yield_flag() || !self.queue.is_empty() {
                        task.set_yield_flag(false);
                        self.return_task(task);
                        break;
                    }
                },
                None => continue,
            }
        }
    }

    fn execute(&self, task: Arc<TaskControlBlock>) {
        let current_task = get_current_task();
        if !(current_task.is_some() && current_task.unwrap().pid() == task.pid()) {
            switch_page_table(task.page_table().ppn());
            set_current_task(Some(task.clone()));
        }
        task.set_status(TaskStatus::Running);
        timer::set_next_timeout();
        unsafe {
            riscv::register::sie::clear_ssoft();
        }

        unsafe {
            set_user_trap();
            _kernel_to_user(task.get_context_ptr());
            set_kernel_trap();
        }

        if task.status() == TaskStatus::Running {
            // It may be set to Sleeping or Exited by other tasks
            task.set_status(TaskStatus::Ready);
        }
        task.inc_runs();
        let scause = riscv::register::scause::read().cause().try_into().unwrap();
        match scause {
            Trap::Interrupt(i) => match i {
                Interrupt::SupervisorTimer => {}
                Interrupt::SupervisorSoft => {
                    unsafe { riscv::register::sip::clear_ssoft() };
                }
                Interrupt::SupervisorExternal => todo!(),
            },
            Trap::Exception(e) => match e {
                Exception::UserEnvCall => syscall::do_syscall(),
                Exception::LoadPageFault
                | Exception::StorePageFault
                | Exception::InstructionPageFault => {
                    let stval = riscv::register::stval::read();
                    let ty = match e {
                        Exception::LoadPageFault => UserPageFaultType::Read,
                        Exception::StorePageFault => UserPageFaultType::Write,
                        Exception::InstructionPageFault => UserPageFaultType::Execute,
                        _ => unreachable!(),
                    };
                    trace!(
                        "User page fault, pid: {:?}, stval: {:#x}",
                        task.pid(),
                        stval,
                    );
                    match task.memory().lock().handle_page_fault(stval, ty) {
                        Ok(()) => {}
                        Err(_) => {
                            warn!(
                                "User page fault, killed. Pid: {:?}, sepc: {:#x}, stval: {:#x}\n Full context: {:?}",
                                task.pid(),
                                task.get_context().sepc,
                                stval,
                                task.get_context(),
                            );
                            task.exit();
                        }
                    }
                }
                Exception::IllegalInstruction
                | Exception::InstructionFault
                | Exception::InstructionMisaligned => {
                    warn!(
                        "User Illegal instruction, killed. Pid: {:?}, sepc: {:#x}",
                        task.pid(),
                        task.get_context().sepc,
                    );
                    task.exit();
                }
                _ => {
                    panic!(
                        "Unhandled exception: {:?}, pid: {:?}, context: {:?}",
                        e,
                        task.pid(),
                        task.get_context()
                    );
                }
            },
        };
        unsafe {
            riscv::register::sie::set_ssoft();
        }
    }
}

pub fn get_task(pid: Pid) -> Option<Arc<TaskControlBlock>> {
    SCHEDULER.tasks.lock().get(&pid).cloned()
}

unsafe extern "C" {
    fn _kernel_to_user(ctx: *mut UserContext);
}
