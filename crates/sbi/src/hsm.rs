use crate::{sbi_call, Sbiret};

const EID_BASE: u64 = 0x48534D;

const FID_HART_START: u64 = 0;
const FID_HART_STOP: u64 = 1;
const FID_HART_GET_STATUS: u64 = 2;
const FID_HART_SUSPEND: u64 = 3;


pub fn sbi_hart_start(hartid: u64, start_addr: u64, opaque: u64) -> Sbiret {
    sbi_call(EID_BASE, FID_HART_START, hartid, start_addr, opaque)
}

pub fn sbi_hart_stop() -> Sbiret {
    sbi_call(EID_BASE, FID_HART_STOP, 0, 0, 0)
}

pub fn sbi_hart_get_status(hartid: u64) -> Sbiret {
    sbi_call(EID_BASE, FID_HART_GET_STATUS, hartid, 0, 0)
}

pub fn sbi_hart_suspend() -> Sbiret {
    sbi_call(EID_BASE, FID_HART_SUSPEND, 0, 0, 0)
}