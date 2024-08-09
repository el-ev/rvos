use core::{cell::UnsafeCell, hint::spin_loop, ops::{Deref, DerefMut}, sync::atomic::{AtomicBool, Ordering}};

pub trait Mutex<T: ?Sized, S: MutexSupport> {
    
}


pub trait MutexSupport {}