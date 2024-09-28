use alloc::{boxed::Box, rc::Weak, sync::Arc, vec::Vec};

use crate::trap::context::UserContext;

use super::{pid::{alloc_pid, PidHandle}, user_space::{alloc_stack_id, StackId}};

pub struct TaskControlBlock {
    pid: PidHandle,
    parent: Option<Weak<TaskControlBlock>>,
    inner: Arc<TaskControlBlockInner>,
}

impl TaskControlBlock {
    pub fn new() -> Arc<Self> {
        todo!()
    }
}

pub struct TaskControlBlockInner {
    stack_id: StackId,
    status: TaskStatus,
    context: Box<UserContext>,
    children: Vec<Arc<TaskControlBlock>>,
}

impl TaskControlBlockInner {
    pub fn new() -> Self {
        Self {
            stack_id: alloc_stack_id(),
            status: TaskStatus::Uninit,
            context: Box::new(UserContext::default()),
            children: Vec::new(),
        }
    }
}

pub enum TaskStatus {
    Uninit,
    Ready,
    Running,
    Sleeping,
    Zombie,
}