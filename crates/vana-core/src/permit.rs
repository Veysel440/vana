//! Permits: the right to run one request on the backend.

use crate::time::Nanos;

/// The right to have one request in flight on the backend.
///
/// A permit is created only by a [`Limiter`](crate::Limiter) and must be
/// handed back to the limiter that granted it with
/// [`Limiter::release`](crate::Limiter::release), exactly once. It is neither
/// `Clone` nor `Copy`, and `release` takes it by value, so it cannot be
/// released twice.
///
/// Dropping a permit without releasing it is a bug: in debug builds the drop
/// panics with "permit dropped without release" (unless the thread is
/// already panicking); in release builds it is silent and the limiter's slot
/// stays taken.
#[must_use = "a permit must be released to the limiter that granted it"]
#[derive(Debug)]
pub struct Permit {
    acquired_at: Nanos,
}

impl Permit {
    /// Creates a permit; only limiters inside this crate grant permits.
    pub(crate) fn new(acquired_at: Nanos) -> Permit {
        Permit { acquired_at }
    }

    /// Returns the time at which the permit was granted.
    pub fn acquired_at(&self) -> Nanos {
        self.acquired_at
    }

    /// Consumes the permit without running the check in `Drop`.
    ///
    /// Called by a limiter when the permit is released.
    pub(crate) fn consume(self) {
        // A permit owns no resources besides plain data, so forgetting it
        // leaks nothing; it only skips the drop check below.
        std::mem::forget(self);
    }
}

impl Drop for Permit {
    fn drop(&mut self) {
        #[cfg(debug_assertions)]
        if !std::thread::panicking() {
            panic!("permit dropped without release");
        }
    }
}
