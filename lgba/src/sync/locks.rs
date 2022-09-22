use crate::sync::Static;
use core::{
    cell::UnsafeCell,
    mem::MaybeUninit,
    ops::{Deref, DerefMut},
    ptr,
    sync::atomic::{compiler_fence, Ordering},
};

#[inline(never)]
fn already_locked() -> ! {
    crate::panic_handler::static_panic("IRQ and main are attempting to access the same Mutex!")
}
#[inline(never)]
fn double_unlock() -> ! {
    crate::panic_handler::static_panic("Attempt to unlock a `RawMutex` which is not locked!")
}
#[inline(never)]
fn not_yet_initialized() -> ! {
    crate::panic_handler::static_panic("Attempt to read an InitOnce that is not yet initialized")
}

/// A mutex that prevents code from running in both an IRQ and normal code at
/// the same time.
///
/// Note that this does not support blocking like a typical mutex, and instead
/// mainly exists for memory safety reasons.
pub struct RawMutex(Static<bool>);
impl RawMutex {
    /// Creates a new lock.
    #[must_use]
    pub const fn new() -> Self {
        RawMutex(Static::new(false))
    }

    /// Locks the mutex and returns whether a lock was successfully acquired.
    fn raw_lock(&self) -> bool {
        if self.0.replace(true) {
            // value was already true, opps.
            false
        } else {
            // prevent any weird reordering, and continue
            compiler_fence(Ordering::Acquire);
            true
        }
    }

    /// Unlocks the mutex.
    fn raw_unlock(&self) {
        compiler_fence(Ordering::Release);
        if !self.0.replace(false) {
            double_unlock()
        }
    }

    /// Returns a guard for this lock, or panics if there is another lock active.
    pub fn lock(&self) -> RawMutexGuard<'_> {
        self.try_lock().unwrap_or_else(|| already_locked())
    }

    /// Returns a guard for this lock, or `None` if there is another lock active.
    pub fn try_lock(&self) -> Option<RawMutexGuard<'_>> {
        if self.raw_lock() {
            Some(RawMutexGuard(self))
        } else {
            None
        }
    }
}
unsafe impl Send for RawMutex {}
unsafe impl Sync for RawMutex {}

/// A guard representing an active lock on an [`RawMutex`].
pub struct RawMutexGuard<'a>(&'a RawMutex);
impl<'a> Drop for RawMutexGuard<'a> {
    fn drop(&mut self) {
        self.0.raw_unlock();
    }
}

/// A mutex that protects an object from being accessed in both an IRQ and
/// normal code at once.
///
/// Note that this does not support blocking like a typical mutex, and instead
/// mainly exists for memory safety reasons.
pub struct Mutex<T> {
    raw: RawMutex,
    data: UnsafeCell<T>,
}
impl<T> Mutex<T> {
    /// Creates a new lock containing a given value.
    #[must_use]
    pub const fn new(t: T) -> Self {
        Mutex { raw: RawMutex::new(), data: UnsafeCell::new(t) }
    }

    /// Returns a guard for this lock, or panics if there is another lock active.
    pub fn lock(&self) -> MutexGuard<'_, T> {
        self.try_lock().unwrap_or_else(|| already_locked())
    }

    /// Returns a guard for this lock or `None` if there is another lock active.
    pub fn try_lock(&self) -> Option<MutexGuard<'_, T>> {
        if self.raw.raw_lock() {
            Some(MutexGuard { underlying: self, ptr: self.data.get() })
        } else {
            None
        }
    }
}
unsafe impl<T> Send for Mutex<T> {}
unsafe impl<T> Sync for Mutex<T> {}

/// A guard representing an active lock on an [`Mutex`].
pub struct MutexGuard<'a, T> {
    underlying: &'a Mutex<T>,
    ptr: *mut T,
}
impl<'a, T> Drop for MutexGuard<'a, T> {
    fn drop(&mut self) {
        self.underlying.raw.raw_unlock();
    }
}
impl<'a, T> Deref for MutexGuard<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.ptr }
    }
}
impl<'a, T> DerefMut for MutexGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.ptr }
    }
}

enum Void {}

/// A helper type that ensures a particular value is only initialized once.
pub struct InitOnce<T> {
    is_initialized: Static<bool>,
    value: UnsafeCell<MaybeUninit<T>>,
}
impl<T> InitOnce<T> {
    /// Creates a new uninitialized object.
    #[must_use]
    pub const fn new() -> Self {
        InitOnce {
            is_initialized: Static::new(false),
            value: UnsafeCell::new(MaybeUninit::uninit()),
        }
    }

    /// Returns whether the contents of the cell have already been initialized.
    ///
    /// This should only be used as an optimization.
    pub fn is_initialized(&self) -> bool {
        self.is_initialized.read()
    }

    /// Gets the contents of this state, or initializes it if it has not already been initialized.
    ///
    /// The initializer function is guaranteed to only be called once.
    ///
    /// This function disables IRQs while it is initializing the inner value. While this can cause
    /// audio skipping and other similar issues, it is not normally a problem as interrupts will
    /// only be disabled once per `InitOnce` during the life cycle of the program.
    pub fn get(&self, initializer: impl FnOnce() -> T) -> &T {
        match self.try_get(|| -> Result<T, Void> { Ok(initializer()) }) {
            Ok(v) => v,
            _ => unimplemented!(),
        }
    }

    /// Gets the contents of this state, or initializes it if it has not already been initialized.
    ///
    /// The initializer function is guaranteed to only be called once if it returns `Ok`. If it
    /// returns `Err`, it will be called again in the future until an attempt at initialization
    /// succeeds.
    ///
    /// This function disables IRQs while it is initializing the inner value. While this can cause
    /// audio skipping and other similar issues, it is not normally a problem as interrupts will
    /// only be disabled once per `InitOnce` during the life cycle of the program.
    pub fn try_get<E>(&self, initializer: impl FnOnce() -> Result<T, E>) -> Result<&T, E> {
        unsafe {
            if !self.is_initialized.read() {
                // We disable interrupts to make this simpler, since this is likely to
                // only occur once in a program anyway.
                crate::irq::disable(|| -> Result<(), E> {
                    // We check again to make sure this function wasn't called in an
                    // interrupt between the first check and when interrupts were
                    // actually disabled.
                    if !self.is_initialized.read() {
                        // Do the actual initialization.
                        ptr::write_volatile((*self.value.get()).as_mut_ptr(), initializer()?);
                        self.is_initialized.write(true);
                    }
                    Ok(())
                })?;
            }
            compiler_fence(Ordering::Acquire);
            Ok(&*(*self.value.get()).as_mut_ptr())
        }
    }

    /// Returns whether the contents of the cell have already been initialized.
    ///
    /// This should only be used as an optimization.
    pub fn get_existing(&self) -> &T {
        self.try_get_existing()
            .unwrap_or_else(|| not_yet_initialized())
    }

    /// Returns whether the contents of the cell have already been initialized.
    ///
    /// This should only be used as an optimization.
    pub fn try_get_existing(&self) -> Option<&T> {
        if self.is_initialized.read() {
            Some(unsafe { &*(*self.value.get()).as_mut_ptr() })
        } else {
            None
        }
    }
}
impl<T> Drop for InitOnce<T> {
    fn drop(&mut self) {
        if self.is_initialized.read() {
            // drop the value inside the `MaybeUninit`
            unsafe {
                ptr::read((*self.value.get()).as_ptr());
            }
        }
    }
}
unsafe impl<T: Send> Send for InitOnce<T> {}
unsafe impl<T: Sync> Sync for InitOnce<T> {}
