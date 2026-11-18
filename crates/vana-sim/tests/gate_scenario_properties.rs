//! Property tests: the invariants hold on traces of random gate scenarios.

use std::collections::HashMap;

use proptest::prelude::*;
use vana_core::{Nanos, RejectReason};
use vana_sim::{
    FixedScenario, GateEvent, GateScenario, TraceEvent, gate_json_lines, run_fixed, run_gate,
    to_json_lines,
};

struct Row {
    kind: &'static str,
    t: u64,
    ticket: u64,
    tenant: u64,
    in_flight: u32,
    queued: u32,
    reason: Option<RejectReason>,
}

fn row(e: &GateEvent) -> Row {
    let (kind, t, ticket, tenant, in_flight, queued, reason) = match e {
        GateEvent::Arrive {
            t,
            ticket,
            tenant,
            in_flight,
            queued,
        } => ("arrive", t, ticket, tenant, in_flight, queued, None),
        GateEvent::Grant {
            t,
            ticket,
            tenant,
            in_flight,
            queued,
        } => ("grant", t, ticket, tenant, in_flight, queued, None),
        GateEvent::Queue {
            t,
            ticket,
            tenant,
            in_flight,
            queued,
        } => ("queue", t, ticket, tenant, in_flight, queued, None),
        GateEvent::Reject {
            t,
            ticket,
            tenant,
            in_flight,
            queued,
            reason,
        } => (
            "reject",
            t,
            ticket,
            tenant,
            in_flight,
            queued,
            Some(*reason),
        ),
        GateEvent::Expire {
            t,
            ticket,
            tenant,
            in_flight,
            queued,
        } => ("expire", t, ticket, tenant, in_flight, queued, None),
        GateEvent::Release {
            t,
            ticket,
            tenant,
            in_flight,
            queued,
        } => ("release", t, ticket, tenant, in_flight, queued, None),
    };
    Row {
        kind,
        t: t.get(),
        ticket: ticket.get(),
        tenant: tenant.get(),
        in_flight: *in_flight,
        queued: *queued,
        reason,
    }
}

/// Where a ticket is in its life.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum State {
    Arrived,
    Queued,
    Running,
    Ended,
}

