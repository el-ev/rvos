use core::arch::{asm, global_asm};

use crate::config::{KERNEL_OFFSET, CPU_NUM};

#[naked]
#[no_mangle]
#[link_section = ".init.boot"]
pub unsafe extern "C" fn _low_entry() -> ! {
    asm!(
        "   mv  tp, a0
            li  s0, {kernel_offset}
            add a1, a1, s0
            call {set_stack}
            add sp, sp, s0
            call {set_boot_page_table}
            la  t1, _high_entry
            add t1, t1, s0
            jr  t1
        ",
        kernel_offset = const KERNEL_OFFSET,
        set_stack   = sym set_stack,
        set_boot_page_table = sym set_boot_page_table,
        options(noreturn)
    )
}

#[naked]
#[no_mangle]
#[link_section = ".init.boot"]
pub unsafe extern "C" fn _second_boot() -> ! {
    asm!(
        "   mv  tp, a0
            li  s0, {kernel_offset}
            add a1, a1, s0
            call {set_stack}
            add sp, sp, s0
            call {set_boot_page_table}
            la  t1, parking
            add t1, t1, s0
            jr  t1
        ",
        kernel_offset = const KERNEL_OFFSET,
        set_stack   = sym set_stack,
        set_boot_page_table = sym set_boot_page_table,
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

extern "C" {
    fn __boot_page_table();
}

// TODO Replace with a real page table struct
global_asm!(
    "   .section .data
        .align 12
    __boot_page_table:
        .quad 0
        .quad 0
        .quad (0x80000 << 10) | 0xcf # 0x0000_0000_8000_0000
        .zero 8 * 505
        .quad (0x80000 << 10) | 0xcf # 0xffff_ffff_8000_0000
        .zero 8 * 3
    "
);

#[repr(C, align(4096))]
struct KernelStack([u8; 1 << 20]); // 1MiB stack

#[link_section = ".bss.stack"]
static mut KERNEL_STACK: core::mem::MaybeUninit<[KernelStack; CPU_NUM]> =
    core::mem::MaybeUninit::uninit();


#[naked]
unsafe extern "C" fn set_stack(hartid: usize) {
    asm!(
        "   add  t0, a0, 1
            slli t0, t0, 20
            la   sp, {stack}
            add  sp, sp, t0
            ret
        ",
        stack = sym KERNEL_STACK,
        options(noreturn),
    )
}


#[naked]
unsafe extern "C" fn set_boot_page_table(hartid: usize) {
    asm!(
        "   la   t0, __boot_page_table
            srli t0, t0, 12
            li   t1, 8 << 60
            or   t0, t0, t1
            csrw satp, t0
            sfence.vma
            ret
        ",
        options(noreturn),
    )

}

