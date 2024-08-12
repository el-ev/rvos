use core::cell::UnsafeCell;
use core::fmt;
use core::mem::MaybeUninit;
use core::ops::{Deref, DerefMut};
use core::sync::atomic::{AtomicU8, Ordering};

pub struct Lazy<T, F = fn() -> T> {
    init_state: AtomicU8,
    data: UnsafeCell<MaybeUninit<T>>,
    init_fn: F,
}

unsafe impl<T: Send, F> Sync for Lazy<T, F> {}
unsafe impl<T: Send, F> Send for Lazy<T, F> {}

impl<T, F: Fn() -> T> Lazy<T, F> {
    pub const fn new(init_fn: F) -> Self {
        Lazy {
            init_state: AtomicU8::new(0),
            data: UnsafeCell::new(MaybeUninit::uninit()),
            init_fn,
        }
    }

    fn force(&self) {
        let state = self.init_state.load(Ordering::Acquire);
        if state == 2 {
            return;
        }
        if state == 1
            || self
                .init_state
                .compare_exchange(0, 1, Ordering::Acquire, Ordering::Relaxed)
                .is_err()
        {
            while self.init_state.load(Ordering::Relaxed) != 2 {
                core::hint::spin_loop();
            }
        } else {
            let data = (self.init_fn)();
            unsafe {
                (*self.data.get()).as_mut_ptr().write(data);
            }
            self.init_state.store(2, Ordering::Release);
        }
    }

    pub fn get(&self) -> &T {
        self.force();
        unsafe { &*(*self.data.get()).as_ptr() }
    }

    pub fn get_mut(&mut self) -> &mut T {
        self.force();
        unsafe { &mut *(*self.data.get()).as_mut_ptr() }
    }

    fn try_get(&self) -> Option<&T> {
        if self.init_state.load(Ordering::Relaxed) == 2 {
            Some(unsafe { &*(*self.data.get()).as_ptr() })
        } else {
            None
        }
    }
}

impl<T: fmt::Debug, F: Fn() -> T> fmt::Debug for Lazy<T, F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.try_get() {
            Some(data) => write!(f, "Lazy {{ data: {:?} }}", data),
            None => write!(f, "Lazy {{ <not initialized> }}"),
        }
    }
}

impl<T> Deref for Lazy<T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.get()
    }
}

impl<T> DerefMut for Lazy<T> {
    fn deref_mut(&mut self) -> &mut T {
        self.get_mut()
    }
}

impl<T, F> Drop for Lazy<T, F> {
    fn drop(&mut self) {
        if self.init_state.load(Ordering::Relaxed) == 2 {
            unsafe {
                core::ptr::drop_in_place((*self.data.get()).as_mut_ptr());
            }
        }
    }
}
