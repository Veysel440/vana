//! Hand-made sequences replayed against `Gate` in front of a `FixedLimiter`.

use vana_core::{
    Admission, Dequeued, FixedLimiter, Gate, Limiter, Nanos, Outcome, Permit, RejectReason,
    TenantId, Ticket, TicketId,
};

fn gate(limit: u32, queue_capacity: u32) -> Gate<FixedLimiter> {
    Gate::new(
        FixedLimiter::new(limit).expect("limit is at least 1"),
        queue_capacity,
    )
}

fn ticket(id: u64, deadline: Option<u64>) -> Ticket {
    let t = Ticket::new(TicketId::new(id), TenantId::new(id % 3), Nanos::new(0));
    match deadline {
        Some(d) => t.with_deadline(Nanos::new(d)),
        None => t,
    }
}

fn n(value: u64) -> Nanos {
    Nanos::new(value)
}

/// The submit result as data, so a test can compare it; a granted permit is
/// handed back separately.
#[derive(Debug, PartialEq, Eq)]
enum Seen {
    Granted(u64),
    Queued,
    Rejected(u64, RejectReason),
}

fn submit(gate: &mut Gate<FixedLimiter>, ticket: Ticket, now: u64) -> (Seen, Option<Permit>) {
    match gate.submit(ticket, n(now)) {
        Admission::Granted(t, p) => (Seen::Granted(t.id().get()), Some(p)),
        Admission::Queued => (Seen::Queued, None),
        Admission::Rejected(t, r) => (Seen::Rejected(t.id().get(), r), None),
    }
}

/// A release result as data; granted permits are pushed to `held`.
#[derive(Debug, PartialEq, Eq)]
enum Left {
    Granted(u64),
    Expired(u64),
}

fn release(
    gate: &mut Gate<FixedLimiter>,
    permit: Permit,
    now: u64,
    held: &mut Vec<(u64, Permit)>,
) -> Vec<Left> {
    gate.release(permit, n(now), Outcome::Success)
        .into_iter()
        .map(|d| match d {
            Dequeued::Granted(t, p) => {
                held.push((t.id().get(), p));
                Left::Granted(t.id().get())
            }
            Dequeued::Expired(t) => Left::Expired(t.id().get()),
        })
        .collect()
}

fn ids(tickets: Vec<Ticket>) -> Vec<u64> {
    tickets.into_iter().map(|t| t.id().get()).collect()
}

/// Releases everything still held so that no permit is dropped.
fn finish(gate: &mut Gate<FixedLimiter>, mut held: Vec<(u64, Permit)>, now: u64) {
    while let Some((_, permit)) = held.pop() {
        let _ = release(gate, permit, now, &mut held);
    }
}

// I8: if a ticket is queued, the limiter is full.
fn assert_work_conserving(gate: &Gate<FixedLimiter>) {
    if gate.queued() > 0 {
        assert_eq!(gate.limiter().in_flight(), gate.limiter().limit());
    }
}

// DESIGN.md Gate, submit: an empty gate with a free slot grants.
#[test]
fn an_empty_gate_grants_a_new_ticket() {
    let mut g = gate(2, 3);
    let (seen, permit) = submit(&mut g, ticket(0, None), 5);
    assert_eq!(seen, Seen::Granted(0));
    assert_eq!(g.limiter().in_flight(), 1);
    assert_eq!(g.queued(), 0);
    let permit = permit.expect("granted");
    assert_eq!(permit.acquired_at(), n(5));
    let _ = g.release(permit, n(6), Outcome::Success);
    assert_eq!(g.limiter().in_flight(), 0);
}

// DESIGN.md Gate, submit: a full limiter with an empty queue queues the ticket.
#[test]
fn a_full_limiter_with_an_empty_queue_queues_the_ticket() {
    let mut g = gate(1, 2);
    let (_, p) = submit(&mut g, ticket(0, None), 0);
    let (seen, none) = submit(&mut g, ticket(1, None), 1);
    assert_eq!(seen, Seen::Queued);
    assert!(none.is_none());
    assert_eq!(g.queued(), 1);
    assert_eq!(g.limiter().in_flight(), 1);
    assert_work_conserving(&g);
    finish(&mut g, vec![(0, p.expect("granted"))], 2);
}

