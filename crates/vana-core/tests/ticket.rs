//! Black-box tests for `TenantId`, `TicketId` and `Ticket`.

use vana_core::{Nanos, TenantId, Ticket, TicketId};

// DESIGN.md, tickets: ids hold the raw value they were created with.
#[test]
fn ids_return_the_raw_value_they_were_created_with() {
    assert_eq!(TenantId::new(0).get(), 0);
    assert_eq!(TenantId::new(u64::MAX).get(), u64::MAX);
    assert_eq!(TicketId::new(0).get(), 0);
    assert_eq!(TicketId::new(u64::MAX).get(), u64::MAX);
}

// DESIGN.md, tickets: ids are ordered by their raw value.
#[test]
fn ids_are_ordered_by_their_raw_value() {
    assert!(TicketId::new(1) < TicketId::new(2));
    assert!(TenantId::new(1) < TenantId::new(2));
    assert_eq!(TicketId::new(3), TicketId::new(3));
    assert_ne!(TenantId::new(3), TenantId::new(4));
}

// DESIGN.md, tickets: ids derive Clone, Copy, Debug, Eq, Ord, Hash.
#[test]
fn ids_have_the_documented_derives() {
    fn assert_traits<T: Clone + Copy + core::fmt::Debug + Eq + Ord + core::hash::Hash>() {}
    assert_traits::<TenantId>();
    assert_traits::<TicketId>();
}

// DESIGN.md, tickets: new and get are const fns.
#[test]
fn id_constructors_work_in_const_context() {
    const TENANT: TenantId = TenantId::new(5);
    const TICKET: TicketId = TicketId::new(6);
    assert_eq!(TENANT.get(), 5);
    assert_eq!(TICKET.get(), 6);
}

// DESIGN.md, tickets: a new ticket carries id, tenant and arrival, and has no deadline.
#[test]
fn a_new_ticket_has_no_deadline() {
    let ticket = Ticket::new(TicketId::new(1), TenantId::new(2), Nanos::new(3));
    assert_eq!(ticket.id(), TicketId::new(1));
    assert_eq!(ticket.tenant(), TenantId::new(2));
    assert_eq!(ticket.arrival(), Nanos::new(3));
    assert_eq!(ticket.deadline(), None);
}

// DESIGN.md, tickets: with_deadline sets the deadline and keeps the other fields.
#[test]
fn with_deadline_sets_only_the_deadline() {
    let ticket = Ticket::new(TicketId::new(1), TenantId::new(2), Nanos::new(3))
        .with_deadline(Nanos::new(10));
    assert_eq!(ticket.deadline(), Some(Nanos::new(10)));
    assert_eq!(ticket.id(), TicketId::new(1));
    assert_eq!(ticket.tenant(), TenantId::new(2));
    assert_eq!(ticket.arrival(), Nanos::new(3));
}

// DESIGN.md, tickets: a deadline equal to the arrival is stored as given.
#[test]
fn a_deadline_equal_to_the_arrival_is_kept() {
    let ticket =
        Ticket::new(TicketId::new(0), TenantId::new(0), Nanos::new(5)).with_deadline(Nanos::new(5));
    assert_eq!(ticket.deadline(), Some(Nanos::new(5)));
}

// DESIGN.md, tickets: the zero and the largest deadline are stored as given.
#[test]
fn the_extreme_deadlines_are_kept() {
    let base = Ticket::new(TicketId::new(0), TenantId::new(0), Nanos::ZERO);
    assert_eq!(
        base.clone().with_deadline(Nanos::ZERO).deadline(),
        Some(Nanos::ZERO)
    );
    assert_eq!(base.with_deadline(Nanos::MAX).deadline(), Some(Nanos::MAX));
}

// DESIGN.md, tickets: a later with_deadline replaces the earlier one.
#[test]
fn setting_a_deadline_twice_keeps_the_last_one() {
    let ticket = Ticket::new(TicketId::new(0), TenantId::new(0), Nanos::ZERO)
        .with_deadline(Nanos::new(1))
        .with_deadline(Nanos::new(2));
    assert_eq!(ticket.deadline(), Some(Nanos::new(2)));
}

// DESIGN.md, tickets: Ticket derives Clone, Debug, PartialEq, Eq.
#[test]
fn tickets_compare_by_all_their_fields() {
    let a = Ticket::new(TicketId::new(1), TenantId::new(2), Nanos::new(3));
    assert_eq!(a, a.clone());
    assert_ne!(a, a.clone().with_deadline(Nanos::new(4)));
    assert_ne!(
        a,
        Ticket::new(TicketId::new(9), TenantId::new(2), Nanos::new(3))
    );
    assert_ne!(
        a,
        Ticket::new(TicketId::new(1), TenantId::new(9), Nanos::new(3))
    );
    assert_ne!(
        a,
        Ticket::new(TicketId::new(1), TenantId::new(2), Nanos::new(9))
    );
    assert!(!format!("{a:?}").is_empty());
}
