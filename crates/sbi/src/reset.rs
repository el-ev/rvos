use crate::{sbi_call, Sbiret};

const SBI_EXT_RESET: u32 = 0x53525354;

pub fn sbi_system_reset(reset_type: u32, reset_reason: u32) -> Sbiret {
    sbi_call(SBI_EXT_RESET as u64, 0, reset_type as u64, reset_reason as u64,  0)
} 

pub fn sbi_shutdown() -> ! {
    sbi_system_reset(0, 0);
    unreachable!()
}

pub fn sbi_cold_reboot() -> ! {
    sbi_system_reset(1, 0);
    unreachable!()
}

pub fn sbi_warm_reboot() -> ! {
    sbi_system_reset(2, 0);
    unreachable!()
}