/// Checks every rule of the simulation that can be seen in the trace alone.
/// Returns the first violation.
fn audit(s: &GateScenario, events: &[GateEvent]) -> Result<(), String> {
    let rows: Vec<Row> = events.iter().map(row).collect();
    let mut in_flight: u32 = 0;
    let mut queued: u32 = 0;
    let mut last_t: u64 = 0;
    let mut next_ticket: u64 = 0;
    let mut state: HashMap<u64, State> = HashMap::new();
    let mut deadline: HashMap<u64, Option<u64>> = HashMap::new();
    let mut grant_t: HashMap<u64, u64> = HashMap::new();
    let mut last_arrival_t: Option<u64> = None;
    let mut last_queue_grant: Option<u64> = None;
    let mut ended = 0u64;

    for (i, r) in rows.iter().enumerate() {
        if r.t < last_t {
            return Err(format!("row {i}: time {} goes back from {last_t}", r.t));
        }
        last_t = r.t;
        if r.in_flight > s.limit {
            return Err(format!(
                "row {i}: I1 in_flight {} > limit {}",
                r.in_flight, s.limit
            ));
        }
        if r.queued > s.queue_capacity {
            return Err(format!(
                "row {i}: queued {} > capacity {}",
                r.queued, s.queue_capacity
            ));
        }
        if r.tenant != r.ticket % s.tenants {
            return Err(format!(
                "row {i}: tenant {} != ticket {} % {}",
                r.tenant, r.ticket, s.tenants
            ));
        }
        if r.reason.is_some() != (r.kind == "reject") {
            return Err(format!("row {i}: reason present only on reject"));
        }
        let cur = state.get(&r.ticket).copied();
        match r.kind {
            "arrive" => {
                if r.ticket != next_ticket {
                    return Err(format!(
                        "row {i}: arrival of {}, expected {next_ticket}",
                        r.ticket
                    ));
                }
                next_ticket += 1;
                if last_arrival_t.is_some_and(|prev| r.t < prev) {
                    return Err(format!("row {i}: arrivals go back in time"));
                }
                last_arrival_t = Some(r.t);
                state.insert(r.ticket, State::Arrived);
                deadline.insert(r.ticket, s.timeout.map(|to| r.t.saturating_add(to.get())));
            }
            "queue" => {
                if cur != Some(State::Arrived) {
                    return Err(format!("row {i}: queue of ticket {} in {cur:?}", r.ticket));
                }
                if i == 0
                    || rows[i - 1].kind != "arrive"
                    || rows[i - 1].ticket != r.ticket
                    || rows[i - 1].t != r.t
                {
                    return Err(format!("row {i}: queue does not follow its arrive"));
                }
                // Work conservation: a ticket queues only when the limiter is full.
                if r.in_flight != s.limit {
                    return Err(format!(
                        "row {i}: I8 queued with in_flight {} < limit",
                        r.in_flight
                    ));
                }
                queued += 1;
                state.insert(r.ticket, State::Queued);
            }
            "reject" => {
                if cur != Some(State::Arrived) {
                    return Err(format!("row {i}: reject of ticket {} in {cur:?}", r.ticket));
                }
                if rows[i - 1].kind != "arrive"
                    || rows[i - 1].ticket != r.ticket
                    || rows[i - 1].t != r.t
                {
                    return Err(format!("row {i}: reject does not follow its arrive"));
                }
                let dl = deadline[&r.ticket];
                match r.reason {
                    Some(RejectReason::Expired) => {
                        if dl.is_none_or(|d| d > r.t) {
                            return Err(format!(
                                "row {i}: expired reject with deadline {dl:?} at {}",
                                r.t
                            ));
                        }
                    }
                    Some(RejectReason::QueueFull) => {
                        if dl.is_some_and(|d| d <= r.t) {
                            return Err(format!("row {i}: queue_full reject of an expired ticket"));
                        }
                        if r.in_flight != s.limit || r.queued != s.queue_capacity {
                            return Err(format!(
                                "row {i}: queue_full with in_flight {} queued {}",
                                r.in_flight, r.queued
                            ));
                        }
                    }
                    None => unreachable!(),
                }
                state.insert(r.ticket, State::Ended);
                ended += 1;
            }
            "grant" => {
                let from_queue = match cur {
                    Some(State::Arrived) => {
                        if rows[i - 1].kind != "arrive"
                            || rows[i - 1].ticket != r.ticket
                            || rows[i - 1].t != r.t
                        {
                            return Err(format!("row {i}: grant does not follow its arrive"));
                        }
                        // No overtaking: a ticket granted on arrival found an empty queue.
                        if r.queued != 0 {
                            return Err(format!(
                                "row {i}: granted on arrival over a queue of {}",
                                r.queued
                            ));
                        }
                        false
                    }
                    Some(State::Queued) => {
                        queued -= 1;
                        true
                    }
                    other => {
                        return Err(format!(
                            "row {i}: grant of ticket {} in {other:?}",
                            r.ticket
                        ));
                    }
                };
                if from_queue {
                    // First in, first out: tickets leave the queue by grant in arrival order.
                    if last_queue_grant.is_some_and(|prev| prev > r.ticket) {
                        return Err(format!(
                            "row {i}: FIFO broken, ticket {} after {:?}",
                            r.ticket, last_queue_grant
                        ));
                    }
                    last_queue_grant = Some(r.ticket);
                }
                // I3: never granted at or after its deadline.
                if deadline[&r.ticket].is_some_and(|d| r.t >= d) {
                    return Err(format!(
                        "row {i}: I3 ticket {} granted at {} with deadline {:?}",
                        r.ticket, r.t, deadline[&r.ticket]
                    ));
                }
                in_flight += 1;
                grant_t.insert(r.ticket, r.t);
                state.insert(r.ticket, State::Running);
            }
            "expire" => {
                if cur != Some(State::Queued) {
                    return Err(format!("row {i}: expire of ticket {} in {cur:?}", r.ticket));
                }
                // The clock stops at each queued deadline, so a ticket is
                // dropped exactly at its deadline.
                match deadline[&r.ticket] {
                    Some(d) if d == r.t => {}
                    other => {
                        return Err(format!("row {i}: expire at {} but deadline {other:?}", r.t));
                    }
                }
                queued -= 1;
                state.insert(r.ticket, State::Ended);
                ended += 1;
            }
            "release" => {
                if cur != Some(State::Running) {
                    return Err(format!(
                        "row {i}: release of ticket {} in {cur:?}",
                        r.ticket
                    ));
                }
                if r.t < grant_t[&r.ticket].saturating_add(1) && r.t != u64::MAX {
                    return Err(format!(
                        "row {i}: ticket {} released at {} after grant at {}",
                        r.ticket, r.t, grant_t[&r.ticket]
                    ));
                }
                in_flight -= 1;
                state.insert(r.ticket, State::Ended);
                ended += 1;
            }
            _ => unreachable!(),
        }
        if r.in_flight != in_flight || r.queued != queued {
            return Err(format!(
                "row {i}: reported in_flight {} queued {}, replay says {in_flight} {queued}",
                r.in_flight, r.queued
            ));
        }
    }

    // I8: at the last row of each time group, a queued ticket means a full limiter.
    for (i, r) in rows.iter().enumerate() {
        let last_of_group = rows.get(i + 1).is_none_or(|n| n.t != r.t);
        if last_of_group && r.queued > 0 && r.in_flight != s.limit {
            return Err(format!(
                "row {i}: I8 queued {} with in_flight {} < {}",
                r.queued, r.in_flight, s.limit
            ));
        }
    }

    // Every ticket reaches exactly one end.
    if next_ticket != u64::from(s.requests) || ended != u64::from(s.requests) {
        return Err(format!(
            "{} arrivals, {ended} ended, {} requests",
            next_ticket, s.requests
        ));
    }
    if in_flight != 0 || queued != 0 {
        return Err(format!(
            "run ended with in_flight {in_flight}, queued {queued}"
        ));
    }
    if s.queue_capacity == 0 && rows.iter().any(|r| r.kind == "queue" || r.kind == "expire") {
        return Err("queue or expire event without a queue".to_string());
    }
    if s.timeout.is_none()
        && rows
            .iter()
            .any(|r| r.kind == "expire" || r.reason == Some(RejectReason::Expired))
    {
        return Err("expiry without a deadline".to_string());
    }
    Ok(())
}

