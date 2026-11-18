//! Property tests: the invariants hold on traces of random scenarios.

use std::collections::HashMap;

use proptest::prelude::*;
use vana_core::Nanos;
use vana_sim::{FixedScenario, TraceEvent, run_fixed, to_json_lines};

fn parts(e: &TraceEvent) -> (&'static str, u64, u64, u64, u32) {
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

/// Checks every rule of the simulation that can be seen in the trace alone.
/// Returns the first violation.
fn audit(s: &FixedScenario, events: &[TraceEvent]) -> Result<(), String> {
    let mut in_flight: u32 = 0;
    let mut last_t: u64 = 0;
    let mut last_arrive_t: Option<u64> = None;
    let mut next_ticket: u64 = 0;
    let mut grants: HashMap<u64, u64> = HashMap::new(); // ticket -> grant time
    let mut rejected: Vec<u64> = Vec::new();
    let mut released: Vec<u64> = Vec::new();
    let mut previous: Option<(&'static str, u64, u64)> = None;

    for (i, e) in events.iter().enumerate() {
        let (kind, t, ticket, tenant, after) = parts(e);
        if t < last_t {
            return Err(format!("event {i}: time {t} goes back from {last_t}"));
        }
        last_t = t;
        if after > s.limit {
            return Err(format!("event {i}: in_flight {after} > limit {}", s.limit));
        }
        if tenant != ticket % s.tenants {
            return Err(format!(
                "event {i}: tenant {tenant} != ticket {ticket} % {}",
                s.tenants
            ));
        }
        match kind {
            "arrive" => {
                if ticket != next_ticket {
                    return Err(format!(
                        "event {i}: arrival of {ticket}, expected {next_ticket}"
                    ));
                }
                next_ticket += 1;
                if after != in_flight {
                    return Err(format!("event {i}: arrive changed in_flight"));
                }
                last_arrive_t = Some(t);
            }
            "grant" => {
                if previous != Some(("arrive", t, ticket)) {
                    return Err(format!("event {i}: grant does not follow its arrive"));
                }
                if in_flight >= s.limit {
                    return Err(format!(
                        "event {i}: grant with in_flight {in_flight} at limit"
                    ));
                }
                if after != in_flight + 1 {
                    return Err(format!(
                        "event {i}: grant in_flight {after}, expected {}",
                        in_flight + 1
                    ));
                }
                in_flight = after;
                grants.insert(ticket, t);
            }
            "reject" => {
                if previous != Some(("arrive", t, ticket)) {
                    return Err(format!("event {i}: reject does not follow its arrive"));
                }
                if in_flight != s.limit {
                    return Err(format!(
                        "event {i}: reject with in_flight {in_flight} below limit"
                    ));
                }
                if after != in_flight {
                    return Err(format!("event {i}: reject changed in_flight"));
                }
                rejected.push(ticket);
            }
            "release" => {
                let Some(granted_at) = grants.get(&ticket) else {
                    return Err(format!(
                        "event {i}: release of ticket {ticket} without a grant"
                    ));
                };
                if t < *granted_at {
                    return Err(format!("event {i}: release before the grant"));
                }
                // At the saturation point a completion can equal the arrival
                // time of the ticket that was just granted.
                if let Some(arrive_t) = last_arrive_t
                    && t <= arrive_t
                    && t != u64::MAX
                {
                    return Err(format!(
                        "event {i}: release at {t} comes after an arrival at {arrive_t}"
                    ));
                }
                if released.contains(&ticket) {
                    return Err(format!("event {i}: ticket {ticket} released twice"));
                }
                if let Some(("release", pt, pticket)) = previous
                    && (t, ticket) <= (pt, pticket)
                {
                    return Err(format!("event {i}: releases out of (time, ticket) order"));
                }
                if in_flight == 0 || after != in_flight - 1 {
                    return Err(format!(
                        "event {i}: release in_flight {after} from {in_flight}"
                    ));
                }
                in_flight = after;
                released.push(ticket);
            }
            _ => unreachable!(),
        }
        previous = Some((kind, t, ticket));
    }

    if next_ticket != u64::from(s.requests) {
        return Err(format!("{next_ticket} arrivals, expected {}", s.requests));
    }
    if in_flight != 0 {
        return Err(format!("{in_flight} still in flight at the end"));
    }
    if released.len() != grants.len() {
        return Err(format!(
            "{} grants but {} releases",
            grants.len(),
            released.len()
        ));
    }
    for ticket in &rejected {
        if grants.contains_key(ticket) || released.contains(ticket) {
            return Err(format!("rejected ticket {ticket} has a grant or release"));
        }
    }
    if grants.len() + rejected.len() != s.requests as usize {
        return Err("grants and rejects do not add up to the requests".to_string());
    }
    Ok(())
}

fn scenario_strategy() -> impl Strategy<Value = FixedScenario> {
    (
        any::<u64>(),
        0u32..60,
        1u64..6,
        1u32..6,
        1u64..500,
        1u64..2000,
    )
        .prop_map(
            |(seed, requests, tenants, limit, gap, service)| FixedScenario {
                seed,
                requests,
                tenants,
                limit,
                mean_gap: Nanos::new(gap),
                mean_service: Nanos::new(service),
            },
        )
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    // I1: in every trace in_flight never exceeds the limit, a grant happens
    // only below the limit and a reject only at the limit.
    #[test]
    fn i1_traces_never_exceed_the_limit(s in scenario_strategy()) {
        let events = run_fixed(&s).expect("valid scenario");
        let result = audit(&s, &events);
        prop_assert!(result.is_ok(), "{:?}", result);
    }

    // I2: every Grant has exactly one Release and no Reject has one.
    #[test]
    fn i2_every_grant_is_released_exactly_once(s in scenario_strategy()) {
        let events = run_fixed(&s).expect("valid scenario");
        let mut grants = 0usize;
        let mut releases = 0usize;
        let mut granted = std::collections::BTreeSet::new();
        let mut released = std::collections::BTreeSet::new();
        for e in &events {
            match parts(e) {
                ("grant", _, ticket, _, _) => { grants += 1; granted.insert(ticket); }
                ("release", _, ticket, _, _) => { releases += 1; released.insert(ticket); }
                ("reject", _, ticket, _, _) => {
                    prop_assert!(!granted.contains(&ticket) && !released.contains(&ticket));
                }
                _ => {}
            }
        }
        prop_assert_eq!(grants, releases);
        prop_assert_eq!(granted, released);
    }

    // I7: the same scenario gives the same bytes every time.
    #[test]
    fn i7_the_same_scenario_gives_the_same_bytes(s in scenario_strategy()) {
        let first = to_json_lines(&run_fixed(&s).expect("valid scenario"));
        let second = to_json_lines(&run_fixed(&s.clone()).expect("valid scenario"));
        prop_assert_eq!(first, second);
    }

    // DESIGN.md, simulation rules: the JSON text has one terminated line per event.
    #[test]
    fn json_has_one_line_per_event(s in scenario_strategy()) {
        let events = run_fixed(&s).expect("valid scenario");
        let text = to_json_lines(&events);
        prop_assert_eq!(text.matches('\n').count(), events.len());
        prop_assert!(text.is_empty() || text.ends_with('\n'));
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    // Saturating arithmetic: the largest means never panic and the trace stays valid.
    #[test]
    fn extreme_means_still_give_a_valid_trace(
        seed in any::<u64>(),
        requests in 0u32..30,
        limit in 1u32..4,
        gap in prop_oneof![Just(1u64), Just(u64::MAX), Just(u64::MAX / 2 + 1)],
        service in prop_oneof![Just(1u64), Just(u64::MAX), Just(u64::MAX / 2 + 1)],
    ) {
        let s = FixedScenario {
            seed,
            requests,
            tenants: 2,
            limit,
            mean_gap: Nanos::new(gap),
            mean_service: Nanos::new(service),
        };
        let events = run_fixed(&s).expect("valid scenario");
        let result = audit(&s, &events);
        prop_assert!(result.is_ok(), "{:?}", result);
    }
}
