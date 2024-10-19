#![no_std]
#![no_main]
#![feature(naked_functions)]

use core::arch::asm;

use userlib_macro::user_main;

#[user_main]
pub fn main() {
    unsafe {
        asm!(
            "addi a0, zero, 0x45",
            "addi a7, zero, 0",
            "ecall"
        )
    }
    loop {
        core::hint::black_box({let  _x = 0;});
    }
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}  
}
