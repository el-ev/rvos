use alloc::format;
use log::{Level, LevelFilter, Log, Metadata, Record};
use sbi::dbcn::sbi_debug_console_write;

use crate::{mm::address_space::KERNEL_OFFSET, println};

struct Logger;

impl Log for Logger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }
        let color = match record.level() {
            Level::Error => "31", // red
            Level::Warn => "93",  // yellow
            Level::Info => "34",  // blue
            Level::Debug => "32", // green
            Level::Trace => "90", // gray
        };
        if record.level() == Level::Error {
            let msg = format!(
                "\x1b[1;{}m[{}][{}:{}][{}] {}\x1b[0m",
                color,
                arch::tp(),
                record.file().unwrap_or("unknown"),
                record.line().unwrap_or(0),
                record.level(),
                record.args()
            );
            sbi_debug_console_write(
                (msg.as_ptr() as usize - KERNEL_OFFSET) as u64,
                msg.len() as u64,
            );
        } else {
            println!(
                "\x1b[1;{}m[{}][{}:{}][{}] {}\x1b[0m",
                color,
                arch::tp(),
                record.file().unwrap_or("unknown"),
                record.line().unwrap_or(0),
                record.level(),
                record.args()
            );
        }
    }

    fn flush(&self) {}
}

pub fn init() {
    static LOGGER: Logger = Logger;
    match log::set_logger(&LOGGER) {
        Ok(_) => {}
        Err(e) => {
            println!("Failed to set logger: {}", e);
            return;
        }
    }
    log::set_max_level(match option_env!("LOG_LEVEL") {
        Some("error") => LevelFilter::Error,
        Some("warn") => LevelFilter::Warn,
        Some("info") => LevelFilter::Info,
        Some("debug") => LevelFilter::Debug,
        Some("trace") => LevelFilter::Trace,
        _ => {
            if cfg!(debug_assertions) {
                LevelFilter::Debug
            } else {
                LevelFilter::Info
            }
        }
    });
}
