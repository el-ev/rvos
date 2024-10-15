use core::{
    cell::UnsafeCell,
    fmt,
    ops::{Deref, DerefMut},
    sync::atomic::{AtomicBool, AtomicI32, Ordering},
};

use crate::MutexHelper;

pub struct Mutex<T: ?Sized, H: MutexHelper> {
    _marker: core::marker::PhantomData<H>,
    lock: AtomicBool,
    hartid: AtomicI32,
    data: UnsafeCell<T>,
}

unsafe impl<T: ?Sized + Send, H: MutexHelper> Sync for Mutex<T, H> {}
unsafe impl<T: ?Sized + Send, H: MutexHelper> Send for Mutex<T, H> {}

impl<T, H: MutexHelper> Mutex<T, H> {
    pub const fn new(data: T) -> Self {
        Self {
            _marker: core::marker::PhantomData,
            lock: AtomicBool::new(false),
            hartid: AtomicI32::new(-1),
            data: UnsafeCell::new(data),
        }
    }

    pub fn into_inner(self) -> T {
        self.data.into_inner()
    }
}

impl<T: ?Sized, H: MutexHelper> Mutex<T, H> {
    pub fn lock(&self) -> MutexGuard<T, H> {
        let helper_data = H::before_lock();
        let hartid = arch::get_hart_id() as i32;
        while self
            .lock
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            let old_hartid = self.hartid.load(Ordering::Relaxed);
            // if old_hartid == hartid {
            //     panic!(
            //         "Deadlock. Hart {} is trying to lock a mutex it already owns",
            //         hartid
            //     );
            // }
            let mut i = 0;
            while self.lock.load(Ordering::Relaxed) {
                H::cpu_relax();
                i += 1;
                if i == 0x100_000 {
                    panic!(
                        "Deadlock. Hart {} is trying to lock a mutex owned by hart {}",
                        hartid, old_hartid
                    );
                }
            }
        }
        self.hartid.store(hartid, Ordering::Relaxed);
        MutexGuard {
            mutex: self,
            helper_data,
        }
    }

    pub fn try_lock(&self) -> Option<MutexGuard<T, H>> {
        let helper_data = H::before_lock();
        let hartid = arch::get_hart_id() as i32;
        if self
            .lock
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_ok()
        {
            self.hartid.store(hartid, Ordering::Relaxed);
            Some(MutexGuard {
                mutex: self,
                helper_data,
            })
        } else {
            None
        }
    }
}

pub struct MutexGuard<'a, T: ?Sized + 'a, H: MutexHelper + 'a> {
    pub(crate) mutex: &'a Mutex<T, H>,
    helper_data: H::HelperData,
}

impl<'a, T: ?Sized, H: MutexHelper> Drop for MutexGuard<'a, T, H> {
    fn drop(&mut self) {
        self.mutex.lock.store(false, Ordering::Release);
        H::after_lock(&self.helper_data);
    }
}

impl<'a, T: ?Sized, H: MutexHelper> Deref for MutexGuard<'a, T, H> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.mutex.data.get() }
    }
}

impl<'a, T: ?Sized, H: MutexHelper> DerefMut for MutexGuard<'a, T, H> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.mutex.data.get() }
    }
}

impl<T: fmt::Debug, H: MutexHelper> fmt::Debug for Mutex<T, H> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.try_lock() {
            Some(guard) => write!(f, "Mutex {{ data: {:?} }}", &*guard),
            None => write!(f, "Mutex {{ <locked> }}"),
        }
    }
}

impl<T: fmt::Debug, H: MutexHelper> fmt::Debug for MutexGuard<'_, T, H> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", &**self)
    }
}
