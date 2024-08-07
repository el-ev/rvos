#![no_std]
#![no_main]
#![feature(naked_functions)]

mod init;
mod panic;

#[no_mangle]
extern "C" fn kernel_main() -> ! {
    loop {
        core::hint::spin_loop(); 
    }
}