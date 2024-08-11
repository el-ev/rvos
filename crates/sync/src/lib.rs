#![no_std]

mod lazy;
pub mod mutex;
mod spin;

pub use lazy::Lazy;

pub type SpinMutex<T> = mutex::Mutex<T, spin::SpinHelper>;
pub type SpinNoIrqMutex<T> = mutex::Mutex<T, spin::SpinNoIrqHelper>;

pub trait MutexHelper {
    type HelperData;
    fn cpu_relax();
    fn before_lock() -> Self::HelperData;
    fn after_lock(helper_data: &Self::HelperData);
}
