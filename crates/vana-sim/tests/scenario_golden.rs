//! Scenario traces: byte-for-byte comparison with independently computed traces.

use vana_core::Nanos;
use vana_sim::{FixedScenario, ScenarioError, run_fixed, to_json_lines};

fn scenario(
    seed: u64,
    requests: u32,
    tenants: u64,
    limit: u32,
    mean_gap: u64,
    mean_service: u64,
) -> FixedScenario {
    FixedScenario {
        seed,
        requests,
        tenants,
        limit,
        mean_gap: Nanos::new(mean_gap),
        mean_service: Nanos::new(mean_service),
    }
}

fn trace_of(scenario: &FixedScenario) -> String {
    to_json_lines(&run_fixed(scenario).expect("the scenario is valid"))
}

// I1, I7; expected trace computed independently from the simulation rules in docs/DESIGN.md.
#[test]
fn golden_trace_a_seed_42_matches_byte_for_byte() {
    let expected = r##"{"t":149,"ev":"arrive","ticket":0,"tenant":0,"in_flight":0}
{"t":149,"ev":"grant","ticket":0,"tenant":0,"in_flight":1}
{"t":205,"ev":"arrive","ticket":1,"tenant":1,"in_flight":1}
{"t":205,"ev":"grant","ticket":1,"tenant":1,"in_flight":2}
{"t":213,"ev":"arrive","ticket":2,"tenant":0,"in_flight":2}
{"t":213,"ev":"reject","ticket":2,"tenant":0,"in_flight":2}
{"t":229,"ev":"release","ticket":0,"tenant":0,"in_flight":1}
{"t":257,"ev":"arrive","ticket":3,"tenant":1,"in_flight":1}
{"t":257,"ev":"grant","ticket":3,"tenant":1,"in_flight":2}
{"t":325,"ev":"arrive","ticket":4,"tenant":0,"in_flight":2}
{"t":325,"ev":"reject","ticket":4,"tenant":0,"in_flight":2}
{"t":366,"ev":"arrive","ticket":5,"tenant":1,"in_flight":2}
{"t":366,"ev":"reject","ticket":5,"tenant":1,"in_flight":2}
{"t":378,"ev":"release","ticket":1,"tenant":1,"in_flight":1}
{"t":469,"ev":"arrive","ticket":6,"tenant":0,"in_flight":1}
{"t":469,"ev":"grant","ticket":6,"tenant":0,"in_flight":2}
{"t":603,"ev":"arrive","ticket":7,"tenant":1,"in_flight":2}
{"t":603,"ev":"reject","ticket":7,"tenant":1,"in_flight":2}
{"t":624,"ev":"arrive","ticket":8,"tenant":0,"in_flight":2}
{"t":624,"ev":"reject","ticket":8,"tenant":0,"in_flight":2}
{"t":643,"ev":"arrive","ticket":9,"tenant":1,"in_flight":2}
{"t":643,"ev":"reject","ticket":9,"tenant":1,"in_flight":2}
{"t":658,"ev":"release","ticket":3,"tenant":1,"in_flight":1}
{"t":730,"ev":"release","ticket":6,"tenant":0,"in_flight":0}
{"t":835,"ev":"arrive","ticket":10,"tenant":0,"in_flight":0}
{"t":835,"ev":"grant","ticket":10,"tenant":0,"in_flight":1}
{"t":872,"ev":"release","ticket":10,"tenant":0,"in_flight":0}
{"t":955,"ev":"arrive","ticket":11,"tenant":1,"in_flight":0}
{"t":955,"ev":"grant","ticket":11,"tenant":1,"in_flight":1}
{"t":1265,"ev":"release","ticket":11,"tenant":1,"in_flight":0}
"##;
    let scenario = scenario(42, 12, 2, 2, 100, 250);
    assert_eq!(trace_of(&scenario), expected);
}

