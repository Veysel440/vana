//! Gate scenarios compared with independently computed traces.

use vana_core::Nanos;
use vana_sim::{GateScenario, gate_json_lines, run_gate};

#[allow(clippy::too_many_arguments)]
fn scenario(
    seed: u64,
    requests: u32,
    tenants: u64,
    limit: u32,
    queue_capacity: u32,
    timeout: Option<u64>,
    mean_gap: u64,
    mean_service: u64,
) -> GateScenario {
    GateScenario {
        seed,
        requests,
        tenants,
        limit,
        queue_capacity,
        timeout: timeout.map(Nanos::new),
        mean_gap: Nanos::new(mean_gap),
        mean_service: Nanos::new(mean_service),
    }
}

fn trace_of(scenario: &GateScenario) -> String {
    gate_json_lines(&run_gate(scenario).expect("the scenario is valid"))
}

// I3, I7, I8; expected trace computed independently from the simulation rules in docs/DESIGN.md.
#[test]
fn gate_golden_trace_two_slots_queue_3_without_a_timeout_matches_byte_for_byte() {
    let expected = r##"{"t":149,"ev":"arrive","ticket":0,"tenant":0,"in_flight":0,"queued":0}
{"t":149,"ev":"grant","ticket":0,"tenant":0,"in_flight":1,"queued":0}
{"t":205,"ev":"arrive","ticket":1,"tenant":1,"in_flight":1,"queued":0}
{"t":205,"ev":"grant","ticket":1,"tenant":1,"in_flight":2,"queued":0}
{"t":213,"ev":"arrive","ticket":2,"tenant":0,"in_flight":2,"queued":0}
{"t":213,"ev":"queue","ticket":2,"tenant":0,"in_flight":2,"queued":1}
{"t":229,"ev":"release","ticket":0,"tenant":0,"in_flight":1,"queued":1}
{"t":229,"ev":"grant","ticket":2,"tenant":0,"in_flight":2,"queued":0}
{"t":257,"ev":"arrive","ticket":3,"tenant":1,"in_flight":2,"queued":0}
{"t":257,"ev":"queue","ticket":3,"tenant":1,"in_flight":2,"queued":1}
{"t":325,"ev":"arrive","ticket":4,"tenant":0,"in_flight":2,"queued":1}
{"t":325,"ev":"queue","ticket":4,"tenant":0,"in_flight":2,"queued":2}
{"t":366,"ev":"arrive","ticket":5,"tenant":1,"in_flight":2,"queued":2}
{"t":366,"ev":"queue","ticket":5,"tenant":1,"in_flight":2,"queued":3}
{"t":378,"ev":"release","ticket":1,"tenant":1,"in_flight":1,"queued":3}
{"t":378,"ev":"grant","ticket":3,"tenant":1,"in_flight":2,"queued":2}
{"t":469,"ev":"arrive","ticket":6,"tenant":0,"in_flight":2,"queued":2}
{"t":469,"ev":"queue","ticket":6,"tenant":0,"in_flight":2,"queued":3}
{"t":603,"ev":"arrive","ticket":7,"tenant":1,"in_flight":2,"queued":3}
{"t":603,"ev":"reject","ticket":7,"tenant":1,"in_flight":2,"queued":3,"reason":"queue_full"}
{"t":624,"ev":"arrive","ticket":8,"tenant":0,"in_flight":2,"queued":3}
{"t":624,"ev":"reject","ticket":8,"tenant":0,"in_flight":2,"queued":3,"reason":"queue_full"}
{"t":643,"ev":"arrive","ticket":9,"tenant":1,"in_flight":2,"queued":3}
{"t":643,"ev":"reject","ticket":9,"tenant":1,"in_flight":2,"queued":3,"reason":"queue_full"}
{"t":664,"ev":"release","ticket":2,"tenant":0,"in_flight":1,"queued":3}
{"t":664,"ev":"grant","ticket":4,"tenant":0,"in_flight":2,"queued":2}
{"t":779,"ev":"release","ticket":3,"tenant":1,"in_flight":1,"queued":2}
{"t":779,"ev":"grant","ticket":5,"tenant":1,"in_flight":2,"queued":1}
{"t":835,"ev":"arrive","ticket":10,"tenant":0,"in_flight":2,"queued":1}
{"t":835,"ev":"queue","ticket":10,"tenant":0,"in_flight":2,"queued":2}
{"t":955,"ev":"arrive","ticket":11,"tenant":1,"in_flight":2,"queued":2}
{"t":955,"ev":"queue","ticket":11,"tenant":1,"in_flight":2,"queued":3}
{"t":970,"ev":"arrive","ticket":12,"tenant":0,"in_flight":2,"queued":3}
{"t":970,"ev":"reject","ticket":12,"tenant":0,"in_flight":2,"queued":3,"reason":"queue_full"}
{"t":974,"ev":"release","ticket":4,"tenant":0,"in_flight":1,"queued":3}
{"t":974,"ev":"grant","ticket":6,"tenant":0,"in_flight":2,"queued":2}
{"t":1026,"ev":"release","ticket":5,"tenant":1,"in_flight":1,"queued":2}
{"t":1026,"ev":"grant","ticket":10,"tenant":0,"in_flight":2,"queued":1}
{"t":1063,"ev":"release","ticket":10,"tenant":0,"in_flight":1,"queued":1}
{"t":1063,"ev":"grant","ticket":11,"tenant":1,"in_flight":2,"queued":0}
{"t":1119,"ev":"arrive","ticket":13,"tenant":1,"in_flight":2,"queued":0}
{"t":1119,"ev":"queue","ticket":13,"tenant":1,"in_flight":2,"queued":1}
{"t":1235,"ev":"release","ticket":6,"tenant":0,"in_flight":1,"queued":1}
{"t":1235,"ev":"grant","ticket":13,"tenant":1,"in_flight":2,"queued":0}
{"t":1308,"ev":"arrive","ticket":14,"tenant":0,"in_flight":2,"queued":0}
{"t":1308,"ev":"queue","ticket":14,"tenant":0,"in_flight":2,"queued":1}
{"t":1373,"ev":"release","ticket":11,"tenant":1,"in_flight":1,"queued":1}
{"t":1373,"ev":"grant","ticket":14,"tenant":0,"in_flight":2,"queued":0}
{"t":1466,"ev":"arrive","ticket":15,"tenant":1,"in_flight":2,"queued":0}
{"t":1466,"ev":"queue","ticket":15,"tenant":1,"in_flight":2,"queued":1}
{"t":1596,"ev":"arrive","ticket":16,"tenant":0,"in_flight":2,"queued":1}
{"t":1596,"ev":"queue","ticket":16,"tenant":0,"in_flight":2,"queued":2}
{"t":1628,"ev":"release","ticket":13,"tenant":1,"in_flight":1,"queued":2}
{"t":1628,"ev":"grant","ticket":15,"tenant":1,"in_flight":2,"queued":1}
{"t":1721,"ev":"release","ticket":14,"tenant":0,"in_flight":1,"queued":1}
{"t":1721,"ev":"grant","ticket":16,"tenant":0,"in_flight":2,"queued":0}
{"t":1724,"ev":"arrive","ticket":17,"tenant":1,"in_flight":2,"queued":0}
{"t":1724,"ev":"queue","ticket":17,"tenant":1,"in_flight":2,"queued":1}
{"t":1737,"ev":"arrive","ticket":18,"tenant":0,"in_flight":2,"queued":1}
{"t":1737,"ev":"queue","ticket":18,"tenant":0,"in_flight":2,"queued":2}
{"t":1890,"ev":"arrive","ticket":19,"tenant":1,"in_flight":2,"queued":2}
{"t":1890,"ev":"queue","ticket":19,"tenant":1,"in_flight":2,"queued":3}
{"t":1997,"ev":"arrive","ticket":20,"tenant":0,"in_flight":2,"queued":3}
{"t":1997,"ev":"reject","ticket":20,"tenant":0,"in_flight":2,"queued":3,"reason":"queue_full"}
{"t":2049,"ev":"release","ticket":15,"tenant":1,"in_flight":1,"queued":3}
{"t":2049,"ev":"grant","ticket":17,"tenant":1,"in_flight":2,"queued":2}
{"t":2052,"ev":"arrive","ticket":21,"tenant":1,"in_flight":2,"queued":2}
{"t":2052,"ev":"queue","ticket":21,"tenant":1,"in_flight":2,"queued":3}
{"t":2113,"ev":"release","ticket":16,"tenant":0,"in_flight":1,"queued":3}
{"t":2113,"ev":"grant","ticket":18,"tenant":0,"in_flight":2,"queued":2}
{"t":2186,"ev":"arrive","ticket":22,"tenant":0,"in_flight":2,"queued":2}
{"t":2186,"ev":"queue","ticket":22,"tenant":0,"in_flight":2,"queued":3}
{"t":2203,"ev":"arrive","ticket":23,"tenant":1,"in_flight":2,"queued":3}
{"t":2203,"ev":"reject","ticket":23,"tenant":1,"in_flight":2,"queued":3,"reason":"queue_full"}
{"t":2240,"ev":"release","ticket":17,"tenant":1,"in_flight":1,"queued":3}
{"t":2240,"ev":"grant","ticket":19,"tenant":1,"in_flight":2,"queued":2}
{"t":2247,"ev":"release","ticket":18,"tenant":0,"in_flight":1,"queued":2}
{"t":2247,"ev":"grant","ticket":21,"tenant":1,"in_flight":2,"queued":1}
{"t":2286,"ev":"release","ticket":19,"tenant":1,"in_flight":1,"queued":1}
{"t":2286,"ev":"grant","ticket":22,"tenant":0,"in_flight":2,"queued":0}
{"t":2304,"ev":"arrive","ticket":24,"tenant":0,"in_flight":2,"queued":0}
{"t":2304,"ev":"queue","ticket":24,"tenant":0,"in_flight":2,"queued":1}
{"t":2378,"ev":"arrive","ticket":25,"tenant":1,"in_flight":2,"queued":1}
{"t":2378,"ev":"queue","ticket":25,"tenant":1,"in_flight":2,"queued":2}
{"t":2409,"ev":"arrive","ticket":26,"tenant":0,"in_flight":2,"queued":2}
{"t":2409,"ev":"queue","ticket":26,"tenant":0,"in_flight":2,"queued":3}
{"t":2415,"ev":"arrive","ticket":27,"tenant":1,"in_flight":2,"queued":3}
{"t":2415,"ev":"reject","ticket":27,"tenant":1,"in_flight":2,"queued":3,"reason":"queue_full"}
{"t":2447,"ev":"release","ticket":22,"tenant":0,"in_flight":1,"queued":3}
{"t":2447,"ev":"grant","ticket":24,"tenant":0,"in_flight":2,"queued":2}
{"t":2552,"ev":"arrive","ticket":28,"tenant":0,"in_flight":2,"queued":2}
{"t":2552,"ev":"queue","ticket":28,"tenant":0,"in_flight":2,"queued":3}
{"t":2635,"ev":"release","ticket":21,"tenant":1,"in_flight":1,"queued":3}
{"t":2635,"ev":"grant","ticket":25,"tenant":1,"in_flight":2,"queued":2}
{"t":2728,"ev":"arrive","ticket":29,"tenant":1,"in_flight":2,"queued":2}
{"t":2728,"ev":"queue","ticket":29,"tenant":1,"in_flight":2,"queued":3}
{"t":2730,"ev":"release","ticket":25,"tenant":1,"in_flight":1,"queued":3}
{"t":2730,"ev":"grant","ticket":26,"tenant":0,"in_flight":2,"queued":2}
{"t":2891,"ev":"release","ticket":26,"tenant":0,"in_flight":1,"queued":2}
{"t":2891,"ev":"grant","ticket":28,"tenant":0,"in_flight":2,"queued":1}
{"t":2932,"ev":"release","ticket":24,"tenant":0,"in_flight":1,"queued":1}
{"t":2932,"ev":"grant","ticket":29,"tenant":1,"in_flight":2,"queued":0}
{"t":2949,"ev":"release","ticket":29,"tenant":1,"in_flight":1,"queued":0}
{"t":3169,"ev":"release","ticket":28,"tenant":0,"in_flight":0,"queued":0}
"##;
    let scenario = scenario(42, 30, 2, 2, 3, None, 100, 250);
    assert_eq!(trace_of(&scenario), expected);
}

