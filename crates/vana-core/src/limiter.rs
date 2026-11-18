//! Limiters: how many requests may be in flight at once.

use core::fmt;

use crate::permit::Permit;
use crate::time::Nanos;

/// How a request that held a permit ended.
// TODO: how a limiter counts errors and cancellations is not decided yet.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// The backend answered.
    Success,
    /// The backend failed or did not answer in time.
    Error,
    /// The client went away before the request finished.
    Cancelled,
}

/// Decides how many requests may be in flight at once and grants permits.
///
/// Invariant I1: a new permit is granted only while `in_flight() < limit()`.
/// If the limit drops below `in_flight()`, requests already in flight are
/// not cut; no new permit is granted until `in_flight() < limit()` again.
///
/// Only code inside `vana-core` can create a [`Permit`], so every limiter is
/// implemented in this crate; other crates use limiters but cannot add one.
///
/// The result of [`try_acquire`](Limiter::try_acquire) must be used: a
/// granted permit that is ignored is dropped without being released.
///
/// ```compile_fail
/// #![deny(unused_must_use)]
/// use vana_core::{FixedLimiter, Limiter, Nanos};
///
/// let mut limiter = FixedLimiter::new(1).unwrap();
/// limiter.try_acquire(Nanos::ZERO);
/// ```
pub trait Limiter {
    /// Returns the current limit: the most permits allowed at once.
    fn limit(&self) -> u32;

    /// Returns the number of permits granted and not yet released.
    fn in_flight(&self) -> u32;

    /// Grants a permit at time `now` if `in_flight() < limit()`, and returns
    /// `None` otherwise.
    #[must_use = "a granted permit must be released to the limiter that granted it"]
    fn try_acquire(&mut self, now: Nanos) -> Option<Permit>;

    /// Takes back a permit at time `now`, recording how its request ended.
    ///
    /// The permit must come from this limiter.
    fn release(&mut self, permit: Permit, now: Nanos, outcome: Outcome);
}

/// A limiter whose limit is set once and never changes.
#[derive(Debug)]
pub struct FixedLimiter {
    limit: u32,
    in_flight: u32,
}

impl FixedLimiter {
    /// Creates a limiter that allows at most `limit` permits at once.
    ///
    /// Returns [`LimitError::Zero`] if `limit` is 0.
    pub fn new(limit: u32) -> Result<FixedLimiter, LimitError> {
        if limit == 0 {
            return Err(LimitError::Zero);
        }
        Ok(FixedLimiter {
            limit,
            in_flight: 0,
        })
    }
}

impl Limiter for FixedLimiter {
    fn limit(&self) -> u32 {
        self.limit
    }

    fn in_flight(&self) -> u32 {
        self.in_flight
    }

    fn try_acquire(&mut self, now: Nanos) -> Option<Permit> {
        if self.in_flight < self.limit {
            self.in_flight += 1;
            Some(Permit::new(now))
        } else {
            None
        }
    }

    /// Takes back a permit; `outcome` is ignored by a fixed limit.
    ///
    /// The permit must be released to the limiter that granted it. Releasing
    /// while `in_flight()` is 0 is a bug: it panics in debug builds, and in
    /// release builds `in_flight()` stays at 0.
    fn release(&mut self, permit: Permit, _now: Nanos, _outcome: Outcome) {
        permit.consume();
        debug_assert!(
            self.in_flight > 0,
            "release called on a limiter with no permit in flight"
        );
        self.in_flight = self.in_flight.saturating_sub(1);
    }
}

/// Error returned when a limit is invalid.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LimitError {
    /// The limit was 0; at least one permit must be allowed.
    Zero,
}

impl fmt::Display for LimitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LimitError::Zero => f.write_str("limit must be at least 1"),
        }
    }
}

impl std::error::Error for LimitError {}
