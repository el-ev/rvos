use log::trace;
use riscv::interrupt::supervisor::Exception;


use super::context::KernelContext;

pub fn handle_exception(ctx: &mut KernelContext, e: Exception) {
    panic!("unhandled exception: {:?}! \n Context: {:?}", e, ctx);
}

pub fn handle_ebreak(ctx: &mut KernelContext) {
    trace!("ebreak at 0x{:x}", ctx.sepc);
    // 32-bit
    ctx.sepc += 2;
}
