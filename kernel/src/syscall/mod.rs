#![allow(dead_code)]
#![allow(unused_variables)]

use core::panic;

use alloc::{ffi::c_str, sync::Arc};
use log::trace;

use crate::{
    console::getchar,
    error::OsError,
    mm::{addr::VirtAddr, address_space::is_illegal_user_va_range, consts::PAGE_SIZE},
    print,
    task::{
        pid::Pid,
        schedule,
        taskdef::{IpcStatus, TaskControlBlock, TaskStatus},
        user_space::UserAreaPerm,
    },
};

#[repr(usize)]
enum Syscall {
    Putchar = 0,
    PrintConsole = 1,
    GetTaskId = 2,
    Yield = 3,
    TaskDestroy = 4,
    SetTlbModEntry = 5,
    MemAlloc = 6,
    MemMap = 7,
    MemUnmap = 8,
    Exofork = 9,
    SetEnvStatus = 10,
    SetTrapframe = 11,
    Panic = 12,
    IpcTrySend = 13,
    IpcRecv = 14,
    Getchar = 15,
    WriteDev = 16,
    ReadDev = 17,
    Open = 18,
    Close = 19,
    Read = 20,
    Write = 21,
    Seek = 22,
    Fstat = 23,
    Fsync = 24,
    Ftruncate = 25,
    Remove = 26,
    Unhandled = 255,
}

impl From<usize> for Syscall {
    fn from(value: usize) -> Self {
        match value {
            0 => Syscall::Putchar,
            1 => Syscall::PrintConsole,
            2 => Syscall::GetTaskId,
            3 => Syscall::Yield,
            4 => Syscall::TaskDestroy,
            5 => Syscall::SetTlbModEntry,
            6 => Syscall::MemAlloc,
            7 => Syscall::MemMap,
            8 => Syscall::MemUnmap,
            9 => Syscall::Exofork,
            10 => Syscall::SetEnvStatus,
            11 => Syscall::SetTrapframe,
            12 => Syscall::Panic,
            13 => Syscall::IpcTrySend,
            14 => Syscall::IpcRecv,
            15 => Syscall::Getchar,
            16 => Syscall::WriteDev,
            17 => Syscall::ReadDev,
            18 => Syscall::Open,
            19 => Syscall::Close,
            20 => Syscall::Read,
            21 => Syscall::Write,
            22 => Syscall::Seek,
            23 => Syscall::Fstat,
            24 => Syscall::Fsync,
            25 => Syscall::Ftruncate,
            26 => Syscall::Remove,
            _ => Syscall::Unhandled,
        }
    }
}

pub fn do_syscall() {
    let task = crate::task::hart::get_current_task().unwrap();
    let syscall = Syscall::from(task.syscall_no());
    let ctx = task.get_context_mut();
    ctx.sepc += 4;
    let args = task.syscall_args();
    ctx.uregs[10] = match syscall {
        Syscall::Putchar => sys_putchar(args[0]),
        Syscall::PrintConsole => sys_print_console(task, args[0], args[1]),
        Syscall::GetTaskId => sys_get_task_id(task),
        Syscall::Yield => sys_yield(task),
        Syscall::TaskDestroy => sys_task_destroy(task, args[0]),
        Syscall::SetTlbModEntry => sys_set_tlb_mod_entry(task, args[0], args[1]),
        Syscall::MemAlloc => sys_mem_alloc(task, args[0], args[1], args[2]),
        Syscall::MemMap => sys_mem_map(task, args[0], args[1], args[2], args[3], args[4]),
        Syscall::MemUnmap => sys_mem_unmap(task, args[0], args[1]),
        Syscall::Exofork => sys_exofork(task),
        Syscall::SetEnvStatus => sys_set_env_status(task, args[0], args[1]),
        Syscall::SetTrapframe => sys_set_trapframe(task, args[0], args[1]),
        Syscall::Panic => sys_panic(task, args[0]),
        Syscall::IpcTrySend => sys_ipc_try_send(task, args[0], args[1], args[2], args[3]),
        Syscall::IpcRecv => sys_ipc_recv(task, args[0]),
        Syscall::Getchar => sys_getchar(),
        Syscall::WriteDev => sys_write_dev(task, args[0], args[1], args[2]),
        Syscall::ReadDev => sys_read_dev(task, args[0], args[1], args[2]),
        _ => OsError::BadSyscall.into(),
    };
}

