use core::panic::PanicInfo;

use alloc::{format, string::String};
use log::error;

use crate::mm::layout::{__text_end, __text_start};

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    unsafe {
        riscv::register::sstatus::clear_sie();
    }
    if let Some(location) = info.location() {
        error!(
            "\x1b[1;31mPanicked: \"{}\" at {}:{}{}\x1b[1;0m",
            info.message(),
            location.file(),
            location.line(),
            backtrace()
        );
    } else {
        error!(
            "\x1b[1;31mPanicked: {}{}\x1b[1;0m",
            info.message(),
            backtrace()
        );
    }
    sbi::reset::sbi_shutdown()
}

fn backtrace() -> String {
    let mut result = String::new();
    unsafe {
        let mut current_ra = arch::ra();
        let mut current_fp = arch::fp();
        let mut depth = 0;
        result.push_str("\nBacktrace:\n");
        while current_ra >= __text_start as usize
            && current_ra <= __text_end as usize
            && current_fp != 0
        {
            result.push_str(&format!(
                "  {:02}: RA = 0x{:016x}, FP = 0x{:016x}\n",
                depth,
                current_ra - size_of::<usize>(),
                current_fp
            ));
            current_ra = *(current_fp as *const usize).sub(1);
            current_fp = *(current_fp as *const usize).sub(2);
            depth += 1;
        }
    }
    result
}