// I1, I7; expected trace computed independently from the simulation rules in docs/DESIGN.md.
#[test]
fn golden_trace_b_seed_7_matches_byte_for_byte() {
    let expected = r##"{"t":39,"ev":"arrive","ticket":0,"tenant":0,"in_flight":0}
{"t":39,"ev":"grant","ticket":0,"tenant":0,"in_flight":1}
{"t":53,"ev":"release","ticket":0,"tenant":0,"in_flight":0}
{"t":130,"ev":"arrive","ticket":1,"tenant":1,"in_flight":0}
{"t":130,"ev":"grant","ticket":1,"tenant":1,"in_flight":1}
{"t":176,"ev":"arrive","ticket":2,"tenant":2,"in_flight":1}
{"t":176,"ev":"reject","ticket":2,"tenant":2,"in_flight":1}
{"t":223,"ev":"arrive","ticket":3,"tenant":0,"in_flight":1}
{"t":223,"ev":"reject","ticket":3,"tenant":0,"in_flight":1}
{"t":237,"ev":"arrive","ticket":4,"tenant":1,"in_flight":1}
{"t":237,"ev":"reject","ticket":4,"tenant":1,"in_flight":1}
{"t":248,"ev":"arrive","ticket":5,"tenant":2,"in_flight":1}
{"t":248,"ev":"reject","ticket":5,"tenant":2,"in_flight":1}
{"t":340,"ev":"arrive","ticket":6,"tenant":0,"in_flight":1}
{"t":340,"ev":"reject","ticket":6,"tenant":0,"in_flight":1}
{"t":427,"ev":"arrive","ticket":7,"tenant":1,"in_flight":1}
{"t":427,"ev":"reject","ticket":7,"tenant":1,"in_flight":1}
{"t":515,"ev":"arrive","ticket":8,"tenant":2,"in_flight":1}
{"t":515,"ev":"reject","ticket":8,"tenant":2,"in_flight":1}
{"t":577,"ev":"arrive","ticket":9,"tenant":0,"in_flight":1}
{"t":577,"ev":"reject","ticket":9,"tenant":0,"in_flight":1}
{"t":597,"ev":"release","ticket":1,"tenant":1,"in_flight":0}
{"t":645,"ev":"arrive","ticket":10,"tenant":1,"in_flight":0}
{"t":645,"ev":"grant","ticket":10,"tenant":1,"in_flight":1}
{"t":680,"ev":"arrive","ticket":11,"tenant":2,"in_flight":1}
{"t":680,"ev":"reject","ticket":11,"tenant":2,"in_flight":1}
{"t":731,"ev":"release","ticket":10,"tenant":1,"in_flight":0}
{"t":771,"ev":"arrive","ticket":12,"tenant":0,"in_flight":0}
{"t":771,"ev":"grant","ticket":12,"tenant":0,"in_flight":1}
{"t":779,"ev":"arrive","ticket":13,"tenant":1,"in_flight":1}
{"t":779,"ev":"reject","ticket":13,"tenant":1,"in_flight":1}
{"t":870,"ev":"arrive","ticket":14,"tenant":2,"in_flight":1}
{"t":870,"ev":"reject","ticket":14,"tenant":2,"in_flight":1}
{"t":968,"ev":"arrive","ticket":15,"tenant":0,"in_flight":1}
{"t":968,"ev":"reject","ticket":15,"tenant":0,"in_flight":1}
{"t":1007,"ev":"arrive","ticket":16,"tenant":1,"in_flight":1}
{"t":1007,"ev":"reject","ticket":16,"tenant":1,"in_flight":1}
{"t":1063,"ev":"arrive","ticket":17,"tenant":2,"in_flight":1}
{"t":1063,"ev":"reject","ticket":17,"tenant":2,"in_flight":1}
{"t":1071,"ev":"arrive","ticket":18,"tenant":0,"in_flight":1}
{"t":1071,"ev":"reject","ticket":18,"tenant":0,"in_flight":1}
{"t":1093,"ev":"arrive","ticket":19,"tenant":1,"in_flight":1}
{"t":1093,"ev":"reject","ticket":19,"tenant":1,"in_flight":1}
{"t":1540,"ev":"release","ticket":12,"tenant":0,"in_flight":0}
"##;
    let scenario = scenario(7, 20, 3, 1, 50, 400);
    assert_eq!(trace_of(&scenario), expected);
}