// DESIGN.md Gate, submit: a full queue rejects with QueueFull.
#[test]
fn a_full_queue_rejects_with_queue_full_and_hands_the_ticket_back() {
    let mut g = gate(1, 2);
    let (_, p) = submit(&mut g, ticket(0, None), 0);
    assert_eq!(submit(&mut g, ticket(1, None), 1).0, Seen::Queued);
    assert_eq!(submit(&mut g, ticket(2, None), 2).0, Seen::Queued);
    assert_eq!(g.queued(), 2);
    let (seen, none) = submit(&mut g, ticket(3, None), 3);
    assert_eq!(seen, Seen::Rejected(3, RejectReason::QueueFull));
    assert!(none.is_none());
    assert_eq!(g.queued(), 2);
    assert_eq!(g.limiter().in_flight(), 1);
    assert_work_conserving(&g);
    finish(&mut g, vec![(0, p.expect("granted"))], 4);
}

// DESIGN.md Gate, submit: a queue with 1 place is full after one ticket.
#[test]
fn a_queue_of_one_holds_exactly_one_ticket() {
    let mut g = gate(1, 1);
    let (_, p) = submit(&mut g, ticket(0, None), 0);
    assert_eq!(submit(&mut g, ticket(1, None), 0).0, Seen::Queued);
    assert_eq!(
        submit(&mut g, ticket(2, None), 0).0,
        Seen::Rejected(2, RejectReason::QueueFull)
    );
    finish(&mut g, vec![(0, p.expect("granted"))], 1);
}

// DESIGN.md Gate: queue capacity 0 means no queue.
#[test]
fn with_no_queue_a_full_limiter_rejects_and_a_free_one_grants() {
    let mut g = gate(1, 0);
    let (seen, p) = submit(&mut g, ticket(0, None), 0);
    assert_eq!(seen, Seen::Granted(0));
    assert_eq!(
        submit(&mut g, ticket(1, None), 1).0,
        Seen::Rejected(1, RejectReason::QueueFull)
    );
    assert_eq!(g.queued(), 0);
    let mut held = Vec::new();
    assert_eq!(
        release(&mut g, p.expect("granted"), 2, &mut held),
        Vec::<Left>::new()
    );
    let (seen, p) = submit(&mut g, ticket(2, None), 3);
    assert_eq!(seen, Seen::Granted(2));
    finish(&mut g, vec![(2, p.expect("granted"))], 4);
}

// I3: a ticket with deadline == now is expired and rejected on submit.
#[test]
fn a_ticket_whose_deadline_is_now_is_rejected_as_expired() {
    let mut g = gate(2, 3);
    let (seen, none) = submit(&mut g, ticket(0, Some(10)), 10);
    assert_eq!(seen, Seen::Rejected(0, RejectReason::Expired));
    assert!(none.is_none());
    assert_eq!(g.limiter().in_flight(), 0);
    assert_eq!(g.queued(), 0);
}

// I3: a deadline before now is expired too.
#[test]
fn a_ticket_whose_deadline_is_before_now_is_rejected_as_expired() {
    let mut g = gate(2, 3);
    assert_eq!(
        submit(&mut g, ticket(0, Some(9)), 10).0,
        Seen::Rejected(0, RejectReason::Expired)
    );
}

// I3: a deadline one nanosecond after now is not expired.
#[test]
fn a_ticket_whose_deadline_is_one_nanosecond_away_is_granted() {
    let mut g = gate(2, 3);
    let (seen, p) = submit(&mut g, ticket(0, Some(11)), 10);
    assert_eq!(seen, Seen::Granted(0));
    finish(&mut g, vec![(0, p.expect("granted"))], 11);
}

// I3: deadline 0 at time 0 is expired (the zero boundary).
#[test]
fn a_ticket_with_deadline_zero_at_time_zero_is_rejected_as_expired() {
    let mut g = gate(1, 1);
    assert_eq!(
        submit(&mut g, ticket(0, Some(0)), 0).0,
        Seen::Rejected(0, RejectReason::Expired)
    );
}

