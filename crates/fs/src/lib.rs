#![no_std]

pub mod cffi;
mod virtio;

static mut PANIC_DISPLAY: Option<fn(&str) -> !> = None;

pub fn set_panic_display(f: fn(&str) -> !) {
    unsafe {
        PANIC_DISPLAY = Some(f);
    }
}

// pub fn fs_panic(info: &str) -> ! {
//     unsafe {
//         if let Some(f) = PANIC_DISPLAY {
//             f(info);
//         }
//     }
//     unreachable!()
// }

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    unsafe {
        if let Some(f) = PANIC_DISPLAY {
            f(info.message().as_str().unwrap_or("panic"));
        }
    }
    unreachable!()
}
