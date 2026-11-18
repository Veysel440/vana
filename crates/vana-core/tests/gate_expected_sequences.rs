//! Hand-made `Gate` sequences replayed in front of a `FixedLimiter` and
//! compared, line by line, with independently computed traces.

use std::collections::HashMap;
use std::fmt::Write as _;

use vana_core::{
    Admission, Dequeued, FixedLimiter, Gate, Limiter, Nanos, Outcome, Permit, RejectReason,
    TenantId, Ticket, TicketId,
};

enum Op {
    Submit {
        ticket: u64,
        tenant: u64,
        deadline: Option<u64>,
        now: u64,
    },
    Release {
        permit: u64,
        now: u64,
        outcome: Outcome,
    },
    Expire {
        now: u64,
    },
}

fn submit(ticket: u64, tenant: u64, deadline: Option<u64>, now: u64) -> Op {
    Op::Submit {
        ticket,
        tenant,
        deadline,
        now,
    }
}

fn release(permit: u64, now: u64, outcome: Outcome) -> Op {
    Op::Release {
        permit,
        now,
        outcome,
    }
}

fn expire(now: u64) -> Op {
    Op::Expire { now }
}

fn num(value: Option<Nanos>) -> String {
    value.map_or("null".to_string(), |v| v.get().to_string())
}

/// Plays `ops` and renders one line per step, then a final line. A permit
/// number is the number of the ticket it was granted to.
fn replay(limit: u32, queue_capacity: u32, ops: &[Op]) -> String {
    let mut gate = Gate::new(
        FixedLimiter::new(limit).expect("limit is at least 1"),
        queue_capacity,
    );
    let mut held: HashMap<u64, Permit> = HashMap::new();
    let mut out = String::new();
    for (step, op) in ops.iter().enumerate() {
        match op {
            Op::Submit {
                ticket,
                tenant,
                deadline,
                now,
            } => {
                let mut t = Ticket::new(
                    TicketId::new(*ticket),
                    TenantId::new(*tenant),
                    Nanos::new(*now),
                );
                if let Some(d) = deadline {
                    t = t.with_deadline(Nanos::new(*d));
                }
                let head = format!(
                    "{{\"step\":{step},\"op\":\"submit\",\"ticket\":{ticket},\"now\":{now},"
                );
                let verdict = match gate.submit(t, Nanos::new(*now)) {
                    Admission::Granted(t, p) => {
                        let at = p.acquired_at().get();
                        held.insert(t.id().get(), p);
                        format!("\"admission\":\"granted\",\"acquired_at\":{at}")
                    }
                    Admission::Queued => "\"admission\":\"queued\"".to_string(),
                    Admission::Rejected(_, RejectReason::Expired) => {
                        "\"admission\":\"rejected_expired\"".to_string()
                    }
                    Admission::Rejected(_, RejectReason::QueueFull) => {
                        "\"admission\":\"rejected_queue_full\"".to_string()
                    }
                };
                let _ = writeln!(
                    out,
                    "{head}{verdict},\"in_flight\":{},\"queued\":{},\"next_deadline\":{}}}",
                    gate.limiter().in_flight(),
                    gate.queued(),
                    num(gate.next_deadline())
                );
            }
            Op::Release {
                permit,
                now,
                outcome,
            } => {
                let p = held.remove(permit).expect("the permit is held");
                let moved = gate.release(p, Nanos::new(*now), *outcome);
                let mut items = Vec::new();
                for d in moved {
                    match d {
                        Dequeued::Granted(t, p) => {
                            let at = p.acquired_at().get();
                            items.push(format!(
                                "{{\"r\":\"granted\",\"ticket\":{},\"acquired_at\":{at}}}",
                                t.id().get()
                            ));
                            held.insert(t.id().get(), p);
                        }
                        Dequeued::Expired(t) => {
                            items.push(format!("{{\"r\":\"expired\",\"ticket\":{}}}", t.id().get()))
                        }
                    }
                }
                let _ = writeln!(
                    out,
                    "{{\"step\":{step},\"op\":\"release\",\"permit\":{permit},\"now\":{now},\"dequeued\":[{}],\"in_flight\":{},\"queued\":{},\"next_deadline\":{}}}",
                    items.join(","),
                    gate.limiter().in_flight(),
                    gate.queued(),
                    num(gate.next_deadline())
                );
            }
            Op::Expire { now } => {
                let ids: Vec<String> = gate
                    .expire(Nanos::new(*now))
                    .iter()
                    .map(|t| t.id().get().to_string())
                    .collect();
                let _ = writeln!(
                    out,
                    "{{\"step\":{step},\"op\":\"expire\",\"now\":{now},\"expired\":[{}],\"in_flight\":{},\"queued\":{},\"next_deadline\":{}}}",
                    ids.join(","),
                    gate.limiter().in_flight(),
                    gate.queued(),
                    num(gate.next_deadline())
                );
            }
        }
    }
    let _ = writeln!(
        out,
        "{{\"final\":true,\"in_flight\":{},\"queued\":{},\"next_deadline\":{}}}",
        gate.limiter().in_flight(),
        gate.queued(),
        num(gate.next_deadline())
    );
    // Release what is still held so that no permit is dropped.
    let mut rest: Vec<Permit> = held.into_values().collect();
    while let Some(p) = rest.pop() {
        for d in gate.release(p, Nanos::MAX, Outcome::Success) {
            if let Dequeued::Granted(_, p) = d {
                rest.push(p);
            }
        }
    }
    out
}