// DESIGN.md Gate, submit: the expired check comes first, even when the queue is full.
#[test]
fn an_expired_ticket_is_rejected_as_expired_even_when_the_queue_is_full() {
    let mut g = gate(1, 1);
    let (_, p) = submit(&mut g, ticket(0, None), 0);
    assert_eq!(submit(&mut g, ticket(1, None), 1).0, Seen::Queued);
    assert_eq!(
        submit(&mut g, ticket(2, Some(5)), 5).0,
        Seen::Rejected(2, RejectReason::Expired)
    );
    assert_eq!(g.queued(), 1);
    finish(&mut g, vec![(0, p.expect("granted"))], 6);
}

// DESIGN.md Gate, submit: the expired check also comes before the no-queue rejection.
#[test]
fn an_expired_ticket_is_rejected_as_expired_when_there_is_no_queue() {
    let mut g = gate(1, 0);
    let (_, p) = submit(&mut g, ticket(0, None), 0);
    assert_eq!(
        submit(&mut g, ticket(1, Some(3)), 3).0,
        Seen::Rejected(1, RejectReason::Expired)
    );
    finish(&mut g, vec![(0, p.expect("granted"))], 4);
}

// I3: a ticket without a deadline never expires, even at the largest time.
#[test]
fn a_ticket_without_a_deadline_is_granted_at_the_largest_time() {
    let mut g = gate(1, 1);
    let (seen, p) = submit(&mut g, ticket(0, None), u64::MAX);
    assert_eq!(seen, Seen::Granted(0));
    finish(&mut g, vec![(0, p.expect("granted"))], u64::MAX);
}

// I3: a deadline equal to the largest time is expired at the largest time.
#[test]
fn a_ticket_with_the_largest_deadline_is_expired_at_the_largest_time() {
    let mut g = gate(1, 1);
    assert_eq!(
        submit(&mut g, ticket(0, Some(u64::MAX)), u64::MAX).0,
        Seen::Rejected(0, RejectReason::Expired)
    );
    let (seen, p) = submit(&mut g, ticket(1, Some(u64::MAX)), u64::MAX - 1);
    assert_eq!(seen, Seen::Granted(1));
    finish(&mut g, vec![(1, p.expect("granted"))], u64::MAX);
}

// DESIGN.md Gate, release: the queue is first in, first out by submission order.
#[test]
fn release_grants_the_queued_tickets_in_arrival_order() {
    let mut g = gate(1, 3);
    let (_, p0) = submit(&mut g, ticket(0, None), 0);
    for id in 1..=3 {
        assert_eq!(submit(&mut g, ticket(id, None), id).0, Seen::Queued);
    }
    let mut held = Vec::new();
    assert_eq!(
        release(&mut g, p0.expect("granted"), 10, &mut held),
        vec![Left::Granted(1)]
    );
    assert_eq!((g.queued(), g.limiter().in_flight()), (2, 1));
    assert_work_conserving(&g);
    let (_, p1) = held.pop().expect("one permit");
    assert_eq!(release(&mut g, p1, 11, &mut held), vec![Left::Granted(2)]);
    let (_, p2) = held.pop().expect("one permit");
    assert_eq!(release(&mut g, p2, 12, &mut held), vec![Left::Granted(3)]);
    assert_eq!((g.queued(), g.limiter().in_flight()), (0, 1));
    finish(&mut g, held, 13);
    assert_eq!(g.limiter().in_flight(), 0);
}

// DESIGN.md Gate, release: a granted ticket's permit carries the release time.
#[test]
fn a_ticket_granted_from_the_queue_gets_a_permit_stamped_with_the_release_time() {
    let mut g = gate(1, 1);
    let (_, p0) = submit(&mut g, ticket(0, None), 0);
    assert_eq!(submit(&mut g, ticket(1, None), 1).0, Seen::Queued);
    let out = g.release(p0.expect("granted"), n(40), Outcome::Success);
    assert_eq!(out.len(), 1);
    match out.into_iter().next().expect("one result") {
        Dequeued::Granted(t, p) => {
            assert_eq!(t.id().get(), 1);
            assert_eq!(p.acquired_at(), n(40));
            let _ = g.release(p, n(41), Outcome::Success);
        }
        other => panic!("expected a grant, got {other:?}"),
    }
}

