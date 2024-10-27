use core::arch::global_asm;

use context::KernelContext;
use riscv::interrupt::{Trap, supervisor::Exception};
use riscv::register::{
    scause::Scause,
    stvec::{self, TrapMode},
};

pub mod context;
mod exception;
mod interrupt;

global_asm!(include_str!("trap.S"));

#[unsafe(no_mangle)]
#[allow(improper_ctypes_definitions)]
pub extern "C" fn kernel_trap_handler(context: &mut KernelContext, scause: Scause, _stval: usize) {
    //trace!("Kernel trap handler: {:?}", scause.cause());
    match scause.cause().try_into().ok().unwrap() {
        Trap::Interrupt(i) => interrupt::handle_interrupt(context, i),
        Trap::Exception(Exception::Breakpoint) => exception::handle_ebreak(context),
        Trap::Exception(e) => exception::handle_exception(context, e),
    }
}

pub fn init() {
    set_kernel_trap();
    unsafe {
        riscv::register::sstatus::set_sie();
        riscv::register::sie::set_sext();
        riscv::register::sie::set_ssoft();
    }
}

#[inline(always)]
pub fn set_kernel_trap() {
    unsafe extern "C" {
        fn _kernel_to_kernel_trap();
    }
    unsafe {
        stvec::write(_kernel_to_kernel_trap as usize, TrapMode::Direct);
    }
    // trace!("Kernel trap vector: 0x{:x}", stvec::read().address());
}

#[inline(always)]
pub fn set_user_trap() {
    unsafe extern "C" {
        fn _user_to_kernel_trap();
    }
    unsafe {
        stvec::write(_user_to_kernel_trap as usize, TrapMode::Direct);
    }
    // trace!("User trap vector: 0x{:x}", stvec::read().address());
}