// I3, I7, I8; expected trace computed independently from the simulation rules in docs/DESIGN.md.
#[test]
fn gate_golden_trace_two_slots_queue_3_with_timeout_400_matches_byte_for_byte() {
    let expected = r##"{"t":149,"ev":"arrive","ticket":0,"tenant":0,"in_flight":0,"queued":0}
{"t":149,"ev":"grant","ticket":0,"tenant":0,"in_flight":1,"queued":0}
{"t":205,"ev":"arrive","ticket":1,"tenant":1,"in_flight":1,"queued":0}
{"t":205,"ev":"grant","ticket":1,"tenant":1,"in_flight":2,"queued":0}
{"t":213,"ev":"arrive","ticket":2,"tenant":0,"in_flight":2,"queued":0}
{"t":213,"ev":"queue","ticket":2,"tenant":0,"in_flight":2,"queued":1}
{"t":229,"ev":"release","ticket":0,"tenant":0,"in_flight":1,"queued":1}
{"t":229,"ev":"grant","ticket":2,"tenant":0,"in_flight":2,"queued":0}
{"t":257,"ev":"arrive","ticket":3,"tenant":1,"in_flight":2,"queued":0}
{"t":257,"ev":"queue","ticket":3,"tenant":1,"in_flight":2,"queued":1}
{"t":325,"ev":"arrive","ticket":4,"tenant":0,"in_flight":2,"queued":1}
{"t":325,"ev":"queue","ticket":4,"tenant":0,"in_flight":2,"queued":2}
{"t":366,"ev":"arrive","ticket":5,"tenant":1,"in_flight":2,"queued":2}
{"t":366,"ev":"queue","ticket":5,"tenant":1,"in_flight":2,"queued":3}
{"t":378,"ev":"release","ticket":1,"tenant":1,"in_flight":1,"queued":3}
{"t":378,"ev":"grant","ticket":3,"tenant":1,"in_flight":2,"queued":2}
{"t":469,"ev":"arrive","ticket":6,"tenant":0,"in_flight":2,"queued":2}
{"t":469,"ev":"queue","ticket":6,"tenant":0,"in_flight":2,"queued":3}
{"t":603,"ev":"arrive","ticket":7,"tenant":1,"in_flight":2,"queued":3}
{"t":603,"ev":"reject","ticket":7,"tenant":1,"in_flight":2,"queued":3,"reason":"queue_full"}
{"t":624,"ev":"arrive","ticket":8,"tenant":0,"in_flight":2,"queued":3}
{"t":624,"ev":"reject","ticket":8,"tenant":0,"in_flight":2,"queued":3,"reason":"queue_full"}
{"t":643,"ev":"arrive","ticket":9,"tenant":1,"in_flight":2,"queued":3}
{"t":643,"ev":"reject","ticket":9,"tenant":1,"in_flight":2,"queued":3,"reason":"queue_full"}
{"t":664,"ev":"release","ticket":2,"tenant":0,"in_flight":1,"queued":3}
{"t":664,"ev":"grant","ticket":4,"tenant":0,"in_flight":2,"queued":2}
{"t":766,"ev":"expire","ticket":5,"tenant":1,"in_flight":2,"queued":1}
{"t":779,"ev":"release","ticket":3,"tenant":1,"in_flight":1,"queued":1}
{"t":779,"ev":"grant","ticket":6,"tenant":0,"in_flight":2,"queued":0}
{"t":835,"ev":"arrive","ticket":10,"tenant":0,"in_flight":2,"queued":0}
{"t":835,"ev":"queue","ticket":10,"tenant":0,"in_flight":2,"queued":1}
{"t":955,"ev":"arrive","ticket":11,"tenant":1,"in_flight":2,"queued":1}
{"t":955,"ev":"queue","ticket":11,"tenant":1,"in_flight":2,"queued":2}
{"t":970,"ev":"arrive","ticket":12,"tenant":0,"in_flight":2,"queued":2}
{"t":970,"ev":"queue","ticket":12,"tenant":0,"in_flight":2,"queued":3}
{"t":974,"ev":"release","ticket":4,"tenant":0,"in_flight":1,"queued":3}
{"t":974,"ev":"grant","ticket":10,"tenant":0,"in_flight":2,"queued":2}
{"t":1011,"ev":"release","ticket":10,"tenant":0,"in_flight":1,"queued":2}
{"t":1011,"ev":"grant","ticket":11,"tenant":1,"in_flight":2,"queued":1}
{"t":1040,"ev":"release","ticket":6,"tenant":0,"in_flight":1,"queued":1}
{"t":1040,"ev":"grant","ticket":12,"tenant":0,"in_flight":2,"queued":0}
{"t":1119,"ev":"arrive","ticket":13,"tenant":1,"in_flight":2,"queued":0}
{"t":1119,"ev":"queue","ticket":13,"tenant":1,"in_flight":2,"queued":1}
{"t":1179,"ev":"release","ticket":12,"tenant":0,"in_flight":1,"queued":1}
{"t":1179,"ev":"grant","ticket":13,"tenant":1,"in_flight":2,"queued":0}
{"t":1308,"ev":"arrive","ticket":14,"tenant":0,"in_flight":2,"queued":0}
{"t":1308,"ev":"queue","ticket":14,"tenant":0,"in_flight":2,"queued":1}
{"t":1321,"ev":"release","ticket":11,"tenant":1,"in_flight":1,"queued":1}
{"t":1321,"ev":"grant","ticket":14,"tenant":0,"in_flight":2,"queued":0}
{"t":1466,"ev":"arrive","ticket":15,"tenant":1,"in_flight":2,"queued":0}
{"t":1466,"ev":"queue","ticket":15,"tenant":1,"in_flight":2,"queued":1}
{"t":1572,"ev":"release","ticket":13,"tenant":1,"in_flight":1,"queued":1}
{"t":1572,"ev":"grant","ticket":15,"tenant":1,"in_flight":2,"queued":0}
{"t":1596,"ev":"arrive","ticket":16,"tenant":0,"in_flight":2,"queued":0}
{"t":1596,"ev":"queue","ticket":16,"tenant":0,"in_flight":2,"queued":1}
{"t":1669,"ev":"release","ticket":14,"tenant":0,"in_flight":1,"queued":1}
{"t":1669,"ev":"grant","ticket":16,"tenant":0,"in_flight":2,"queued":0}
{"t":1724,"ev":"arrive","ticket":17,"tenant":1,"in_flight":2,"queued":0}
{"t":1724,"ev":"queue","ticket":17,"tenant":1,"in_flight":2,"queued":1}
{"t":1737,"ev":"arrive","ticket":18,"tenant":0,"in_flight":2,"queued":1}
{"t":1737,"ev":"queue","ticket":18,"tenant":0,"in_flight":2,"queued":2}
{"t":1890,"ev":"arrive","ticket":19,"tenant":1,"in_flight":2,"queued":2}
{"t":1890,"ev":"queue","ticket":19,"tenant":1,"in_flight":2,"queued":3}
{"t":1993,"ev":"release","ticket":15,"tenant":1,"in_flight":1,"queued":3}
{"t":1993,"ev":"grant","ticket":17,"tenant":1,"in_flight":2,"queued":2}
{"t":1997,"ev":"arrive","ticket":20,"tenant":0,"in_flight":2,"queued":2}
{"t":1997,"ev":"queue","ticket":20,"tenant":0,"in_flight":2,"queued":3}
{"t":2052,"ev":"arrive","ticket":21,"tenant":1,"in_flight":2,"queued":3}
{"t":2052,"ev":"reject","ticket":21,"tenant":1,"in_flight":2,"queued":3,"reason":"queue_full"}
{"t":2061,"ev":"release","ticket":16,"tenant":0,"in_flight":1,"queued":3}
{"t":2061,"ev":"grant","ticket":18,"tenant":0,"in_flight":2,"queued":2}
{"t":2184,"ev":"release","ticket":17,"tenant":1,"in_flight":1,"queued":2}
{"t":2184,"ev":"grant","ticket":19,"tenant":1,"in_flight":2,"queued":1}
{"t":2186,"ev":"arrive","ticket":22,"tenant":0,"in_flight":2,"queued":1}
{"t":2186,"ev":"queue","ticket":22,"tenant":0,"in_flight":2,"queued":2}
{"t":2195,"ev":"release","ticket":18,"tenant":0,"in_flight":1,"queued":2}
{"t":2195,"ev":"grant","ticket":20,"tenant":0,"in_flight":2,"queued":1}
{"t":2203,"ev":"arrive","ticket":23,"tenant":1,"in_flight":2,"queued":1}
{"t":2203,"ev":"queue","ticket":23,"tenant":1,"in_flight":2,"queued":2}
{"t":2230,"ev":"release","ticket":19,"tenant":1,"in_flight":1,"queued":2}
{"t":2230,"ev":"grant","ticket":22,"tenant":0,"in_flight":2,"queued":1}
{"t":2275,"ev":"release","ticket":20,"tenant":0,"in_flight":1,"queued":1}
{"t":2275,"ev":"grant","ticket":23,"tenant":1,"in_flight":2,"queued":0}
{"t":2304,"ev":"arrive","ticket":24,"tenant":0,"in_flight":2,"queued":0}
{"t":2304,"ev":"queue","ticket":24,"tenant":0,"in_flight":2,"queued":1}
{"t":2347,"ev":"release","ticket":23,"tenant":1,"in_flight":1,"queued":1}
{"t":2347,"ev":"grant","ticket":24,"tenant":0,"in_flight":2,"queued":0}
{"t":2378,"ev":"arrive","ticket":25,"tenant":1,"in_flight":2,"queued":0}
{"t":2378,"ev":"queue","ticket":25,"tenant":1,"in_flight":2,"queued":1}
{"t":2391,"ev":"release","ticket":22,"tenant":0,"in_flight":1,"queued":1}
{"t":2391,"ev":"grant","ticket":25,"tenant":1,"in_flight":2,"queued":0}
{"t":2409,"ev":"arrive","ticket":26,"tenant":0,"in_flight":2,"queued":0}
{"t":2409,"ev":"queue","ticket":26,"tenant":0,"in_flight":2,"queued":1}
{"t":2415,"ev":"arrive","ticket":27,"tenant":1,"in_flight":2,"queued":1}
{"t":2415,"ev":"queue","ticket":27,"tenant":1,"in_flight":2,"queued":2}
{"t":2486,"ev":"release","ticket":25,"tenant":1,"in_flight":1,"queued":2}
{"t":2486,"ev":"grant","ticket":26,"tenant":0,"in_flight":2,"queued":1}
{"t":2552,"ev":"arrive","ticket":28,"tenant":0,"in_flight":2,"queued":1}
{"t":2552,"ev":"queue","ticket":28,"tenant":0,"in_flight":2,"queued":2}
{"t":2647,"ev":"release","ticket":26,"tenant":0,"in_flight":1,"queued":2}
{"t":2647,"ev":"grant","ticket":27,"tenant":1,"in_flight":2,"queued":1}
{"t":2728,"ev":"arrive","ticket":29,"tenant":1,"in_flight":2,"queued":1}
{"t":2728,"ev":"queue","ticket":29,"tenant":1,"in_flight":2,"queued":2}
{"t":2832,"ev":"release","ticket":24,"tenant":0,"in_flight":1,"queued":2}
{"t":2832,"ev":"grant","ticket":28,"tenant":0,"in_flight":2,"queued":1}
{"t":3059,"ev":"release","ticket":27,"tenant":1,"in_flight":1,"queued":1}
{"t":3059,"ev":"grant","ticket":29,"tenant":1,"in_flight":2,"queued":0}
{"t":3076,"ev":"release","ticket":29,"tenant":1,"in_flight":1,"queued":0}
{"t":3110,"ev":"release","ticket":28,"tenant":0,"in_flight":0,"queued":0}
"##;
    let scenario = scenario(42, 30, 2, 2, 3, Some(400), 100, 250);
    assert_eq!(trace_of(&scenario), expected);
}