fn scenario() -> impl Strategy<Value = GateScenario> {
    (
        any::<u64>(),
        0u32..40,
        1u64..5,
        1u32..5,
        0u32..6,
        prop::option::weighted(0.6, 0u64..800),
        1u64..200,
        1u64..400,
    )
        .prop_map(
            |(seed, requests, tenants, limit, queue_capacity, timeout, gap, service)| {
                GateScenario {
                    seed,
                    requests,
                    tenants,
                    limit,
                    queue_capacity,
                    timeout: timeout.map(Nanos::new),
                    mean_gap: Nanos::new(gap),
                    mean_service: Nanos::new(service),
                }
            },
        )
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(300))]

    // I1, I3, I8, FIFO, one end per ticket: every rule that shows in the trace.
    #[test]
    fn i1_i3_i8_and_one_end_per_ticket_hold_on_random_scenarios(s in scenario()) {
        let events = run_gate(&s).expect("the scenario is valid");
        prop_assert_eq!(audit(&s, &events), Ok(()));
    }

    // I7: the same scenario gives the same trace, byte for byte.
    #[test]
    fn i7_the_same_gate_scenario_gives_the_same_trace(s in scenario()) {
        let first = run_gate(&s).expect("the scenario is valid");
        let second = run_gate(&s.clone()).expect("the scenario is valid");
        prop_assert_eq!(gate_json_lines(&first), gate_json_lines(&second));
        prop_assert_eq!(first, second);
    }

    // DESIGN.md gate run: with no queue and no deadline the decisions are the fixed run's.
    #[test]
    fn with_no_queue_and_no_timeout_the_decisions_equal_run_fixed(
        seed in any::<u64>(),
        requests in 0u32..40,
        tenants in 1u64..5,
        limit in 1u32..5,
        gap in 1u64..200,
        service in 1u64..400,
    ) {
        let gate = GateScenario {
            seed,
            requests,
            tenants,
            limit,
            queue_capacity: 0,
            timeout: None,
            mean_gap: Nanos::new(gap),
            mean_service: Nanos::new(service),
        };
        let fixed = FixedScenario {
            seed,
            requests,
            tenants,
            limit,
            mean_gap: Nanos::new(gap),
            mean_service: Nanos::new(service),
        };
        let from_gate = run_gate(&gate).expect("the scenario is valid");
        let from_fixed = run_fixed(&fixed).expect("the scenario is valid");
        prop_assert_eq!(from_gate.len(), from_fixed.len());
        for (g, f) in from_gate.iter().zip(from_fixed.iter()) {
            prop_assert_eq!(projection_gate(g), projection_fixed(f));
        }
    }
}

