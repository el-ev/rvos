use alloc::{collections::vec_deque::VecDeque, sync::Arc, vec::Vec};
use log::info;
use sync::Lazy;

use crate::Mutex;

use super::taskdef::TaskControlBlock;

pub static SCHEDULER: Lazy<Scheduler> = Lazy::new(Scheduler::new);

pub struct Processor {
    hart_id: usize,
    current_task: Option<Arc<TaskControlBlock>>,
}

impl Processor {
    pub const fn new(hart_id: usize) -> Self {
        Self {
            hart_id,
            current_task: None,
        }
    }

    pub fn current(&self) -> Option<Arc<TaskControlBlock>> {
        self.current_task.clone()
    }

    pub fn take_current(&mut self) -> Option<Arc<TaskControlBlock>> {
        self.current_task.take()
    }
}

pub struct Scheduler {
    processors: Mutex<Vec<Processor>>,
    tasks: Mutex<VecDeque<Arc<TaskControlBlock>>>,
}

impl Scheduler {
    pub fn new() -> Self {
        Self {
            processors: Mutex::new(Vec::new()),
            tasks: Mutex::new(VecDeque::new()),
        }
    }

    pub fn add_processor(&self, processor: Processor) {
        info!("Add processor {}", processor.hart_id);
        self.processors.lock().push(processor);
    }

    pub fn add_task(&self, task: Arc<TaskControlBlock>) {
        self.tasks.lock().push_back(task);
    }

    pub fn get_task(&self) -> Option<Arc<TaskControlBlock>> {
        self.tasks.lock().pop_front()
    }

    pub fn main_loop(&self) -> ! {
        loop {
            if let Some(task) = self.get_task() {
            } else {
                panic!("No task to run!");
            }
        }
    }
}
