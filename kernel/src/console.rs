use alloc::boxed::Box;
use core::{
    fmt::{self, Write},
    sync::atomic::{AtomicBool, Ordering},
};
use sync::Lazy;

use crate::{
    Mutex,
    drivers::serial::ConsoleDevice,
    mm::address_space::{K_HARDWARE_BEG, KERNEL_OFFSET},
};
use crate::{config::UART_BASE, drivers::serial::Uart};

static PRINT_LOCK: Mutex<()> = Mutex::new(());

pub static CONSOLE: Lazy<Box<dyn ConsoleDevice + Send + Sync>> = Lazy::new(|| {
    let uart = Uart::new(
        unsafe { UART_BASE } + K_HARDWARE_BEG,
        0x0038_4000,
        115200,
        1,
        0,
    );
    Box::new(uart)
});
pub static CUSTOM_PRINT: AtomicBool = AtomicBool::new(false);

pub struct Stdout;

impl fmt::Write for Stdout {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        if CUSTOM_PRINT.load(Ordering::Relaxed) {
            CONSOLE.puts(s);
        } else {
            // TODO: Here is a bug when printing user space string
            sbi::dbcn::sbi_debug_console_write((s.as_ptr() as usize - KERNEL_OFFSET) as u64, s.len() as u64);
        }
        Ok(())
    }
}

pub fn _print(args: fmt::Arguments<'_>) {
    let _lock = PRINT_LOCK.lock();
    Stdout.write_fmt(args).unwrap();
}

pub unsafe fn poison_lock() {
    let mut i = 0;
    loop {
        let _lock = PRINT_LOCK.try_lock();
        if _lock.is_some() {
            core::mem::forget(_lock);
            break;
        }
        i += 1;
        if i >= 0x100_000 {
            unsafe {
                PRINT_LOCK.force_unlock();
            }
        }
    }
}

pub fn getchar() -> u8 {
    CONSOLE.getc()
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
