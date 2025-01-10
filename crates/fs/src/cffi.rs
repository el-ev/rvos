static mut PANIC_DISPLAY: Option<fn(&str) -> !> = None;

#[no_mangle]
pub extern "C" fn set_panic_display(f: extern "C" fn(*const core::ffi::c_char) -> !) {
    _set_panic_display(unsafe { core::mem::transmute::<extern "C" fn(*const u8) -> !, for<'a> fn(&'a str) -> !>(f) });
}

#[no_mangle]
pub extern "C" fn __panic() -> ! {
    panic!("Panic message\0");
}

fn _set_panic_display(f: fn(&str) -> !) {
    unsafe {
        PANIC_DISPLAY = Some(f);
    }
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    unsafe {
        if let Some(f) = PANIC_DISPLAY {
            f(info.message().as_str().unwrap_or("panic"));
        }
    }
    unreachable!()
}
