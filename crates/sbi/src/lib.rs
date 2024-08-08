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
            0 => SbiError::SbiSuccess,
            -1 => SbiError::SbiErrFailed,
            -2 => SbiError::SbiErrNotSupported,
            -3 => SbiError::SbiErrInvalidParam,
            -4 => SbiError::SbiErrDenied,
            -5 => SbiError::SbiErrInvalidAddress,
            -6 => SbiError::SbiErrAlreadyAvailable,
            -7 => SbiError::SbiErrAlreadyStarted,
            -8 => SbiError::SbiErrAlreadyStopped,
            -9 => SbiError::SbiErrNoShrem,
            _ => panic!("Unknown SBI error code: {}", error),
        },
        value,
    }
}

pub struct Sbiret {
    pub error: SbiError,
    pub value: u64,
}

#[derive(PartialEq, Eq)]
pub enum SbiError {
    SbiSuccess = 0,
    SbiErrFailed = -1,
    SbiErrNotSupported = -2,
    SbiErrInvalidParam = -3,
    SbiErrDenied = -4,
    SbiErrInvalidAddress = -5,
    SbiErrAlreadyAvailable = -6,
    SbiErrAlreadyStarted = -7,
    SbiErrAlreadyStopped = -8,
    SbiErrNoShrem = -9
}
impl Sbiret {
    pub fn is_success(&self) -> bool {
        self.error == SbiError::SbiSuccess
    }
}