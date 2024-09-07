use riscv::register::scause::Exception;

use super::context::KernelContext;

pub fn handle_exception(_ctx: &mut KernelContext, e: Exception) {
    panic!("unhandled exception: {:?}!", e)
}
