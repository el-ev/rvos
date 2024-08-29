use riscv::register::scause::Exception;

use super::context::Context;

pub fn handle_exception(_ctx: &mut Context, e: Exception) {
    panic!("unhandled exception: {:?}!", e)
}