macro_rules! syscall_trace {
    ($syscall:ty, $fmt:tt $(, $arg:expr)*) => {
        trace!(concat!("[{:?}] syscall {}: ", $fmt), $crate::task::hart::get_current_task().unwrap().pid(), stringify!($syscall), $($arg),*);
    };
}

fn sys_putchar(c: usize) -> usize {
    syscall_trace!(Syscall::Putchar, "{}", c);
    print!("{}", c as u8 as char);
    OsError::Success.into()
}

fn sys_print_console(task: Arc<TaskControlBlock>, ptr: usize, len: usize) -> usize {
    syscall_trace!(Syscall::PrintConsole, "ptr: 0x{:x}, len: {}", ptr, len);
    if is_illegal_user_va_range(ptr, len) {
        return OsError::InvalidParam.into();
    }
    unsafe {
        riscv::register::sstatus::set_sum();
    }
    let p = ptr as *const u8;
    let mut q = p;
    let maxlen = len;
    let mut rlen = 0;
    while unsafe { *q } != 0 {
        rlen += 1;
        if rlen > maxlen {
            break;
        }
        q = unsafe { q.add(1) };
    }
    let str = unsafe { core::slice::from_raw_parts(p, rlen) };
    match core::str::from_utf8(str) {
        Ok(s) => {
            print!("{}", &s[..len.min(s.len())]);
            unsafe {
                riscv::register::sstatus::clear_sum();
            }
            OsError::Success
        }
        Err(_) => {
            unsafe {
                riscv::register::sstatus::clear_sum();
            }
            OsError::InvalidParam
        }
    }
    .into()
}

fn sys_get_task_id(task: Arc<TaskControlBlock>) -> usize {
    syscall_trace!(Syscall::GetTaskId, "{}", task.pid().0);
    task.pid().0
}

fn sys_yield(task: Arc<TaskControlBlock>) -> usize {
    syscall_trace!(Syscall::Yield, "");
    task.set_yield_flag(true);
    OsError::Success.into()
}

fn sys_task_destroy(task: Arc<TaskControlBlock>, pid: usize) -> usize {
    syscall_trace!(Syscall::TaskDestroy, "{}", pid);
    if let Some(task) = task.get_task(Pid(pid)) {
        if task.status() == TaskStatus::Running {
            task.set_yield_flag(true);
            while task.status() == TaskStatus::Running {}
        }
        task.exit();
        OsError::Success
    } else {
        OsError::BadTask
    }
    .into()
}

fn sys_set_tlb_mod_entry(task: Arc<TaskControlBlock>, pid: usize, entry: usize) -> usize {
    syscall_trace!(
        Syscall::SetTlbModEntry,
        "pid: {}, entry: 0x{:x}",
        pid,
        entry
    );
    let task = task.get_task(Pid(pid));
    if let Some(task) = task {
        task.set_user_exception_entry(entry);
        OsError::Success
    } else {
        OsError::BadTask
    }
    .into()
}

pub fn sys_mem_alloc(task: Arc<TaskControlBlock>, pid: usize, va: usize, perm: usize) -> usize {
    syscall_trace!(
        Syscall::MemAlloc,
        "pid: {}, va: 0x{:x}, perm: 0x{:x}",
        pid,
        va,
        perm
    );
    if is_illegal_user_va_range(va, PAGE_SIZE) {
        return OsError::InvalidParam.into();
    }
    let task = task.get_task(Pid(pid));
    if let Some(task) = task {
        let perm = match UserAreaPerm::from_bits(perm) {
            Some(perm) => perm,
            None => return OsError::InvalidParam.into(),
        };
        match task.memory().lock().alloc(VirtAddr(va).floor_page(), perm) {
            Ok(_) => OsError::Success,
            Err(e) => e,
        }
    } else {
        OsError::BadTask
    }
    .into()
}

