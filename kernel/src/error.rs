#[allow(dead_code)]
#[repr(usize)]
#[derive(Debug, PartialEq)]
pub enum OsError {
    Success = 0,
    Unspecified = 1,
    BadTask = 2,
    InvalidParam = 3,
    NoMem = 4,
    BadSyscall = 5,
    NoFreeTask = 6,
    IpcNotRecv = 7,
    NoDisk = 8,
    MaxOpen = 9,
    NotFound = 10,
    BadPath = 11,
    FileExists = 12,
    NotExec = 13,
}

impl OsError {
    pub fn is_success(&self) -> bool {
        *self == OsError::Success
    }
}

impl From<OsError> for usize {
    fn from(e: OsError) -> usize {
        (-(e as isize)) as usize
    }
}

impl From<usize> for OsError {
    fn from(e: usize) -> OsError {
        match e {
            0 => OsError::Success,
            1 => OsError::Unspecified,
            2 => OsError::BadTask,
            3 => OsError::InvalidParam,
            4 => OsError::NoMem,
            5 => OsError::BadSyscall,
            6 => OsError::NoFreeTask,
            7 => OsError::IpcNotRecv,
            8 => OsError::NoDisk,
            9 => OsError::MaxOpen,
            10 => OsError::NotFound,
            11 => OsError::BadPath,
            12 => OsError::FileExists,
            13 => OsError::NotExec,
            _ => OsError::Unspecified,
        }
    }
}
