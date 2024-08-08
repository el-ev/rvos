use core::arch::{asm, global_asm};

#[naked]
#[no_mangle]
#[link_section = ".init.boot"]
pub unsafe extern "C" fn _low_entry() -> ! {
    asm!(
        "   
            mv  tp, a0
            li  s0, 0xffffffff00000000
            add a1, a1, s0

            add  t0, tp, 1
            slli t0, t0, 18
            la   sp, {stack}
            add  sp, sp, t0
            add  sp, sp, s0

            la   t0, __boot_page_table
            srli t0, t0, 12
            li   t1, 8 << 60
            or   t0, t0, t1
            csrw satp, t0
            sfence.vma

            la  t1, _high_entry
            add t1, t1, s0
            jr  t1
        ",
        stack = sym KERNEL_STACK,
        options(noreturn)
    )
}

#[naked]
#[no_mangle]
#[link_section = ".text.entry"]
unsafe extern "C" fn _high_entry() -> ! {
    core::arch::asm!(
        "
            la   t0, kernel_main
            jr   t0
        ",
        options(noreturn),
    )
}

global_asm!(
    "   .section .data
        .align 12
    __boot_page_table:
        .quad 0
        .quad 0
        .quad (0x80000 << 10) | 0xcf # 0x0000_0000_8000_0000
        .zero 8 * 507
        .quad (0x80000 << 10) | 0xcf # 0xffff_ffff_8000_0000
        .quad 0
    "
);

#[repr(C, align(4096))]
struct KernelStack([u8; 1 << 18]); // 1MiB stack

#[link_section = ".bss.stack"]
static mut KERNEL_STACK: core::mem::MaybeUninit<[KernelStack; 8]> =
    core::mem::MaybeUninit::uninit();

extern "C" {
    fn __boot_page_table();
}