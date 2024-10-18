use core::fmt::{self, Display};

use alloc::sync::Arc;

use crate::{mask, mm::{addr::VirtAddr, address_space::U_END, consts::PAGE_SIZE_BITS, paging::pte::PteFlags}, task::{taskdef::TaskControlBlock, user_space::UserAreaPerm}};

#[derive(Clone)]
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
        let mut vpn = VirtAddr(self.ptr as usize).floor_page();
        let end = VirtAddr(self.ptr as usize + self.len).ceil_page();
        while vpn < end {
            // let pte = self.task.page_table().query(vpn);
            // if pte.is_none() || !pte.unwrap().flags().contains(PteFlags::U | PteFlags::V | PteFlags::R) {
            //     return None;
            // }
            if !self.task.memory().lock().check_perm(vpn, UserAreaPerm::R) {
                return None;
            }
            vpn += 1;
        }
        Some(UserString { inner: self.clone() })
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