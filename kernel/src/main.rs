#![no_std]
#![no_main]
#![feature(naked_functions)]

mod init;
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
extern "C" fn kernel_main() -> ! {
    // unsafe {
	// 	*(0x100000 as *mut u32) = 0x5555;
	// }
	sbi::reset::sbi_shutdown();
}
