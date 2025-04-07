use core::arch::asm;

use super::id::SyscallId;

#[inline(always)]
pub fn syscall_0(id: SyscallId) -> isize {
    let ret: isize;
    unsafe {
        asm!("ecall",
            in("a7") id as usize,
            lateout("a0") ret,
        );
    }
    ret
}

#[inline(always)]
pub fn syscall_1(id: SyscallId, a0: usize) -> isize {
    let ret: isize;
    unsafe {
        asm!("ecall",
            in("a7") id as usize,
            in("a0") a0,
            lateout("a0") ret,
        );
    }
    ret
}

#[inline(always)]
pub fn syscall_2(id: SyscallId, a0: usize, a1: usize) -> isize {
    let ret: isize;
    unsafe {
        asm!("ecall",
            in("a7") id as usize,
            in("a0") a0,
            in("a1") a1,
            lateout("a0") ret,
        );
    }
    ret
}

#[inline(always)]
pub fn syscall_3(id: SyscallId, a0: usize, a1: usize, a2: usize) -> isize {
    let ret: isize;
    unsafe {
        asm!("ecall",
            in("a7") id as usize,
            in("a0") a0,
            in("a1") a1,
            in("a2") a2,
            lateout("a0") ret,
        );
    }
    ret
}

#[inline(always)]
pub fn syscall_4(id: SyscallId, a0: usize, a1: usize, a2: usize, a3: usize) -> isize {
    let ret: isize;
    unsafe {
        asm!("ecall",
            in("a7") id as usize,
            in("a0") a0,
            in("a1") a1,
            in("a2") a2,
            in("a3") a3,
            lateout("a0") ret,
        );
    }
    ret
}

#[inline(always)]
pub fn syscall_5(id: SyscallId, a0: usize, a1: usize, a2: usize, a3: usize, a4: usize) -> isize {
    let ret: isize;
    unsafe {
        asm!("ecall",
            in("a7") id as usize,
            in("a0") a0,
            in("a1") a1,
            in("a2") a2,
            in("a3") a3,
            in("a4") a4,
            lateout("a0") ret,
        );
    }
    ret
}