// I1: release with several queued tickets frees one slot, so one head is granted.
#[test]
fn one_release_grants_only_as_many_heads_as_there_are_free_slots() {
    let mut g = gate(2, 4);
    let (_, p0) = submit(&mut g, ticket(0, None), 0);
    let (_, p1) = submit(&mut g, ticket(1, None), 1);
    for id in 2..=4 {
        assert_eq!(submit(&mut g, ticket(id, None), 2).0, Seen::Queued);
    }
    let mut held = vec![(1, p1.expect("granted"))];
    let out = release(&mut g, p0.expect("granted"), 5, &mut held);
    assert_eq!(out, vec![Left::Granted(2)]);
    assert_eq!(g.limiter().in_flight(), 2);
    assert_eq!(g.queued(), 2);
    assert_work_conserving(&g);
    finish(&mut g, held, 6);
}

// DESIGN.md Gate, release: expired heads are dropped in order, then the first live head is granted.
#[test]
fn release_drops_expired_heads_in_order_then_grants_the_first_live_one() {
    let mut g = gate(1, 5);
    let (_, p0) = submit(&mut g, ticket(0, None), 0);
    assert_eq!(submit(&mut g, ticket(1, Some(5)), 1).0, Seen::Queued);
    assert_eq!(submit(&mut g, ticket(2, Some(6)), 2).0, Seen::Queued);
    assert_eq!(submit(&mut g, ticket(3, Some(100)), 3).0, Seen::Queued);
    assert_eq!(submit(&mut g, ticket(4, None), 4).0, Seen::Queued);
    let mut held = Vec::new();
    let out = release(&mut g, p0.expect("granted"), 10, &mut held);
    assert_eq!(
        out,
        vec![Left::Expired(1), Left::Expired(2), Left::Granted(3)]
    );
    assert_eq!((g.queued(), g.limiter().in_flight()), (1, 1));
    assert_work_conserving(&g);
    finish(&mut g, held, 11);
}

// I3: a deadline equal to the release time counts as expired at the head.
#[test]
fn a_head_whose_deadline_equals_the_release_time_is_dropped() {
    let mut g = gate(1, 2);
    let (_, p0) = submit(&mut g, ticket(0, None), 0);
    assert_eq!(submit(&mut g, ticket(1, Some(10)), 1).0, Seen::Queued);
    assert_eq!(submit(&mut g, ticket(2, Some(11)), 2).0, Seen::Queued);
    let mut held = Vec::new();
    let out = release(&mut g, p0.expect("granted"), 10, &mut held);
    assert_eq!(out, vec![Left::Expired(1), Left::Granted(2)]);
    finish(&mut g, held, 11);
}

// I3: a head whose deadline is one nanosecond after the release time is granted.
#[test]
fn a_head_whose_deadline_is_one_nanosecond_after_the_release_time_is_granted() {
    let mut g = gate(1, 1);
    let (_, p0) = submit(&mut g, ticket(0, None), 0);
    assert_eq!(submit(&mut g, ticket(1, Some(11)), 1).0, Seen::Queued);
    let mut held = Vec::new();
    let out = release(&mut g, p0.expect("granted"), 10, &mut held);
    assert_eq!(out, vec![Left::Granted(1)]);
    finish(&mut g, held, 11);
}