// DESIGN.md Gate section; expected trace computed independently from the rules in docs/DESIGN.md.
#[test]
fn gate_sequence_an_empty_gate_grants_and_stamps_the_permit_with_now_matches_the_expected_trace() {
    let ops = [submit(1, 0, None, 10)];
    let expected = r##"{"step":0,"op":"submit","ticket":1,"now":10,"admission":"granted","acquired_at":10,"in_flight":1,"queued":0,"next_deadline":null}
{"final":true,"in_flight":1,"queued":0,"next_deadline":null}
"##;
    assert_eq!(replay(2, 3, &ops), expected);
}

// DESIGN.md Gate section; expected trace computed independently from the rules in docs/DESIGN.md.
#[test]
fn gate_sequence_a_full_limiter_queues_and_reports_the_earliest_deadline_matches_the_expected_trace()
 {
    let ops = [
        submit(1, 0, None, 0),
        submit(2, 0, None, 1),
        submit(3, 0, Some(50), 2),
    ];
    let expected = r##"{"step":0,"op":"submit","ticket":1,"now":0,"admission":"granted","acquired_at":0,"in_flight":1,"queued":0,"next_deadline":null}
{"step":1,"op":"submit","ticket":2,"now":1,"admission":"queued","in_flight":1,"queued":1,"next_deadline":null}
{"step":2,"op":"submit","ticket":3,"now":2,"admission":"queued","in_flight":1,"queued":2,"next_deadline":50}
{"final":true,"in_flight":1,"queued":2,"next_deadline":50}
"##;
    assert_eq!(replay(1, 3, &ops), expected);
}

// DESIGN.md Gate section; expected trace computed independently from the rules in docs/DESIGN.md.
#[test]
fn gate_sequence_a_full_queue_rejects_without_changing_the_queue_matches_the_expected_trace() {
    let ops = [
        submit(1, 0, None, 0),
        submit(2, 0, None, 1),
        submit(3, 0, None, 2),
        submit(4, 0, None, 3),
        submit(5, 0, None, 4),
    ];
    let expected = r##"{"step":0,"op":"submit","ticket":1,"now":0,"admission":"granted","acquired_at":0,"in_flight":1,"queued":0,"next_deadline":null}
{"step":1,"op":"submit","ticket":2,"now":1,"admission":"queued","in_flight":1,"queued":1,"next_deadline":null}
{"step":2,"op":"submit","ticket":3,"now":2,"admission":"queued","in_flight":1,"queued":2,"next_deadline":null}
{"step":3,"op":"submit","ticket":4,"now":3,"admission":"rejected_queue_full","in_flight":1,"queued":2,"next_deadline":null}
{"step":4,"op":"submit","ticket":5,"now":4,"admission":"rejected_queue_full","in_flight":1,"queued":2,"next_deadline":null}
{"final":true,"in_flight":1,"queued":2,"next_deadline":null}
"##;
    assert_eq!(replay(1, 2, &ops), expected);
}

// DESIGN.md Gate section; expected trace computed independently from the rules in docs/DESIGN.md.
#[test]
fn gate_sequence_a_deadline_at_or_before_now_is_rejected_as_expired_matches_the_expected_trace() {
    let ops = [
        submit(1, 0, Some(10), 10),
        submit(2, 0, Some(9), 10),
        submit(3, 0, Some(11), 10),
        expire(10),
    ];
    let expected = r##"{"step":0,"op":"submit","ticket":1,"now":10,"admission":"rejected_expired","in_flight":0,"queued":0,"next_deadline":null}
{"step":1,"op":"submit","ticket":2,"now":10,"admission":"rejected_expired","in_flight":0,"queued":0,"next_deadline":null}
{"step":2,"op":"submit","ticket":3,"now":10,"admission":"granted","acquired_at":10,"in_flight":1,"queued":0,"next_deadline":null}
{"step":3,"op":"expire","now":10,"expired":[],"in_flight":1,"queued":0,"next_deadline":null}
{"final":true,"in_flight":1,"queued":0,"next_deadline":null}
"##;
    assert_eq!(replay(2, 2, &ops), expected);
}

// DESIGN.md Gate section; expected trace computed independently from the rules in docs/DESIGN.md.
#[test]
fn gate_sequence_an_expired_ticket_is_rejected_as_expired_before_the_queue_check_matches_the_expected_trace()
 {
    let ops = [
        submit(1, 0, None, 0),
        submit(2, 0, None, 1),
        submit(3, 0, Some(5), 5),
        submit(4, 0, Some(4), 5),
    ];
    let expected = r##"{"step":0,"op":"submit","ticket":1,"now":0,"admission":"granted","acquired_at":0,"in_flight":1,"queued":0,"next_deadline":null}
{"step":1,"op":"submit","ticket":2,"now":1,"admission":"queued","in_flight":1,"queued":1,"next_deadline":null}
{"step":2,"op":"submit","ticket":3,"now":5,"admission":"rejected_expired","in_flight":1,"queued":1,"next_deadline":null}
{"step":3,"op":"submit","ticket":4,"now":5,"admission":"rejected_expired","in_flight":1,"queued":1,"next_deadline":null}
{"final":true,"in_flight":1,"queued":1,"next_deadline":null}
"##;
    assert_eq!(replay(1, 1, &ops), expected);
}

