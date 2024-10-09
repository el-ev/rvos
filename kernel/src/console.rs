use alloc::boxed::Box;
use core::{
    fmt::{self, Write},
    sync::atomic::{AtomicBool, Ordering},
};
use sbi::dbcn::sbi_debug_console_write;
use sync::Lazy;

use crate::drivers::serial::Uart;
use crate::{
    Mutex,
    drivers::serial::ConsoleDevice,
    mm::address_space::{K_HARDWARE_BEG, KERNEL_OFFSET},
};

static PRINT_LOCK: Mutex<()> = Mutex::new(());
// TODO Device Tree
pub static CONSOLE: Lazy<Box<dyn ConsoleDevice + Send + Sync>> = Lazy::new(|| {
    let uart = Uart::new(0x1000_0000 + K_HARDWARE_BEG, 0x0038_4000, 115200, 1, 0);
    Box::new(uart)
});
pub static CUSTOM_PRINT: AtomicBool = AtomicBool::new(false);

pub struct Stdout;

impl fmt::Write for Stdout {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        if CUSTOM_PRINT.load(Ordering::Relaxed) {
            CONSOLE.puts(s);
        } else {
            sbi_debug_console_write((s.as_ptr() as usize - KERNEL_OFFSET) as u64, s.len() as u64);
        }
        Ok(())
    }
}

pub fn _print(args: fmt::Arguments<'_>) {
    let _lock = PRINT_LOCK.lock();
    Stdout.write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {{
        $crate::console::_print(format_args!($($arg)*));
    }};
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}
