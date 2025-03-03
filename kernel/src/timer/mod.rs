mod consts;

use core::arch::asm;

use arch::tp;
use log::info;
use riscv::register::{sie, time};

use self::consts::{CLOCK_FREQ, INTERRUPT_PER_SEC};

// TODO: this is hart-local
// static mut TICKS: usize = 0;

pub fn init() {
    unsafe {
        sie::set_stimer();
    }
    set_next_timeout();
    info!("timer initialized for hart {}", tp());
}

pub fn set_next_timeout() {
    // sbi_set_timer(get_next_int_time());
    unsafe {
        asm!(
            "csrw stimecmp, {0}",
            in(reg) get_next_int_time()
        )
    }
}

fn get_next_int_time() -> u64 {
    (time::read() + CLOCK_FREQ / INTERRUPT_PER_SEC) as u64
}

#[allow(dead_code)]
pub fn sleep(duration: usize) {
    let end = time::read() + duration * CLOCK_FREQ;
    while time::read() < end {}
}

// pub fn get_ticks() -> usize {
//     unsafe { TICKS }
// }
//
// pub fn get_time_sec() -> usize {
//     get_ticks() / INTERRUPT_PER_SEC
// }
//
// pub fn get_time_usec() -> usize {
//     get_ticks() % INTERRUPT_PER_SEC * (consts::USEC_PER_SEC / INTERRUPT_PER_SEC)
// }

pub fn tick() {
    set_next_timeout();
}
