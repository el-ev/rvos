use core::{fmt, fmt::Write};
use sbi::dbcn::sbi_debug_console_write;

pub struct DebugOut;

impl fmt::Write for DebugOut {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        // TODO: const KERNEL_BASE: u64 = 0xFFFFFFFF00000000;
        static SPIN_LOCK: spin::Mutex<()> = spin::Mutex::new(());
        let _lock = SPIN_LOCK.lock();

        sbi_debug_console_write(s.as_ptr() as u64 - 0xFFFFFFFF00000000, s.len() as u64);
        Ok(())
    }
}

pub fn _debug_print(args: fmt::Arguments<'_>) {
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