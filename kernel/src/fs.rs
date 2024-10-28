unsafe extern "C" {
    pub fn set_panic_display(f: extern "C" fn(*const core::ffi::c_char) -> !);
}