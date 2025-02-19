use core::arch::{global_asm, naked_asm};

use crate::timer;
use riscv::register::stvec::{self, TrapMode};

pub mod context;

global_asm!(include_str!("trap.S"));

#[unsafe(link_section = ".text.stvec_table")]
#[naked]
#[repr(align(16))]
pub unsafe extern "C" fn stvec_table() {
    unsafe {
        naked_asm!(
            "
             0: j {exception_handler}
             1: j {default_interrupt_handler} #ssi
             2: j {default_interrupt_handler}
             3: j {default_interrupt_handler} 
             4: j {default_interrupt_handler}
             5: j {timer_handler} # sti
             6: j {default_interrupt_handler}
             7: j {default_interrupt_handler}
             8: j {default_interrupt_handler}
             9: j {default_interrupt_handler} # sei
             10: j {default_interrupt_handler}
             11: j {default_interrupt_handler} 
             12: j {default_interrupt_handler}
             13: j {default_interrupt_handler} # counter-overflow
             14: j {default_interrupt_handler}
             15: j {default_interrupt_handler}
            ",
            exception_handler = sym exception_handler,
            default_interrupt_handler = sym default_interrupt_handler,
            timer_handler = sym timer_handler,
        )
    }
}

pub extern "riscv-interrupt-s" fn exception_handler() {
    let scause = riscv::register::scause::read();
    panic!("unhandled exception: {:?}", scause.cause());
}

pub extern "riscv-interrupt-s" fn default_interrupt_handler() {
    let scause = riscv::register::scause::read();
    panic!("unhandled interrupt: {:?}", scause.cause());
}

pub extern "riscv-interrupt-s" fn timer_handler() {
    timer::tick();
}

pub fn init() {
    unsafe {
        set_kernel_trap();
        riscv::register::sstatus::set_sie();
        riscv::register::sie::set_sext();
        // riscv::register::sie::set_ssoft();
    }
}

#[inline(always)]
pub unsafe fn set_kernel_trap() {
    unsafe {
        stvec::write(stvec_table as usize, TrapMode::Vectored);
    }
}

#[inline(always)]
pub unsafe fn set_user_trap() {
    unsafe extern "C" {
        fn _user_to_kernel_trap();
    }
    unsafe {
        stvec::write(_user_to_kernel_trap as usize, TrapMode::Direct);
    }
}
