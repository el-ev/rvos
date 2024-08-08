use core::arch::asm;

const EID_SET_TIMER: u64 = 0x0;
const EID_CONSOLE_PUTCHAR: u64 = 0x1;
const EID_CONSOLE_GETCHAR: u64 = 0x2;
const EID_CLEAR_IPI: u64 = 0x3;
const EID_SEND_IPI: u64 = 0x4;
const EID_REMOTE_FENCE_I: u64 = 0x5;
const EID_REMOTE_SFENCE_VMA: u64 = 0x6;
const EID_REMOTE_SFENCE_VMA_ASID: u64 = 0x7;
const EID_SHUTDOWN: u64 = 0x8;

pub fn sbi_set_timer(stime_value: u64) -> i64 {
    sbi_call_legacy(EID_SET_TIMER, stime_value, 0, 0)
}

pub fn sbi_console_putchar(c: u8) -> i64 {
    sbi_call_legacy(EID_CONSOLE_PUTCHAR, c as u64, 0, 0)
}

pub fn sbi_console_getchar() -> i64 {
    sbi_call_legacy(EID_CONSOLE_GETCHAR, 0, 0, 0)
}

pub fn sbi_shutdown() -> ! {
    sbi_call_legacy(EID_SHUTDOWN, 0, 0, 0);
    unreachable!()
}

#[inline(always)]
pub fn sbi_call_legacy(eid: u64, arg0: u64, arg1: u64, arg2: u64) -> i64 {
    let value : i64;
    unsafe {
        asm! {
            "ecall",
            in("x17") eid,
            in("x10") arg0,
            in("x11") arg1,
            in("x12") arg2,
            lateout("x10") value,
        };
    }
    value
}