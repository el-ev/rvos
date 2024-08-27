use riscv::register::scause::Exception;

use super::context::Context;

pub fn handle_exception(_ctx: &mut Context, e: Exception) {
    match e {
        _ => panic!("unhandled exception: {:?}!", e),
    }
}
