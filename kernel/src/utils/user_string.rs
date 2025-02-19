use core::fmt::{self, Display};

use alloc::{
    string::{String, ToString},
    sync::Arc,
};

use crate::{
    mm::{addr::VirtAddr, address_space::U_END, consts::PAGE_SIZE},
    task::{taskdef::TaskControlBlock, user_space::UserAreaPerm},
};

#[derive(Clone)]
pub struct UnsafeUserString {
    task: Arc<TaskControlBlock>,
    ptr: *const u8,
    len: Option<usize>,
}

impl UnsafeUserString {
    pub fn new(task: Arc<TaskControlBlock>, ptr: *const u8, len: Option<usize>) -> Self {
        Self { task, ptr, len }
    }

    unsafe fn as_str(&self) -> &str {
        unsafe {
            core::str::from_utf8_unchecked(core::slice::from_raw_parts(self.ptr, self.len.unwrap()))
        }
    }

    #[allow(dead_code)]
    unsafe fn copied(&self) -> String {
        unsafe { self.as_str().to_string() }
    }

    pub fn checked(&self) -> Option<UserString> {
        if self.len.is_none() {
            if self.ptr as usize >= U_END {
                return None;
            }
            let mut ptr = self.ptr as usize;
            let mut len = 0;

            loop {
                if ptr >= U_END {
                    return None;
                }

                let vpn = VirtAddr(ptr).floor_page();
                if !self.task.memory().lock().check_perm(vpn, UserAreaPerm::R) {
                    return None;
                }
                while ptr % PAGE_SIZE != 0 || ptr >= U_END {
                    if unsafe { *(ptr as *const u8) } == 0 {
                        break;
                    }

                    ptr += 1;
                    len += 1;
                }
                if unsafe { *(ptr as *const u8) } == 0 {
                    break;
                }
                ptr += 1;
                len += 1;
            }
            Some(UserString {
                inner: UnsafeUserString {
                    task: self.task.clone(),
                    ptr: self.ptr,
                    len: Some(len),
                },
            })
        } else {
            if self.ptr as usize >= U_END
                || (self.ptr as usize).checked_add(self.len.unwrap()).is_none()
                || (self.ptr as usize + self.len.unwrap()) >= U_END
            {
                return None;
            }
            let mut vpn = VirtAddr(self.ptr as usize).floor_page();
            let end = VirtAddr(self.ptr as usize + self.len.unwrap()).ceil_page();
            while vpn < end {
                if !self.task.memory().lock().check_perm(vpn, UserAreaPerm::R) {
                    return None;
                }
                vpn += 1;
            }
            Some(UserString {
                inner: self.clone(),
            })
        }
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
