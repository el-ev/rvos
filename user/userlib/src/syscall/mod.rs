use core::{convert::Infallible, hint::unreachable_unchecked};

use crate::error::ErrorCode;
use id::SyscallId;

mod asm;
mod id;

#[inline(always)]
pub fn syscall_putchar(ch: u8) -> Result<(), ErrorCode> {
    match asm::syscall_1(SyscallId::SysPutchar, ch as usize) {
        0 => Ok(()),
        err => Err(ErrorCode::from(err)),
    }
}

#[inline(always)]
pub fn syscall_print_cons(s: &str) -> Result<(), ErrorCode> {
    match asm::syscall_2(SyscallId::SysPrintCons, s.as_ptr() as usize, s.len()) {
        0 => Ok(()),
        err => Err(ErrorCode::from(err)),
    }
}

#[inline(always)]
pub fn syscall_getenvid() -> Result<usize, Infallible> {
    match asm::syscall_0(SyscallId::SysGetEnvId) {
        envid if envid >= 0 => Ok(envid as usize),
        _ => unreachable!(),
    }
}

#[inline(always)]
pub fn syscall_yield() -> Result<(), Infallible> {
    match asm::syscall_0(SyscallId::SysYield) {
        0 => Ok(()),
        _ => unreachable!(),
    }
}

#[inline(always)]
pub fn syscall_env_destroy(envid: usize) -> Result<(), ErrorCode> {
    match asm::syscall_1(SyscallId::SysEnvDestory, envid) {
        0 => Ok(()),
        err => Err(ErrorCode::from(err)),
    }
}

// TODO VA struct
#[inline(always)]
pub fn syscall_mem_alloc(envid: usize, va: usize, perm: usize) -> Result<(), ErrorCode> {
    match asm::syscall_3(SyscallId::SysMemAlloc, envid, va, perm) {
        0 => Ok(()),
        err => Err(ErrorCode::from(err)),
    }
}

#[inline(always)]
pub fn syscall_mem_map(
    srcenvid: usize,
    srcva: usize,
    dstenvid: usize,
    dstva: usize,
    perm: usize,
) -> Result<(), ErrorCode> {
    match asm::syscall_5(SyscallId::SysMemMap, srcenvid, srcva, dstenvid, dstva, perm) {
        0 => Ok(()),
        err => Err(ErrorCode::from(err)),
    }
}

#[inline(always)]
pub fn syscall_mem_unmap(envid: usize, va: usize) -> Result<(), ErrorCode> {
    match asm::syscall_2(SyscallId::SysMemUnmap, envid, va) {
        0 => Ok(()),
        err => Err(ErrorCode::from(err)),
    }
}

// TODO EnvId Wrapper
#[inline(always)]
pub fn syscall_exofork() -> Result<Option<usize>, ErrorCode> {
    match asm::syscall_0(SyscallId::SysExofork) {
        0 => Ok(None),
        envid if envid >= 0 => Ok(Some(envid as usize)),
        err => Err(ErrorCode::from(err)),
    }
}

#[inline(always)]
pub fn syscall_set_env_status(envid: usize, status: usize) -> Result<(), ErrorCode> {
    match asm::syscall_2(SyscallId::SysSetEnvStatus, envid, status) {
        0 => Ok(()),
        err => Err(ErrorCode::from(err)),
    }
}

#[inline(always)]
pub fn syscall_panic(msg: &str) -> ! {
    asm::syscall_1(SyscallId::SysPanic, msg.as_ptr() as usize);
    unsafe { unreachable_unchecked() }
}

#[inline(always)]
pub fn syscall_ipc_try_send(
    to_envid: usize,
    value: usize,
    srcva: usize,
    perm: usize,
) -> Result<(), ErrorCode> {
    match asm::syscall_4(SyscallId::SysIpcTrySend, to_envid, value, srcva, perm) {
        0 => Ok(()),
        err => Err(ErrorCode::from(err)),
    }
}

#[inline(always)]
pub fn syscall_ipc_recv(dstva: usize) -> Result<isize, Infallible> {
    match asm::syscall_1(SyscallId::SysIpcRecv, dstva) {
        value => Ok(value),
    }
}

#[inline(always)]
pub fn syscall_cgetc() -> Result<u8, Infallible> {
    match asm::syscall_0(SyscallId::SysCGetc) {
        ch if ch >= 0 => Ok(ch as u8),
        _ => unreachable!(),
    }
}

#[inline(always)]
pub fn syscall_write_dev(dev: usize, buf: &[u8]) -> Result<(), ErrorCode> {
    if !buf.len().is_power_of_two() {
        return Err(ErrorCode::Inval);
    }
    match asm::syscall_3(
        SyscallId::SysWriteDev,
        dev,
        buf.as_ptr() as usize,
        buf.len(),
    ) {
        0 => Ok(()),
        err => Err(ErrorCode::from(err)),
    }
}

#[inline(always)]
pub fn syscall_read_dev(dev: usize, buf: &mut [u8]) -> Result<usize, ErrorCode> {
    if !buf.len().is_power_of_two() {
        return Err(ErrorCode::Inval);
    }
    match asm::syscall_3(
        SyscallId::SysReadDev,
        dev,
        buf.as_mut_ptr() as usize,
        buf.len(),
    ) {
        len if len >= 0 => Ok(len as usize),
        err => Err(ErrorCode::from(err)),
    }
}
