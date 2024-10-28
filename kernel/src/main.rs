#![no_std]
#![no_main]
#![feature(alloc_error_handler)]
#![feature(naked_functions)]
#![feature(postfix_match)]

extern crate alloc;
use core::{ptr::write_bytes, sync::atomic::AtomicU8};

use log::{error, info, warn};
use mm::address_space::KERNEL_OFFSET;
use riscv::asm::ebreak;
use sbi::hsm::sbi_hart_get_status;
use sync::Lazy;

pub type Mutex<T> = sync::SpinNoIrqMutex<T>;

mod config;
mod console;
mod device_tree;
mod drivers;
mod entry;
mod error;
mod logging;
mod mm;
mod panic;
mod syscall;
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

#[unsafe(no_mangle)]
extern "C" fn kernel_main(hartid: usize, dtb: usize) -> ! {
    clear_bss();
    logging::init();
    print!("{}", BANNER);
    info!("RVOS Started on hart {}", hartid);
    STARTED_HART.fetch_add(1, core::sync::atomic::Ordering::SeqCst);
    let _device_tree = device_tree::parse_fdt(dtb);
    mm::init();
    mm::map_kernel_regions(dtb);
    trap::init();
    console::CONSOLE.init();
    console::CUSTOM_PRINT.store(true, core::sync::atomic::Ordering::SeqCst);
    info!("Switched to custom uart driver.");
    timer::init();
    #[cfg(feature = "smp")]
    {
        for i in 0..get_hart_count() {
            if i != hartid {
                start_hart(i);
            }
        }
        let mut i = 0;
        loop {
            i += 1;
            if i > 100000 {
                warn!("Some harts failed to start.");
                break;
            }
            core::hint::spin_loop();
            if STARTED_HART.load(core::sync::atomic::Ordering::SeqCst) == get_hart_count() as u8 {
                break;
            }
        }
    }
    mm::paging::unmap_low_memory();
    unsafe {
        ebreak();
    }
    unsafe {
        set_panic_display(panic::panic_str);
    };
    task::run()
}

#[unsafe(no_mangle)]
pub extern "C" fn other_hart_main(hartid: usize) -> ! {
    STARTED_HART.fetch_add(1, core::sync::atomic::Ordering::SeqCst);
    trap::init();
    info!("Hart {} started.", hartid);
    riscv::asm::wfi();
    task::schedule::SCHEDULER.hart_loop()
}

fn clear_bss() {
    unsafe extern "C" {
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
    static HART_COUNT: Lazy<usize> = Lazy::new(|| {
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
        hart_cnt.min(config::CPU_NUM)
    });
    *HART_COUNT
}

#[cfg(feature = "smp")]
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

unsafe extern "C" {
    fn set_panic_display(f: extern "C" fn(*const core::ffi::c_char) -> !);
}