use core::{
    cell::UnsafeCell,
    fmt::Debug,
    mem::MaybeUninit,
    sync::atomic::{AtomicU8, Ordering},
};

pub struct Once {
    state: AtomicU8,
}

unsafe impl Sync for Once {}
unsafe impl Send for Once {}

impl Default for Once {
    fn default() -> Self {
        Self::new()
    }
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq)]
enum OnceState {
    Incomplete = 0x00,
    Running = 0x01,
    Complete = 0x02,
    Poisoned = 0x03,
}

impl OnceState {
    unsafe fn from_u8_unchecked(v: u8) -> Self {
        // SAFETY: Guaranteed by the caller
        unsafe { core::mem::transmute(v) }
    }
}

impl Debug for Once {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut d = f.debug_tuple("Once");
        match self.state.load(core::sync::atomic::Ordering::Relaxed) {
            0 => d.field(&"Incomplete"),
            1 => d.field(&"Running"),
            2 => d.field(&"Complete"),
            _ => d.field(&"Poisoned"),
        }
        .finish()
    }
}

impl Once {
    pub const fn new() -> Self {
        Self {
            state: AtomicU8::new(OnceState::Incomplete as u8),
        }
    }

    pub fn call_once<F>(&self, f: F)
    where
        F: FnOnce(),
    {
        // Fast path
        if self.state.load(Ordering::Acquire) == OnceState::Complete as u8 {
            return;
        }
        self.call_inner(f);
    }

    pub fn is_completed(&self) -> bool {
        self.state.load(Ordering::Acquire) == OnceState::Complete as u8
    }

    pub fn is_running(&self) -> bool {
        self.state.load(Ordering::Acquire) == OnceState::Running as u8
    }

    pub fn is_poisoned(&self) -> bool {
        self.state.load(Ordering::Acquire) == OnceState::Poisoned as u8
    }

    fn call_inner<F>(&self, f: F)
    where
        F: FnOnce(),
    {
        loop {
            match self
                .state
                .compare_exchange(
                    OnceState::Incomplete as u8,
                    OnceState::Running as u8,
                    Ordering::Acquire,
                    Ordering::Acquire,
                )
                // SAFETY: The state is only modified in [`Once::call_inner`], which ensures the validity
                .map_err(|v| unsafe { OnceState::from_u8_unchecked(v) })
            {
                Ok(_) => break, // must be `OnceState::Running`
                Err(OnceState::Incomplete) => continue,
                Err(OnceState::Running) => return self.wait(),
                Err(OnceState::Complete) => return,
                Err(OnceState::Poisoned) => panic!("Once was poisoned previously"),
            }
        }
        let mut finish = Finish {
            state: Some(&self.state),
        };

        f();

        // If `f()` panics, this line will not be reached, and the `Finish` guard will poison the `Once`
        finish.state = None;

        self.state
            .store(OnceState::Complete as u8, Ordering::Release);
    }

    fn wait(&self) {
        loop {
            // SAFETY: The state is only modified in [`Once::call_inner`], which ensures the validity
            match unsafe { OnceState::from_u8_unchecked(self.state.load(Ordering::Acquire)) } {
                OnceState::Complete => break,
                OnceState::Poisoned => panic!("Once was poisoned previously"),
                _ => {}
            }
            core::hint::spin_loop();
        }
    }
}

struct Finish<'a> {
    state: Option<&'a AtomicU8>,
}

// This is a guard that ensures that the `Once` is marked as `Poisoned` if the initialization function panics.
// not used in our case though, as we have no support for panic unwinding
impl Drop for Finish<'_> {
    fn drop(&mut self) {
        if let Some(s) = self.state {
            s.store(OnceState::Poisoned as u8, Ordering::SeqCst)
        }
    }
}

pub struct OnceCell<T> {
    once: Once,
    data: UnsafeCell<MaybeUninit<T>>,
}

unsafe impl<T: Send + Sync> Sync for OnceCell<T> {}
unsafe impl<T: Send> Send for OnceCell<T> {}

impl<T> Default for OnceCell<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Debug> Debug for OnceCell<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut d = f.debug_tuple("OnceCell");
        match self.get() {
            Some(v) => d.field(v),
            None => d.field(&"<uninit>"),
        }
        .finish()
    }
}

impl<T> OnceCell<T> {
    pub const fn new() -> OnceCell<T> {
        OnceCell {
            once: Once::new(),
            data: UnsafeCell::new(MaybeUninit::uninit()),
        }
    }

    pub fn is_initialized(&self) -> bool {
        self.once.is_completed()
    }

    pub fn get(&self) -> Option<&T> {
        if self.once.is_completed() {
            // SAFETY: `data` is initialized when `once` is completed
            Some(unsafe { (*self.data.get()).assume_init_ref() })
        } else {
            None
        }
    }

    pub fn get_mut(&mut self) -> Option<&mut T> {
        if self.once.is_completed() {
            Some(unsafe { (*self.data.get()).assume_init_mut() })
        } else {
            None
        }
    }

    pub fn take(&mut self) -> Option<T> {
        if self.once.is_completed() {
            self.once = Once::new();
            Some(unsafe { (*self.data.get()).assume_init_read() })
        } else {
            None
        }
    }

    // #[allow(clippy::result_unit_err)]
    // pub fn initialize(&self, value: T) -> Result<(), ()> {
    //     if self.once.is_completed() {
    //         Err(())
    //     } else {
    //         self.once.call_once(|| {
    //             // SAFETY: We only write to the `data` field on the first call
    //             unsafe {
    //                 *self.data.get() = MaybeUninit::new(value);
    //             }
    //         });
    //         Ok(())
    //     }
    // }
    #[allow(clippy::result_unit_err)]
    pub fn initialize(&self, f: impl FnOnce() -> T) -> Result<(), ()> {
        if self.once.is_completed() || self.once.is_running() {
            Err(())
        } else {
            self.once.call_once(|| {
                // SAFETY: We only write to the `data` field on the first call
                unsafe {
                    *self.data.get() = MaybeUninit::new(f());
                }
            });
            Ok(())
        }
    }

    pub fn wait(&self) {
        self.once.wait();
    }
}

impl<T> Drop for OnceCell<T> {
    fn drop(&mut self) {
        if self.once.is_completed() {
            // SAFETY: We only write to the `data` field once the `Once` is complete
            unsafe {
                (*self.data.get()).assume_init_drop();
            }
        }
    }
}
