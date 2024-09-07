use riscv::register::scause::Interrupt;

use crate::{println, timer};

use super::context::Context;

pub fn handle_interrupt(_ctx: &mut Context, i: Interrupt) {
    println!("ctx: {:x?}", _ctx);
    match i {
        Interrupt::SupervisorTimer => timer_interrupt(),
        _ => panic!("unhandled interrupt: {:?}!", i),
    }
}

fn timer_interrupt() {
    timer::tick();
}
