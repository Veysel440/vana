//! Black-box tests for scenario errors, the JSON trace format and determinism.

use vana_core::{Nanos, TenantId, TicketId};
use vana_sim::{FixedScenario, ScenarioError, TraceEvent, run_fixed, to_json_lines};

fn scenario(seed: u64) -> FixedScenario {
    FixedScenario {
        seed,
        requests: 40,
        tenants: 3,
        limit: 2,
        mean_gap: Nanos::new(50),
        mean_service: Nanos::new(200),
    }
}

const VARIANTS: [ScenarioError; 4] = [
    ScenarioError::ZeroLimit,
    ScenarioError::ZeroTenants,
    ScenarioError::ZeroMeanGap,
    ScenarioError::ZeroMeanService,
];

fn event(kind: &str, t: u64, ticket: u64, tenant: u64, in_flight: u32) -> TraceEvent {
    let (t, ticket, tenant) = (Nanos::new(t), TicketId::new(ticket), TenantId::new(tenant));
    match kind {
        "arrive" => TraceEvent::Arrive {
            t,
            ticket,
            tenant,
            in_flight,
        },
        "grant" => TraceEvent::Grant {
            t,
            ticket,
            tenant,
            in_flight,
        },
        "reject" => TraceEvent::Reject {
            t,
            ticket,
            tenant,
            in_flight,
        },
        "release" => TraceEvent::Release {
            t,
            ticket,
            tenant,
            in_flight,
        },
        other => panic!("unknown kind {other}"),
    }
}

// DESIGN.md, simulation rules (invalid scenarios): each message is non-empty.
#[test]
fn every_scenario_error_has_a_message() {
    for error in VARIANTS {
        assert!(!error.to_string().is_empty(), "{error:?}");
    }
}

// DESIGN.md, simulation rules (invalid scenarios): the four messages differ.
#[test]
fn the_four_scenario_error_messages_are_pairwise_distinct() {
    for (i, a) in VARIANTS.iter().enumerate() {
        for b in &VARIANTS[i + 1..] {
            assert_ne!(a.to_string(), b.to_string(), "{a:?} and {b:?}");
        }
    }
}

// DESIGN.md, simulation rules (invalid scenarios): ScenarioError is a std error.
#[test]
fn a_scenario_error_is_a_standard_error() {
    fn assert_traits<T: Clone + Copy + core::fmt::Debug + PartialEq + Eq + std::error::Error>() {}
    assert_traits::<ScenarioError>();
    let boxed: Box<dyn std::error::Error> = Box::new(ScenarioError::ZeroMeanGap);
    assert_eq!(boxed.to_string(), ScenarioError::ZeroMeanGap.to_string());
}

// DESIGN.md, simulation rules (invalid scenarios): every field alone triggers its own variant.
#[test]
fn each_zero_field_gives_its_own_error() {
    let mut s = scenario(1);
    s.limit = 0;
    assert_eq!(run_fixed(&s).unwrap_err(), ScenarioError::ZeroLimit);
    let mut s = scenario(1);
    s.tenants = 0;
    assert_eq!(run_fixed(&s).unwrap_err(), ScenarioError::ZeroTenants);
    let mut s = scenario(1);
    s.mean_gap = Nanos::ZERO;
    assert_eq!(run_fixed(&s).unwrap_err(), ScenarioError::ZeroMeanGap);
    let mut s = scenario(1);
    s.mean_service = Nanos::ZERO;
    assert_eq!(run_fixed(&s).unwrap_err(), ScenarioError::ZeroMeanService);
}

// DESIGN.md, simulation rules: with several zero fields the first in enum order is reported.
#[test]
fn the_first_zero_field_in_enum_order_wins() {
    let mut s = scenario(1);
    s.tenants = 0;
    s.mean_gap = Nanos::ZERO;
    s.mean_service = Nanos::ZERO;
    assert_eq!(run_fixed(&s).unwrap_err(), ScenarioError::ZeroTenants);

    let mut s = scenario(1);
    s.mean_gap = Nanos::ZERO;
    s.mean_service = Nanos::ZERO;
    assert_eq!(run_fixed(&s).unwrap_err(), ScenarioError::ZeroMeanGap);

    let mut s = scenario(1);
    s.limit = 0;
    s.mean_service = Nanos::ZERO;
    assert_eq!(run_fixed(&s).unwrap_err(), ScenarioError::ZeroLimit);
}

// DESIGN.md, simulation rules: one request with the smallest valid values works.
#[test]
fn the_smallest_valid_scenario_runs() {
    let s = FixedScenario {
        seed: 0,
        requests: 1,
        tenants: 1,
        limit: 1,
        mean_gap: Nanos::new(1),
        mean_service: Nanos::new(1),
    };
    let trace = run_fixed(&s).expect("valid");
    assert_eq!(trace.len(), 3);
}

// DESIGN.md, simulation rules: a trace of an empty slice is the empty string.
#[test]
fn the_json_of_no_events_is_the_empty_string() {
    assert_eq!(to_json_lines(&[]), "");
}