// DESIGN.md Gate, release: when every queued ticket is expired the queue empties and nothing is granted.
#[test]
fn release_with_only_expired_tickets_queued_empties_the_queue_without_a_grant() {
    let mut g = gate(1, 3);
    let (_, p0) = submit(&mut g, ticket(0, None), 0);
    assert_eq!(submit(&mut g, ticket(1, Some(2)), 1).0, Seen::Queued);
    assert_eq!(submit(&mut g, ticket(2, Some(3)), 1).0, Seen::Queued);
    let mut held = Vec::new();
    let out = release(&mut g, p0.expect("granted"), 9, &mut held);
    assert_eq!(out, vec![Left::Expired(1), Left::Expired(2)]);
    assert!(held.is_empty());
    assert_eq!((g.queued(), g.limiter().in_flight()), (0, 0));
}

// DESIGN.md Gate, release: a live head after an expired head stays queued while the limiter is full.
#[test]
fn release_stops_when_the_limiter_is_full_and_keeps_the_rest_in_order() {
    let mut g = gate(1, 4);
    let (_, p0) = submit(&mut g, ticket(0, None), 0);
    assert_eq!(submit(&mut g, ticket(1, Some(4)), 1).0, Seen::Queued);
    assert_eq!(submit(&mut g, ticket(2, None), 2).0, Seen::Queued);
    assert_eq!(submit(&mut g, ticket(3, None), 3).0, Seen::Queued);
    let mut held = Vec::new();
    let out = release(&mut g, p0.expect("granted"), 8, &mut held);
    assert_eq!(out, vec![Left::Expired(1), Left::Granted(2)]);
    let (_, p2) = held.pop().expect("one permit");
    let out = release(&mut g, p2, 9, &mut held);
    assert_eq!(out, vec![Left::Granted(3)]);
    finish(&mut g, held, 10);
}

// DESIGN.md Gate, release: release with an empty queue returns nothing.
#[test]
fn release_with_an_empty_queue_returns_an_empty_list() {
    let mut g = gate(2, 2);
    let (_, p) = submit(&mut g, ticket(0, None), 0);
    let mut held = Vec::new();
    assert!(release(&mut g, p.expect("granted"), 1, &mut held).is_empty());
    assert_eq!(g.limiter().in_flight(), 0);
}

// DESIGN.md Gate, release: the outcome does not change which tickets are granted.
#[test]
fn every_outcome_frees_the_slot_for_the_head_of_the_queue() {
    for outcome in [Outcome::Success, Outcome::Error, Outcome::Cancelled] {
        let mut g = gate(1, 1);
        let (_, p0) = submit(&mut g, ticket(0, None), 0);
        assert_eq!(submit(&mut g, ticket(1, None), 1).0, Seen::Queued);
        let out = g.release(p0.expect("granted"), n(2), outcome);
        assert_eq!(out.len(), 1);
        match out.into_iter().next().expect("one result") {
            Dequeued::Granted(t, p) => {
                assert_eq!(t.id().get(), 1);
                let _ = g.release(p, n(3), outcome);
            }
            other => panic!("expected a grant, got {other:?}"),
        }
    }
}

// DESIGN.md Gate, expire: a ticket in the middle of the queue is removed and the order of the rest is kept.
#[test]
fn expire_removes_a_ticket_from_the_middle_of_the_queue() {
    let mut g = gate(1, 4);
    let (_, p0) = submit(&mut g, ticket(0, None), 0);
    assert_eq!(submit(&mut g, ticket(1, None), 1).0, Seen::Queued);
    assert_eq!(submit(&mut g, ticket(2, Some(10)), 2).0, Seen::Queued);
    assert_eq!(submit(&mut g, ticket(3, None), 3).0, Seen::Queued);
    assert_eq!(ids(g.expire(n(10))), vec![2]);
    assert_eq!(g.queued(), 2);
    assert_eq!(g.limiter().in_flight(), 1);
    assert_work_conserving(&g);
    let mut held = Vec::new();
    assert_eq!(
        release(&mut g, p0.expect("granted"), 11, &mut held),
        vec![Left::Granted(1)]
    );
    let (_, p1) = held.pop().expect("one permit");
    assert_eq!(release(&mut g, p1, 12, &mut held), vec![Left::Granted(3)]);
    finish(&mut g, held, 13);
}