// I3, I7, I8; expected trace computed independently from the simulation rules in docs/DESIGN.md.
#[test]
fn gate_golden_trace_two_slots_no_queue_without_a_timeout_matches_byte_for_byte() {
    let expected = r##"{"t":149,"ev":"arrive","ticket":0,"tenant":0,"in_flight":0,"queued":0}
{"t":149,"ev":"grant","ticket":0,"tenant":0,"in_flight":1,"queued":0}
{"t":205,"ev":"arrive","ticket":1,"tenant":1,"in_flight":1,"queued":0}
{"t":205,"ev":"grant","ticket":1,"tenant":1,"in_flight":2,"queued":0}
{"t":213,"ev":"arrive","ticket":2,"tenant":0,"in_flight":2,"queued":0}
{"t":213,"ev":"reject","ticket":2,"tenant":0,"in_flight":2,"queued":0,"reason":"queue_full"}
{"t":229,"ev":"release","ticket":0,"tenant":0,"in_flight":1,"queued":0}
{"t":257,"ev":"arrive","ticket":3,"tenant":1,"in_flight":1,"queued":0}
{"t":257,"ev":"grant","ticket":3,"tenant":1,"in_flight":2,"queued":0}
{"t":325,"ev":"arrive","ticket":4,"tenant":0,"in_flight":2,"queued":0}
{"t":325,"ev":"reject","ticket":4,"tenant":0,"in_flight":2,"queued":0,"reason":"queue_full"}
{"t":366,"ev":"arrive","ticket":5,"tenant":1,"in_flight":2,"queued":0}
{"t":366,"ev":"reject","ticket":5,"tenant":1,"in_flight":2,"queued":0,"reason":"queue_full"}
{"t":378,"ev":"release","ticket":1,"tenant":1,"in_flight":1,"queued":0}
{"t":469,"ev":"arrive","ticket":6,"tenant":0,"in_flight":1,"queued":0}
{"t":469,"ev":"grant","ticket":6,"tenant":0,"in_flight":2,"queued":0}
{"t":603,"ev":"arrive","ticket":7,"tenant":1,"in_flight":2,"queued":0}
{"t":603,"ev":"reject","ticket":7,"tenant":1,"in_flight":2,"queued":0,"reason":"queue_full"}
{"t":624,"ev":"arrive","ticket":8,"tenant":0,"in_flight":2,"queued":0}
{"t":624,"ev":"reject","ticket":8,"tenant":0,"in_flight":2,"queued":0,"reason":"queue_full"}
{"t":643,"ev":"arrive","ticket":9,"tenant":1,"in_flight":2,"queued":0}
{"t":643,"ev":"reject","ticket":9,"tenant":1,"in_flight":2,"queued":0,"reason":"queue_full"}
{"t":658,"ev":"release","ticket":3,"tenant":1,"in_flight":1,"queued":0}
{"t":730,"ev":"release","ticket":6,"tenant":0,"in_flight":0,"queued":0}
{"t":835,"ev":"arrive","ticket":10,"tenant":0,"in_flight":0,"queued":0}
{"t":835,"ev":"grant","ticket":10,"tenant":0,"in_flight":1,"queued":0}
{"t":872,"ev":"release","ticket":10,"tenant":0,"in_flight":0,"queued":0}
{"t":955,"ev":"arrive","ticket":11,"tenant":1,"in_flight":0,"queued":0}
{"t":955,"ev":"grant","ticket":11,"tenant":1,"in_flight":1,"queued":0}
{"t":1265,"ev":"release","ticket":11,"tenant":1,"in_flight":0,"queued":0}
"##;
    let scenario = scenario(42, 12, 2, 2, 0, None, 100, 250);
    assert_eq!(trace_of(&scenario), expected);
}