// I1, I7; expected trace computed independently from the simulation rules in docs/DESIGN.md.
#[test]
fn golden_trace_c_without_rejects_matches_byte_for_byte() {
    let expected = r##"{"t":114,"ev":"arrive","ticket":0,"tenant":0,"in_flight":0}
{"t":114,"ev":"grant","ticket":0,"tenant":0,"in_flight":1}
{"t":264,"ev":"release","ticket":0,"tenant":0,"in_flight":0}
{"t":309,"ev":"arrive","ticket":1,"tenant":0,"in_flight":0}
{"t":309,"ev":"grant","ticket":1,"tenant":0,"in_flight":1}
{"t":398,"ev":"release","ticket":1,"tenant":0,"in_flight":0}
{"t":398,"ev":"arrive","ticket":2,"tenant":0,"in_flight":0}
{"t":398,"ev":"grant","ticket":2,"tenant":0,"in_flight":1}
{"t":551,"ev":"release","ticket":2,"tenant":0,"in_flight":0}
{"t":574,"ev":"arrive","ticket":3,"tenant":0,"in_flight":0}
{"t":574,"ev":"grant","ticket":3,"tenant":0,"in_flight":1}
{"t":632,"ev":"arrive","ticket":4,"tenant":0,"in_flight":1}
{"t":632,"ev":"grant","ticket":4,"tenant":0,"in_flight":2}
{"t":679,"ev":"release","ticket":3,"tenant":0,"in_flight":1}
{"t":713,"ev":"arrive","ticket":5,"tenant":0,"in_flight":1}
{"t":713,"ev":"grant","ticket":5,"tenant":0,"in_flight":2}
{"t":791,"ev":"release","ticket":4,"tenant":0,"in_flight":1}
{"t":804,"ev":"arrive","ticket":6,"tenant":0,"in_flight":1}
{"t":804,"ev":"grant","ticket":6,"tenant":0,"in_flight":2}
{"t":835,"ev":"release","ticket":5,"tenant":0,"in_flight":1}
{"t":892,"ev":"arrive","ticket":7,"tenant":0,"in_flight":1}
{"t":892,"ev":"grant","ticket":7,"tenant":0,"in_flight":2}
{"t":911,"ev":"release","ticket":6,"tenant":0,"in_flight":1}
{"t":926,"ev":"release","ticket":7,"tenant":0,"in_flight":0}
{"t":1022,"ev":"arrive","ticket":8,"tenant":0,"in_flight":0}
{"t":1022,"ev":"grant","ticket":8,"tenant":0,"in_flight":1}
{"t":1159,"ev":"arrive","ticket":9,"tenant":0,"in_flight":1}
{"t":1159,"ev":"grant","ticket":9,"tenant":0,"in_flight":2}
{"t":1186,"ev":"release","ticket":8,"tenant":0,"in_flight":1}
{"t":1336,"ev":"release","ticket":9,"tenant":0,"in_flight":0}
"##;
    let scenario = scenario(1, 10, 1, 1000, 100, 100);
    assert_eq!(trace_of(&scenario), expected);
}

// DESIGN.md, simulation rules; expected trace computed independently from the simulation rules in docs/DESIGN.md.
#[test]
fn a_single_request_arrives_is_granted_and_released() {
    let expected = r##"{"t":8,"ev":"arrive","ticket":0,"tenant":0,"in_flight":0}
{"t":8,"ev":"grant","ticket":0,"tenant":0,"in_flight":1}
{"t":24,"ev":"release","ticket":0,"tenant":0,"in_flight":0}
"##;
    let scenario = scenario(5, 1, 1, 1, 10, 10);
    assert_eq!(trace_of(&scenario), expected);
}

