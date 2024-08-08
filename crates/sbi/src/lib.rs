#![no_std]
#![allow(dead_code)]
use core::arch::asm;

pub mod hsm;
pub mod base;
pub mod dbcn;
pub mod legacy;
pub mod reset;


#[inline(always)]
pub fn sbi_call(eid: u64, fid: u64, arg0: u64, arg1: u64, arg2: u64) -> Sbiret {
    let error : i64;
    let value : u64;
    unsafe {
        asm! {
            "ecall",
            in("x17") eid,
            in("x16") fid,
            in("x10") arg0,
            in("x11") arg1,
            in("x12") arg2,
            lateout("x10") error,
            lateout("x11") value,
        };
    }
    Sbiret {
        error: match error {
            0 => SbiError::Success,
            -1 => SbiError::ErrFailed,
            -2 => SbiError::ErrNotSupported,
            -3 => SbiError::ErrInvalidParam,
            -4 => SbiError::ErrDenied,
            -5 => SbiError::ErrInvalidAddress,
            -6 => SbiError::ErrAlreadyAvailable,
            -7 => SbiError::ErrAlreadyStarted,
            -8 => SbiError::ErrAlreadyStopped,
            -9 => SbiError::ErrNoShrem,
            _ => panic!("Unknown SBI error code: {}", error),
        },
        value,
    }
}

pub struct Sbiret {
    pub error: SbiError,
    pub value: u64,
}

#[derive(PartialEq, Eq, Debug)]
pub enum SbiError {
    Success = 0,
    ErrFailed = -1,
    ErrNotSupported = -2,
    ErrInvalidParam = -3,
    ErrDenied = -4,
    ErrInvalidAddress = -5,
    ErrAlreadyAvailable = -6,
    ErrAlreadyStarted = -7,
    ErrAlreadyStopped = -8,
    ErrNoShrem = -9
}
impl Sbiret {
    pub fn is_success(&self) -> bool {
        self.error == SbiError::Success
    }
}