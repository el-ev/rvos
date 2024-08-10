use core::{cell::UnsafeCell, fmt, hint::spin_loop, ops::{Deref, DerefMut}, sync::atomic::{AtomicBool, Ordering}};

use crate::MutexHelper;

pub struct Mutex<T: ?Sized, H: MutexHelper> {
    _marker: core::marker::PhantomData<H>,
    lock: AtomicBool,
    data: UnsafeCell<T>,
}

unsafe impl<T: ?Sized + Send, H: MutexHelper> Sync for Mutex<T, H> {}
unsafe impl<T: ?Sized + Send, H: MutexHelper> Send for Mutex<T, H> {}

pub struct MutexGuard<'a, T: ?Sized + 'a, H: MutexHelper + 'a> {
    mutex: &'a Mutex<T, H>,
}

impl<T, H: MutexHelper> Mutex<T, H> {
    pub const fn new(data: T) -> Self {
        Mutex {
            lock: AtomicBool::new(false),
            data: UnsafeCell::new(data),
            _marker: core::marker::PhantomData,
        }
    }

    pub fn into_inner(self) -> T {
        self.data.into_inner()
    }
}

impl<T: ?Sized, H: MutexHelper> Mutex<T, H> {
    fn is_locked(&self) -> bool {
        self.lock.load(Ordering::Relaxed)
    }

    pub fn lock(&self) -> MutexGuard<T, H> {
        H::before_lock();
        while self
            .lock
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            while self.is_locked() {
                spin_loop();
            }
        }
        H::after_lock();
        MutexGuard { mutex: self }
    }

    pub fn try_lock(&self) -> Option<MutexGuard<T, H>> {
        H::before_lock();
        if self
            .lock
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_ok()
        {
            H::after_lock();
            Some(MutexGuard { mutex: self })
        } else {
            H::after_lock();
            None
        }
    }

    /// # Safety
    /// unsafe.
    pub unsafe fn force_unlock(&self) {
        self.lock.store(false, Ordering::Release);
    }
}

impl<T: ?Sized + fmt::Debug, H: MutexHelper> fmt::Debug for Mutex<T, H> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.try_lock() {
            Some(guard) => {
                f.debug_struct("Mutex")
                    .field("data", &&*guard)
                    .finish()
            }
            None => {
                struct LockedPlaceholder;
                impl fmt::Debug for LockedPlaceholder {
                    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                        f.write_str("<locked>")
                    }
                }
                f.debug_struct("Mutex")
                    .field("data", &LockedPlaceholder)
                    .finish()
            }
        }
    }
}

impl<'a, T: ?Sized, H: MutexHelper> Deref for MutexGuard<'a, T, H> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self.mutex.data.get() }
    }
}

impl<'a, T: ?Sized, H: MutexHelper> DerefMut for MutexGuard<'a, T, H> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.mutex.data.get() }
    }
}

impl<'a, T: ?Sized, H: MutexHelper> Drop for MutexGuard<'a, T, H> {
    fn drop(&mut self) {
        unsafe {
            self.mutex.force_unlock();
        }
    }
}