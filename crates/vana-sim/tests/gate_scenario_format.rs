//! Gate scenario errors and the JSON line format of gate events.

use vana_core::{Nanos, RejectReason, TenantId, TicketId};
use vana_sim::{GateEvent, GateScenario, ScenarioError, gate_json_lines, run_gate};

fn base() -> GateScenario {
    GateScenario {
        seed: 1,
        requests: 5,
        tenants: 1,
        limit: 1,
        queue_capacity: 3,
        timeout: None,
        mean_gap: Nanos::new(10),
        mean_service: Nanos::new(10),
    }
}

fn error_of(scenario: &GateScenario) -> ScenarioError {
    run_gate(scenario).expect_err("the scenario is invalid")
}

// DESIGN.md gate run: a limit of 0 is invalid (ScenarioError::ZeroLimit).
#[test]
fn a_zero_limit_is_rejected_with_zero_limit() {
    let s = GateScenario { limit: 0, ..base() };
    assert_eq!(error_of(&s), ScenarioError::ZeroLimit);
}

// DESIGN.md gate run: zero tenants is invalid (ScenarioError::ZeroTenants).
#[test]
fn zero_tenants_are_rejected_with_zero_tenants() {
    let s = GateScenario {
        tenants: 0,
        ..base()
    };
    assert_eq!(error_of(&s), ScenarioError::ZeroTenants);
}

// DESIGN.md gate run: a mean gap of 0 is invalid (ScenarioError::ZeroMeanGap).
#[test]
fn a_zero_mean_gap_is_rejected_with_zero_mean_gap() {
    let s = GateScenario {
        mean_gap: Nanos::ZERO,
        ..base()
    };
    assert_eq!(error_of(&s), ScenarioError::ZeroMeanGap);
}

// DESIGN.md gate run: a mean service of 0 is invalid (ScenarioError::ZeroMeanService).
#[test]
fn a_zero_mean_service_is_rejected_with_zero_mean_service() {
    let s = GateScenario {
        mean_service: Nanos::ZERO,
        ..base()
    };
    assert_eq!(error_of(&s), ScenarioError::ZeroMeanService);
}

// DESIGN.md gate run: the checks run in the order limit, tenants, mean gap, mean service.
#[test]
fn the_first_invalid_field_in_the_documented_order_gives_the_error() {
    let all = GateScenario {
        limit: 0,
        tenants: 0,
        mean_gap: Nanos::ZERO,
        mean_service: Nanos::ZERO,
        ..base()
    };
    assert_eq!(error_of(&all), ScenarioError::ZeroLimit);
    let no_limit_problem = GateScenario {
        limit: 1,
        ..all.clone()
    };
    assert_eq!(error_of(&no_limit_problem), ScenarioError::ZeroTenants);
    let gap_and_service = GateScenario {
        tenants: 1,
        ..no_limit_problem
    };
    assert_eq!(error_of(&gap_and_service), ScenarioError::ZeroMeanGap);
    let only_service = GateScenario {
        mean_gap: Nanos::new(1),
        ..gap_and_service
    };
    assert_eq!(error_of(&only_service), ScenarioError::ZeroMeanService);
}

// DESIGN.md gate run: the fields are checked even when there are no requests.
#[test]
fn an_invalid_scenario_with_no_requests_is_still_rejected() {
    let s = GateScenario {
        requests: 0,
        limit: 0,
        ..base()
    };
    assert_eq!(error_of(&s), ScenarioError::ZeroLimit);
}

// DESIGN.md gate run: no requests give an empty trace.
#[test]
fn no_requests_give_an_empty_trace() {
    let s = GateScenario {
        requests: 0,
        ..base()
    };
    assert_eq!(run_gate(&s).expect("valid"), Vec::new());
    assert_eq!(gate_json_lines(&[]), "");
}

// DESIGN.md gate run: a queue capacity of 0 and a timeout of 0 are valid.
#[test]
fn a_zero_queue_capacity_and_a_zero_timeout_are_valid() {
    let s = GateScenario {
        queue_capacity: 0,
        timeout: Some(Nanos::ZERO),
        ..base()
    };
    let events = run_gate(&s).expect("valid");
    assert!(!events.is_empty());
}

fn ids(t: u64, ticket: u64, tenant: u64) -> (Nanos, TicketId, TenantId) {
    (Nanos::new(t), TicketId::new(ticket), TenantId::new(tenant))
}

