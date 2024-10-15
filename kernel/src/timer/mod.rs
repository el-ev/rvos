#![allow(unused)]

mod consts;

use core::cell::Cell;

use arch::tp;
use log::info;
use riscv::register::{sie, sstatus, time};
use sbi::legacy::sbi_set_timer;
use sync::Lazy;

use self::consts::{CLOCK_FREQ, INTERRUPT_PER_SEC};

static mut TICKS: usize = 0;

pub fn init() {
    unsafe {
        sie::set_stimer();
        sstatus::set_sie();
    }
    set_next_timeout();
    info!("timer initialized.");
}

pub fn set_next_timeout() {
    sbi_set_timer(get_next_int_time());
}

fn get_next_int_time() -> u64 {
    (time::read() + CLOCK_FREQ / INTERRUPT_PER_SEC) as u64
}

pub fn get_ticks() -> usize {
    unsafe { TICKS }
}

pub fn get_time_sec() -> usize {
    get_ticks() / INTERRUPT_PER_SEC
}

pub fn get_time_usec() -> usize {
    get_ticks() % INTERRUPT_PER_SEC * (consts::USEC_PER_SEC / INTERRUPT_PER_SEC)
}

pub fn tick() {
    set_next_timeout();
    if unsafe {tp() != 0} {
        return;
    }
    unsafe {
        TICKS += 1;
    }
    if unsafe {
        TICKS
    } % INTERRUPT_PER_SEC == 0 {
        info!("{} seconds passed.", get_time_sec());
    }
}
