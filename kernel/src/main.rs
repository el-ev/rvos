#![no_std]
#![no_main]
#![feature(naked_functions)]

use sbi::hsm::sbi_hart_get_status;

mod debug_console;
mod entry;
mod panic;

// Every custom kernel needs a banner
const BANNER: &str = 
r#"
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
	let hart_count = get_hart_count();
	debugln!("RVOS Started in hart {} with {} harts in total", hartid, hart_count);

	sbi::reset::sbi_shutdown()
}

fn clear_bss() {
	extern "C" {
		fn __bss_start();
		fn __bss_end();
	}
	(__bss_start as usize..__bss_end as usize).for_each(|addr| {
		unsafe {
			(addr as *mut u8).write_volatile(0);
		}
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