// DESIGN.md Gate, expire: several expired tickets come back in queue order.
#[test]
fn expire_returns_every_expired_ticket_in_queue_order() {
    let mut g = gate(1, 5);
    let (_, p0) = submit(&mut g, ticket(0, None), 0);
    assert_eq!(submit(&mut g, ticket(1, Some(30)), 1).0, Seen::Queued);
    assert_eq!(submit(&mut g, ticket(2, Some(5)), 2).0, Seen::Queued);
    assert_eq!(submit(&mut g, ticket(3, None), 3).0, Seen::Queued);
    assert_eq!(submit(&mut g, ticket(4, Some(20)), 4).0, Seen::Queued);
    assert_eq!(submit(&mut g, ticket(5, Some(21)), 5).0, Seen::Queued);
    assert_eq!(ids(g.expire(n(20))), vec![2, 4]);
    assert_eq!(g.queued(), 3);
    assert_eq!(ids(g.expire(n(100))), vec![1, 5]);
    assert_eq!(g.queued(), 1);
    assert_work_conserving(&g);
    finish(&mut g, vec![(0, p0.expect("granted"))], 101);
}

// DESIGN.md Gate, expire: a deadline equal to now is expired; one after now is not.
#[test]
fn expire_takes_deadline_equal_to_now_but_not_one_nanosecond_later() {
    let mut g = gate(1, 3);
    let (_, p0) = submit(&mut g, ticket(0, None), 0);
    assert_eq!(submit(&mut g, ticket(1, Some(10)), 1).0, Seen::Queued);
    assert_eq!(submit(&mut g, ticket(2, Some(11)), 1).0, Seen::Queued);
    assert!(g.expire(n(9)).is_empty());
    assert_eq!(ids(g.expire(n(10))), vec![1]);
    assert_eq!(g.queued(), 1);
    finish(&mut g, vec![(0, p0.expect("granted"))], 12);
}

// DESIGN.md Gate, expire: an empty gate returns an empty list.
#[test]
fn expire_on_an_empty_gate_returns_an_empty_list() {
    let mut g = gate(1, 3);
    assert!(g.expire(n(0)).is_empty());
    assert!(g.expire(Nanos::MAX).is_empty());
    assert_eq!(g.queued(), 0);
    assert_eq!(g.limiter().in_flight(), 0);
}

// DESIGN.md Gate, expire: tickets without a deadline are never taken.
#[test]
fn expire_never_takes_tickets_without_a_deadline() {
    let mut g = gate(1, 3);
    let (_, p0) = submit(&mut g, ticket(0, None), 0);
    assert_eq!(submit(&mut g, ticket(1, None), 1).0, Seen::Queued);
    assert_eq!(submit(&mut g, ticket(2, None), 1).0, Seen::Queued);
    assert!(g.expire(Nanos::MAX).is_empty());
    assert_eq!(g.queued(), 2);
    finish(&mut g, vec![(0, p0.expect("granted"))], 5);
}

// DESIGN.md Gate, expire: expire does not grant and does not touch running requests.
#[test]
fn expire_leaves_the_running_requests_and_grants_nothing() {
    let mut g = gate(2, 3);
    let (_, p0) = submit(&mut g, ticket(0, Some(5)), 0);
    let (_, p1) = submit(&mut g, ticket(1, None), 0);
    assert_eq!(submit(&mut g, ticket(2, Some(5)), 1).0, Seen::Queued);
    assert_eq!(ids(g.expire(n(50))), vec![2]);
    assert_eq!(g.limiter().in_flight(), 2);
    finish(
        &mut g,
        vec![(0, p0.expect("granted")), (1, p1.expect("granted"))],
        60,
    );
}

// DESIGN.md Gate, next_deadline: an empty queue has none.
#[test]
fn next_deadline_of_an_empty_queue_is_none() {
    let g = gate(1, 3);
    assert_eq!(g.next_deadline(), None);
}

// DESIGN.md Gate, next_deadline: a granted ticket's deadline is not in the queue.
#[test]
fn next_deadline_ignores_tickets_that_are_running() {
    let mut g = gate(1, 3);
    let (_, p0) = submit(&mut g, ticket(0, Some(7)), 0);
    assert_eq!(g.next_deadline(), None);
    finish(&mut g, vec![(0, p0.expect("granted"))], 1);
}

