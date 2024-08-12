use crate::MutexHelper;

pub struct SpinHelper {}
impl MutexHelper for SpinHelper {
    type HelperData = ();
    fn cpu_relax() {
        core::hint::spin_loop();
    }
    fn before_lock() {}
    fn after_lock(_helper_data: &()) {}
}

pub struct SpinNoIrqHelper {}

impl MutexHelper for SpinNoIrqHelper {
    type HelperData = bool;
    fn cpu_relax() {
        core::hint::spin_loop();
    }
    fn before_lock() -> bool {
        let sie = arch::read_sie();
        arch::disable_sie();
        sie
    }
    fn after_lock(sie: &bool) {
        if *sie {
            arch::enable_sie();
        }
    }
}