pub fn sys_mem_map(
    task: Arc<TaskControlBlock>,
    src_pid: usize,
    src_va: usize,
    dst_pid: usize,
    dst_va: usize,
    perm: usize,
) -> usize {
    syscall_trace!(
        Syscall::MemMap,
        "src_pid: {}, src_va: 0x{:x}, dst_pid: 0x{:x}, dst_va: 0x{:x}, perm: 0x{:x}",
        src_pid,
        src_va,
        dst_pid,
        dst_va,
        perm
    );
    if is_illegal_user_va_range(src_va, PAGE_SIZE) || is_illegal_user_va_range(dst_va, PAGE_SIZE) {
        return OsError::InvalidParam.into();
    }
    let (src_task, dst_task) = match (
        schedule::get_task(Pid(src_pid)),
        schedule::get_task(Pid(dst_pid)),
    ) {
        (Some(src), Some(dst)) => (src, dst),
        _ => return OsError::BadTask.into(),
    };
    match src_task
        .memory()
        .lock()
        .find_frame(VirtAddr(src_va).floor_page())
    {
        Ok(frame) => {
            let perm = match UserAreaPerm::from_bits(perm) {
                Some(perm) => perm,
                None => return OsError::InvalidParam.into(),
            };
            dst_task
                .memory()
                .lock()
                .map(VirtAddr(dst_va).floor_page(), frame, perm)
                .map(|_| OsError::Success)
                .unwrap_or_else(|e| e)
        }
        Err(e) => e,
    }
    .into()
}

pub fn sys_mem_unmap(task: Arc<TaskControlBlock>, pid: usize, va: usize) -> usize {
    syscall_trace!(Syscall::MemUnmap, "pid: {}, va: 0x{:x}", pid, va);
    if is_illegal_user_va_range(va, PAGE_SIZE) {
        return OsError::InvalidParam.into();
    }
    let task = task.get_task(Pid(pid));
    if let Some(task) = task {
        match task.memory().lock().unmap(VirtAddr(va).floor_page()) {
            Ok(_) => OsError::Success,
            Err(e) => e,
        }
    } else {
        OsError::BadTask
    }
    .into()
}

pub fn sys_exofork(task: Arc<TaskControlBlock>) -> usize {
    syscall_trace!(Syscall::Exofork, "");
    unimplemented!()
}

pub fn sys_set_env_status(task: Arc<TaskControlBlock>, pid: usize, status: usize) -> usize {
    syscall_trace!(Syscall::SetEnvStatus, "pid: {}, status: {}", pid, status);
    let status = match status {
        0 => TaskStatus::Sleeping,
        1 => TaskStatus::Ready,
        _ => return OsError::InvalidParam.into(),
    };
    let task = task.get_task(Pid(pid));
    if let Some(task) = task {
        if task.status() == TaskStatus::Running {
            task.set_yield_flag(true);
            while task.status() == TaskStatus::Running {}
        }
        task.set_status(status);
        OsError::Success
    } else {
        OsError::BadTask
    }
    .into()
}

pub fn sys_set_trapframe(task: Arc<TaskControlBlock>, pid: usize, ptr: usize) -> usize {
    syscall_trace!(Syscall::SetTrapframe, "pid: {}, ptr: 0x{:x}", pid, ptr);
    if is_illegal_user_va_range(ptr, 34 * size_of::<usize>()) {
        return OsError::InvalidParam.into();
    }
    if let Some(task) = task.get_task(Pid(pid)) {
        if task.status() == TaskStatus::Running {
            task.set_yield_flag(true);
            while task.status() == TaskStatus::Running {}
        }
        unsafe {
            task.set_user_context(ptr);
        }
        OsError::Success
    } else {
        OsError::BadTask
    }
    .into()
}

