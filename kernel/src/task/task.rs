use alloc::{boxed::Box, rc::Weak, sync::Arc, vec::Vec};

use crate::trap::context::UserContext;

use super::{pid::PidHandle, user_space::StackId};

pub struct TaskControlBlock {
    pid: PidHandle,
    parent: Option<Weak<TaskControlBlock>>,
    inner: Arc<TaskControlBlockInner>,
}

pub struct TaskControlBlockInner {
    stack_id: StackId,
    status: TaskStatus,
    context: Box<UserContext>,
    children: Vec<Arc<TaskControlBlock>>,
}

pub enum TaskStatus {
    Uninit,
    Ready,
    Running,
    Sleeping,
    Zombie,
}