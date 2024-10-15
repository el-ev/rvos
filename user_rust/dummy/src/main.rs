#![no_std]
#![no_main]
#![feature(naked_functions)]

use userlib::user_main;

#[user_main]
pub fn main() {
    loop {
        core::hint::black_box({let x = 0;});
    }
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}  
}
