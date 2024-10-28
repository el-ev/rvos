use riscv::interrupt::supervisor::Interrupt;

use crate::timer;

use super::context::KernelContext;

pub fn handle_interrupt(_ctx: &mut KernelContext, i: Interrupt) {
    match i {
        Interrupt::SupervisorTimer => timer_interrupt(),
        Interrupt::SupervisorSoft => {},
        Interrupt::SupervisorExternal => todo!(),
    }
}

fn timer_interrupt() {
    timer::tick();
}
