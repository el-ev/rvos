#![allow(dead_code)]
#![allow(unused_variables)]

use core::panic;

use alloc::sync::Arc;

use crate::{
    console::getchar,
    error::OsError,
    mm::{addr::VirtAddr, address_space::is_illegal_user_va_range, consts::PAGE_SIZE},
    print,
    task::{pid::Pid, schedule, taskdef::{TaskControlBlock, TaskStatus}, user_space::UserAreaPerm},
    utils::user_string::UnsafeUserString,
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
    // WriteDev = 16,
    // ReadDev = 17,
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
            // 16 => Syscall::WriteDev,
            // 17 => Syscall::ReadDev,
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
        // Syscall::WriteDev => sys_write_dev(task, args[0], args[1], args[2]),
        // Syscall::ReadDev => sys_read_dev(task, args[0], args[1], args[2]),
        _ => OsError::BadSyscall.into(),
    };
}

fn sys_putchar(c: usize) -> usize {
    print!("{}", c as u8 as char);
    OsError::Success.into()
}

fn sys_print_console(task: Arc<TaskControlBlock>, ptr: usize, len: usize) -> usize {
    match UnsafeUserString::new(task, ptr as *const _, Some(len)).checked() {
        Some(s) => {
            unsafe {
                riscv::register::sstatus::set_sum();
            }
            print!("{}", s);
            unsafe {
                riscv::register::sstatus::clear_sum();
            }
            OsError::Success
        }
        None => OsError::InvalidParam,
    }
    .into()
}

fn sys_get_task_id(task: Arc<TaskControlBlock>) -> usize {
    task.pid().0
}

fn sys_yield(task: Arc<TaskControlBlock>) -> usize {
    task.set_yield_flag(true);
    OsError::Success.into()
}

fn sys_task_destroy(task: Arc<TaskControlBlock>, pid: usize) -> usize {
    if let Some(task) = task.get_task(Pid(pid)) {
        if task.status() == TaskStatus::Running {
            task.set_yield_flag(true);
            while task.status() == TaskStatus::Running {}
        }
        task.exit();
        OsError::Success.into()
    } else {
        OsError::BadTask.into()
    }
}

fn sys_set_tlb_mod_entry(task: Arc<TaskControlBlock>, pid: usize, entry: usize) -> usize {
    let task = task.get_task(Pid(pid));
    if let Some(task) = task {
        task.set_user_exception_entry(entry);
        OsError::Success.into()
    } else {
        OsError::BadTask.into()
    }
}

pub fn sys_mem_alloc(task: Arc<TaskControlBlock>, pid: usize, va: usize, perm: usize) -> usize {
    if is_illegal_user_va_range(va, PAGE_SIZE) {
        return OsError::InvalidParam.into();
    }
    let task = task.get_task(Pid(pid));
    if let Some(task) = task {
        let perm: UserAreaPerm = UserAreaPerm::from_bits(perm).unwrap();
        match task.memory().lock().alloc(VirtAddr(va).floor_page(), perm) {
            Ok(_) => OsError::Success,
            Err(e) => e,
        }
        .into()
    } else {
        OsError::BadTask.into()
    }
}

pub fn sys_mem_map(
    task: Arc<TaskControlBlock>,
    src_pid: usize,
    src_va: usize,
    dst_pid: usize,
    dst_va: usize,
    perm: usize,
) -> usize {
    if is_illegal_user_va_range(src_va, PAGE_SIZE) || is_illegal_user_va_range(dst_va, PAGE_SIZE) {
        return OsError::InvalidParam.into();
    }
    let src_task = schedule::get_task(Pid(src_pid));
    if src_task.is_none() {
        return OsError::BadTask.into();
    }
    let dst_task = schedule::get_task(Pid(dst_pid));
    if dst_task.is_none() {
        return OsError::BadTask.into();
    }
    let src_task = src_task.unwrap();
    let dst_task = dst_task.unwrap();
    match src_task
        .memory()
        .lock()
        .find_frame(VirtAddr(src_va).floor_page())
    {
        Ok(frame) => {
            let perm: UserAreaPerm = UserAreaPerm::from_bits(perm).unwrap();
            dst_task
                .memory()
                .lock()
                .map(VirtAddr(dst_va).floor_page(), frame, perm);
            OsError::Success.into()
        }
        Err(e) => e.into(),
    }
}

pub fn sys_mem_unmap(task: Arc<TaskControlBlock>, pid: usize, va: usize) -> usize {
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
    }.into()
}

pub fn sys_exofork(task: Arc<TaskControlBlock>) -> usize {
    0
}

pub fn sys_set_env_status(task: Arc<TaskControlBlock>, pid: usize, status: usize) -> usize {
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
        OsError::Success.into()
    } else {
        OsError::BadTask.into()
    }
}

pub fn sys_set_trapframe(task: Arc<TaskControlBlock>, pid: usize, ptr: usize) -> usize {
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
        OsError::Success.into()
    } else {
        OsError::BadTask.into()
    }
}

pub fn sys_panic(task: Arc<TaskControlBlock>, ptr: usize) -> usize {
    unsafe {
        riscv::register::sstatus::set_sum();
    }
    let panic_info = UnsafeUserString::new(task, ptr as *const u8, None).checked();
    match panic_info {
        Some(info) => {
            panic!("{}", info);
        }
        None => {
            panic!("User explicit panic");
        }
    }
}

pub fn sys_ipc_try_send(
    task: Arc<TaskControlBlock>,
    pid: usize,
    value: usize,
    src_va: usize,
    perm: usize,
) -> usize {
    0
}

pub fn sys_ipc_recv(task: Arc<TaskControlBlock>, dst_va: usize) -> usize {
    0
}

pub fn sys_getchar() -> usize {
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

// pub fn sys_write_dev(task: Arc<TaskControlBlock>, dev: usize, pa: usize, len: usize) -> usize {
//     0
// }

// pub fn sys_read_dev(task: Arc<TaskControlBlock>, dev: usize, pa: usize, len: usize) -> usize {
//     0
// }

pub fn sys_file_op(task: Arc<TaskControlBlock>, args: &[usize]) -> usize {
    0
}

pub fn sys_unhandled() -> usize {
    OsError::BadSyscall.into()
}