// DESIGN.md, simulation rules (completion time <= t is released first); expected trace computed independently from the simulation rules in docs/DESIGN.md.
#[test]
fn a_completion_at_the_arrival_time_is_released_before_the_arrival() {
    let expected = r##"{"t":1,"ev":"arrive","ticket":0,"tenant":0,"in_flight":0}
{"t":1,"ev":"grant","ticket":0,"tenant":0,"in_flight":1}
{"t":3,"ev":"release","ticket":0,"tenant":0,"in_flight":0}
{"t":3,"ev":"arrive","ticket":1,"tenant":1,"in_flight":0}
{"t":3,"ev":"grant","ticket":1,"tenant":1,"in_flight":1}
{"t":4,"ev":"release","ticket":1,"tenant":1,"in_flight":0}
{"t":4,"ev":"arrive","ticket":2,"tenant":0,"in_flight":0}
{"t":4,"ev":"grant","ticket":2,"tenant":0,"in_flight":1}
{"t":5,"ev":"arrive","ticket":3,"tenant":1,"in_flight":1}
{"t":5,"ev":"reject","ticket":3,"tenant":1,"in_flight":1}
{"t":6,"ev":"release","ticket":2,"tenant":0,"in_flight":0}
{"t":6,"ev":"arrive","ticket":4,"tenant":0,"in_flight":0}
{"t":6,"ev":"grant","ticket":4,"tenant":0,"in_flight":1}
{"t":8,"ev":"release","ticket":4,"tenant":0,"in_flight":0}
{"t":8,"ev":"arrive","ticket":5,"tenant":1,"in_flight":0}
{"t":8,"ev":"grant","ticket":5,"tenant":1,"in_flight":1}
{"t":9,"ev":"arrive","ticket":6,"tenant":0,"in_flight":1}
{"t":9,"ev":"reject","ticket":6,"tenant":0,"in_flight":1}
{"t":10,"ev":"release","ticket":5,"tenant":1,"in_flight":0}
{"t":11,"ev":"arrive","ticket":7,"tenant":1,"in_flight":0}
{"t":11,"ev":"grant","ticket":7,"tenant":1,"in_flight":1}
{"t":12,"ev":"arrive","ticket":8,"tenant":0,"in_flight":1}
{"t":12,"ev":"reject","ticket":8,"tenant":0,"in_flight":1}
{"t":13,"ev":"release","ticket":7,"tenant":1,"in_flight":0}
{"t":13,"ev":"arrive","ticket":9,"tenant":1,"in_flight":0}
{"t":13,"ev":"grant","ticket":9,"tenant":1,"in_flight":1}
{"t":15,"ev":"release","ticket":9,"tenant":1,"in_flight":0}
{"t":15,"ev":"arrive","ticket":10,"tenant":0,"in_flight":0}
{"t":15,"ev":"grant","ticket":10,"tenant":0,"in_flight":1}
{"t":17,"ev":"release","ticket":10,"tenant":0,"in_flight":0}
{"t":17,"ev":"arrive","ticket":11,"tenant":1,"in_flight":0}
{"t":17,"ev":"grant","ticket":11,"tenant":1,"in_flight":1}
{"t":18,"ev":"release","ticket":11,"tenant":1,"in_flight":0}
{"t":19,"ev":"arrive","ticket":12,"tenant":0,"in_flight":0}
{"t":19,"ev":"grant","ticket":12,"tenant":0,"in_flight":1}
{"t":20,"ev":"arrive","ticket":13,"tenant":1,"in_flight":1}
{"t":20,"ev":"reject","ticket":13,"tenant":1,"in_flight":1}
{"t":21,"ev":"release","ticket":12,"tenant":0,"in_flight":0}
{"t":22,"ev":"arrive","ticket":14,"tenant":0,"in_flight":0}
{"t":22,"ev":"grant","ticket":14,"tenant":0,"in_flight":1}
{"t":24,"ev":"release","ticket":14,"tenant":0,"in_flight":0}
{"t":24,"ev":"arrive","ticket":15,"tenant":1,"in_flight":0}
{"t":24,"ev":"grant","ticket":15,"tenant":1,"in_flight":1}
{"t":25,"ev":"arrive","ticket":16,"tenant":0,"in_flight":1}
{"t":25,"ev":"reject","ticket":16,"tenant":0,"in_flight":1}
{"t":26,"ev":"release","ticket":15,"tenant":1,"in_flight":0}
{"t":26,"ev":"arrive","ticket":17,"tenant":1,"in_flight":0}
{"t":26,"ev":"grant","ticket":17,"tenant":1,"in_flight":1}
{"t":27,"ev":"release","ticket":17,"tenant":1,"in_flight":0}
{"t":27,"ev":"arrive","ticket":18,"tenant":0,"in_flight":0}
{"t":27,"ev":"grant","ticket":18,"tenant":0,"in_flight":1}
{"t":28,"ev":"arrive","ticket":19,"tenant":1,"in_flight":1}
{"t":28,"ev":"reject","ticket":19,"tenant":1,"in_flight":1}
{"t":29,"ev":"release","ticket":18,"tenant":0,"in_flight":0}
{"t":29,"ev":"arrive","ticket":20,"tenant":0,"in_flight":0}
{"t":29,"ev":"grant","ticket":20,"tenant":0,"in_flight":1}
{"t":31,"ev":"release","ticket":20,"tenant":0,"in_flight":0}
{"t":31,"ev":"arrive","ticket":21,"tenant":1,"in_flight":0}
{"t":31,"ev":"grant","ticket":21,"tenant":1,"in_flight":1}
{"t":32,"ev":"arrive","ticket":22,"tenant":0,"in_flight":1}
{"t":32,"ev":"reject","ticket":22,"tenant":0,"in_flight":1}
{"t":33,"ev":"release","ticket":21,"tenant":1,"in_flight":0}
{"t":34,"ev":"arrive","ticket":23,"tenant":1,"in_flight":0}
{"t":34,"ev":"grant","ticket":23,"tenant":1,"in_flight":1}
{"t":35,"ev":"release","ticket":23,"tenant":1,"in_flight":0}
{"t":36,"ev":"arrive","ticket":24,"tenant":0,"in_flight":0}
{"t":36,"ev":"grant","ticket":24,"tenant":0,"in_flight":1}
{"t":37,"ev":"release","ticket":24,"tenant":0,"in_flight":0}
{"t":38,"ev":"arrive","ticket":25,"tenant":1,"in_flight":0}
{"t":38,"ev":"grant","ticket":25,"tenant":1,"in_flight":1}
{"t":40,"ev":"release","ticket":25,"tenant":1,"in_flight":0}
{"t":40,"ev":"arrive","ticket":26,"tenant":0,"in_flight":0}
{"t":40,"ev":"grant","ticket":26,"tenant":0,"in_flight":1}
{"t":41,"ev":"arrive","ticket":27,"tenant":1,"in_flight":1}
{"t":41,"ev":"reject","ticket":27,"tenant":1,"in_flight":1}
{"t":42,"ev":"release","ticket":26,"tenant":0,"in_flight":0}
{"t":42,"ev":"arrive","ticket":28,"tenant":0,"in_flight":0}
{"t":42,"ev":"grant","ticket":28,"tenant":0,"in_flight":1}
{"t":44,"ev":"release","ticket":28,"tenant":0,"in_flight":0}
{"t":44,"ev":"arrive","ticket":29,"tenant":1,"in_flight":0}
{"t":44,"ev":"grant","ticket":29,"tenant":1,"in_flight":1}
{"t":45,"ev":"release","ticket":29,"tenant":1,"in_flight":0}
"##;
    let scenario = scenario(3, 30, 2, 1, 1, 1);
    assert_eq!(trace_of(&scenario), expected);
}

