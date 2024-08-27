use core::arch::global_asm;

use context::Context;
use log::debug;
use riscv::register::{
    scause::{Scause, Trap},
    sie,
    stvec::{self, TrapMode},
};

mod context;
mod exception;
mod interrupt;

global_asm!(include_str!("trap.S"));

#[no_mangle]
#[allow(improper_ctypes_definitions)]
pub extern "C" fn kernel_trap_handler(context: &mut Context, scause: Scause, _stval: usize) {
    match scause.cause() {
        Trap::Interrupt(i) => interrupt::handle_interrupt(context, i),
        Trap::Exception(e) => exception::handle_exception(context, e),
    }
}

#[inline(always)]
pub fn set_kernel_trap() {
    extern "C" {
        fn _kernel_trap();
    }
    unsafe {
        stvec::write(_kernel_trap as usize, TrapMode::Direct);
        sie::set_sext();
    }
    debug!("Kernel trap vector: 0x{:x}", stvec::read().address());
}
