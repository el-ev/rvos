use alloc::{collections::vec_deque::VecDeque, sync::Arc, vec::Vec};
use arch::{SIEGuard, tp};
use log::{debug, info, warn};
use riscv::register::{
    scause::{self, Exception, Interrupt, Trap},
    sscratch,
};
use sync::Lazy;

use core::arch::naked_asm;

use crate::{
    Mutex,
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
    tasks: Mutex<VecDeque<Arc<TaskControlBlock>>>,
}

impl Scheduler {
    pub fn new() -> Self {
        Self {
            tasks: Mutex::new(VecDeque::new()),
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
        loop {
            core::hint::spin_loop();
            
            let task: Arc<TaskControlBlock>;
            if let Some(taskk) = self.get_task() {
                task = taskk.clone();
            } else {
                // debug!("Hart {} has no task to run", tp());
                continue;
            }
            //debug!("Hart {} is running task {:?}", tp(), task.pid());
            if task.status() != TaskStatus::Ready {
                panic!("Task {:?} is {:?}, expected Ready", task.pid(), task.status());
            }
            // TODO: Refactor here
            let current_task = get_current_task();
            if !(current_task.is_some() && current_task.unwrap().pid() == task.pid()) {
                switch_page_table(task.page_table());
                set_current_task(task.clone());
            }
            task.set_status(TaskStatus::Running);
            // Switch to user space
            set_user_trap();
            unsafe {
                _kernel_to_user(task.get_context_ptr());
            }
            set_kernel_trap();

            let scause = scause::read();
            match scause.cause() {
                Trap::Interrupt(i) => match i {
                    Interrupt::SupervisorTimer => {timer::tick()},
                    _ => {
                        panic!("Unhandled interrupt: {:?}", i);
                    }
                },
                Trap::Exception(e) => match e {
                    Exception::UserEnvCall => syscall::do_syscall(task.clone()),
                    Exception::LoadPageFault | Exception::StorePageFault | Exception::InstructionPageFault => {
                        // kill for now
                        // TODO Handle page fault
                        warn!(
                            "User page fault, pid: {:?}, sepc: {:#x}",
                            task.pid(),
                            task.get_context().sepc,
                        );
                        task.exit();
                    }
                    Exception::IllegalInstruction | Exception::InstructionFault | Exception::InstructionMisaligned => {
                        warn!(
                            "User Illegal instruction, pid: {:?}, sepc: {:#x}",
                            task.pid(),
                            task.get_context().sepc,
                        );
                        task.exit();
                    }
                    _ => {
                        panic!("Unhandled exception: {:?}, pid: {:?}, context: {:?}", e, task.pid(), task.get_context());
                    }
                },
            };
            if task.is_exited() {
                task.do_exit();
            } else {
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
