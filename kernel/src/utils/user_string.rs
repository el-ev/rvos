use core::fmt::{self, Display};

use alloc::sync::Arc;

use crate::{mm::address_space::U_END, task::taskdef::TaskControlBlock};

pub struct UnsafeUserString {
    task: Arc<TaskControlBlock>,
    ptr: *const u8,
    len: usize,
}

impl UnsafeUserString {
    pub fn new(task: Arc<TaskControlBlock>, ptr: *const u8, len: usize) -> Self {
        Self { task, ptr, len }
    }

    pub unsafe fn as_str(&self) -> &str {
        unsafe { core::str::from_utf8_unchecked(core::slice::from_raw_parts(self.ptr, self.len)) }
    }

    pub fn checked(&self) -> Option<UserString> {
        if self.ptr as usize >= U_END {
            return None;
        }
        
        None
    }
}

pub struct UserString {
    inner: UnsafeUserString,
}

impl Display for UserString {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", unsafe { self.inner.as_str() })
    }
}