// DESIGN.md Gate section; expected trace computed independently from the rules in docs/DESIGN.md.
#[test]
fn gate_sequence_without_a_queue_expired_and_live_tickets_get_different_reasons_matches_the_expected_trace()
 {
    let ops = [
        submit(1, 0, None, 0),
        submit(2, 0, Some(5), 5),
        submit(3, 0, Some(6), 5),
    ];
    let expected = r##"{"step":0,"op":"submit","ticket":1,"now":0,"admission":"granted","acquired_at":0,"in_flight":1,"queued":0,"next_deadline":null}
{"step":1,"op":"submit","ticket":2,"now":5,"admission":"rejected_expired","in_flight":1,"queued":0,"next_deadline":null}
{"step":2,"op":"submit","ticket":3,"now":5,"admission":"rejected_queue_full","in_flight":1,"queued":0,"next_deadline":null}
{"final":true,"in_flight":1,"queued":0,"next_deadline":null}
"##;
    assert_eq!(replay(1, 0, &ops), expected);
}

// DESIGN.md Gate section; expected trace computed independently from the rules in docs/DESIGN.md.
#[test]
fn gate_sequence_without_a_queue_a_release_frees_the_slot_for_the_next_submit_matches_the_expected_trace()
 {
    let ops = [
        submit(1, 0, None, 0),
        submit(2, 0, None, 1),
        submit(3, 0, None, 2),
        release(1, 3, Outcome::Success),
        submit(4, 0, None, 4),
        submit(5, 0, None, 5),
    ];
    let expected = r##"{"step":0,"op":"submit","ticket":1,"now":0,"admission":"granted","acquired_at":0,"in_flight":1,"queued":0,"next_deadline":null}
{"step":1,"op":"submit","ticket":2,"now":1,"admission":"rejected_queue_full","in_flight":1,"queued":0,"next_deadline":null}
{"step":2,"op":"submit","ticket":3,"now":2,"admission":"rejected_queue_full","in_flight":1,"queued":0,"next_deadline":null}
{"step":3,"op":"release","permit":1,"now":3,"dequeued":[],"in_flight":0,"queued":0,"next_deadline":null}
{"step":4,"op":"submit","ticket":4,"now":4,"admission":"granted","acquired_at":4,"in_flight":1,"queued":0,"next_deadline":null}
{"step":5,"op":"submit","ticket":5,"now":5,"admission":"rejected_queue_full","in_flight":1,"queued":0,"next_deadline":null}
{"final":true,"in_flight":1,"queued":0,"next_deadline":null}
"##;
    assert_eq!(replay(1, 0, &ops), expected);
}

// DESIGN.md Gate section; expected trace computed independently from the rules in docs/DESIGN.md.
#[test]
fn gate_sequence_each_release_grants_the_head_of_the_queue_in_arrival_order_matches_the_expected_trace()
 {
    let ops = [
        submit(1, 0, None, 0),
        submit(2, 0, None, 1),
        submit(3, 0, None, 2),
        submit(4, 0, None, 3),
        release(1, 10, Outcome::Success),
        release(2, 11, Outcome::Success),
        release(3, 12, Outcome::Success),
        release(4, 13, Outcome::Success),
    ];
    let expected = r##"{"step":0,"op":"submit","ticket":1,"now":0,"admission":"granted","acquired_at":0,"in_flight":1,"queued":0,"next_deadline":null}
{"step":1,"op":"submit","ticket":2,"now":1,"admission":"queued","in_flight":1,"queued":1,"next_deadline":null}
{"step":2,"op":"submit","ticket":3,"now":2,"admission":"queued","in_flight":1,"queued":2,"next_deadline":null}
{"step":3,"op":"submit","ticket":4,"now":3,"admission":"queued","in_flight":1,"queued":3,"next_deadline":null}
{"step":4,"op":"release","permit":1,"now":10,"dequeued":[{"r":"granted","ticket":2,"acquired_at":10}],"in_flight":1,"queued":2,"next_deadline":null}
{"step":5,"op":"release","permit":2,"now":11,"dequeued":[{"r":"granted","ticket":3,"acquired_at":11}],"in_flight":1,"queued":1,"next_deadline":null}
{"step":6,"op":"release","permit":3,"now":12,"dequeued":[{"r":"granted","ticket":4,"acquired_at":12}],"in_flight":1,"queued":0,"next_deadline":null}
{"step":7,"op":"release","permit":4,"now":13,"dequeued":[],"in_flight":0,"queued":0,"next_deadline":null}
{"final":true,"in_flight":0,"queued":0,"next_deadline":null}
"##;
    assert_eq!(replay(1, 3, &ops), expected);
}

