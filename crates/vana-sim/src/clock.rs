//! A clock that moves only when told to.

use vana_core::{Clock, Nanos};

/// A [`Clock`] that starts at a given time and moves only when
/// [`advance`](ManualClock::advance) is called.
#[derive(Debug)]
pub struct ManualClock {
    now: Nanos,
}

impl ManualClock {
    /// Creates a clock that reads `start`.
    pub fn new(start: Nanos) -> ManualClock {
        ManualClock { now: start }
    }

    /// Moves the clock forward by `by`, stopping at [`Nanos::MAX`].
    pub fn advance(&mut self, by: Nanos) {
        self.now = self.now.saturating_add(by);
    }
}

impl Clock for ManualClock {
    fn now(&self) -> Nanos {
        self.now
    }
}
