//! Time as the core sees it: nanoseconds passed in by the caller.

/// A count of nanoseconds.
///
/// `Nanos` is used both as a point in time, counted from an arbitrary origin
/// chosen by the [`Clock`] that produced it, and as a duration. Arithmetic
/// saturates at [`Nanos::ZERO`] and [`Nanos::MAX`] instead of wrapping. There
/// are deliberately no `Add` or `Sub` operators, so every computation states
/// that it saturates.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Nanos(u64);

impl Nanos {
    /// Zero nanoseconds.
    pub const ZERO: Nanos = Nanos(0);

    /// The largest representable value; saturating arithmetic stops here.
    pub const MAX: Nanos = Nanos(u64::MAX);

    /// Creates a value from a number of nanoseconds.
    pub const fn new(nanos: u64) -> Nanos {
        Nanos(nanos)
    }

    /// Returns the number of nanoseconds.
    pub const fn get(self) -> u64 {
        self.0
    }

    /// Adds `rhs`, stopping at [`Nanos::MAX`] instead of overflowing.
    pub fn saturating_add(self, rhs: Nanos) -> Nanos {
        Nanos(self.0.saturating_add(rhs.0))
    }

    /// Subtracts `rhs`, stopping at [`Nanos::ZERO`] instead of underflowing.
    pub fn saturating_sub(self, rhs: Nanos) -> Nanos {
        Nanos(self.0.saturating_sub(rhs.0))
    }
}

/// A source of time.
///
/// Implementations must be monotonic: a later call never returns a smaller
/// value than an earlier one. `vana-core` itself never reads a clock; every
/// function that needs the current time takes it as a [`Nanos`] parameter,
/// and the caller decides where that value comes from.
pub trait Clock {
    /// Returns the current time.
    fn now(&self) -> Nanos;
}
