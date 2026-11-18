//! The gate: a bounded FIFO queue in front of a limiter, with deadlines.

use std::collections::VecDeque;

use crate::limiter::{Limiter, Outcome};
use crate::permit::Permit;
use crate::ticket::Ticket;
use crate::time::Nanos;

/// Why a ticket was turned away.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RejectReason {
    /// The limiter was full and the queue had no free place.
    QueueFull,
    /// The ticket's deadline had passed (`deadline <= now`).
    Expired,
}

/// The answer of [`Gate::submit`].
#[derive(Debug)]
pub enum Admission {
    /// The ticket runs now; the permit must be released to the gate.
    Granted(Ticket, Permit),
    /// The ticket waits in the queue; it leaves it through
    /// [`Gate::release`] or [`Gate::expire`].
    Queued,
    /// The ticket was turned away.
    Rejected(Ticket, RejectReason),
}

/// A ticket that left the queue during [`Gate::release`].
#[derive(Debug)]
pub enum Dequeued {
    /// The ticket runs now; the permit must be released to the gate.
    Granted(Ticket, Permit),
    /// The ticket reached the head of the queue after its deadline and was
    /// dropped without a permit.
    Expired(Ticket),
}

/// A bounded FIFO queue in front of a [`Limiter`], with deadline-aware
/// admission.
///
/// A ticket is expired at time `t` when its deadline is at most `t`; a
/// ticket without a deadline never expires. The queue is first in, first
/// out by submission order, and a new ticket never overtakes a queued one.
///
/// Invariants, after every public method:
/// - I3: an expired ticket is never granted.
/// - I8: if a ticket is queued, the limiter is full.
#[derive(Debug)]
pub struct Gate<L: Limiter> {
    limiter: L,
    queue: VecDeque<Ticket>,
    capacity: u32,
}

impl<L: Limiter> Gate<L> {
    /// Creates a gate in front of `limiter` with room for `queue_capacity`
    /// waiting tickets; 0 means no queue.
    pub fn new(limiter: L, queue_capacity: u32) -> Gate<L> {
        Gate {
            limiter,
            queue: VecDeque::new(),
            capacity: queue_capacity,
        }
    }

    /// Returns the limiter.
    pub fn limiter(&self) -> &L {
        &self.limiter
    }

    /// Returns the number of tickets in the queue.
    pub fn queued(&self) -> u32 {
        // The queue never holds more than `capacity` tickets, so the length
        // fits in a u32.
        self.queue.len() as u32
    }

    /// Returns the earliest deadline among the queued tickets, if any has
    /// one.
    pub fn next_deadline(&self) -> Option<Nanos> {
        self.queue.iter().filter_map(Ticket::deadline).min()
    }

    /// Decides what happens to `ticket` at time `now`.
    ///
    /// An expired ticket is rejected with [`RejectReason::Expired`]. If the
    /// queue is empty and the limiter grants a permit, the ticket runs.
    /// Otherwise it is queued if the queue has room, and rejected with
    /// [`RejectReason::QueueFull`] if not.
    #[must_use = "a granted permit must be released to the gate"]
    pub fn submit(&mut self, ticket: Ticket, now: Nanos) -> Admission {
        if is_expired(&ticket, now) {
            return Admission::Rejected(ticket, RejectReason::Expired);
        }
        if self.queue.is_empty()
            && let Some(permit) = self.limiter.try_acquire(now)
        {
            return Admission::Granted(ticket, permit);
        }
        if self.queued() < self.capacity {
            self.queue.push_back(ticket);
            Admission::Queued
        } else {
            Admission::Rejected(ticket, RejectReason::QueueFull)
        }
    }

    /// Releases `permit` to the limiter at time `now`, then moves tickets
    /// from the head of the queue.
    ///
    /// An expired ticket at the head is dropped as [`Dequeued::Expired`];
    /// otherwise the head is granted a permit if the limiter has room. This
    /// stops at the first unexpired head the limiter cannot take, or when
    /// the queue is empty; expired heads behind the last grant are still
    /// dropped. The results come in the order they happened.
    ///
    /// Expired tickets that are not at the head stay in the queue and count
    /// against its capacity until [`expire`](Gate::expire) runs, so a caller
    /// that wants them out of the way calls `expire` before `submit`.
    #[must_use = "granted permits must be released to the gate"]
    pub fn release(&mut self, permit: Permit, now: Nanos, outcome: Outcome) -> Vec<Dequeued> {
        self.limiter.release(permit, now, outcome);
        let mut out = Vec::new();
        while let Some(ticket) = self.queue.pop_front() {
            if is_expired(&ticket, now) {
                out.push(Dequeued::Expired(ticket));
                continue;
            }
            match self.limiter.try_acquire(now) {
                Some(permit) => out.push(Dequeued::Granted(ticket, permit)),
                None => {
                    // The limiter is full: the ticket keeps its place.
                    self.queue.push_front(ticket);
                    break;
                }
            }
        }
        out
    }

    /// Removes every expired ticket from the queue, wherever it is, and
    /// returns them in queue order.
    #[must_use = "expired tickets are removed from the queue and returned"]
    pub fn expire(&mut self, now: Nanos) -> Vec<Ticket> {
        let mut expired = Vec::new();
        let mut kept = VecDeque::with_capacity(self.queue.len());
        for ticket in self.queue.drain(..) {
            if is_expired(&ticket, now) {
                expired.push(ticket);
            } else {
                kept.push_back(ticket);
            }
        }
        self.queue = kept;
        expired
    }
}

/// A ticket is expired at `now` when its deadline is at most `now`.
fn is_expired(ticket: &Ticket, now: Nanos) -> bool {
    ticket.deadline().is_some_and(|deadline| deadline <= now)
}