// DESIGN.md Gate section; expected trace computed independently from the rules in docs/DESIGN.md.
#[test]
fn gate_sequence_with_two_slots_releases_in_any_order_still_grant_in_arrival_order_matches_the_expected_trace()
 {
    let ops = [
        submit(1, 0, None, 0),
        submit(2, 0, None, 0),
        submit(3, 0, None, 1),
        submit(4, 0, None, 1),
        submit(5, 0, None, 2),
        release(2, 5, Outcome::Success),
        release(1, 6, Outcome::Success),
        release(4, 7, Outcome::Success),
    ];
    let expected = r##"{"step":0,"op":"submit","ticket":1,"now":0,"admission":"granted","acquired_at":0,"in_flight":1,"queued":0,"next_deadline":null}
{"step":1,"op":"submit","ticket":2,"now":0,"admission":"granted","acquired_at":0,"in_flight":2,"queued":0,"next_deadline":null}
{"step":2,"op":"submit","ticket":3,"now":1,"admission":"queued","in_flight":2,"queued":1,"next_deadline":null}
{"step":3,"op":"submit","ticket":4,"now":1,"admission":"queued","in_flight":2,"queued":2,"next_deadline":null}
{"step":4,"op":"submit","ticket":5,"now":2,"admission":"rejected_queue_full","in_flight":2,"queued":2,"next_deadline":null}
{"step":5,"op":"release","permit":2,"now":5,"dequeued":[{"r":"granted","ticket":3,"acquired_at":5}],"in_flight":2,"queued":1,"next_deadline":null}
{"step":6,"op":"release","permit":1,"now":6,"dequeued":[{"r":"granted","ticket":4,"acquired_at":6}],"in_flight":2,"queued":0,"next_deadline":null}
{"step":7,"op":"release","permit":4,"now":7,"dequeued":[],"in_flight":1,"queued":0,"next_deadline":null}
{"final":true,"in_flight":1,"queued":0,"next_deadline":null}
"##;
    assert_eq!(replay(2, 2, &ops), expected);
}

// DESIGN.md Gate section; expected trace computed independently from the rules in docs/DESIGN.md.
#[test]
fn gate_sequence_a_release_with_an_empty_queue_returns_nothing_matches_the_expected_trace() {
    let ops = [
        submit(1, 0, None, 0),
        release(1, 4, Outcome::Success),
        submit(2, 0, None, 5),
    ];
    let expected = r##"{"step":0,"op":"submit","ticket":1,"now":0,"admission":"granted","acquired_at":0,"in_flight":1,"queued":0,"next_deadline":null}
{"step":1,"op":"release","permit":1,"now":4,"dequeued":[],"in_flight":0,"queued":0,"next_deadline":null}
{"step":2,"op":"submit","ticket":2,"now":5,"admission":"granted","acquired_at":5,"in_flight":1,"queued":0,"next_deadline":null}
{"final":true,"in_flight":1,"queued":0,"next_deadline":null}
"##;
    assert_eq!(replay(1, 3, &ops), expected);
}

// DESIGN.md Gate section; expected trace computed independently from the rules in docs/DESIGN.md.
#[test]
fn gate_sequence_a_release_drops_an_expired_head_then_grants_the_next_matches_the_expected_trace() {
    let ops = [
        submit(1, 0, None, 0),
        submit(2, 0, Some(10), 1),
        submit(3, 0, None, 2),
        release(1, 10, Outcome::Success),
    ];
    let expected = r##"{"step":0,"op":"submit","ticket":1,"now":0,"admission":"granted","acquired_at":0,"in_flight":1,"queued":0,"next_deadline":null}
{"step":1,"op":"submit","ticket":2,"now":1,"admission":"queued","in_flight":1,"queued":1,"next_deadline":10}
{"step":2,"op":"submit","ticket":3,"now":2,"admission":"queued","in_flight":1,"queued":2,"next_deadline":10}
{"step":3,"op":"release","permit":1,"now":10,"dequeued":[{"r":"expired","ticket":2},{"r":"granted","ticket":3,"acquired_at":10}],"in_flight":1,"queued":0,"next_deadline":null}
{"final":true,"in_flight":1,"queued":0,"next_deadline":null}
"##;
    assert_eq!(replay(1, 3, &ops), expected);
}

// DESIGN.md Gate section; expected trace computed independently from the rules in docs/DESIGN.md.
#[test]
fn gate_sequence_a_release_drops_every_expired_ticket_and_grants_none_matches_the_expected_trace() {
    let ops = [
        submit(1, 0, None, 0),
        submit(2, 0, Some(10), 1),
        submit(3, 0, Some(11), 2),
        release(1, 20, Outcome::Success),
        submit(4, 0, None, 21),
    ];
    let expected = r##"{"step":0,"op":"submit","ticket":1,"now":0,"admission":"granted","acquired_at":0,"in_flight":1,"queued":0,"next_deadline":null}
{"step":1,"op":"submit","ticket":2,"now":1,"admission":"queued","in_flight":1,"queued":1,"next_deadline":10}
{"step":2,"op":"submit","ticket":3,"now":2,"admission":"queued","in_flight":1,"queued":2,"next_deadline":10}
{"step":3,"op":"release","permit":1,"now":20,"dequeued":[{"r":"expired","ticket":2},{"r":"expired","ticket":3}],"in_flight":0,"queued":0,"next_deadline":null}
{"step":4,"op":"submit","ticket":4,"now":21,"admission":"granted","acquired_at":21,"in_flight":1,"queued":0,"next_deadline":null}
{"final":true,"in_flight":1,"queued":0,"next_deadline":null}
"##;
    assert_eq!(replay(1, 3, &ops), expected);
}

