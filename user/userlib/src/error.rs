#[repr(usize)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ErrorCode {
    Success,
    Unspecified,
    BadEnv,
    Inval,
    NoMem,
    NoSys,
    NoFreeEnv,
    IpcNotRecv,
    NoDisk,
    MaxOpen,
    NotFound,
    BadPath,
    FileExists,
    NotExec,
}

impl From<isize> for ErrorCode {
    fn from(err: isize) -> Self {
        match err {
            0 => Self::Success,
            -1 => Self::Unspecified,
            -2 => Self::BadEnv,
            -3 => Self::Inval,
            -4 => Self::NoMem,
            -5 => Self::NoSys,
            -6 => Self::NoFreeEnv,
            -7 => Self::IpcNotRecv,
            -8 => Self::NoDisk,
            -9 => Self::MaxOpen,
            -10 => Self::NotFound,
            -11 => Self::BadPath,
            -12 => Self::FileExists,
            -13 => Self::NotExec,
            _ => unreachable!(),
        }
    }
}
