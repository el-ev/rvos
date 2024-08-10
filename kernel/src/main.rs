#![no_std]
#![no_main]
#![feature(asm_const)]
#![feature(naked_functions)]

use core::sync::atomic::{AtomicBool, Ordering};

use config::KERNEL_OFFSET;
use log::{error, info};
use sbi::hsm::sbi_hart_get_status;

mod config;
mod debug_console;
mod entry;
mod logging;
mod mm;
mod panic;
mod utils;

// Every custom kernel needs a banner
const BANNER: &str = r#"
  _______      ______   _____ 
 |  __ \ \    / / __ \ / ____|
 | |__) \ \  / / |  | | (___  
 |  _  / \ \/ /| |  | |\___ \ 
 | | \ \  \  / | |__| |____) |
 |_|  \_\  \/   \____/|_____/ 
                                                              
"#;


#[no_mangle]
extern "C" fn kernel_main(hartid: usize, _dtb_pa: usize) -> ! {
    clear_bss();
    debug!("{}", BANNER);
    logging::init();
	info!("RVOS Started.");
    info!("Hart {} has been started.", hartid);
	#[cfg(feature = "smp")]
    for i in 0..get_hart_count() {
        if i != hartid {
				start_hart(i);
            }
    }
    loop {
        core::hint::spin_loop();
    }
}

#[no_mangle]
extern "C" fn parking(hartid: usize) -> ! {
    info!("Hart {} has been started.", hartid);
    loop {
        core::hint::spin_loop();
    }
}

fn clear_bss() {
    extern "C" {
        fn __bss_start();
        fn __bss_end();
    }
    (__bss_start as usize..__bss_end as usize).for_each(|addr| unsafe {
        (addr as *mut u8).write_volatile(0);
    });
}

// TODO Move this to a separate module
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
