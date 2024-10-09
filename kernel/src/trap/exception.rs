use log::info;
use riscv::register::scause::Exception;

use super::context::KernelContext;

pub fn handle_exception(_ctx: &mut KernelContext, e: Exception) {
    panic!("unhandled exception: {:?}!", e)
}

pub fn handle_ebreak(ctx: &mut KernelContext) {
    info!("ebreak at 0x{:x}", ctx.sepc);
    // 32-bit
    ctx.sepc += 2;
}