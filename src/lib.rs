//! https://docs.rs/pin-cell/latest/pin_cell/struct.PinCell.html but for [`Mutex`] instead of [`RefCell`](std::cell::RefCell)

use std::{
    ops::Deref,
    pin::Pin,
    sync::{Mutex, MutexGuard},
};

pub struct PinLock<T: ?Sized> {
    inner: Mutex<T>,
}

impl<T> PinLock<T> {
    /// Creates a new `PinCell` containing `value`.
    pub const fn new(value: T) -> PinLock<T> {
        PinLock {
            inner: Mutex::new(value),
        }
    }
}

impl<T: ?Sized> PinLock<T> {
    /// Acquires a mutex, blocking the current thread until it is able to do so.
    ///
    /// This function will block the local thread until it is available to acquire
    /// the mutex. Upon returning, the thread is the only thread with the lock
    /// held. An RAII guard is returned to allow scoped unlock of the lock. When
    /// the guard goes out of scope, the mutex will be unlocked.
    pub fn lock<'a>(self: Pin<&'a Self>) -> PinLockGuard<'a, T> {
        let ref_mut: MutexGuard<'a, T> = Pin::get_ref(self).inner.lock().unwrap();

        // this is a pin projection from Pin<&PinLock<T>> to Pin<Mutex<T>>
        // projecting is safe because:
        //
        // - for<T: ?Sized> (PinLock<T>: Unpin) imples (Mutex<T>: Unpin)
        //   holds true
        // - PinLock does not implement Drop
        //
        // see discussion on tracking issue #49150 about pin projection
        // invariants
        let pin_ref_mut: Pin<MutexGuard<'a, T>> = unsafe { Pin::new_unchecked(ref_mut) };

        PinLockGuard { inner: pin_ref_mut }
    }
}

#[derive(Debug)]
/// A wrapper type for a mutably borrowed value from a `PinLock<T>`.
pub struct PinLockGuard<'a, T: ?Sized> {
    pub(crate) inner: Pin<MutexGuard<'a, T>>,
}

impl<'a, T: ?Sized> Deref for PinLockGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.inner
    }
}

impl<'a, T: ?Sized> PinLockGuard<'a, T> {
    /// Get a pinned mutable reference to the value inside this wrapper.
    pub fn as_mut<'b>(self: &'b mut PinLockGuard<'a, T>) -> Pin<&'b mut T> {
        self.inner.as_mut()
    }
}