// DESIGN.md Gate section; expected trace computed independently from the rules in docs/DESIGN.md.
#[test]
fn gate_sequence_expire_removes_a_ticket_from_the_middle_matches_the_expected_trace() {
    let ops = [
        submit(1, 0, None, 0),
        submit(2, 0, None, 1),
        submit(3, 0, Some(5), 2),
        submit(4, 0, None, 3),
        submit(5, 0, Some(5), 4),
        expire(4),
        expire(5),
        release(1, 6, Outcome::Success),
        release(2, 7, Outcome::Success),
        release(4, 8, Outcome::Success),
    ];
    let expected = r##"{"step":0,"op":"submit","ticket":1,"now":0,"admission":"granted","acquired_at":0,"in_flight":1,"queued":0,"next_deadline":null}
{"step":1,"op":"submit","ticket":2,"now":1,"admission":"queued","in_flight":1,"queued":1,"next_deadline":null}
{"step":2,"op":"submit","ticket":3,"now":2,"admission":"queued","in_flight":1,"queued":2,"next_deadline":5}
{"step":3,"op":"submit","ticket":4,"now":3,"admission":"queued","in_flight":1,"queued":3,"next_deadline":5}
{"step":4,"op":"submit","ticket":5,"now":4,"admission":"queued","in_flight":1,"queued":4,"next_deadline":5}
{"step":5,"op":"expire","now":4,"expired":[],"in_flight":1,"queued":4,"next_deadline":5}
{"step":6,"op":"expire","now":5,"expired":[3,5],"in_flight":1,"queued":2,"next_deadline":null}
{"step":7,"op":"release","permit":1,"now":6,"dequeued":[{"r":"granted","ticket":2,"acquired_at":6}],"in_flight":1,"queued":1,"next_deadline":null}
{"step":8,"op":"release","permit":2,"now":7,"dequeued":[{"r":"granted","ticket":4,"acquired_at":7}],"in_flight":1,"queued":0,"next_deadline":null}
{"step":9,"op":"release","permit":4,"now":8,"dequeued":[],"in_flight":0,"queued":0,"next_deadline":null}
{"final":true,"in_flight":0,"queued":0,"next_deadline":null}
"##;
    assert_eq!(replay(1, 5, &ops), expected);
}

// DESIGN.md Gate section; expected trace computed independently from the rules in docs/DESIGN.md.
#[test]
fn gate_sequence_expire_removes_the_head_and_a_middle_ticket_in_queue_order_matches_the_expected_trace()
 {
    let ops = [
        submit(1, 0, None, 0),
        submit(2, 0, Some(7), 1),
        submit(3, 0, None, 2),
        submit(4, 0, Some(7), 3),
        submit(5, 0, Some(3), 4),
        expire(7),
    ];
    let expected = r##"{"step":0,"op":"submit","ticket":1,"now":0,"admission":"granted","acquired_at":0,"in_flight":1,"queued":0,"next_deadline":null}
{"step":1,"op":"submit","ticket":2,"now":1,"admission":"queued","in_flight":1,"queued":1,"next_deadline":7}
{"step":2,"op":"submit","ticket":3,"now":2,"admission":"queued","in_flight":1,"queued":2,"next_deadline":7}
{"step":3,"op":"submit","ticket":4,"now":3,"admission":"queued","in_flight":1,"queued":3,"next_deadline":7}
{"step":4,"op":"submit","ticket":5,"now":4,"admission":"rejected_expired","in_flight":1,"queued":3,"next_deadline":7}
{"step":5,"op":"expire","now":7,"expired":[2,4],"in_flight":1,"queued":1,"next_deadline":null}
{"final":true,"in_flight":1,"queued":1,"next_deadline":null}
"##;
    assert_eq!(replay(1, 5, &ops), expected);
}

// DESIGN.md Gate section; expected trace computed independently from the rules in docs/DESIGN.md.
#[test]
fn gate_sequence_expire_before_every_deadline_removes_nothing_matches_the_expected_trace() {
    let ops = [
        expire(0),
        submit(1, 0, None, 0),
        submit(2, 0, Some(100), 1),
        expire(99),
        expire(100),
    ];
    let expected = r##"{"step":0,"op":"expire","now":0,"expired":[],"in_flight":0,"queued":0,"next_deadline":null}
{"step":1,"op":"submit","ticket":1,"now":0,"admission":"granted","acquired_at":0,"in_flight":1,"queued":0,"next_deadline":null}
{"step":2,"op":"submit","ticket":2,"now":1,"admission":"queued","in_flight":1,"queued":1,"next_deadline":100}
{"step":3,"op":"expire","now":99,"expired":[],"in_flight":1,"queued":1,"next_deadline":100}
{"step":4,"op":"expire","now":100,"expired":[2],"in_flight":1,"queued":0,"next_deadline":null}
{"final":true,"in_flight":1,"queued":0,"next_deadline":null}
"##;
    assert_eq!(replay(1, 3, &ops), expected);
}

