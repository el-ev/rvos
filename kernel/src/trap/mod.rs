use core::{
    arch::{asm, global_asm, naked_asm},
    hint::unreachable_unchecked,
};

use crate::timer;
use riscv::{
    interrupt::{Trap, supervisor::Exception, supervisor::Interrupt},
    register::stvec::{self, TrapMode},
};

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
             1: j {ssoft_handler} #ssi
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
            ssoft_handler = sym ssoft_handler,
            default_interrupt_handler = sym default_interrupt_handler,
            timer_handler = sym timer_handler,
        )
    }
}

// This shouldn't use interrupt abi but it works for now
pub extern "riscv-interrupt-s" fn exception_handler() {
    match riscv::register::scause::read()
        .cause()
        .try_into::<Interrupt, Exception>()
        .unwrap()
    {
        Trap::Exception(e) => match e {
            Exception::Breakpoint => unsafe {
                asm!(
                    "
                    csrr t0, sepc
                    addi t0, t0, 2
                    csrw sepc, t0
                "
                )
            },
            _ => panic!("unhandled exception: {:?}", e),
        },
        Trap::Interrupt(_i) => unsafe { unreachable_unchecked() },
    }
}

pub extern "riscv-interrupt-s" fn default_interrupt_handler() {
    match riscv::register::scause::read()
        .cause()
        .try_into::<Interrupt, Exception>()
        .unwrap()
    {
        Trap::Interrupt(i) => panic!("unhandled interrupt: {:?}", i),
        Trap::Exception(_e) => unsafe { unreachable_unchecked() },
    }
}

#[naked]
pub extern "C" fn ssoft_handler() {
    unsafe {
        naked_asm!(
            "
                addi sp, sp, -16
                sd t0, 0(sp)
                sd t1, 8(sp)
                csrr t0, sip
                li t1, !(1 << 1)
                and t0, t0, t1
                csrw sip, t0
                ld t0, 0(sp)
                ld t1, 8(sp)
                addi sp, sp, 16
                sret
            "
        )
    }
}

pub extern "riscv-interrupt-s" fn timer_handler() {
    timer::tick();
}

pub fn init() {
    unsafe {
        set_kernel_trap();
        riscv::register::sstatus::set_sie();
        riscv::register::sie::set_sext();
        riscv::register::sie::set_ssoft();
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