type Projection = (&'static str, u64, u64, u64, u32);

fn projection_gate(e: &GateEvent) -> Projection {
    let r = row(e);
    assert_eq!(r.queued, 0, "a gate with no queue never queues");
    if r.kind == "reject" {
        assert_eq!(r.reason, Some(RejectReason::QueueFull));
    }
    (r.kind, r.t, r.ticket, r.tenant, r.in_flight)
}

fn projection_fixed(e: &TraceEvent) -> Projection {
    match e {
        TraceEvent::Arrive {
            t,
            ticket,
            tenant,
            in_flight,
        } => ("arrive", t.get(), ticket.get(), tenant.get(), *in_flight),
        TraceEvent::Grant {
            t,
            ticket,
            tenant,
            in_flight,
        } => ("grant", t.get(), ticket.get(), tenant.get(), *in_flight),
        TraceEvent::Reject {
            t,
            ticket,
            tenant,
            in_flight,
        } => ("reject", t.get(), ticket.get(), tenant.get(), *in_flight),
        TraceEvent::Release {
            t,
            ticket,
            tenant,
            in_flight,
        } => ("release", t.get(), ticket.get(), tenant.get(), *in_flight),
    }
}

// I7: JSON lines of the gate run and of the fixed run agree on the shared fields (text level).
#[test]
fn the_json_text_of_a_no_queue_run_extends_the_fixed_text_line_by_line() {
    let gate = GateScenario {
        seed: 42,
        requests: 12,
        tenants: 2,
        limit: 2,
        queue_capacity: 0,
        timeout: None,
        mean_gap: Nanos::new(100),
        mean_service: Nanos::new(250),
    };
    let fixed = FixedScenario {
        seed: 42,
        requests: 12,
        tenants: 2,
        limit: 2,
        mean_gap: Nanos::new(100),
        mean_service: Nanos::new(250),
    };
    let g = gate_json_lines(&run_gate(&gate).expect("valid"));
    let f = to_json_lines(&run_fixed(&fixed).expect("valid"));
    assert_eq!(g.lines().count(), f.lines().count());
    for (gl, fl) in g.lines().zip(f.lines()) {
        // fixed: {...,"in_flight":N}   gate: {...,"in_flight":N,"queued":0[,"reason":"queue_full"]}
        let prefix = fl.strip_suffix('}').expect("closing brace");
        assert!(gl.starts_with(prefix), "{gl} does not start with {prefix}");
        let rest = &gl[prefix.len()..];
        let want_reject = ",\"queued\":0,\"reason\":\"queue_full\"}";
        let want_other = ",\"queued\":0}";
        assert!(
            rest == want_other || (fl.contains("\"ev\":\"reject\"") && rest == want_reject),
            "unexpected tail {rest:?} on {gl}"
        );
    }
}
