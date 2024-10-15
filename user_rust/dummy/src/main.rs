#![no_std]
#![no_main]
#![feature(naked_functions)]

use userlib::user_main;

#[user_main]
pub fn main() {
    loop {}
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}  
}
