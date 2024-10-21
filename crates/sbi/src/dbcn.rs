use crate::{Sbiret, sbi_call};

const SBI_EXT_DBCN: u64 = 0x4442434E;

const FID_DEBUG_CONSOLE_WRITE: u64 = 0x0;
const FID_DEBUG_CONSOLE_READ: u64 = 0x1;
const FID_DEBUG_CONSOLE_WRITE_BYTE: u64 = 0x2;

pub fn sbi_debug_console_write(pa: u64, len: u64) -> Sbiret {
    sbi_call(SBI_EXT_DBCN, FID_DEBUG_CONSOLE_WRITE, len, pa, 0)
}

pub fn sbi_debug_console_read(pa: u64, len: u64) -> Sbiret {
    sbi_call(SBI_EXT_DBCN, FID_DEBUG_CONSOLE_READ, len, pa, 0)
}

pub fn sbi_debug_console_write_byte(c: u8) -> Sbiret {
    sbi_call(SBI_EXT_DBCN, FID_DEBUG_CONSOLE_WRITE_BYTE, c as u64, 0, 0)
}