// I3, I7, I8; expected trace computed independently from the simulation rules in docs/DESIGN.md.
#[test]
fn gate_golden_trace_timeout_0_rejects_everything_as_expired_matches_byte_for_byte() {
    let expected = r##"{"t":114,"ev":"arrive","ticket":0,"tenant":0,"in_flight":0,"queued":0}
{"t":114,"ev":"reject","ticket":0,"tenant":0,"in_flight":0,"queued":0,"reason":"expired"}
{"t":309,"ev":"arrive","ticket":1,"tenant":0,"in_flight":0,"queued":0}
{"t":309,"ev":"reject","ticket":1,"tenant":0,"in_flight":0,"queued":0,"reason":"expired"}
{"t":398,"ev":"arrive","ticket":2,"tenant":0,"in_flight":0,"queued":0}
{"t":398,"ev":"reject","ticket":2,"tenant":0,"in_flight":0,"queued":0,"reason":"expired"}
{"t":574,"ev":"arrive","ticket":3,"tenant":0,"in_flight":0,"queued":0}
{"t":574,"ev":"reject","ticket":3,"tenant":0,"in_flight":0,"queued":0,"reason":"expired"}
{"t":632,"ev":"arrive","ticket":4,"tenant":0,"in_flight":0,"queued":0}
{"t":632,"ev":"reject","ticket":4,"tenant":0,"in_flight":0,"queued":0,"reason":"expired"}
{"t":713,"ev":"arrive","ticket":5,"tenant":0,"in_flight":0,"queued":0}
{"t":713,"ev":"reject","ticket":5,"tenant":0,"in_flight":0,"queued":0,"reason":"expired"}
{"t":804,"ev":"arrive","ticket":6,"tenant":0,"in_flight":0,"queued":0}
{"t":804,"ev":"reject","ticket":6,"tenant":0,"in_flight":0,"queued":0,"reason":"expired"}
{"t":892,"ev":"arrive","ticket":7,"tenant":0,"in_flight":0,"queued":0}
{"t":892,"ev":"reject","ticket":7,"tenant":0,"in_flight":0,"queued":0,"reason":"expired"}
{"t":1022,"ev":"arrive","ticket":8,"tenant":0,"in_flight":0,"queued":0}
{"t":1022,"ev":"reject","ticket":8,"tenant":0,"in_flight":0,"queued":0,"reason":"expired"}
{"t":1159,"ev":"arrive","ticket":9,"tenant":0,"in_flight":0,"queued":0}
{"t":1159,"ev":"reject","ticket":9,"tenant":0,"in_flight":0,"queued":0,"reason":"expired"}
"##;
    let scenario = scenario(1, 10, 1, 1, 5, Some(0), 100, 250);
    assert_eq!(trace_of(&scenario), expected);
}