// DESIGN.md gate run: the JSON line has no spaces and the keys in a fixed order.
#[test]
fn every_event_kind_renders_as_one_json_line_with_the_documented_keys() {
    let (t, ticket, tenant) = ids(12, 3, 1);
    let line = |e: GateEvent| e.to_json_line();
    assert_eq!(
        line(GateEvent::Arrive {
            t,
            ticket,
            tenant,
            in_flight: 2,
            queued: 1
        }),
        r#"{"t":12,"ev":"arrive","ticket":3,"tenant":1,"in_flight":2,"queued":1}"#
    );
    assert_eq!(
        line(GateEvent::Grant {
            t,
            ticket,
            tenant,
            in_flight: 2,
            queued: 0
        }),
        r#"{"t":12,"ev":"grant","ticket":3,"tenant":1,"in_flight":2,"queued":0}"#
    );
    assert_eq!(
        line(GateEvent::Queue {
            t,
            ticket,
            tenant,
            in_flight: 2,
            queued: 4
        }),
        r#"{"t":12,"ev":"queue","ticket":3,"tenant":1,"in_flight":2,"queued":4}"#
    );
    assert_eq!(
        line(GateEvent::Expire {
            t,
            ticket,
            tenant,
            in_flight: 2,
            queued: 3
        }),
        r#"{"t":12,"ev":"expire","ticket":3,"tenant":1,"in_flight":2,"queued":3}"#
    );
    assert_eq!(
        line(GateEvent::Release {
            t,
            ticket,
            tenant,
            in_flight: 1,
            queued: 3
        }),
        r#"{"t":12,"ev":"release","ticket":3,"tenant":1,"in_flight":1,"queued":3}"#
    );
}

// DESIGN.md gate run: a reject line ends with the reason.
#[test]
fn a_reject_line_ends_with_its_reason() {
    let (t, ticket, tenant) = ids(7, 2, 0);
    let full = GateEvent::Reject {
        t,
        ticket,
        tenant,
        in_flight: 1,
        queued: 2,
        reason: RejectReason::QueueFull,
    };
    assert_eq!(
        full.to_json_line(),
        r#"{"t":7,"ev":"reject","ticket":2,"tenant":0,"in_flight":1,"queued":2,"reason":"queue_full"}"#
    );
    let expired = GateEvent::Reject {
        t,
        ticket,
        tenant,
        in_flight: 0,
        queued: 0,
        reason: RejectReason::Expired,
    };
    assert_eq!(
        expired.to_json_line(),
        r#"{"t":7,"ev":"reject","ticket":2,"tenant":0,"in_flight":0,"queued":0,"reason":"expired"}"#
    );
}

// DESIGN.md gate run: the largest values print in full.
#[test]
fn the_largest_values_print_in_full() {
    let e = GateEvent::Release {
        t: Nanos::MAX,
        ticket: TicketId::new(u64::MAX),
        tenant: TenantId::new(u64::MAX),
        in_flight: u32::MAX,
        queued: u32::MAX,
    };
    assert_eq!(
        e.to_json_line(),
        r#"{"t":18446744073709551615,"ev":"release","ticket":18446744073709551615,"tenant":18446744073709551615,"in_flight":4294967295,"queued":4294967295}"#
    );
}

// DESIGN.md gate run: gate_json_lines ends every line with a newline.
#[test]
fn json_lines_end_every_line_with_a_newline() {
    let (t, ticket, tenant) = ids(1, 0, 0);
    let events = [
        GateEvent::Arrive {
            t,
            ticket,
            tenant,
            in_flight: 0,
            queued: 0,
        },
        GateEvent::Grant {
            t,
            ticket,
            tenant,
            in_flight: 1,
            queued: 0,
        },
    ];
    assert_eq!(
        gate_json_lines(&events),
        "{\"t\":1,\"ev\":\"arrive\",\"ticket\":0,\"tenant\":0,\"in_flight\":0,\"queued\":0}\n{\"t\":1,\"ev\":\"grant\",\"ticket\":0,\"tenant\":0,\"in_flight\":1,\"queued\":0}\n"
    );
}

// DESIGN.md gate run: GateEvent is Clone, Debug, PartialEq and Eq.
#[test]
fn gate_events_can_be_cloned_compared_and_formatted() {
    let (t, ticket, tenant) = ids(1, 0, 0);
    let a = GateEvent::Queue {
        t,
        ticket,
        tenant,
        in_flight: 1,
        queued: 1,
    };
    let b = a.clone();
    assert_eq!(a, b);
    assert_ne!(
        a,
        GateEvent::Queue {
            t,
            ticket,
            tenant,
            in_flight: 1,
            queued: 2
        }
    );
    assert!(!format!("{a:?}").is_empty());
}

// I7: running a scenario does not consume it; the same value can run again with the same result.
#[test]
fn running_the_same_scenario_value_twice_gives_equal_traces() {
    let s = GateScenario {
        seed: 9,
        requests: 25,
        tenants: 3,
        limit: 2,
        queue_capacity: 2,
        timeout: Some(Nanos::new(30)),
        mean_gap: Nanos::new(5),
        mean_service: Nanos::new(20),
    };
    assert_eq!(run_gate(&s), run_gate(&s));
}
