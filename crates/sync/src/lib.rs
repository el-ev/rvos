#![no_std]

mod helpers;
mod lazy;
pub mod mutex;
mod once;

pub use lazy::Lazy;
pub use once::{Once, OnceCell};

pub type SpinMutex<T> = mutex::Mutex<T, helpers::SpinHelper>;
pub type SpinNoIrqMutex<T> = mutex::Mutex<T, helpers::SpinNoIrqHelper>;

pub trait MutexHelper {
    type HelperData;
    fn relax();
    fn before_lock() -> Self::HelperData;
    fn after_lock(helper_data: &Self::HelperData);
}
