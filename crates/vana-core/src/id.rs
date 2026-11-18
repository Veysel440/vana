//! Identifiers for tenants and tickets.

/// Identifies the tenant a request belongs to.
///
/// The value is opaque to the core: it is only compared and hashed, never
/// interpreted. The layer that creates it decides how a request maps to a
/// tenant.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TenantId(u64);

impl TenantId {
    /// Creates a tenant id from a raw value.
    pub const fn new(id: u64) -> TenantId {
        TenantId(id)
    }

    /// Returns the raw value.
    pub const fn get(self) -> u64 {
        self.0
    }
}

/// Identifies one ticket, that is, one request waiting for a decision.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TicketId(u64);

impl TicketId {
    /// Creates a ticket id from a raw value.
    pub const fn new(id: u64) -> TicketId {
        TicketId(id)
    }

    /// Returns the raw value.
    pub const fn get(self) -> u64 {
        self.0
    }
}