// DESIGN.md Gate section; expected trace computed independently from the rules in docs/DESIGN.md.
#[test]
fn gate_sequence_next_deadline_is_the_earliest_queued_deadline_matches_the_expected_trace() {
    let ops = [
        submit(1, 0, None, 0),
        submit(2, 0, Some(30), 1),
        submit(3, 0, Some(10), 2),
        submit(4, 0, None, 3),
        submit(5, 0, Some(20), 4),
        expire(10),
        expire(20),
        expire(30),
    ];
    let expected = r##"{"step":0,"op":"submit","ticket":1,"now":0,"admission":"granted","acquired_at":0,"in_flight":1,"queued":0,"next_deadline":null}
{"step":1,"op":"submit","ticket":2,"now":1,"admission":"queued","in_flight":1,"queued":1,"next_deadline":30}
{"step":2,"op":"submit","ticket":3,"now":2,"admission":"queued","in_flight":1,"queued":2,"next_deadline":10}
{"step":3,"op":"submit","ticket":4,"now":3,"admission":"queued","in_flight":1,"queued":3,"next_deadline":10}
{"step":4,"op":"submit","ticket":5,"now":4,"admission":"queued","in_flight":1,"queued":4,"next_deadline":10}
{"step":5,"op":"expire","now":10,"expired":[3],"in_flight":1,"queued":3,"next_deadline":20}
{"step":6,"op":"expire","now":20,"expired":[5],"in_flight":1,"queued":2,"next_deadline":30}
{"step":7,"op":"expire","now":30,"expired":[2],"in_flight":1,"queued":1,"next_deadline":null}
{"final":true,"in_flight":1,"queued":1,"next_deadline":null}
"##;
    assert_eq!(replay(1, 6, &ops), expected);
}

// DESIGN.md Gate section; expected trace computed independently from the rules in docs/DESIGN.md.
#[test]
fn gate_sequence_a_ticket_without_a_deadline_is_never_expired_matches_the_expected_trace() {
    let ops = [
        submit(1, 0, None, 0),
        submit(2, 0, None, 1),
        expire(18446744073709551615),
        release(1, 18446744073709551615, Outcome::Success),
    ];
    let expected = r##"{"step":0,"op":"submit","ticket":1,"now":0,"admission":"granted","acquired_at":0,"in_flight":1,"queued":0,"next_deadline":null}
{"step":1,"op":"submit","ticket":2,"now":1,"admission":"queued","in_flight":1,"queued":1,"next_deadline":null}
{"step":2,"op":"expire","now":18446744073709551615,"expired":[],"in_flight":1,"queued":1,"next_deadline":null}
{"step":3,"op":"release","permit":1,"now":18446744073709551615,"dequeued":[{"r":"granted","ticket":2,"acquired_at":18446744073709551615}],"in_flight":1,"queued":0,"next_deadline":null}
{"final":true,"in_flight":1,"queued":0,"next_deadline":null}
"##;
    assert_eq!(replay(1, 3, &ops), expected);
}

// DESIGN.md Gate section; expected trace computed independently from the rules in docs/DESIGN.md.
#[test]
fn gate_sequence_submit_does_not_remove_expired_tickets_from_the_queue_matches_the_expected_trace()
{
    let ops = [
        submit(1, 0, None, 0),
        submit(2, 0, Some(5), 1),
        submit(3, 0, None, 10),
        expire(10),
        submit(4, 0, None, 11),
    ];
    let expected = r##"{"step":0,"op":"submit","ticket":1,"now":0,"admission":"granted","acquired_at":0,"in_flight":1,"queued":0,"next_deadline":null}
{"step":1,"op":"submit","ticket":2,"now":1,"admission":"queued","in_flight":1,"queued":1,"next_deadline":5}
{"step":2,"op":"submit","ticket":3,"now":10,"admission":"rejected_queue_full","in_flight":1,"queued":1,"next_deadline":5}
{"step":3,"op":"expire","now":10,"expired":[2],"in_flight":1,"queued":0,"next_deadline":null}
{"step":4,"op":"submit","ticket":4,"now":11,"admission":"queued","in_flight":1,"queued":1,"next_deadline":null}
{"final":true,"in_flight":1,"queued":1,"next_deadline":null}
"##;
    assert_eq!(replay(1, 1, &ops), expected);
}

// DESIGN.md Gate section; expected trace computed independently from the rules in docs/DESIGN.md.
#[test]
fn gate_sequence_a_new_ticket_waits_behind_the_queued_ones_matches_the_expected_trace() {
    let ops = [
        submit(1, 0, None, 0),
        submit(2, 0, None, 1),
        release(1, 2, Outcome::Success),
        submit(3, 0, None, 3),
        release(2, 4, Outcome::Success),
    ];
    let expected = r##"{"step":0,"op":"submit","ticket":1,"now":0,"admission":"granted","acquired_at":0,"in_flight":1,"queued":0,"next_deadline":null}
{"step":1,"op":"submit","ticket":2,"now":1,"admission":"queued","in_flight":1,"queued":1,"next_deadline":null}
{"step":2,"op":"release","permit":1,"now":2,"dequeued":[{"r":"granted","ticket":2,"acquired_at":2}],"in_flight":1,"queued":0,"next_deadline":null}
{"step":3,"op":"submit","ticket":3,"now":3,"admission":"queued","in_flight":1,"queued":1,"next_deadline":null}
{"step":4,"op":"release","permit":2,"now":4,"dequeued":[{"r":"granted","ticket":3,"acquired_at":4}],"in_flight":1,"queued":0,"next_deadline":null}
{"final":true,"in_flight":1,"queued":0,"next_deadline":null}
"##;
    assert_eq!(replay(1, 3, &ops), expected);
}