// DESIGN.md, simulation rules (JSON line): keys in the documented order, no spaces.
#[test]
fn an_arrive_event_is_one_json_line_with_the_documented_keys() {
    assert_eq!(
        event("arrive", 5, 0, 1, 0).to_json_line(),
        r#"{"t":5,"ev":"arrive","ticket":0,"tenant":1,"in_flight":0}"#
    );
}

// DESIGN.md, simulation rules (JSON line): the four event names.
#[test]
fn each_event_kind_has_its_own_name_in_the_json_line() {
    assert_eq!(
        event("grant", 6, 2, 0, 1).to_json_line(),
        r#"{"t":6,"ev":"grant","ticket":2,"tenant":0,"in_flight":1}"#
    );
    assert_eq!(
        event("reject", 7, 3, 1, 2).to_json_line(),
        r#"{"t":7,"ev":"reject","ticket":3,"tenant":1,"in_flight":2}"#
    );
    assert_eq!(
        event("release", 8, 4, 2, 1).to_json_line(),
        r#"{"t":8,"ev":"release","ticket":4,"tenant":2,"in_flight":1}"#
    );
}

// DESIGN.md, simulation rules (JSON line): the largest values print in full.
#[test]
fn the_largest_values_are_printed_in_full() {
    assert_eq!(
        event("release", u64::MAX, u64::MAX, u64::MAX, u32::MAX).to_json_line(),
        r#"{"t":18446744073709551615,"ev":"release","ticket":18446744073709551615,"tenant":18446744073709551615,"in_flight":4294967295}"#
    );
}

// DESIGN.md, simulation rules (JSON line): the zero values print as 0.
#[test]
fn zero_values_print_as_zero() {
    assert_eq!(
        event("arrive", 0, 0, 0, 0).to_json_line(),
        r#"{"t":0,"ev":"arrive","ticket":0,"tenant":0,"in_flight":0}"#
    );
}

// DESIGN.md, simulation rules (JSON line): to_json_lines ends every line with a newline.
#[test]
fn json_lines_end_each_line_with_a_newline() {
    let events = [event("arrive", 1, 0, 0, 0), event("grant", 1, 0, 0, 1)];
    assert_eq!(
        to_json_lines(&events),
        concat!(
            r#"{"t":1,"ev":"arrive","ticket":0,"tenant":0,"in_flight":0}"#,
            "\n",
            r#"{"t":1,"ev":"grant","ticket":0,"tenant":0,"in_flight":1}"#,
            "\n"
        )
    );
}

// DESIGN.md, simulation rules: a single event gives one line plus a newline.
#[test]
fn one_event_gives_exactly_one_terminated_line() {
    let text = to_json_lines(&[event("reject", 9, 1, 0, 2)]);
    assert!(text.ends_with('\n'));
    assert_eq!(text.matches('\n').count(), 1);
    assert_eq!(
        text.trim_end_matches('\n'),
        event("reject", 9, 1, 0, 2).to_json_line()
    );
}

// I7: the same scenario twice gives the same trace, and the same bytes.
#[test]
fn running_the_same_scenario_twice_gives_the_same_trace() {
    let s = scenario(2024);
    let first = run_fixed(&s).expect("valid");
    let second = run_fixed(&s).expect("valid");
    assert_eq!(first, second);
    assert_eq!(to_json_lines(&first), to_json_lines(&second));
}

// I7: a cloned scenario gives the same trace as the original.
#[test]
fn a_cloned_scenario_gives_the_same_trace() {
    let s = scenario(77);
    assert_eq!(run_fixed(&s), run_fixed(&s.clone()));
}

// I7: running one scenario does not change the next run (no hidden global state).
#[test]
fn another_scenario_in_between_does_not_change_the_trace() {
    let s = scenario(5);
    let before = run_fixed(&s).expect("valid");
    let _ = run_fixed(&scenario(6)).expect("valid");
    assert_eq!(before, run_fixed(&s).expect("valid"));
}

// I7: a different seed gives a different trace (checked for seeds 1 to 10).
#[test]
fn different_seeds_give_different_traces() {
    let base = to_json_lines(&run_fixed(&scenario(0)).expect("valid"));
    for seed in 1..=10 {
        let other = to_json_lines(&run_fixed(&scenario(seed)).expect("valid"));
        assert_ne!(base, other, "seed {seed} matched seed 0");
    }
}

// DESIGN.md, simulation rules: the trace type compares by value.
#[test]
fn trace_events_compare_by_value() {
    assert_eq!(event("grant", 1, 2, 3, 4), event("grant", 1, 2, 3, 4));
    assert_ne!(event("grant", 1, 2, 3, 4), event("reject", 1, 2, 3, 4));
    assert_ne!(event("grant", 1, 2, 3, 4), event("grant", 9, 2, 3, 4));
    assert_ne!(event("grant", 1, 2, 3, 4), event("grant", 1, 9, 3, 4));
    assert_ne!(event("grant", 1, 2, 3, 4), event("grant", 1, 2, 9, 4));
    assert_ne!(event("grant", 1, 2, 3, 4), event("grant", 1, 2, 3, 9));
}
