#![no_std]
#![feature(const_trait_impl)]

pub mod mutex;

pub type SpinMutex<T> = mutex::Mutex<T, DummyHelper>;

pub trait MutexHelper {
    fn before_lock() {}
    fn after_lock() {}
}

pub struct DummyHelper;
impl MutexHelper for DummyHelper {}