// DESIGN.md Gate section; expected trace computed independently from the rules in docs/DESIGN.md.
#[test]
fn gate_sequence_deadlines_and_times_at_the_largest_value_matches_the_expected_trace() {
    let ops = [
        submit(1, 0, Some(18446744073709551615), 18446744073709551615),
        submit(2, 0, Some(18446744073709551615), 18446744073709551614),
        submit(3, 0, Some(18446744073709551615), 18446744073709551614),
        submit(4, 0, None, 18446744073709551614),
        expire(18446744073709551614),
        expire(18446744073709551615),
    ];
    let expected = r##"{"step":0,"op":"submit","ticket":1,"now":18446744073709551615,"admission":"rejected_expired","in_flight":0,"queued":0,"next_deadline":null}
{"step":1,"op":"submit","ticket":2,"now":18446744073709551614,"admission":"granted","acquired_at":18446744073709551614,"in_flight":1,"queued":0,"next_deadline":null}
{"step":2,"op":"submit","ticket":3,"now":18446744073709551614,"admission":"queued","in_flight":1,"queued":1,"next_deadline":18446744073709551615}
{"step":3,"op":"submit","ticket":4,"now":18446744073709551614,"admission":"queued","in_flight":1,"queued":2,"next_deadline":18446744073709551615}
{"step":4,"op":"expire","now":18446744073709551614,"expired":[],"in_flight":1,"queued":2,"next_deadline":18446744073709551615}
{"step":5,"op":"expire","now":18446744073709551615,"expired":[3],"in_flight":1,"queued":1,"next_deadline":null}
{"final":true,"in_flight":1,"queued":1,"next_deadline":null}
"##;
    assert_eq!(replay(1, 2, &ops), expected);
}

// DESIGN.md Gate section; expected trace computed independently from the rules in docs/DESIGN.md.
#[test]
fn gate_sequence_the_largest_queue_capacity_is_accepted_matches_the_expected_trace() {
    let ops = [
        submit(1, 0, None, 0),
        submit(2, 0, None, 1),
        submit(3, 0, None, 2),
        submit(4, 0, None, 3),
        release(1, 4, Outcome::Success),
    ];
    let expected = r##"{"step":0,"op":"submit","ticket":1,"now":0,"admission":"granted","acquired_at":0,"in_flight":1,"queued":0,"next_deadline":null}
{"step":1,"op":"submit","ticket":2,"now":1,"admission":"queued","in_flight":1,"queued":1,"next_deadline":null}
{"step":2,"op":"submit","ticket":3,"now":2,"admission":"queued","in_flight":1,"queued":2,"next_deadline":null}
{"step":3,"op":"submit","ticket":4,"now":3,"admission":"queued","in_flight":1,"queued":3,"next_deadline":null}
{"step":4,"op":"release","permit":1,"now":4,"dequeued":[{"r":"granted","ticket":2,"acquired_at":4}],"in_flight":1,"queued":2,"next_deadline":null}
{"final":true,"in_flight":1,"queued":2,"next_deadline":null}
"##;
    assert_eq!(replay(1, 4294967295, &ops), expected);
}

// DESIGN.md Gate section; expected trace computed independently from the rules in docs/DESIGN.md.
#[test]
fn gate_sequence_the_largest_limit_never_queues_matches_the_expected_trace() {
    let ops = [
        submit(1, 0, None, 0),
        submit(2, 0, None, 0),
        release(1, 1, Outcome::Success),
        release(2, 1, Outcome::Success),
    ];
    let expected = r##"{"step":0,"op":"submit","ticket":1,"now":0,"admission":"granted","acquired_at":0,"in_flight":1,"queued":0,"next_deadline":null}
{"step":1,"op":"submit","ticket":2,"now":0,"admission":"granted","acquired_at":0,"in_flight":2,"queued":0,"next_deadline":null}
{"step":2,"op":"release","permit":1,"now":1,"dequeued":[],"in_flight":1,"queued":0,"next_deadline":null}
{"step":3,"op":"release","permit":2,"now":1,"dequeued":[],"in_flight":0,"queued":0,"next_deadline":null}
{"final":true,"in_flight":0,"queued":0,"next_deadline":null}
"##;
    assert_eq!(replay(4294967295, 0, &ops), expected);
}

// DESIGN.md Gate section; expected trace computed independently from the rules in docs/DESIGN.md.
#[test]
fn gate_sequence_a_permit_from_the_queue_is_stamped_with_the_release_time_matches_the_expected_trace()
 {
    let ops = [
        submit(1, 0, None, 3),
        submit(2, 0, None, 4),
        release(1, 90, Outcome::Success),
    ];
    let expected = r##"{"step":0,"op":"submit","ticket":1,"now":3,"admission":"granted","acquired_at":3,"in_flight":1,"queued":0,"next_deadline":null}
{"step":1,"op":"submit","ticket":2,"now":4,"admission":"queued","in_flight":1,"queued":1,"next_deadline":null}
{"step":2,"op":"release","permit":1,"now":90,"dequeued":[{"r":"granted","ticket":2,"acquired_at":90}],"in_flight":1,"queued":0,"next_deadline":null}
{"final":true,"in_flight":1,"queued":0,"next_deadline":null}
"##;
    assert_eq!(replay(1, 2, &ops), expected);
}