// I3, I7, I8; expected trace computed independently from the simulation rules in docs/DESIGN.md.
#[test]
fn gate_golden_trace_one_slot_queue_5_timeout_900_matches_byte_for_byte() {
    let expected = r##"{"t":39,"ev":"arrive","ticket":0,"tenant":0,"in_flight":0,"queued":0}
{"t":39,"ev":"grant","ticket":0,"tenant":0,"in_flight":1,"queued":0}
{"t":53,"ev":"release","ticket":0,"tenant":0,"in_flight":0,"queued":0}
{"t":130,"ev":"arrive","ticket":1,"tenant":1,"in_flight":0,"queued":0}
{"t":130,"ev":"grant","ticket":1,"tenant":1,"in_flight":1,"queued":0}
{"t":176,"ev":"arrive","ticket":2,"tenant":2,"in_flight":1,"queued":0}
{"t":176,"ev":"queue","ticket":2,"tenant":2,"in_flight":1,"queued":1}
{"t":223,"ev":"arrive","ticket":3,"tenant":0,"in_flight":1,"queued":1}
{"t":223,"ev":"queue","ticket":3,"tenant":0,"in_flight":1,"queued":2}
{"t":237,"ev":"arrive","ticket":4,"tenant":1,"in_flight":1,"queued":2}
{"t":237,"ev":"queue","ticket":4,"tenant":1,"in_flight":1,"queued":3}
{"t":248,"ev":"arrive","ticket":5,"tenant":2,"in_flight":1,"queued":3}
{"t":248,"ev":"queue","ticket":5,"tenant":2,"in_flight":1,"queued":4}
{"t":340,"ev":"arrive","ticket":6,"tenant":0,"in_flight":1,"queued":4}
{"t":340,"ev":"queue","ticket":6,"tenant":0,"in_flight":1,"queued":5}
{"t":427,"ev":"arrive","ticket":7,"tenant":1,"in_flight":1,"queued":5}
{"t":427,"ev":"reject","ticket":7,"tenant":1,"in_flight":1,"queued":5,"reason":"queue_full"}
{"t":515,"ev":"arrive","ticket":8,"tenant":2,"in_flight":1,"queued":5}
{"t":515,"ev":"reject","ticket":8,"tenant":2,"in_flight":1,"queued":5,"reason":"queue_full"}
{"t":577,"ev":"arrive","ticket":9,"tenant":0,"in_flight":1,"queued":5}
{"t":577,"ev":"reject","ticket":9,"tenant":0,"in_flight":1,"queued":5,"reason":"queue_full"}
{"t":597,"ev":"release","ticket":1,"tenant":1,"in_flight":0,"queued":5}
{"t":597,"ev":"grant","ticket":2,"tenant":2,"in_flight":1,"queued":4}
{"t":645,"ev":"arrive","ticket":10,"tenant":1,"in_flight":1,"queued":4}
{"t":645,"ev":"queue","ticket":10,"tenant":1,"in_flight":1,"queued":5}
{"t":680,"ev":"arrive","ticket":11,"tenant":2,"in_flight":1,"queued":5}
{"t":680,"ev":"reject","ticket":11,"tenant":2,"in_flight":1,"queued":5,"reason":"queue_full"}
{"t":771,"ev":"arrive","ticket":12,"tenant":0,"in_flight":1,"queued":5}
{"t":771,"ev":"reject","ticket":12,"tenant":0,"in_flight":1,"queued":5,"reason":"queue_full"}
{"t":779,"ev":"arrive","ticket":13,"tenant":1,"in_flight":1,"queued":5}
{"t":779,"ev":"reject","ticket":13,"tenant":1,"in_flight":1,"queued":5,"reason":"queue_full"}
{"t":797,"ev":"release","ticket":2,"tenant":2,"in_flight":0,"queued":5}
{"t":797,"ev":"grant","ticket":3,"tenant":0,"in_flight":1,"queued":4}
{"t":870,"ev":"arrive","ticket":14,"tenant":2,"in_flight":1,"queued":4}
{"t":870,"ev":"queue","ticket":14,"tenant":2,"in_flight":1,"queued":5}
{"t":968,"ev":"arrive","ticket":15,"tenant":0,"in_flight":1,"queued":5}
{"t":968,"ev":"reject","ticket":15,"tenant":0,"in_flight":1,"queued":5,"reason":"queue_full"}
{"t":1007,"ev":"arrive","ticket":16,"tenant":1,"in_flight":1,"queued":5}
{"t":1007,"ev":"reject","ticket":16,"tenant":1,"in_flight":1,"queued":5,"reason":"queue_full"}
{"t":1060,"ev":"release","ticket":3,"tenant":0,"in_flight":0,"queued":5}
{"t":1060,"ev":"grant","ticket":4,"tenant":1,"in_flight":1,"queued":4}
{"t":1063,"ev":"arrive","ticket":17,"tenant":2,"in_flight":1,"queued":4}
{"t":1063,"ev":"queue","ticket":17,"tenant":2,"in_flight":1,"queued":5}
{"t":1071,"ev":"arrive","ticket":18,"tenant":0,"in_flight":1,"queued":5}
{"t":1071,"ev":"reject","ticket":18,"tenant":0,"in_flight":1,"queued":5,"reason":"queue_full"}
{"t":1093,"ev":"arrive","ticket":19,"tenant":1,"in_flight":1,"queued":5}
{"t":1093,"ev":"reject","ticket":19,"tenant":1,"in_flight":1,"queued":5,"reason":"queue_full"}
{"t":1148,"ev":"expire","ticket":5,"tenant":2,"in_flight":1,"queued":4}
{"t":1158,"ev":"arrive","ticket":20,"tenant":2,"in_flight":1,"queued":4}
{"t":1158,"ev":"queue","ticket":20,"tenant":2,"in_flight":1,"queued":5}
{"t":1197,"ev":"arrive","ticket":21,"tenant":0,"in_flight":1,"queued":5}
{"t":1197,"ev":"reject","ticket":21,"tenant":0,"in_flight":1,"queued":5,"reason":"queue_full"}
{"t":1202,"ev":"arrive","ticket":22,"tenant":1,"in_flight":1,"queued":5}
{"t":1202,"ev":"reject","ticket":22,"tenant":1,"in_flight":1,"queued":5,"reason":"queue_full"}
{"t":1240,"ev":"expire","ticket":6,"tenant":0,"in_flight":1,"queued":4}
{"t":1256,"ev":"arrive","ticket":23,"tenant":2,"in_flight":1,"queued":4}
{"t":1256,"ev":"queue","ticket":23,"tenant":2,"in_flight":1,"queued":5}
{"t":1341,"ev":"arrive","ticket":24,"tenant":0,"in_flight":1,"queued":5}
{"t":1341,"ev":"reject","ticket":24,"tenant":0,"in_flight":1,"queued":5,"reason":"queue_full"}
{"t":1391,"ev":"release","ticket":4,"tenant":1,"in_flight":0,"queued":5}
{"t":1391,"ev":"grant","ticket":10,"tenant":1,"in_flight":1,"queued":4}
{"t":1409,"ev":"arrive","ticket":25,"tenant":1,"in_flight":1,"queued":4}
{"t":1409,"ev":"queue","ticket":25,"tenant":1,"in_flight":1,"queued":5}
{"t":1416,"ev":"arrive","ticket":26,"tenant":2,"in_flight":1,"queued":5}
{"t":1416,"ev":"reject","ticket":26,"tenant":2,"in_flight":1,"queued":5,"reason":"queue_full"}
{"t":1474,"ev":"arrive","ticket":27,"tenant":0,"in_flight":1,"queued":5}
{"t":1474,"ev":"reject","ticket":27,"tenant":0,"in_flight":1,"queued":5,"reason":"queue_full"}
{"t":1477,"ev":"release","ticket":10,"tenant":1,"in_flight":0,"queued":5}
{"t":1477,"ev":"grant","ticket":14,"tenant":2,"in_flight":1,"queued":4}
{"t":1524,"ev":"arrive","ticket":28,"tenant":1,"in_flight":1,"queued":4}
{"t":1524,"ev":"queue","ticket":28,"tenant":1,"in_flight":1,"queued":5}
{"t":1582,"ev":"arrive","ticket":29,"tenant":2,"in_flight":1,"queued":5}
{"t":1582,"ev":"reject","ticket":29,"tenant":2,"in_flight":1,"queued":5,"reason":"queue_full"}
{"t":1618,"ev":"arrive","ticket":30,"tenant":0,"in_flight":1,"queued":5}
{"t":1618,"ev":"reject","ticket":30,"tenant":0,"in_flight":1,"queued":5,"reason":"queue_full"}
{"t":1706,"ev":"arrive","ticket":31,"tenant":1,"in_flight":1,"queued":5}
{"t":1706,"ev":"reject","ticket":31,"tenant":1,"in_flight":1,"queued":5,"reason":"queue_full"}
{"t":1753,"ev":"arrive","ticket":32,"tenant":2,"in_flight":1,"queued":5}
{"t":1753,"ev":"reject","ticket":32,"tenant":2,"in_flight":1,"queued":5,"reason":"queue_full"}
{"t":1810,"ev":"release","ticket":14,"tenant":2,"in_flight":0,"queued":5}
{"t":1810,"ev":"grant","ticket":17,"tenant":2,"in_flight":1,"queued":4}
{"t":1833,"ev":"arrive","ticket":33,"tenant":0,"in_flight":1,"queued":4}
{"t":1833,"ev":"queue","ticket":33,"tenant":0,"in_flight":1,"queued":5}
{"t":1932,"ev":"arrive","ticket":34,"tenant":1,"in_flight":1,"queued":5}
{"t":1932,"ev":"reject","ticket":34,"tenant":1,"in_flight":1,"queued":5,"reason":"queue_full"}
{"t":1969,"ev":"arrive","ticket":35,"tenant":2,"in_flight":1,"queued":5}
{"t":1969,"ev":"reject","ticket":35,"tenant":2,"in_flight":1,"queued":5,"reason":"queue_full"}
{"t":2022,"ev":"arrive","ticket":36,"tenant":0,"in_flight":1,"queued":5}
{"t":2022,"ev":"reject","ticket":36,"tenant":0,"in_flight":1,"queued":5,"reason":"queue_full"}
{"t":2058,"ev":"expire","ticket":20,"tenant":2,"in_flight":1,"queued":4}
{"t":2088,"ev":"arrive","ticket":37,"tenant":1,"in_flight":1,"queued":4}
{"t":2088,"ev":"queue","ticket":37,"tenant":1,"in_flight":1,"queued":5}
{"t":2111,"ev":"arrive","ticket":38,"tenant":2,"in_flight":1,"queued":5}
{"t":2111,"ev":"reject","ticket":38,"tenant":2,"in_flight":1,"queued":5,"reason":"queue_full"}
{"t":2156,"ev":"expire","ticket":23,"tenant":2,"in_flight":1,"queued":4}
{"t":2164,"ev":"arrive","ticket":39,"tenant":0,"in_flight":1,"queued":4}
{"t":2164,"ev":"queue","ticket":39,"tenant":0,"in_flight":1,"queued":5}
{"t":2296,"ev":"release","ticket":17,"tenant":2,"in_flight":0,"queued":5}
{"t":2296,"ev":"grant","ticket":25,"tenant":1,"in_flight":1,"queued":4}
{"t":2424,"ev":"expire","ticket":28,"tenant":1,"in_flight":1,"queued":3}
{"t":2586,"ev":"release","ticket":25,"tenant":1,"in_flight":0,"queued":3}
{"t":2586,"ev":"grant","ticket":33,"tenant":0,"in_flight":1,"queued":2}
{"t":2988,"ev":"expire","ticket":37,"tenant":1,"in_flight":1,"queued":1}
{"t":3064,"ev":"expire","ticket":39,"tenant":0,"in_flight":1,"queued":0}
{"t":3209,"ev":"release","ticket":33,"tenant":0,"in_flight":0,"queued":0}
"##;
    let scenario = scenario(7, 40, 3, 1, 5, Some(900), 50, 400);
    assert_eq!(trace_of(&scenario), expected);
}