// DESIGN.md, simulation rules (tenant = ticket % tenants); expected trace computed independently from the simulation rules in docs/DESIGN.md.
#[test]
fn with_more_tenants_than_requests_the_tenant_is_the_ticket() {
    let expected = r##"{"t":28,"ev":"arrive","ticket":0,"tenant":0,"in_flight":0}
{"t":28,"ev":"grant","ticket":0,"tenant":0,"in_flight":1}
{"t":39,"ev":"arrive","ticket":1,"tenant":1,"in_flight":1}
{"t":39,"ev":"grant","ticket":1,"tenant":1,"in_flight":2}
{"t":50,"ev":"arrive","ticket":2,"tenant":2,"in_flight":2}
{"t":50,"ev":"reject","ticket":2,"tenant":2,"in_flight":2}
{"t":59,"ev":"release","ticket":0,"tenant":0,"in_flight":1}
{"t":71,"ev":"release","ticket":1,"tenant":1,"in_flight":0}
{"t":76,"ev":"arrive","ticket":3,"tenant":3,"in_flight":0}
{"t":76,"ev":"grant","ticket":3,"tenant":3,"in_flight":1}
{"t":116,"ev":"release","ticket":3,"tenant":3,"in_flight":0}
"##;
    let scenario = scenario(9, 4, 100, 2, 20, 20);
    assert_eq!(trace_of(&scenario), expected);
}

