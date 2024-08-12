use core::{fmt, fmt::Write};
use sbi::dbcn::sbi_debug_console_write;

use crate::config::KERNEL_OFFSET;

// TODO: this is ugly
pub struct DebugOut;

impl fmt::Write for DebugOut {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        sbi_debug_console_write(s.as_ptr() as u64 - KERNEL_OFFSET as u64, s.len() as u64);
        Ok(())
    }
}

pub fn _debug_print(args: fmt::Arguments<'_>) {
    static LOCK: sync::SpinNoIrqMutex<()> = sync::SpinNoIrqMutex::new(());
    let _lock = LOCK.lock();
    DebugOut.write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {{
        $crate::debug_console::_debug_print(format_args!($($arg)*));
    }};
}

#[macro_export]
macro_rules! debugln {
    () => ($crate::debug!("\n"));
    ($($arg:tt)*) => ($crate::debug!("{}\n", format_args!($($arg)*)));
}