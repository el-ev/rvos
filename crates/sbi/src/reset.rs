use crate::{sbi_call, Sbiret};

const SBI_EXT_RESET: u64 = 0x53525354;

const FID_SYSTEM_RESET: u64 = 0x0;

pub fn sbi_system_reset(reset_type: u64, reset_reason: u64) -> Sbiret {
    sbi_call(SBI_EXT_RESET, FID_SYSTEM_RESET, reset_type, reset_reason, 0)
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