// I1, DESIGN.md, simulation rules; expected trace computed independently from the simulation rules in docs/DESIGN.md.
#[test]
fn one_tenant_and_limit_one_under_load_rejects_often() {
    let expected = r##"{"t":7,"ev":"arrive","ticket":0,"tenant":0,"in_flight":0}
{"t":7,"ev":"grant","ticket":0,"tenant":0,"in_flight":1}
{"t":20,"ev":"arrive","ticket":1,"tenant":0,"in_flight":1}
{"t":20,"ev":"reject","ticket":1,"tenant":0,"in_flight":1}
{"t":24,"ev":"arrive","ticket":2,"tenant":0,"in_flight":1}
{"t":24,"ev":"reject","ticket":2,"tenant":0,"in_flight":1}
{"t":27,"ev":"arrive","ticket":3,"tenant":0,"in_flight":1}
{"t":27,"ev":"reject","ticket":3,"tenant":0,"in_flight":1}
{"t":34,"ev":"arrive","ticket":4,"tenant":0,"in_flight":1}
{"t":34,"ev":"reject","ticket":4,"tenant":0,"in_flight":1}
{"t":38,"ev":"arrive","ticket":5,"tenant":0,"in_flight":1}
{"t":38,"ev":"reject","ticket":5,"tenant":0,"in_flight":1}
{"t":57,"ev":"arrive","ticket":6,"tenant":0,"in_flight":1}
{"t":57,"ev":"reject","ticket":6,"tenant":0,"in_flight":1}
{"t":60,"ev":"release","ticket":0,"tenant":0,"in_flight":0}
{"t":77,"ev":"arrive","ticket":7,"tenant":0,"in_flight":0}
{"t":77,"ev":"grant","ticket":7,"tenant":0,"in_flight":1}
{"t":90,"ev":"arrive","ticket":8,"tenant":0,"in_flight":1}
{"t":90,"ev":"reject","ticket":8,"tenant":0,"in_flight":1}
{"t":105,"ev":"arrive","ticket":9,"tenant":0,"in_flight":1}
{"t":105,"ev":"reject","ticket":9,"tenant":0,"in_flight":1}
{"t":109,"ev":"arrive","ticket":10,"tenant":0,"in_flight":1}
{"t":109,"ev":"reject","ticket":10,"tenant":0,"in_flight":1}
{"t":125,"ev":"arrive","ticket":11,"tenant":0,"in_flight":1}
{"t":125,"ev":"reject","ticket":11,"tenant":0,"in_flight":1}
{"t":127,"ev":"release","ticket":7,"tenant":0,"in_flight":0}
{"t":139,"ev":"arrive","ticket":12,"tenant":0,"in_flight":0}
{"t":139,"ev":"grant","ticket":12,"tenant":0,"in_flight":1}
{"t":157,"ev":"arrive","ticket":13,"tenant":0,"in_flight":1}
{"t":157,"ev":"reject","ticket":13,"tenant":0,"in_flight":1}
{"t":165,"ev":"arrive","ticket":14,"tenant":0,"in_flight":1}
{"t":165,"ev":"reject","ticket":14,"tenant":0,"in_flight":1}
{"t":326,"ev":"release","ticket":12,"tenant":0,"in_flight":0}
"##;
    let scenario = scenario(11, 15, 1, 1, 10, 100);
    assert_eq!(trace_of(&scenario), expected);
}