// DESIGN.md Gate section; expected trace computed independently from the rules in docs/DESIGN.md.
#[test]
fn gate_sequence_every_outcome_frees_the_slot_the_same_way_matches_the_expected_trace() {
    let ops = [
        submit(1, 0, None, 0),
        submit(2, 0, None, 1),
        submit(3, 0, None, 2),
        release(1, 5, Outcome::Error),
        release(2, 6, Outcome::Cancelled),
        release(3, 7, Outcome::Success),
    ];
    let expected = r##"{"step":0,"op":"submit","ticket":1,"now":0,"admission":"granted","acquired_at":0,"in_flight":1,"queued":0,"next_deadline":null}
{"step":1,"op":"submit","ticket":2,"now":1,"admission":"queued","in_flight":1,"queued":1,"next_deadline":null}
{"step":2,"op":"submit","ticket":3,"now":2,"admission":"queued","in_flight":1,"queued":2,"next_deadline":null}
{"step":3,"op":"release","permit":1,"now":5,"dequeued":[{"r":"granted","ticket":2,"acquired_at":5}],"in_flight":1,"queued":1,"next_deadline":null}
{"step":4,"op":"release","permit":2,"now":6,"dequeued":[{"r":"granted","ticket":3,"acquired_at":6}],"in_flight":1,"queued":0,"next_deadline":null}
{"step":5,"op":"release","permit":3,"now":7,"dequeued":[],"in_flight":0,"queued":0,"next_deadline":null}
{"final":true,"in_flight":0,"queued":0,"next_deadline":null}
"##;
    assert_eq!(replay(1, 2, &ops), expected);
}

// DESIGN.md Gate section; expected trace computed independently from the rules in docs/DESIGN.md.
#[test]
fn gate_sequence_an_expired_head_stays_queued_until_a_release_or_expire_matches_the_expected_trace()
{
    let ops = [
        submit(1, 0, None, 0),
        submit(2, 0, Some(4), 1),
        submit(3, 0, Some(4), 2),
        submit(4, 0, None, 9),
        release(1, 9, Outcome::Success),
    ];
    let expected = r##"{"step":0,"op":"submit","ticket":1,"now":0,"admission":"granted","acquired_at":0,"in_flight":1,"queued":0,"next_deadline":null}
{"step":1,"op":"submit","ticket":2,"now":1,"admission":"queued","in_flight":1,"queued":1,"next_deadline":4}
{"step":2,"op":"submit","ticket":3,"now":2,"admission":"queued","in_flight":1,"queued":2,"next_deadline":4}
{"step":3,"op":"submit","ticket":4,"now":9,"admission":"queued","in_flight":1,"queued":3,"next_deadline":4}
{"step":4,"op":"release","permit":1,"now":9,"dequeued":[{"r":"expired","ticket":2},{"r":"expired","ticket":3},{"r":"granted","ticket":4,"acquired_at":9}],"in_flight":1,"queued":0,"next_deadline":null}
{"final":true,"in_flight":1,"queued":0,"next_deadline":null}
"##;
    assert_eq!(replay(1, 3, &ops), expected);
}

// DESIGN.md Gate section; expected trace computed independently from the rules in docs/DESIGN.md.
#[test]
fn gate_sequence_a_release_drops_expired_heads_behind_the_last_grant_matches_the_expected_trace() {
    let ops = [
        submit(1, 0, None, 0),
        submit(2, 0, Some(5), 1),
        submit(3, 0, None, 2),
        submit(4, 0, Some(8), 3),
        submit(5, 0, None, 4),
        release(1, 9, Outcome::Success),
    ];
    let expected = r##"{"step":0,"op":"submit","ticket":1,"now":0,"admission":"granted","acquired_at":0,"in_flight":1,"queued":0,"next_deadline":null}
{"step":1,"op":"submit","ticket":2,"now":1,"admission":"queued","in_flight":1,"queued":1,"next_deadline":5}
{"step":2,"op":"submit","ticket":3,"now":2,"admission":"queued","in_flight":1,"queued":2,"next_deadline":5}
{"step":3,"op":"submit","ticket":4,"now":3,"admission":"queued","in_flight":1,"queued":3,"next_deadline":5}
{"step":4,"op":"submit","ticket":5,"now":4,"admission":"queued","in_flight":1,"queued":4,"next_deadline":5}
{"step":5,"op":"release","permit":1,"now":9,"dequeued":[{"r":"expired","ticket":2},{"r":"granted","ticket":3,"acquired_at":9},{"r":"expired","ticket":4}],"in_flight":1,"queued":1,"next_deadline":null}
{"final":true,"in_flight":1,"queued":1,"next_deadline":null}
"##;
    assert_eq!(replay(1, 5, &ops), expected);
}