pub fn sys_panic(task: Arc<TaskControlBlock>, ptr: usize) -> usize {
    syscall_trace!(Syscall::Panic, "ptr: 0x{:x}", ptr);
    if is_illegal_user_va_range(ptr, PAGE_SIZE) {
        return OsError::InvalidParam.into();
    }
    unsafe {
        riscv::register::sstatus::set_sum();
    }
    let p = ptr as *const u8;
    let mut q = p;
    let maxlen = 512;
    let mut len = 0;
    while unsafe { *q } != 0 {
        len += 1;
        if len > maxlen {
            break;
        }
        q = unsafe { q.add(1) };
    }
    let panic_info = unsafe { core::slice::from_raw_parts(p, len) };
    panic!(
        "{}",
        core::str::from_utf8(panic_info).unwrap_or("Invalid user panic info")
    );
}

pub fn sys_ipc_try_send(
    task: Arc<TaskControlBlock>,
    pid: usize,
    value: usize,
    src_va: usize,
    perm: usize,
) -> usize {
    syscall_trace!(
        Syscall::IpcTrySend,
        "pid: {}, value: 0x{:x}, src_va: 0x{:x}, perm: 0x{:x}",
        pid,
        value,
        src_va,
        perm
    );
    if src_va != 0 && is_illegal_user_va_range(src_va, PAGE_SIZE) {
        return OsError::InvalidParam.into();
    }
    match schedule::get_task(Pid(pid)) {
        Some(dst) => {
            let mut ipc_info = dst.get_ipc_info().lock();
            if ipc_info.recving == IpcStatus::NotReceiving {
                return OsError::IpcNotRecv.into();
            }
            ipc_info.from = task.pid().0;
            ipc_info.value = value;
            let perm = match UserAreaPerm::from_bits(perm) {
                Some(perm) => perm,
                None => return OsError::InvalidParam.into(),
            };
            ipc_info.perm = perm.bits();
            if src_va != 0 {
                if is_illegal_user_va_range(src_va, PAGE_SIZE) {
                    return OsError::InvalidParam.into();
                }
                match task
                    .memory()
                    .lock()
                    .find_frame(VirtAddr(src_va).floor_page())
                {
                    Ok(frame) => {
                        match dst
                            .memory()
                            .lock()
                            .map(ipc_info.dstva.floor_page(), frame, perm)
                        {
                            Ok(_) => OsError::Success,
                            Err(e) => e,
                        }
                    }
                    Err(e) => e,
                }
            } else {
                OsError::Success
            }
        }
        None => OsError::BadTask,
    }
    .into()
}

pub fn sys_ipc_recv(task: Arc<TaskControlBlock>, dst_va: usize) -> usize {
    syscall_trace!(Syscall::IpcRecv, "dst_va: 0x{:x}", dst_va);
    if dst_va != 0 && is_illegal_user_va_range(dst_va, PAGE_SIZE) {
        return OsError::InvalidParam.into();
    }
    let dst_va = VirtAddr(dst_va);
    let mut ipc_info = task.get_ipc_info().lock();
    ipc_info.recving = IpcStatus::Receiving;
    ipc_info.dstva = dst_va;
    task.set_status(TaskStatus::Sleeping);
    task.set_yield_flag(true);
    OsError::Success.into()
}

pub fn sys_getchar() -> usize {
    syscall_trace!(Syscall::Getchar, "");
    let mut c: u8;
    // TODO: interrupt instead of busy waiting
    loop {
        c = getchar();
        if c != 0 {
            break;
        }
    }
    c as usize
}

pub fn sys_write_dev(task: Arc<TaskControlBlock>, dev: usize, pa: usize, len: usize) -> usize {
    syscall_trace!(
        Syscall::WriteDev,
        "dev: 0x{:x}, pa: 0x{:x}, len: {}",
        dev,
        pa,
        len
    );
    0
}

pub fn sys_read_dev(task: Arc<TaskControlBlock>, dev: usize, pa: usize, len: usize) -> usize {
    syscall_trace!(
        Syscall::ReadDev,
        "dev: 0x{:x}, pa: 0x{:x}, len: {}",
        dev,
        pa,
        len
    );
    0
}

pub fn sys_unhandled() -> usize {
    OsError::BadSyscall.into()
}