// I1, DESIGN.md, simulation rules; expected trace computed independently from the simulation rules in docs/DESIGN.md.
#[test]
fn a_limit_equal_to_the_request_count_rejects_nothing() {
    let expected = r##"{"t":6,"ev":"arrive","ticket":0,"tenant":0,"in_flight":0}
{"t":6,"ev":"grant","ticket":0,"tenant":0,"in_flight":1}
{"t":9,"ev":"arrive","ticket":1,"tenant":1,"in_flight":1}
{"t":9,"ev":"grant","ticket":1,"tenant":1,"in_flight":2}
{"t":18,"ev":"arrive","ticket":2,"tenant":2,"in_flight":2}
{"t":18,"ev":"grant","ticket":2,"tenant":2,"in_flight":3}
{"t":20,"ev":"arrive","ticket":3,"tenant":0,"in_flight":3}
{"t":20,"ev":"grant","ticket":3,"tenant":0,"in_flight":4}
{"t":24,"ev":"arrive","ticket":4,"tenant":1,"in_flight":4}
{"t":24,"ev":"grant","ticket":4,"tenant":1,"in_flight":5}
{"t":34,"ev":"arrive","ticket":5,"tenant":2,"in_flight":5}
{"t":34,"ev":"grant","ticket":5,"tenant":2,"in_flight":6}
{"t":41,"ev":"arrive","ticket":6,"tenant":0,"in_flight":6}
{"t":41,"ev":"grant","ticket":6,"tenant":0,"in_flight":7}
{"t":48,"ev":"arrive","ticket":7,"tenant":1,"in_flight":7}
{"t":48,"ev":"grant","ticket":7,"tenant":1,"in_flight":8}
{"t":176,"ev":"release","ticket":6,"tenant":0,"in_flight":7}
{"t":305,"ev":"release","ticket":4,"tenant":1,"in_flight":6}
{"t":418,"ev":"release","ticket":7,"tenant":1,"in_flight":5}
{"t":607,"ev":"release","ticket":2,"tenant":2,"in_flight":4}
{"t":1388,"ev":"release","ticket":5,"tenant":2,"in_flight":3}
{"t":1522,"ev":"release","ticket":3,"tenant":0,"in_flight":2}
{"t":1819,"ev":"release","ticket":1,"tenant":1,"in_flight":1}
{"t":1885,"ev":"release","ticket":0,"tenant":0,"in_flight":0}
"##;
    let scenario = scenario(12, 8, 3, 8, 5, 1000);
    assert_eq!(trace_of(&scenario), expected);
}

// DESIGN.md, simulation rules (saturating arithmetic); expected trace computed independently from the simulation rules in docs/DESIGN.md.
#[test]
fn saturating_times_stop_at_the_largest_value() {
    let expected = r##"{"t":10905525725756348110,"ev":"arrive","ticket":0,"tenant":0,"in_flight":0}
{"t":10905525725756348110,"ev":"grant","ticket":0,"tenant":0,"in_flight":1}
{"t":18446744073709551615,"ev":"release","ticket":0,"tenant":0,"in_flight":0}
{"t":18446744073709551615,"ev":"arrive","ticket":1,"tenant":1,"in_flight":0}
{"t":18446744073709551615,"ev":"grant","ticket":1,"tenant":1,"in_flight":1}
{"t":18446744073709551615,"ev":"release","ticket":1,"tenant":1,"in_flight":0}
{"t":18446744073709551615,"ev":"arrive","ticket":2,"tenant":0,"in_flight":0}
{"t":18446744073709551615,"ev":"grant","ticket":2,"tenant":0,"in_flight":1}
{"t":18446744073709551615,"ev":"release","ticket":2,"tenant":0,"in_flight":0}
{"t":18446744073709551615,"ev":"arrive","ticket":3,"tenant":1,"in_flight":0}
{"t":18446744073709551615,"ev":"grant","ticket":3,"tenant":1,"in_flight":1}
{"t":18446744073709551615,"ev":"release","ticket":3,"tenant":1,"in_flight":0}
{"t":18446744073709551615,"ev":"arrive","ticket":4,"tenant":0,"in_flight":0}
{"t":18446744073709551615,"ev":"grant","ticket":4,"tenant":0,"in_flight":1}
{"t":18446744073709551615,"ev":"release","ticket":4,"tenant":0,"in_flight":0}
{"t":18446744073709551615,"ev":"arrive","ticket":5,"tenant":1,"in_flight":0}
{"t":18446744073709551615,"ev":"grant","ticket":5,"tenant":1,"in_flight":1}
{"t":18446744073709551615,"ev":"release","ticket":5,"tenant":1,"in_flight":0}
"##;
    let scenario = scenario(2, 6, 2, 1, 18446744073709551615, 18446744073709551615);
    assert_eq!(trace_of(&scenario), expected);
}

