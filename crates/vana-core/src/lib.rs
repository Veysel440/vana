//! Decision engine of vana.
//!
//! This crate holds the pure decision logic that sits between a request and a
//! PHP backend. Today it provides time ([`Nanos`], [`Clock`]), identifiers
//! ([`TenantId`], [`TicketId`]), [`Ticket`], [`Permit`] and the [`Limiter`]
//! trait with one implementation, [`FixedLimiter`].
//!
//! Rule: no I/O, no async runtime, no clock reads. Time is passed in by the
//! caller as [`Nanos`], so the proxy and the simulator (`vana-sim`) can drive
//! the same code.

mod id;
mod limiter;
mod permit;
mod ticket;
mod time;

pub use id::{TenantId, TicketId};
pub use limiter::{FixedLimiter, LimitError, Limiter, Outcome};
pub use permit::Permit;
pub use ticket::Ticket;
pub use time::{Clock, Nanos};