// DESIGN.md Gate, next_deadline: queued tickets without a deadline are ignored.
#[test]
fn next_deadline_ignores_queued_tickets_without_a_deadline() {
    let mut g = gate(1, 4);
    let (_, p0) = submit(&mut g, ticket(0, None), 0);
    assert_eq!(submit(&mut g, ticket(1, None), 1).0, Seen::Queued);
    assert_eq!(submit(&mut g, ticket(2, None), 1).0, Seen::Queued);
    assert_eq!(g.next_deadline(), None);
    assert_eq!(submit(&mut g, ticket(3, Some(40)), 1).0, Seen::Queued);
    assert_eq!(g.next_deadline(), Some(n(40)));
    finish(&mut g, vec![(0, p0.expect("granted"))], 2);
}

// DESIGN.md Gate, next_deadline: the earliest deadline wherever it sits in the queue.
#[test]
fn next_deadline_is_the_earliest_deadline_wherever_it_is_queued() {
    let mut g = gate(1, 5);
    let (_, p0) = submit(&mut g, ticket(0, None), 0);
    assert_eq!(submit(&mut g, ticket(1, Some(50)), 1).0, Seen::Queued);
    assert_eq!(g.next_deadline(), Some(n(50)));
    assert_eq!(submit(&mut g, ticket(2, Some(30)), 1).0, Seen::Queued);
    assert_eq!(g.next_deadline(), Some(n(30)));
    assert_eq!(submit(&mut g, ticket(3, None), 1).0, Seen::Queued);
    assert_eq!(submit(&mut g, ticket(4, Some(40)), 1).0, Seen::Queued);
    assert_eq!(g.next_deadline(), Some(n(30)));
    assert_eq!(ids(g.expire(n(30))), vec![2]);
    assert_eq!(g.next_deadline(), Some(n(40)));
    let mut held = Vec::new();
    let _ = release(&mut g, p0.expect("granted"), 35, &mut held);
    // Ticket 1 (deadline 50) was granted; tickets 3 and 4 are still queued.
    assert_eq!(g.next_deadline(), Some(n(40)));
    finish(&mut g, held, 36);
}

// DESIGN.md Gate, next_deadline: the earliest deadline can be the largest time.
#[test]
fn next_deadline_can_be_the_largest_time() {
    let mut g = gate(1, 2);
    let (_, p0) = submit(&mut g, ticket(0, None), 0);
    assert_eq!(submit(&mut g, ticket(1, Some(u64::MAX)), 1).0, Seen::Queued);
    assert_eq!(g.next_deadline(), Some(Nanos::MAX));
    finish(&mut g, vec![(0, p0.expect("granted"))], 2);
}

// I8: the gate is work-conserving after a sequence of every kind of operation.
#[test]
fn i8_the_limiter_is_full_whenever_a_ticket_is_queued_through_a_mixed_sequence() {
    let mut g = gate(2, 3);
    let mut held: Vec<(u64, Permit)> = Vec::new();
    let mut take = |g: &mut Gate<FixedLimiter>, id: u64, dl: Option<u64>, now: u64| {
        let (seen, p) = submit(g, ticket(id, dl), now);
        if let Some(p) = p {
            held.push((id, p));
        }
        assert_work_conserving(g);
        seen
    };
    assert_eq!(take(&mut g, 0, None, 0), Seen::Granted(0));
    assert_eq!(take(&mut g, 1, Some(20), 1), Seen::Granted(1));
    assert_eq!(take(&mut g, 2, Some(8), 2), Seen::Queued);
    assert_eq!(take(&mut g, 3, None, 3), Seen::Queued);
    assert_eq!(take(&mut g, 4, Some(50), 4), Seen::Queued);
    assert_eq!(
        take(&mut g, 5, None, 5),
        Seen::Rejected(5, RejectReason::QueueFull)
    );
    assert_eq!(ids(g.expire(n(8))), vec![2]);
    assert_work_conserving(&g);
    let (_, p0) = held.remove(0);
    let out = release(&mut g, p0, 9, &mut held);
    assert_eq!(out, vec![Left::Granted(3)]);
    assert_work_conserving(&g);
    let (_, p1) = held.remove(0);
    let out = release(&mut g, p1, 10, &mut held);
    assert_eq!(out, vec![Left::Granted(4)]);
    assert_work_conserving(&g);
    assert_eq!(g.queued(), 0);
    finish(&mut g, held, 11);
}

