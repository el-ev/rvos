#![no_std]
#![no_main]
#![feature(alloc_error_handler)]
#![feature(naked_functions)]

extern crate alloc;
use core::{ptr::write_bytes, sync::atomic::AtomicU8};

use config::KERNEL_OFFSET;
use log::{error, info};
use sbi::hsm::sbi_hart_get_status;

mod config;
mod console;
mod drivers;
mod entry;
mod logging;
mod mm;
mod panic;
mod task;
mod timer;
mod trap;
mod utils;

static STARTED_HART: AtomicU8 = AtomicU8::new(0);

// Every custom kernel needs a banner
const BANNER: &str = r#"  _______      ______   _____ 
 |  __ \ \    / / __ \ / ____|
 | |__) \ \  / / |  | | (___  
 |  _  / \ \/ /| |  | |\___ \ 
 | | \ \  \  / | |__| |____) |
 |_|  \_\  \/   \____/|_____/  
"#;

#[no_mangle]
extern "C" fn kernel_main(hartid: usize, _dtb_pa: usize) -> ! {
    clear_bss();
    logging::init();
    print!("{}", BANNER);
    info!("RVOS Started.");
    STARTED_HART.fetch_add(1, core::sync::atomic::Ordering::SeqCst);
    mm::init();
    trap::set_kernel_trap();
    timer::init();
    #[cfg(feature = "smp")]
    for i in 0..get_hart_count() {
        if i != hartid {
            start_hart(i);
        }
    }
    loop {
        core::hint::spin_loop();
        if STARTED_HART.load(core::sync::atomic::Ordering::SeqCst) == get_hart_count() as u8 {
            // TODO: Enable when we have device memory region
            // mm::paging::unmap_low_memory();
            break;
        }
    }
    console::CONSOLE.init();
    console::CUSTOM_PRINT.store(true, core::sync::atomic::Ordering::SeqCst);
    info!("Switched to custom uart driver.");
    panic!()
}

#[no_mangle]
extern "C" fn parking(_hartid: usize) -> ! {
    STARTED_HART.fetch_add(1, core::sync::atomic::Ordering::SeqCst);
    loop {
        arch::wfi();
    }
}

fn clear_bss() {
    extern "C" {
        fn __bss_start();
        fn __bss_end();
    }
    unsafe {
        write_bytes(
            __bss_start as *mut u8,
            0,
            (__bss_end as usize - __bss_start as usize) as usize,
        );
    }
}

#[inline]
pub fn get_hart_count() -> usize {
    let mut hart_cnt = 0;
    let mut hart_id = 0;
    loop {
        let status = sbi_hart_get_status(hart_id);
        if status.is_success() {
            hart_cnt += 1;
            hart_id += 1;
        } else {
            break;
        }
    }
    hart_cnt
}

#[allow(unused)]
pub fn start_hart(hartid: usize) {
    match sbi::hsm::sbi_hart_start(
        hartid as u64,
        entry::_second_boot as usize as u64 - KERNEL_OFFSET as u64,
        0,
    )
    .error
    {
        sbi::SbiError::Success => (),
        e => error!("Failed to start hart {}: {:?}", hartid, e),
    }
}
