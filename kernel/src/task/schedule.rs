use alloc::{collections::vec_deque::VecDeque, sync::Arc, vec::Vec};
use arch::{tp, SIEGuard};
use log::{debug, info};
use riscv::register::{
    scause::{self, Interrupt, Trap},
    sscratch,
};
use sync::Lazy;

use core::arch::naked_asm;

use crate::{
    mm::paging::switch_page_table, print, println, timer, trap::{self, context::UserContext, set_kernel_trap, set_user_trap}, Mutex
};

use super::{
    hart::{get_current_task, set_current_task},
    taskdef::TaskControlBlock,
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
        loop {
            core::hint::spin_loop();
            let task: Arc<TaskControlBlock>;
            if let Some(taskk) = self.get_task() {
                task = taskk.clone();
            } else {
                // debug!("Hart {} has no task to run", tp());
                continue;
            }
            // debug!("Hart {} is running task {:?}", tp(), task.pid());
            // TODO: Refactor here
            let current_task = get_current_task();
            if !(current_task.is_some() && current_task.unwrap().pid() == task.pid()) {
                switch_page_table(task.page_table());
                set_current_task(task.clone());
            }
            set_user_trap();
            unsafe {
                _kernel_to_user(task.get_context());
            }
            set_kernel_trap();
            // TODO: Exception handling here
            let scause = scause::read();
            if scause.cause() == Trap::Interrupt(Interrupt::SupervisorTimer) {
                timer::tick();
            } else {
                debug!("Unhandled exception: {:?}", scause.cause());
            }
            // debug!("Hart {} returned from user task", tp());
            self.add_task(task);
        }
    }
}

unsafe extern "C" {
    fn _kernel_to_user(ctx: *mut UserContext);
    fn _user_to_kernel_trap();
}