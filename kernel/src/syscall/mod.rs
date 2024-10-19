use alloc::sync::Arc;

use crate::{error::OsError, print, task::taskdef::TaskControlBlock};

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
    FileOp = 18,
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
            18 => Syscall::FileOp,
            _ => Syscall::Unhandled,
        }
    }
}

pub fn do_syscall(task: Arc<TaskControlBlock>) {
    let syscall = Syscall::from(task.syscall_no());
    let ctx = task.get_context_mut();
    ctx.sepc += 4;
    let args = task.syscall_args();
    ctx.uregs[10] = match syscall {
        Syscall::Putchar => sys_putchar(args[0]),
        _ => OsError::BadSyscall ,
    }.into();
}

pub fn sys_putchar(c: usize) -> OsError {
    print!("{}", c as u8 as char);
    OsError::Success
}

