//! Tickets: requests as the core sees them.

use crate::id::{TenantId, TicketId};
use crate::time::Nanos;

/// A request waiting for a decision.
///
/// A ticket carries its id, its tenant, its arrival time and an optional
/// deadline.
// TODO: priority class and cost are not part of a ticket yet.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ticket {
    id: TicketId,
    tenant: TenantId,
    arrival: Nanos,
    deadline: Option<Nanos>,
}

impl Ticket {
    /// Creates a ticket without a deadline.
    pub fn new(id: TicketId, tenant: TenantId, arrival: Nanos) -> Ticket {
        Ticket {
            id,
            tenant,
            arrival,
            deadline: None,
        }
    }

    /// Returns the ticket with its deadline set to `deadline`.
    pub fn with_deadline(self, deadline: Nanos) -> Ticket {
        Ticket {
            deadline: Some(deadline),
            ..self
        }
    }

    /// Returns the ticket's id.
    pub fn id(&self) -> TicketId {
        self.id
    }

    /// Returns the tenant the ticket belongs to.
    pub fn tenant(&self) -> TenantId {
        self.tenant
    }

    /// Returns the time the ticket arrived.
    pub fn arrival(&self) -> Nanos {
        self.arrival
    }

    /// Returns the ticket's deadline, if it has one.
    pub fn deadline(&self) -> Option<Nanos> {
        self.deadline
    }
}
