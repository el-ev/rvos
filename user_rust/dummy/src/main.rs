#![no_std]
#![no_main]
#![feature(naked_functions)]

use core::arch::asm;

use userlib_macro::user_main;

const STR: &str = "Hello, world!\n\x00123123123123123123";
const PANIC_MSG: &str = "Panic!11213123";

#[user_main]
pub fn main() {
    unsafe {
        asm!("addi a0, zero, 0x45", "addi a7, zero, 0", "ecall"); // Putchar 'E'
        asm!(
            "li a7, 1",
            "ecall",
            in("a0") STR.as_ptr(),
            in("a1") STR.len(),
        );
    }
    (0..100000).for_each(|_| {
        unsafe {
            asm!(
                "li a7, 3",
                "ecall",
            )
        }
    });
    unsafe {
        // asm!(
        //     "li a7, 12",
        //     "ecall",
        //     in("a0") PANIC_MSG.as_ptr(),
        // );
        asm!("li a0, 11",
            "sd zero, 0(zero)",)
    }
    loop {
        core::hint::black_box({
            let _x = 0;
        });
    }
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    unsafe {asm!(
        "li a7, 12",
        "ecall",
        in("a0") PANIC_MSG.as_ptr(),
    );}
    loop {
    }
}
