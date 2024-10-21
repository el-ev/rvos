use alloc::{collections::vec_deque::VecDeque, sync::Arc, vec::Vec};
use arch::{SIEGuard, tp};
use log::{debug, info, warn};
use riscv::register::{scause, sscratch};
use riscv::interrupt::{Trap, supervisor::{Exception, Interrupt}};

use sync::Lazy;

use core::{
    arch::naked_asm,
    sync::atomic::{AtomicUsize, Ordering},
};

use crate::{
    Mutex,
    config::MAX_TASKS,
    error::OsError,
    mm::paging::switch_page_table,
    print, println, syscall, timer,
    trap::{self, context::UserContext, set_kernel_trap, set_user_trap},
};

use super::{
    hart::{get_current_task, set_current_task},
    taskdef::{TaskControlBlock, TaskStatus},
};

pub static SCHEDULER: Lazy<Scheduler> = Lazy::new(Scheduler::new);

pub struct Scheduler {
    alive_task_count: AtomicUsize,
    tasks: Mutex<VecDeque<Arc<TaskControlBlock>>>,
}

impl Scheduler {
    pub fn new() -> Self {
        Self {
            alive_task_count: AtomicUsize::new(0),
            tasks: Mutex::new(VecDeque::new()),
        }
    }

    pub fn new_task(&self, task: Arc<TaskControlBlock>) -> Result<(), OsError> {
        loop {
            let current_count = self.alive_task_count.load(Ordering::Relaxed);
            if current_count >= MAX_TASKS {
                return Err(OsError::NoFreeTask);
            }
            if self
                .alive_task_count
                .compare_exchange(
                    current_count,
                    current_count + 1,
                    Ordering::Acquire,
                    Ordering::Relaxed,
                )
                .ok()
                == Some(current_count)
            {
                self.tasks.lock().push_back(task);
                return Ok(());
            }
        }
    }

    pub fn add_task(&self, task: Arc<TaskControlBlock>) {
        self.tasks.lock().push_back(task);
    }

    pub fn get_task(&self) -> Option<Arc<TaskControlBlock>> {
        self.tasks.lock().pop_front()
    }

    pub fn main_loop(&self) -> ! {
        // Currently all harts just busy spin until they have a task to run
        // TODO: Better scheduling
        // TODO: Software interrupt
        loop {
            core::hint::spin_loop();
            let sie_guard = SIEGuard::new();
            if self
                .alive_task_count
                .load(core::sync::atomic::Ordering::SeqCst)
                == 0
            {
                panic!("No task to run");
            }

            let task = self.get_task();
            if task.is_none() {
                continue;
            }
            let task = task.unwrap();
            //debug!("Hart {} is running task {:?}", tp(), task.pid());
            match task.status() {
                TaskStatus::Sleeping => {
                    self.add_task(task);
                    continue;
                }
                TaskStatus::Exited => {
                    debug!("Task {:?} exited, runs: {}", task.pid(), task.runs());
                    task.do_exit();
                    self.alive_task_count
                        .fetch_sub(1, core::sync::atomic::Ordering::SeqCst);
                    continue;
                }
                TaskStatus::Ready => {}
                _ => {
                    panic!(
                        "Task {:?} is in an invalid state: {:?}",
                        task.pid(),
                        task.status()
                    );
                }
            }
            let current_task = get_current_task();
            if !(current_task.is_some() && current_task.unwrap().pid() == task.pid()) {
                switch_page_table(task.page_table().ppn());
                set_current_task(Some(task.clone()));
            }
            task.set_status(TaskStatus::Running);
            drop(sie_guard);

            set_user_trap();
            unsafe {
                _kernel_to_user(task.get_context_ptr());
            }
            set_kernel_trap();

            let sie_guard = SIEGuard::new();
            task.inc_runs();
            let scause = scause::read().cause().try_into().unwrap();
            match scause {
                Trap::Interrupt(i) => match i {
                    Interrupt::SupervisorTimer => timer::tick(),
                    _ => {
                        panic!("Unhandled interrupt: {:?}", i);
                    }
                },
                Trap::Exception(e) => match e {
                    Exception::UserEnvCall => syscall::do_syscall(task.clone()),
                    Exception::LoadPageFault
                    | Exception::StorePageFault
                    | Exception::InstructionPageFault => {
                        // kill for now
                        // TODO Handle page fault
                        warn!(
                            "User page fault, killed. Pid: {:?}, sepc: {:#x}, stval: {:#x}",
                            task.pid(),
                            task.get_context().sepc,
                            riscv::register::stval::read()
                        );
                        task.exit();
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
            if task.is_exited() {
                debug!("Task {:?} exited, runs: {}", task.pid(), task.runs());
                task.do_exit();
                self.alive_task_count
                    .fetch_sub(1, core::sync::atomic::Ordering::SeqCst);
            } else {
                set_current_task(None);
                task.set_status(TaskStatus::Ready);
                self.add_task(task);
            }
        }
    }
}

unsafe extern "C" {
    fn _kernel_to_user(ctx: *mut UserContext);
    fn _user_to_kernel_trap();
}
