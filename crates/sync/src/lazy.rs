use core::{
    cell::UnsafeCell,
    fmt,
    mem::ManuallyDrop,
    ops::{Deref, DerefMut},
};

use crate::Once;

union LazyData<T, F> {
    value: ManuallyDrop<T>,
    init: ManuallyDrop<F>,
}

pub struct Lazy<T, F = fn() -> T> {
    once: Once,
    data: UnsafeCell<LazyData<T, F>>,
}

unsafe impl<T: Send + Sync, F> Sync for Lazy<T, F> {}
unsafe impl<T: Send, F> Send for Lazy<T, F> {}

impl<T, F: Fn() -> T> Lazy<T, F> {
    pub const fn new(init_fn: F) -> Self {
        Lazy {
            once: Once::new(),
            data: UnsafeCell::new(LazyData {
                init: ManuallyDrop::new(init_fn),
            }),
        }
    }

    fn force(&self) {
        if self.once.is_completed() {
            return;
        }
        self.once.call_once(|| {
            // SAFETY: On the first call, `self.data` contains the initializer function.
            let data = unsafe { &mut *self.data.get() };
            let f = unsafe { ManuallyDrop::take(&mut data.init) };
            let value = f();
            data.value = ManuallyDrop::new(value);
        });
    }

    pub fn get(&self) -> &T {
        self.force();
        unsafe { &(*self.data.get()).value }
    }

    pub fn get_mut(&mut self) -> &mut T {
        self.force();
        unsafe { &mut (*self.data.get()).value }
    }

    fn try_get(&self) -> Option<&T> {
        if self.once.is_completed() {
            Some(unsafe { &*(*self.data.get()).value })
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
        if self.once.is_completed() {
            unsafe {
                ManuallyDrop::drop(&mut (*self.data.get()).value);
            }
        }
    }
}