// DESIGN.md, simulation rules (saturating arithmetic); expected trace computed independently from the simulation rules in docs/DESIGN.md.
#[test]
fn a_saturating_service_range_gives_the_expected_trace() {
    let expected = r##"{"t":9,"ev":"arrive","ticket":0,"tenant":0,"in_flight":0}
{"t":9,"ev":"grant","ticket":0,"tenant":0,"in_flight":1}
{"t":27,"ev":"arrive","ticket":1,"tenant":0,"in_flight":1}
{"t":27,"ev":"grant","ticket":1,"tenant":0,"in_flight":2}
{"t":35,"ev":"arrive","ticket":2,"tenant":0,"in_flight":2}
{"t":35,"ev":"reject","ticket":2,"tenant":0,"in_flight":2}
{"t":54,"ev":"arrive","ticket":3,"tenant":0,"in_flight":2}
{"t":54,"ev":"reject","ticket":3,"tenant":0,"in_flight":2}
{"t":58,"ev":"arrive","ticket":4,"tenant":0,"in_flight":2}
{"t":58,"ev":"reject","ticket":4,"tenant":0,"in_flight":2}
{"t":9071633986856679609,"ev":"release","ticket":1,"tenant":0,"in_flight":1}
{"t":16462000697783136313,"ev":"release","ticket":0,"tenant":0,"in_flight":0}
"##;
    let scenario = scenario(4, 5, 1, 2, 10, 18446744073709551615);
    assert_eq!(trace_of(&scenario), expected);
}

// DESIGN.md, simulation rules (invalid scenarios); expected error computed independently.
#[test]
fn a_zero_limit_is_rejected() {
    let scenario = scenario(1, 5, 1, 0, 10, 10);
    assert_eq!(run_fixed(&scenario).unwrap_err(), ScenarioError::ZeroLimit);
}

// DESIGN.md, simulation rules (invalid scenarios); expected error computed independently.
#[test]
fn a_zero_tenant_count_is_rejected() {
    let scenario = scenario(1, 5, 0, 1, 10, 10);
    assert_eq!(
        run_fixed(&scenario).unwrap_err(),
        ScenarioError::ZeroTenants
    );
}

// DESIGN.md, simulation rules (invalid scenarios); expected error computed independently.
#[test]
fn a_zero_mean_gap_is_rejected() {
    let scenario = scenario(1, 5, 1, 1, 0, 10);
    assert_eq!(
        run_fixed(&scenario).unwrap_err(),
        ScenarioError::ZeroMeanGap
    );
}

// DESIGN.md, simulation rules (invalid scenarios); expected error computed independently.
#[test]
fn a_zero_mean_service_is_rejected() {
    let scenario = scenario(1, 5, 1, 1, 10, 0);
    assert_eq!(
        run_fixed(&scenario).unwrap_err(),
        ScenarioError::ZeroMeanService
    );
}

// DESIGN.md, simulation rules (invalid scenarios); expected error computed independently.
#[test]
fn with_every_field_zero_the_limit_error_comes_first() {
    let scenario = scenario(1, 5, 0, 0, 0, 0);
    assert_eq!(run_fixed(&scenario).unwrap_err(), ScenarioError::ZeroLimit);
}

// DESIGN.md, simulation rules (invalid scenarios); expected error computed independently.
#[test]
fn an_invalid_scenario_is_rejected_even_with_no_requests() {
    let scenario = scenario(1, 0, 1, 0, 10, 10);
    assert_eq!(run_fixed(&scenario).unwrap_err(), ScenarioError::ZeroLimit);
}

// DESIGN.md, simulation rules (no requests, no events); expected trace computed independently.
#[test]
fn no_requests_gives_an_empty_trace() {
    let scenario = scenario(5, 0, 1, 1, 10, 10);
    assert_eq!(run_fixed(&scenario).expect("valid"), Vec::new());
    assert_eq!(trace_of(&scenario), "");
}
