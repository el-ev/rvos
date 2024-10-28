#[no_mangle]
pub extern "C" fn set_panic_display(f: extern "C" fn(*const core::ffi::c_char) -> !) {
    crate::set_panic_display(unsafe { core::mem::transmute::<extern "C" fn(*const i8) -> !, for<'a> fn(&'a str) -> !>(f) });
}

#[no_mangle]
pub extern "C" fn panic() -> ! {
    panic!("Panic message\0");
}