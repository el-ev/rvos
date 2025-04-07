#![no_std]
extern crate alloc;

use syscall::syscall_panic;

pub mod console;
pub mod consts;
pub mod error;
pub mod syscall;

#[cfg(feature = "allocator")]
pub mod allocator;

#[cfg(feature = "allocator")]
#[global_allocator]
static ALLOCATOR: allocator::Allocator = allocator::Allocator::new();

#[cfg(feature = "allocator")]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    let msg = alloc::format!(
        "User panic: {} at {}:{}",
        _info.message().as_str().unwrap_or("<Unknown>"),
        _info.location().unwrap().file(),
        _info.location().unwrap().line()
    );
    syscall_panic(msg.as_str());
}

#[cfg(not(feature = "allocator"))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    syscall_panic(_info.message().as_str().unwrap_or("<Unknown>"));
}