// I8: after a release that drains the queue, nothing waits while a slot is free.
#[test]
fn i8_after_a_release_either_the_queue_is_empty_or_the_limiter_is_full() {
    let mut g = gate(3, 6);
    let mut held = Vec::new();
    for id in 0..3 {
        let (_, p) = submit(&mut g, ticket(id, None), 0);
        held.push((id, p.expect("granted")));
    }
    for id in 3..9 {
        assert_eq!(submit(&mut g, ticket(id, Some(100)), 1).0, Seen::Queued);
    }
    let mut now = 2;
    while let Some((_, p)) = held.pop() {
        let _ = release(&mut g, p, now, &mut held);
        assert_work_conserving(&g);
        now += 1;
    }
    assert_eq!((g.queued(), g.limiter().in_flight()), (0, 0));
}

// DESIGN.md Gate: the limiter accessor shows the limit and the permits held.
#[test]
fn the_limiter_accessor_shows_limit_and_in_flight() {
    let mut g = gate(3, 1);
    assert_eq!((g.limiter().limit(), g.limiter().in_flight()), (3, 0));
    let (_, p) = submit(&mut g, ticket(0, None), 0);
    assert_eq!((g.limiter().limit(), g.limiter().in_flight()), (3, 1));
    finish(&mut g, vec![(0, p.expect("granted"))], 1);
}

// DESIGN.md Gate: the largest queue capacity is accepted.
#[test]
fn the_largest_queue_capacity_is_accepted() {
    let mut g = gate(1, u32::MAX);
    let (_, p) = submit(&mut g, ticket(0, None), 0);
    assert_eq!(submit(&mut g, ticket(1, None), 0).0, Seen::Queued);
    assert_eq!(g.queued(), 1);
    let mut held = Vec::new();
    let out = release(&mut g, p.expect("granted"), 1, &mut held);
    assert_eq!(out, vec![Left::Granted(1)]);
    finish(&mut g, held, 2);
}

// DESIGN.md Gate: RejectReason is Clone, Copy, Debug, PartialEq and Eq; Admission and Gate are Debug.
#[test]
fn the_gate_types_can_be_copied_compared_and_formatted() {
    let a = RejectReason::QueueFull;
    let b = a;
    assert_eq!(a, b);
    assert_ne!(a, RejectReason::Expired);
    assert!(!format!("{a:?}").is_empty());
    let mut g = gate(1, 1);
    assert!(!format!("{g:?}").is_empty());
    let admission = g.submit(ticket(0, None), n(0));
    assert!(!format!("{admission:?}").is_empty());
    match admission {
        Admission::Granted(_, p) => {
            let out = g.release(p, n(1), Outcome::Success);
            assert!(out.is_empty());
        }
        other => panic!("expected a grant, got {other:?}"),
    }
}

// DESIGN.md Gate, release: the Expired variant of Dequeued is Debug and carries the ticket.
#[test]
fn a_dequeued_expired_ticket_carries_the_ticket_and_is_debug() {
    let mut g = gate(1, 1);
    let (_, p0) = submit(&mut g, ticket(0, None), 0);
    assert_eq!(submit(&mut g, ticket(9, Some(3)), 1).0, Seen::Queued);
    let out = g.release(p0.expect("granted"), n(3), Outcome::Success);
    assert_eq!(out.len(), 1);
    assert!(!format!("{out:?}").is_empty());
    match &out[0] {
        Dequeued::Expired(t) => assert_eq!((t.id().get(), t.tenant().get()), (9, 0)),
        other => panic!("expected an expiry, got {other:?}"),
    }
}
