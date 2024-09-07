use riscv::register::scause::Interrupt;

use crate::timer;

use super::context::KernelContext;

pub fn handle_interrupt(_ctx: &mut KernelContext, i: Interrupt) {
    match i {
        Interrupt::SupervisorTimer => timer_interrupt(),
        _ => panic!("unhandled interrupt: {:?}!", i),
    }
}

fn timer_interrupt() {
    timer::tick();
}
