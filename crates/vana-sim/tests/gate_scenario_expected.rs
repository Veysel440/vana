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

/// FNV-1a over the bytes of a trace, for the long traces that are compared by digest.
fn fnv1a(text: &str) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in text.bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

/// How many lines of each kind a trace has: arrive, grant, queue, reject, expire, release.
fn kinds(text: &str) -> [usize; 6] {
    let names = ["arrive", "grant", "queue", "reject", "expire", "release"];
    let mut counts = [0usize; 6];
    for line in text.lines() {
        for (i, name) in names.iter().enumerate() {
            if line.contains(&format!("\"ev\":\"{name}\"")) {
                counts[i] += 1;
            }
        }
    }
    counts
}

// DESIGN.md gate run rules; expected trace computed independently from the simulation rules in docs/DESIGN.md.
#[test]
fn gate_run_a_run_with_zero_requests_is_empty_matches_the_expected_trace() {
    let expected = r##""##;
    let scenario = scenario(5, 0, 1, 1, 3, None, 10, 10);
    assert_eq!(trace_of(&scenario), expected);
}

// DESIGN.md gate run rules; expected trace computed independently from the simulation rules in docs/DESIGN.md.
#[test]
fn gate_run_a_run_with_one_request_arrives_grants_and_releases_matches_the_expected_trace() {
    let expected = r##"{"t":8,"ev":"arrive","ticket":0,"tenant":0,"in_flight":0,"queued":0}
{"t":8,"ev":"grant","ticket":0,"tenant":0,"in_flight":1,"queued":0}
{"t":24,"ev":"release","ticket":0,"tenant":0,"in_flight":0,"queued":0}
"##;
    let scenario = scenario(5, 1, 1, 1, 3, Some(1000), 10, 10);
    assert_eq!(trace_of(&scenario), expected);
}

// DESIGN.md gate run rules; expected trace computed independently from the simulation rules in docs/DESIGN.md.
#[test]
fn gate_run_without_a_queue_a_timeout_never_expires_anything_matches_the_expected_trace() {
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
{"t":970,"ev":"arrive","ticket":12,"tenant":0,"in_flight":1,"queued":0}
{"t":970,"ev":"grant","ticket":12,"tenant":0,"in_flight":2,"queued":0}
{"t":1109,"ev":"release","ticket":12,"tenant":0,"in_flight":1,"queued":0}
{"t":1119,"ev":"arrive","ticket":13,"tenant":1,"in_flight":1,"queued":0}
{"t":1119,"ev":"grant","ticket":13,"tenant":1,"in_flight":2,"queued":0}
{"t":1265,"ev":"release","ticket":11,"tenant":1,"in_flight":1,"queued":0}
{"t":1308,"ev":"arrive","ticket":14,"tenant":0,"in_flight":1,"queued":0}
{"t":1308,"ev":"grant","ticket":14,"tenant":0,"in_flight":2,"queued":0}
{"t":1466,"ev":"arrive","ticket":15,"tenant":1,"in_flight":2,"queued":0}
{"t":1466,"ev":"reject","ticket":15,"tenant":1,"in_flight":2,"queued":0,"reason":"queue_full"}
{"t":1512,"ev":"release","ticket":13,"tenant":1,"in_flight":1,"queued":0}
{"t":1596,"ev":"arrive","ticket":16,"tenant":0,"in_flight":1,"queued":0}
{"t":1596,"ev":"grant","ticket":16,"tenant":0,"in_flight":2,"queued":0}
{"t":1656,"ev":"release","ticket":14,"tenant":0,"in_flight":1,"queued":0}
{"t":1724,"ev":"arrive","ticket":17,"tenant":1,"in_flight":1,"queued":0}
{"t":1724,"ev":"grant","ticket":17,"tenant":1,"in_flight":2,"queued":0}
{"t":1737,"ev":"arrive","ticket":18,"tenant":0,"in_flight":2,"queued":0}
{"t":1737,"ev":"reject","ticket":18,"tenant":0,"in_flight":2,"queued":0,"reason":"queue_full"}
{"t":1890,"ev":"arrive","ticket":19,"tenant":1,"in_flight":2,"queued":0}
{"t":1890,"ev":"reject","ticket":19,"tenant":1,"in_flight":2,"queued":0,"reason":"queue_full"}
{"t":1915,"ev":"release","ticket":17,"tenant":1,"in_flight":1,"queued":0}
{"t":1988,"ev":"release","ticket":16,"tenant":0,"in_flight":0,"queued":0}
{"t":1997,"ev":"arrive","ticket":20,"tenant":0,"in_flight":0,"queued":0}
{"t":1997,"ev":"grant","ticket":20,"tenant":0,"in_flight":1,"queued":0}
{"t":2052,"ev":"arrive","ticket":21,"tenant":1,"in_flight":1,"queued":0}
{"t":2052,"ev":"grant","ticket":21,"tenant":1,"in_flight":2,"queued":0}
{"t":2077,"ev":"release","ticket":20,"tenant":0,"in_flight":1,"queued":0}
{"t":2186,"ev":"arrive","ticket":22,"tenant":0,"in_flight":1,"queued":0}
{"t":2186,"ev":"grant","ticket":22,"tenant":0,"in_flight":2,"queued":0}
{"t":2203,"ev":"arrive","ticket":23,"tenant":1,"in_flight":2,"queued":0}
{"t":2203,"ev":"reject","ticket":23,"tenant":1,"in_flight":2,"queued":0,"reason":"queue_full"}
{"t":2304,"ev":"arrive","ticket":24,"tenant":0,"in_flight":2,"queued":0}
{"t":2304,"ev":"reject","ticket":24,"tenant":0,"in_flight":2,"queued":0,"reason":"queue_full"}
{"t":2347,"ev":"release","ticket":22,"tenant":0,"in_flight":1,"queued":0}
{"t":2378,"ev":"arrive","ticket":25,"tenant":1,"in_flight":1,"queued":0}
{"t":2378,"ev":"grant","ticket":25,"tenant":1,"in_flight":2,"queued":0}
{"t":2409,"ev":"arrive","ticket":26,"tenant":0,"in_flight":2,"queued":0}
{"t":2409,"ev":"reject","ticket":26,"tenant":0,"in_flight":2,"queued":0,"reason":"queue_full"}
{"t":2415,"ev":"arrive","ticket":27,"tenant":1,"in_flight":2,"queued":0}
{"t":2415,"ev":"reject","ticket":27,"tenant":1,"in_flight":2,"queued":0,"reason":"queue_full"}
{"t":2440,"ev":"release","ticket":21,"tenant":1,"in_flight":1,"queued":0}
{"t":2473,"ev":"release","ticket":25,"tenant":1,"in_flight":0,"queued":0}
{"t":2552,"ev":"arrive","ticket":28,"tenant":0,"in_flight":0,"queued":0}
{"t":2552,"ev":"grant","ticket":28,"tenant":0,"in_flight":1,"queued":0}
{"t":2728,"ev":"arrive","ticket":29,"tenant":1,"in_flight":1,"queued":0}
{"t":2728,"ev":"grant","ticket":29,"tenant":1,"in_flight":2,"queued":0}
{"t":2745,"ev":"release","ticket":29,"tenant":1,"in_flight":1,"queued":0}
{"t":2830,"ev":"release","ticket":28,"tenant":0,"in_flight":0,"queued":0}
"##;
    let scenario = scenario(42, 30, 2, 2, 0, Some(400), 100, 250);
    assert_eq!(trace_of(&scenario), expected);
}

// DESIGN.md gate run rules; expected trace computed independently from the simulation rules in docs/DESIGN.md.
#[test]
fn gate_run_without_a_queue_a_zero_timeout_rejects_everything_as_expired_matches_the_expected_trace()
 {
    let expected = r##"{"t":23,"ev":"arrive","ticket":0,"tenant":0,"in_flight":0,"queued":0}
{"t":23,"ev":"reject","ticket":0,"tenant":0,"in_flight":0,"queued":0,"reason":"expired"}
{"t":146,"ev":"arrive","ticket":1,"tenant":1,"in_flight":0,"queued":0}
{"t":146,"ev":"reject","ticket":1,"tenant":1,"in_flight":0,"queued":0,"reason":"expired"}
{"t":190,"ev":"arrive","ticket":2,"tenant":0,"in_flight":0,"queued":0}
{"t":190,"ev":"reject","ticket":2,"tenant":0,"in_flight":0,"queued":0,"reason":"expired"}
{"t":218,"ev":"arrive","ticket":3,"tenant":1,"in_flight":0,"queued":0}
{"t":218,"ev":"reject","ticket":3,"tenant":1,"in_flight":0,"queued":0,"reason":"expired"}
{"t":317,"ev":"arrive","ticket":4,"tenant":0,"in_flight":0,"queued":0}
{"t":317,"ev":"reject","ticket":4,"tenant":0,"in_flight":0,"queued":0,"reason":"expired"}
{"t":457,"ev":"arrive","ticket":5,"tenant":1,"in_flight":0,"queued":0}
{"t":457,"ev":"reject","ticket":5,"tenant":1,"in_flight":0,"queued":0,"reason":"expired"}
{"t":554,"ev":"arrive","ticket":6,"tenant":0,"in_flight":0,"queued":0}
{"t":554,"ev":"reject","ticket":6,"tenant":0,"in_flight":0,"queued":0,"reason":"expired"}
{"t":698,"ev":"arrive","ticket":7,"tenant":1,"in_flight":0,"queued":0}
{"t":698,"ev":"reject","ticket":7,"tenant":1,"in_flight":0,"queued":0,"reason":"expired"}
"##;
    let scenario = scenario(3, 8, 2, 2, 0, Some(0), 100, 250);
    assert_eq!(trace_of(&scenario), expected);
}

// DESIGN.md gate run rules; expected trace computed independently from the simulation rules in docs/DESIGN.md.
#[test]
fn gate_run_a_timeout_of_one_nanosecond_expires_queued_tickets_at_once_matches_the_expected_trace()
{
    let expected = r##"{"t":2,"ev":"arrive","ticket":0,"tenant":0,"in_flight":0,"queued":0}
{"t":2,"ev":"grant","ticket":0,"tenant":0,"in_flight":1,"queued":0}
{"t":6,"ev":"arrive","ticket":1,"tenant":1,"in_flight":1,"queued":0}
{"t":6,"ev":"queue","ticket":1,"tenant":1,"in_flight":1,"queued":1}
{"t":7,"ev":"expire","ticket":1,"tenant":1,"in_flight":1,"queued":0}
{"t":7,"ev":"arrive","ticket":2,"tenant":0,"in_flight":1,"queued":0}
{"t":7,"ev":"queue","ticket":2,"tenant":0,"in_flight":1,"queued":1}
{"t":8,"ev":"expire","ticket":2,"tenant":0,"in_flight":1,"queued":0}
{"t":8,"ev":"arrive","ticket":3,"tenant":1,"in_flight":1,"queued":0}
{"t":8,"ev":"queue","ticket":3,"tenant":1,"in_flight":1,"queued":1}
{"t":9,"ev":"release","ticket":0,"tenant":0,"in_flight":0,"queued":1}
{"t":9,"ev":"expire","ticket":3,"tenant":1,"in_flight":0,"queued":0}
{"t":11,"ev":"arrive","ticket":4,"tenant":0,"in_flight":0,"queued":0}
{"t":11,"ev":"grant","ticket":4,"tenant":0,"in_flight":1,"queued":0}
{"t":12,"ev":"arrive","ticket":5,"tenant":1,"in_flight":1,"queued":0}
{"t":12,"ev":"queue","ticket":5,"tenant":1,"in_flight":1,"queued":1}
{"t":13,"ev":"expire","ticket":5,"tenant":1,"in_flight":1,"queued":0}
{"t":18,"ev":"arrive","ticket":6,"tenant":0,"in_flight":1,"queued":0}
{"t":18,"ev":"queue","ticket":6,"tenant":0,"in_flight":1,"queued":1}
{"t":19,"ev":"expire","ticket":6,"tenant":0,"in_flight":1,"queued":0}
{"t":24,"ev":"arrive","ticket":7,"tenant":1,"in_flight":1,"queued":0}
{"t":24,"ev":"queue","ticket":7,"tenant":1,"in_flight":1,"queued":1}
{"t":25,"ev":"expire","ticket":7,"tenant":1,"in_flight":1,"queued":0}
{"t":28,"ev":"arrive","ticket":8,"tenant":0,"in_flight":1,"queued":0}
{"t":28,"ev":"queue","ticket":8,"tenant":0,"in_flight":1,"queued":1}
{"t":29,"ev":"expire","ticket":8,"tenant":0,"in_flight":1,"queued":0}
{"t":33,"ev":"arrive","ticket":9,"tenant":1,"in_flight":1,"queued":0}
{"t":33,"ev":"queue","ticket":9,"tenant":1,"in_flight":1,"queued":1}
{"t":34,"ev":"release","ticket":4,"tenant":0,"in_flight":0,"queued":1}
{"t":34,"ev":"expire","ticket":9,"tenant":1,"in_flight":0,"queued":0}
{"t":35,"ev":"arrive","ticket":10,"tenant":0,"in_flight":0,"queued":0}
{"t":35,"ev":"grant","ticket":10,"tenant":0,"in_flight":1,"queued":0}
{"t":40,"ev":"arrive","ticket":11,"tenant":1,"in_flight":1,"queued":0}
{"t":40,"ev":"queue","ticket":11,"tenant":1,"in_flight":1,"queued":1}
{"t":41,"ev":"expire","ticket":11,"tenant":1,"in_flight":1,"queued":0}
{"t":44,"ev":"arrive","ticket":12,"tenant":0,"in_flight":1,"queued":0}
{"t":44,"ev":"queue","ticket":12,"tenant":0,"in_flight":1,"queued":1}
{"t":45,"ev":"expire","ticket":12,"tenant":0,"in_flight":1,"queued":0}
{"t":50,"ev":"arrive","ticket":13,"tenant":1,"in_flight":1,"queued":0}
{"t":50,"ev":"queue","ticket":13,"tenant":1,"in_flight":1,"queued":1}
{"t":51,"ev":"expire","ticket":13,"tenant":1,"in_flight":1,"queued":0}
{"t":53,"ev":"arrive","ticket":14,"tenant":0,"in_flight":1,"queued":0}
{"t":53,"ev":"queue","ticket":14,"tenant":0,"in_flight":1,"queued":1}
{"t":54,"ev":"expire","ticket":14,"tenant":0,"in_flight":1,"queued":0}
{"t":55,"ev":"release","ticket":10,"tenant":0,"in_flight":0,"queued":0}
{"t":57,"ev":"arrive","ticket":15,"tenant":1,"in_flight":0,"queued":0}
{"t":57,"ev":"grant","ticket":15,"tenant":1,"in_flight":1,"queued":0}
{"t":61,"ev":"arrive","ticket":16,"tenant":0,"in_flight":1,"queued":0}
{"t":61,"ev":"queue","ticket":16,"tenant":0,"in_flight":1,"queued":1}
{"t":62,"ev":"release","ticket":15,"tenant":1,"in_flight":0,"queued":1}
{"t":62,"ev":"expire","ticket":16,"tenant":0,"in_flight":0,"queued":0}
{"t":64,"ev":"arrive","ticket":17,"tenant":1,"in_flight":0,"queued":0}
{"t":64,"ev":"grant","ticket":17,"tenant":1,"in_flight":1,"queued":0}
{"t":66,"ev":"arrive","ticket":18,"tenant":0,"in_flight":1,"queued":0}
{"t":66,"ev":"queue","ticket":18,"tenant":0,"in_flight":1,"queued":1}
{"t":67,"ev":"expire","ticket":18,"tenant":0,"in_flight":1,"queued":0}
{"t":70,"ev":"arrive","ticket":19,"tenant":1,"in_flight":1,"queued":0}
{"t":70,"ev":"queue","ticket":19,"tenant":1,"in_flight":1,"queued":1}
{"t":71,"ev":"expire","ticket":19,"tenant":1,"in_flight":1,"queued":0}
{"t":73,"ev":"arrive","ticket":20,"tenant":0,"in_flight":1,"queued":0}
{"t":73,"ev":"queue","ticket":20,"tenant":0,"in_flight":1,"queued":1}
{"t":74,"ev":"expire","ticket":20,"tenant":0,"in_flight":1,"queued":0}
{"t":79,"ev":"arrive","ticket":21,"tenant":1,"in_flight":1,"queued":0}
{"t":79,"ev":"queue","ticket":21,"tenant":1,"in_flight":1,"queued":1}
{"t":80,"ev":"expire","ticket":21,"tenant":1,"in_flight":1,"queued":0}
{"t":83,"ev":"arrive","ticket":22,"tenant":0,"in_flight":1,"queued":0}
{"t":83,"ev":"queue","ticket":22,"tenant":0,"in_flight":1,"queued":1}
{"t":84,"ev":"expire","ticket":22,"tenant":0,"in_flight":1,"queued":0}
{"t":87,"ev":"release","ticket":17,"tenant":1,"in_flight":0,"queued":0}
{"t":89,"ev":"arrive","ticket":23,"tenant":1,"in_flight":0,"queued":0}
{"t":89,"ev":"grant","ticket":23,"tenant":1,"in_flight":1,"queued":0}
{"t":95,"ev":"arrive","ticket":24,"tenant":0,"in_flight":1,"queued":0}
{"t":95,"ev":"queue","ticket":24,"tenant":0,"in_flight":1,"queued":1}
{"t":96,"ev":"expire","ticket":24,"tenant":0,"in_flight":1,"queued":0}
{"t":100,"ev":"arrive","ticket":25,"tenant":1,"in_flight":1,"queued":0}
{"t":100,"ev":"queue","ticket":25,"tenant":1,"in_flight":1,"queued":1}
{"t":101,"ev":"expire","ticket":25,"tenant":1,"in_flight":1,"queued":0}
{"t":105,"ev":"arrive","ticket":26,"tenant":0,"in_flight":1,"queued":0}
{"t":105,"ev":"queue","ticket":26,"tenant":0,"in_flight":1,"queued":1}
{"t":106,"ev":"release","ticket":23,"tenant":1,"in_flight":0,"queued":1}
{"t":106,"ev":"expire","ticket":26,"tenant":0,"in_flight":0,"queued":0}
{"t":108,"ev":"arrive","ticket":27,"tenant":1,"in_flight":0,"queued":0}
{"t":108,"ev":"grant","ticket":27,"tenant":1,"in_flight":1,"queued":0}
{"t":109,"ev":"arrive","ticket":28,"tenant":0,"in_flight":1,"queued":0}
{"t":109,"ev":"queue","ticket":28,"tenant":0,"in_flight":1,"queued":1}
{"t":110,"ev":"expire","ticket":28,"tenant":0,"in_flight":1,"queued":0}
{"t":114,"ev":"arrive","ticket":29,"tenant":1,"in_flight":1,"queued":0}
{"t":114,"ev":"queue","ticket":29,"tenant":1,"in_flight":1,"queued":1}
{"t":115,"ev":"expire","ticket":29,"tenant":1,"in_flight":1,"queued":0}
{"t":128,"ev":"release","ticket":27,"tenant":1,"in_flight":0,"queued":0}
"##;
    let scenario = scenario(11, 30, 2, 1, 4, Some(1), 3, 12);
    assert_eq!(trace_of(&scenario), expected);
}

// DESIGN.md gate run rules; digest and counts computed independently from the simulation rules in docs/DESIGN.md.
#[test]
fn gate_run_dense_ties_of_completions_deadlines_and_arrivals_keep_the_documented_order_matches_the_expected_digest()
 {
    let scenario = scenario(3, 60, 2, 1, 3, Some(2), 1, 1);
    let trace = trace_of(&scenario);
    assert_eq!(trace.lines().count(), 197);
    assert_eq!(kinds(&trace), [60, 54, 23, 0, 6, 54]);
    assert_eq!(fnv1a(&trace), 0x7f144d415740dd48);
}

// DESIGN.md gate run rules; digest and counts computed independently from the simulation rules in docs/DESIGN.md.
#[test]
fn gate_run_dense_ties_with_two_slots_keep_the_documented_order_matches_the_expected_digest() {
    let scenario = scenario(8, 60, 3, 2, 4, Some(3), 2, 2);
    let trace = trace_of(&scenario);
    assert_eq!(trace.lines().count(), 182);
    assert_eq!(kinds(&trace), [60, 60, 2, 0, 0, 60]);
    assert_eq!(fnv1a(&trace), 0xeaa9d5c92221d7d3);
}

// DESIGN.md gate run rules; expected trace computed independently from the simulation rules in docs/DESIGN.md.
#[test]
fn gate_run_one_slot_and_a_queue_of_one_matches_the_expected_trace() {
    let expected = r##"{"t":2,"ev":"arrive","ticket":0,"tenant":0,"in_flight":0,"queued":0}
{"t":2,"ev":"grant","ticket":0,"tenant":0,"in_flight":1,"queued":0}
{"t":23,"ev":"arrive","ticket":1,"tenant":1,"in_flight":1,"queued":0}
{"t":23,"ev":"queue","ticket":1,"tenant":1,"in_flight":1,"queued":1}
{"t":27,"ev":"arrive","ticket":2,"tenant":0,"in_flight":1,"queued":1}
{"t":27,"ev":"reject","ticket":2,"tenant":0,"in_flight":1,"queued":1,"reason":"queue_full"}
{"t":61,"ev":"arrive","ticket":3,"tenant":1,"in_flight":1,"queued":1}
{"t":61,"ev":"reject","ticket":3,"tenant":1,"in_flight":1,"queued":1,"reason":"queue_full"}
{"t":76,"ev":"arrive","ticket":4,"tenant":0,"in_flight":1,"queued":1}
{"t":76,"ev":"reject","ticket":4,"tenant":0,"in_flight":1,"queued":1,"reason":"queue_full"}
{"t":107,"ev":"arrive","ticket":5,"tenant":1,"in_flight":1,"queued":1}
{"t":107,"ev":"reject","ticket":5,"tenant":1,"in_flight":1,"queued":1,"reason":"queue_full"}
{"t":112,"ev":"release","ticket":0,"tenant":0,"in_flight":0,"queued":1}
{"t":112,"ev":"grant","ticket":1,"tenant":1,"in_flight":1,"queued":0}
{"t":134,"ev":"arrive","ticket":6,"tenant":0,"in_flight":1,"queued":0}
{"t":134,"ev":"queue","ticket":6,"tenant":0,"in_flight":1,"queued":1}
{"t":147,"ev":"arrive","ticket":7,"tenant":1,"in_flight":1,"queued":1}
{"t":147,"ev":"reject","ticket":7,"tenant":1,"in_flight":1,"queued":1,"reason":"queue_full"}
{"t":165,"ev":"arrive","ticket":8,"tenant":0,"in_flight":1,"queued":1}
{"t":165,"ev":"reject","ticket":8,"tenant":0,"in_flight":1,"queued":1,"reason":"queue_full"}
{"t":191,"ev":"release","ticket":1,"tenant":1,"in_flight":0,"queued":1}
{"t":191,"ev":"grant","ticket":6,"tenant":0,"in_flight":1,"queued":0}
{"t":198,"ev":"arrive","ticket":9,"tenant":1,"in_flight":1,"queued":0}
{"t":198,"ev":"queue","ticket":9,"tenant":1,"in_flight":1,"queued":1}
{"t":210,"ev":"arrive","ticket":10,"tenant":0,"in_flight":1,"queued":1}
{"t":210,"ev":"reject","ticket":10,"tenant":0,"in_flight":1,"queued":1,"reason":"queue_full"}
{"t":218,"ev":"arrive","ticket":11,"tenant":1,"in_flight":1,"queued":1}
{"t":218,"ev":"reject","ticket":11,"tenant":1,"in_flight":1,"queued":1,"reason":"queue_full"}
{"t":237,"ev":"arrive","ticket":12,"tenant":0,"in_flight":1,"queued":1}
{"t":237,"ev":"reject","ticket":12,"tenant":0,"in_flight":1,"queued":1,"reason":"queue_full"}
{"t":260,"ev":"release","ticket":6,"tenant":0,"in_flight":0,"queued":1}
{"t":260,"ev":"grant","ticket":9,"tenant":1,"in_flight":1,"queued":0}
{"t":261,"ev":"arrive","ticket":13,"tenant":1,"in_flight":1,"queued":0}
{"t":261,"ev":"queue","ticket":13,"tenant":1,"in_flight":1,"queued":1}
{"t":282,"ev":"arrive","ticket":14,"tenant":0,"in_flight":1,"queued":1}
{"t":282,"ev":"reject","ticket":14,"tenant":0,"in_flight":1,"queued":1,"reason":"queue_full"}
{"t":289,"ev":"arrive","ticket":15,"tenant":1,"in_flight":1,"queued":1}
{"t":289,"ev":"reject","ticket":15,"tenant":1,"in_flight":1,"queued":1,"reason":"queue_full"}
{"t":310,"ev":"arrive","ticket":16,"tenant":0,"in_flight":1,"queued":1}
{"t":310,"ev":"reject","ticket":16,"tenant":0,"in_flight":1,"queued":1,"reason":"queue_full"}
{"t":313,"ev":"arrive","ticket":17,"tenant":1,"in_flight":1,"queued":1}
{"t":313,"ev":"reject","ticket":17,"tenant":1,"in_flight":1,"queued":1,"reason":"queue_full"}
{"t":331,"ev":"release","ticket":9,"tenant":1,"in_flight":0,"queued":1}
{"t":331,"ev":"grant","ticket":13,"tenant":1,"in_flight":1,"queued":0}
{"t":352,"ev":"arrive","ticket":18,"tenant":0,"in_flight":1,"queued":0}
{"t":352,"ev":"queue","ticket":18,"tenant":0,"in_flight":1,"queued":1}
{"t":378,"ev":"arrive","ticket":19,"tenant":1,"in_flight":1,"queued":1}
{"t":378,"ev":"reject","ticket":19,"tenant":1,"in_flight":1,"queued":1,"reason":"queue_full"}
{"t":391,"ev":"arrive","ticket":20,"tenant":0,"in_flight":1,"queued":1}
{"t":391,"ev":"reject","ticket":20,"tenant":0,"in_flight":1,"queued":1,"reason":"queue_full"}
{"t":400,"ev":"arrive","ticket":21,"tenant":1,"in_flight":1,"queued":1}
{"t":400,"ev":"reject","ticket":21,"tenant":1,"in_flight":1,"queued":1,"reason":"queue_full"}
{"t":428,"ev":"arrive","ticket":22,"tenant":0,"in_flight":1,"queued":1}
{"t":428,"ev":"reject","ticket":22,"tenant":0,"in_flight":1,"queued":1,"reason":"queue_full"}
{"t":436,"ev":"release","ticket":13,"tenant":1,"in_flight":0,"queued":1}
{"t":436,"ev":"grant","ticket":18,"tenant":0,"in_flight":1,"queued":0}
{"t":440,"ev":"release","ticket":18,"tenant":0,"in_flight":0,"queued":0}
{"t":447,"ev":"arrive","ticket":23,"tenant":1,"in_flight":0,"queued":0}
{"t":447,"ev":"grant","ticket":23,"tenant":1,"in_flight":1,"queued":0}
{"t":460,"ev":"arrive","ticket":24,"tenant":0,"in_flight":1,"queued":0}
{"t":460,"ev":"queue","ticket":24,"tenant":0,"in_flight":1,"queued":1}
{"t":461,"ev":"release","ticket":23,"tenant":1,"in_flight":0,"queued":1}
{"t":461,"ev":"grant","ticket":24,"tenant":0,"in_flight":1,"queued":0}
{"t":515,"ev":"release","ticket":24,"tenant":0,"in_flight":0,"queued":0}
"##;
    let scenario = scenario(21, 25, 2, 1, 1, None, 20, 60);
    assert_eq!(trace_of(&scenario), expected);
}

// DESIGN.md gate run rules; expected trace computed independently from the simulation rules in docs/DESIGN.md.
#[test]
fn gate_run_a_limit_at_or_above_the_requests_never_queues_matches_the_expected_trace() {
    let expected = r##"{"t":6,"ev":"arrive","ticket":0,"tenant":0,"in_flight":0,"queued":0}
{"t":6,"ev":"grant","ticket":0,"tenant":0,"in_flight":1,"queued":0}
{"t":9,"ev":"arrive","ticket":1,"tenant":1,"in_flight":1,"queued":0}
{"t":9,"ev":"grant","ticket":1,"tenant":1,"in_flight":2,"queued":0}
{"t":18,"ev":"arrive","ticket":2,"tenant":2,"in_flight":2,"queued":0}
{"t":18,"ev":"grant","ticket":2,"tenant":2,"in_flight":3,"queued":0}
{"t":20,"ev":"arrive","ticket":3,"tenant":0,"in_flight":3,"queued":0}
{"t":20,"ev":"grant","ticket":3,"tenant":0,"in_flight":4,"queued":0}
{"t":24,"ev":"arrive","ticket":4,"tenant":1,"in_flight":4,"queued":0}
{"t":24,"ev":"grant","ticket":4,"tenant":1,"in_flight":5,"queued":0}
{"t":34,"ev":"arrive","ticket":5,"tenant":2,"in_flight":5,"queued":0}
{"t":34,"ev":"grant","ticket":5,"tenant":2,"in_flight":6,"queued":0}
{"t":41,"ev":"arrive","ticket":6,"tenant":0,"in_flight":6,"queued":0}
{"t":41,"ev":"grant","ticket":6,"tenant":0,"in_flight":7,"queued":0}
{"t":48,"ev":"arrive","ticket":7,"tenant":1,"in_flight":7,"queued":0}
{"t":48,"ev":"grant","ticket":7,"tenant":1,"in_flight":8,"queued":0}
{"t":176,"ev":"release","ticket":6,"tenant":0,"in_flight":7,"queued":0}
{"t":305,"ev":"release","ticket":4,"tenant":1,"in_flight":6,"queued":0}
{"t":418,"ev":"release","ticket":7,"tenant":1,"in_flight":5,"queued":0}
{"t":607,"ev":"release","ticket":2,"tenant":2,"in_flight":4,"queued":0}
{"t":1388,"ev":"release","ticket":5,"tenant":2,"in_flight":3,"queued":0}
{"t":1522,"ev":"release","ticket":3,"tenant":0,"in_flight":2,"queued":0}
{"t":1819,"ev":"release","ticket":1,"tenant":1,"in_flight":1,"queued":0}
{"t":1885,"ev":"release","ticket":0,"tenant":0,"in_flight":0,"queued":0}
"##;
    let scenario = scenario(12, 8, 3, 8, 3, Some(500), 5, 1000);
    assert_eq!(trace_of(&scenario), expected);
}

// DESIGN.md gate run rules; expected trace computed independently from the simulation rules in docs/DESIGN.md.
#[test]
fn gate_run_the_largest_queue_capacity_never_rejects_for_a_full_queue_matches_the_expected_trace() {
    let expected = r##"{"t":39,"ev":"arrive","ticket":0,"tenant":0,"in_flight":0,"queued":0}
{"t":39,"ev":"grant","ticket":0,"tenant":0,"in_flight":1,"queued":0}
{"t":63,"ev":"arrive","ticket":1,"tenant":1,"in_flight":1,"queued":0}
{"t":63,"ev":"queue","ticket":1,"tenant":1,"in_flight":1,"queued":1}
{"t":82,"ev":"arrive","ticket":2,"tenant":0,"in_flight":1,"queued":1}
{"t":82,"ev":"queue","ticket":2,"tenant":0,"in_flight":1,"queued":2}
{"t":181,"ev":"arrive","ticket":3,"tenant":1,"in_flight":1,"queued":2}
{"t":181,"ev":"queue","ticket":3,"tenant":1,"in_flight":1,"queued":3}
{"t":224,"ev":"arrive","ticket":4,"tenant":0,"in_flight":1,"queued":3}
{"t":224,"ev":"queue","ticket":4,"tenant":0,"in_flight":1,"queued":4}
{"t":270,"ev":"arrive","ticket":5,"tenant":1,"in_flight":1,"queued":4}
{"t":270,"ev":"queue","ticket":5,"tenant":1,"in_flight":1,"queued":5}
{"t":340,"ev":"release","ticket":0,"tenant":0,"in_flight":0,"queued":5}
{"t":340,"ev":"grant","ticket":1,"tenant":1,"in_flight":1,"queued":4}
{"t":358,"ev":"arrive","ticket":6,"tenant":0,"in_flight":1,"queued":4}
{"t":358,"ev":"queue","ticket":6,"tenant":0,"in_flight":1,"queued":5}
{"t":380,"ev":"release","ticket":1,"tenant":1,"in_flight":0,"queued":5}
{"t":380,"ev":"grant","ticket":2,"tenant":0,"in_flight":1,"queued":4}
{"t":454,"ev":"arrive","ticket":7,"tenant":1,"in_flight":1,"queued":4}
{"t":454,"ev":"queue","ticket":7,"tenant":1,"in_flight":1,"queued":5}
{"t":533,"ev":"release","ticket":2,"tenant":0,"in_flight":0,"queued":5}
{"t":533,"ev":"grant","ticket":3,"tenant":1,"in_flight":1,"queued":4}
{"t":538,"ev":"arrive","ticket":8,"tenant":0,"in_flight":1,"queued":4}
{"t":538,"ev":"queue","ticket":8,"tenant":0,"in_flight":1,"queued":5}
{"t":554,"ev":"arrive","ticket":9,"tenant":1,"in_flight":1,"queued":5}
{"t":554,"ev":"queue","ticket":9,"tenant":1,"in_flight":1,"queued":6}
{"t":566,"ev":"arrive","ticket":10,"tenant":0,"in_flight":1,"queued":6}
{"t":566,"ev":"queue","ticket":10,"tenant":0,"in_flight":1,"queued":7}
{"t":585,"ev":"arrive","ticket":11,"tenant":1,"in_flight":1,"queued":7}
{"t":585,"ev":"queue","ticket":11,"tenant":1,"in_flight":1,"queued":8}
{"t":601,"ev":"arrive","ticket":12,"tenant":0,"in_flight":1,"queued":8}
{"t":601,"ev":"queue","ticket":12,"tenant":0,"in_flight":1,"queued":9}
{"t":695,"ev":"arrive","ticket":13,"tenant":1,"in_flight":1,"queued":9}
{"t":695,"ev":"queue","ticket":13,"tenant":1,"in_flight":1,"queued":10}
{"t":738,"ev":"release","ticket":3,"tenant":1,"in_flight":0,"queued":10}
{"t":738,"ev":"grant","ticket":4,"tenant":0,"in_flight":1,"queued":9}
{"t":741,"ev":"arrive","ticket":14,"tenant":0,"in_flight":1,"queued":9}
{"t":741,"ev":"queue","ticket":14,"tenant":0,"in_flight":1,"queued":10}
{"t":769,"ev":"arrive","ticket":15,"tenant":1,"in_flight":1,"queued":10}
{"t":769,"ev":"queue","ticket":15,"tenant":1,"in_flight":1,"queued":11}
{"t":867,"ev":"arrive","ticket":16,"tenant":0,"in_flight":1,"queued":11}
{"t":867,"ev":"queue","ticket":16,"tenant":0,"in_flight":1,"queued":12}
{"t":965,"ev":"arrive","ticket":17,"tenant":1,"in_flight":1,"queued":12}
{"t":965,"ev":"queue","ticket":17,"tenant":1,"in_flight":1,"queued":13}
{"t":980,"ev":"release","ticket":4,"tenant":0,"in_flight":0,"queued":13}
{"t":980,"ev":"grant","ticket":5,"tenant":1,"in_flight":1,"queued":12}
{"t":1003,"ev":"arrive","ticket":18,"tenant":0,"in_flight":1,"queued":12}
{"t":1003,"ev":"queue","ticket":18,"tenant":0,"in_flight":1,"queued":13}
{"t":1016,"ev":"arrive","ticket":19,"tenant":1,"in_flight":1,"queued":13}
{"t":1016,"ev":"queue","ticket":19,"tenant":1,"in_flight":1,"queued":14}
{"t":1035,"ev":"release","ticket":5,"tenant":1,"in_flight":0,"queued":14}
{"t":1035,"ev":"grant","ticket":6,"tenant":0,"in_flight":1,"queued":13}
{"t":1069,"ev":"arrive","ticket":20,"tenant":0,"in_flight":1,"queued":13}
{"t":1069,"ev":"queue","ticket":20,"tenant":0,"in_flight":1,"queued":14}
{"t":1116,"ev":"arrive","ticket":21,"tenant":1,"in_flight":1,"queued":14}
{"t":1116,"ev":"queue","ticket":21,"tenant":1,"in_flight":1,"queued":15}
{"t":1182,"ev":"arrive","ticket":22,"tenant":0,"in_flight":1,"queued":15}
{"t":1182,"ev":"queue","ticket":22,"tenant":0,"in_flight":1,"queued":16}
{"t":1204,"ev":"arrive","ticket":23,"tenant":1,"in_flight":1,"queued":16}
{"t":1204,"ev":"queue","ticket":23,"tenant":1,"in_flight":1,"queued":17}
{"t":1217,"ev":"release","ticket":6,"tenant":0,"in_flight":0,"queued":17}
{"t":1217,"ev":"grant","ticket":7,"tenant":1,"in_flight":1,"queued":16}
{"t":1219,"ev":"arrive","ticket":24,"tenant":0,"in_flight":1,"queued":16}
{"t":1219,"ev":"queue","ticket":24,"tenant":0,"in_flight":1,"queued":17}
{"t":1593,"ev":"release","ticket":7,"tenant":1,"in_flight":0,"queued":17}
{"t":1593,"ev":"grant","ticket":8,"tenant":0,"in_flight":1,"queued":16}
{"t":1778,"ev":"release","ticket":8,"tenant":0,"in_flight":0,"queued":16}
{"t":1778,"ev":"grant","ticket":9,"tenant":1,"in_flight":1,"queued":15}
{"t":1961,"ev":"release","ticket":9,"tenant":1,"in_flight":0,"queued":15}
{"t":1961,"ev":"grant","ticket":10,"tenant":0,"in_flight":1,"queued":14}
{"t":2356,"ev":"release","ticket":10,"tenant":0,"in_flight":0,"queued":14}
{"t":2356,"ev":"grant","ticket":11,"tenant":1,"in_flight":1,"queued":13}
{"t":2707,"ev":"release","ticket":11,"tenant":1,"in_flight":0,"queued":13}
{"t":2707,"ev":"grant","ticket":12,"tenant":0,"in_flight":1,"queued":12}
{"t":2993,"ev":"release","ticket":12,"tenant":0,"in_flight":0,"queued":12}
{"t":2993,"ev":"grant","ticket":13,"tenant":1,"in_flight":1,"queued":11}
{"t":3042,"ev":"release","ticket":13,"tenant":1,"in_flight":0,"queued":11}
{"t":3042,"ev":"grant","ticket":14,"tenant":0,"in_flight":1,"queued":10}
{"t":3380,"ev":"release","ticket":14,"tenant":0,"in_flight":0,"queued":10}
{"t":3380,"ev":"grant","ticket":15,"tenant":1,"in_flight":1,"queued":9}
{"t":3458,"ev":"release","ticket":15,"tenant":1,"in_flight":0,"queued":9}
{"t":3458,"ev":"grant","ticket":16,"tenant":0,"in_flight":1,"queued":8}
{"t":3615,"ev":"release","ticket":16,"tenant":0,"in_flight":0,"queued":8}
{"t":3615,"ev":"grant","ticket":17,"tenant":1,"in_flight":1,"queued":7}
{"t":3899,"ev":"release","ticket":17,"tenant":1,"in_flight":0,"queued":7}
{"t":3899,"ev":"grant","ticket":18,"tenant":0,"in_flight":1,"queued":6}
{"t":4079,"ev":"release","ticket":18,"tenant":0,"in_flight":0,"queued":6}
{"t":4079,"ev":"grant","ticket":19,"tenant":1,"in_flight":1,"queued":5}
{"t":4080,"ev":"release","ticket":19,"tenant":1,"in_flight":0,"queued":5}
{"t":4080,"ev":"grant","ticket":20,"tenant":0,"in_flight":1,"queued":4}
{"t":4332,"ev":"release","ticket":20,"tenant":0,"in_flight":0,"queued":4}
{"t":4332,"ev":"grant","ticket":21,"tenant":1,"in_flight":1,"queued":3}
{"t":4446,"ev":"release","ticket":21,"tenant":1,"in_flight":0,"queued":3}
{"t":4446,"ev":"grant","ticket":22,"tenant":0,"in_flight":1,"queued":2}
{"t":4751,"ev":"release","ticket":22,"tenant":0,"in_flight":0,"queued":2}
{"t":4751,"ev":"grant","ticket":23,"tenant":1,"in_flight":1,"queued":1}
{"t":5143,"ev":"release","ticket":23,"tenant":1,"in_flight":0,"queued":1}
{"t":5143,"ev":"grant","ticket":24,"tenant":0,"in_flight":1,"queued":0}
{"t":5309,"ev":"release","ticket":24,"tenant":0,"in_flight":0,"queued":0}
"##;
    let scenario = scenario(5, 25, 2, 1, 4294967295, None, 50, 200);
    assert_eq!(trace_of(&scenario), expected);
}

// DESIGN.md gate run rules; expected trace computed independently from the simulation rules in docs/DESIGN.md.
#[test]
fn gate_run_the_largest_limit_never_queues_matches_the_expected_trace() {
    let expected = r##"{"t":39,"ev":"arrive","ticket":0,"tenant":0,"in_flight":0,"queued":0}
{"t":39,"ev":"grant","ticket":0,"tenant":0,"in_flight":1,"queued":0}
{"t":63,"ev":"arrive","ticket":1,"tenant":1,"in_flight":1,"queued":0}
{"t":63,"ev":"grant","ticket":1,"tenant":1,"in_flight":2,"queued":0}
{"t":82,"ev":"arrive","ticket":2,"tenant":0,"in_flight":2,"queued":0}
{"t":82,"ev":"grant","ticket":2,"tenant":0,"in_flight":3,"queued":0}
{"t":103,"ev":"release","ticket":1,"tenant":1,"in_flight":2,"queued":0}
{"t":181,"ev":"arrive","ticket":3,"tenant":1,"in_flight":2,"queued":0}
{"t":181,"ev":"grant","ticket":3,"tenant":1,"in_flight":3,"queued":0}
{"t":224,"ev":"arrive","ticket":4,"tenant":0,"in_flight":3,"queued":0}
{"t":224,"ev":"grant","ticket":4,"tenant":0,"in_flight":4,"queued":0}
{"t":235,"ev":"release","ticket":2,"tenant":0,"in_flight":3,"queued":0}
{"t":270,"ev":"arrive","ticket":5,"tenant":1,"in_flight":3,"queued":0}
{"t":270,"ev":"grant","ticket":5,"tenant":1,"in_flight":4,"queued":0}
{"t":325,"ev":"release","ticket":5,"tenant":1,"in_flight":3,"queued":0}
{"t":340,"ev":"release","ticket":0,"tenant":0,"in_flight":2,"queued":0}
{"t":358,"ev":"arrive","ticket":6,"tenant":0,"in_flight":2,"queued":0}
{"t":358,"ev":"grant","ticket":6,"tenant":0,"in_flight":3,"queued":0}
{"t":386,"ev":"release","ticket":3,"tenant":1,"in_flight":2,"queued":0}
{"t":454,"ev":"arrive","ticket":7,"tenant":1,"in_flight":2,"queued":0}
{"t":454,"ev":"grant","ticket":7,"tenant":1,"in_flight":3,"queued":0}
{"t":466,"ev":"release","ticket":4,"tenant":0,"in_flight":2,"queued":0}
{"t":538,"ev":"arrive","ticket":8,"tenant":0,"in_flight":2,"queued":0}
{"t":538,"ev":"grant","ticket":8,"tenant":0,"in_flight":3,"queued":0}
{"t":540,"ev":"release","ticket":6,"tenant":0,"in_flight":2,"queued":0}
{"t":554,"ev":"arrive","ticket":9,"tenant":1,"in_flight":2,"queued":0}
{"t":554,"ev":"grant","ticket":9,"tenant":1,"in_flight":3,"queued":0}
{"t":566,"ev":"arrive","ticket":10,"tenant":0,"in_flight":3,"queued":0}
{"t":566,"ev":"grant","ticket":10,"tenant":0,"in_flight":4,"queued":0}
{"t":585,"ev":"arrive","ticket":11,"tenant":1,"in_flight":4,"queued":0}
{"t":585,"ev":"grant","ticket":11,"tenant":1,"in_flight":5,"queued":0}
{"t":601,"ev":"arrive","ticket":12,"tenant":0,"in_flight":5,"queued":0}
{"t":601,"ev":"grant","ticket":12,"tenant":0,"in_flight":6,"queued":0}
{"t":695,"ev":"arrive","ticket":13,"tenant":1,"in_flight":6,"queued":0}
{"t":695,"ev":"grant","ticket":13,"tenant":1,"in_flight":7,"queued":0}
{"t":723,"ev":"release","ticket":8,"tenant":0,"in_flight":6,"queued":0}
{"t":737,"ev":"release","ticket":9,"tenant":1,"in_flight":5,"queued":0}
{"t":741,"ev":"arrive","ticket":14,"tenant":0,"in_flight":5,"queued":0}
{"t":741,"ev":"grant","ticket":14,"tenant":0,"in_flight":6,"queued":0}
{"t":744,"ev":"release","ticket":13,"tenant":1,"in_flight":5,"queued":0}
{"t":769,"ev":"arrive","ticket":15,"tenant":1,"in_flight":5,"queued":0}
{"t":769,"ev":"grant","ticket":15,"tenant":1,"in_flight":6,"queued":0}
{"t":830,"ev":"release","ticket":7,"tenant":1,"in_flight":5,"queued":0}
{"t":847,"ev":"release","ticket":15,"tenant":1,"in_flight":4,"queued":0}
{"t":867,"ev":"arrive","ticket":16,"tenant":0,"in_flight":4,"queued":0}
{"t":867,"ev":"grant","ticket":16,"tenant":0,"in_flight":5,"queued":0}
{"t":887,"ev":"release","ticket":12,"tenant":0,"in_flight":4,"queued":0}
{"t":936,"ev":"release","ticket":11,"tenant":1,"in_flight":3,"queued":0}
{"t":961,"ev":"release","ticket":10,"tenant":0,"in_flight":2,"queued":0}
{"t":965,"ev":"arrive","ticket":17,"tenant":1,"in_flight":2,"queued":0}
{"t":965,"ev":"grant","ticket":17,"tenant":1,"in_flight":3,"queued":0}
{"t":1003,"ev":"arrive","ticket":18,"tenant":0,"in_flight":3,"queued":0}
{"t":1003,"ev":"grant","ticket":18,"tenant":0,"in_flight":4,"queued":0}
{"t":1016,"ev":"arrive","ticket":19,"tenant":1,"in_flight":4,"queued":0}
{"t":1016,"ev":"grant","ticket":19,"tenant":1,"in_flight":5,"queued":0}
{"t":1017,"ev":"release","ticket":19,"tenant":1,"in_flight":4,"queued":0}
{"t":1024,"ev":"release","ticket":16,"tenant":0,"in_flight":3,"queued":0}
{"t":1069,"ev":"arrive","ticket":20,"tenant":0,"in_flight":3,"queued":0}
{"t":1069,"ev":"grant","ticket":20,"tenant":0,"in_flight":4,"queued":0}
{"t":1079,"ev":"release","ticket":14,"tenant":0,"in_flight":3,"queued":0}
{"t":1116,"ev":"arrive","ticket":21,"tenant":1,"in_flight":3,"queued":0}
{"t":1116,"ev":"grant","ticket":21,"tenant":1,"in_flight":4,"queued":0}
{"t":1182,"ev":"arrive","ticket":22,"tenant":0,"in_flight":4,"queued":0}
{"t":1182,"ev":"grant","ticket":22,"tenant":0,"in_flight":5,"queued":0}
{"t":1183,"ev":"release","ticket":18,"tenant":0,"in_flight":4,"queued":0}
{"t":1204,"ev":"arrive","ticket":23,"tenant":1,"in_flight":4,"queued":0}
{"t":1204,"ev":"grant","ticket":23,"tenant":1,"in_flight":5,"queued":0}
{"t":1219,"ev":"arrive","ticket":24,"tenant":0,"in_flight":5,"queued":0}
{"t":1219,"ev":"grant","ticket":24,"tenant":0,"in_flight":6,"queued":0}
{"t":1230,"ev":"release","ticket":21,"tenant":1,"in_flight":5,"queued":0}
{"t":1249,"ev":"release","ticket":17,"tenant":1,"in_flight":4,"queued":0}
{"t":1321,"ev":"release","ticket":20,"tenant":0,"in_flight":3,"queued":0}
{"t":1385,"ev":"release","ticket":24,"tenant":0,"in_flight":2,"queued":0}
{"t":1487,"ev":"release","ticket":22,"tenant":0,"in_flight":1,"queued":0}
{"t":1596,"ev":"release","ticket":23,"tenant":1,"in_flight":0,"queued":0}
"##;
    let scenario = scenario(5, 25, 2, 4294967295, 3, Some(100), 50, 200);
    assert_eq!(trace_of(&scenario), expected);
}

// DESIGN.md gate run rules; digest and counts computed independently from the simulation rules in docs/DESIGN.md.
#[test]
fn gate_run_a_deep_queue_with_a_timeout_drops_tickets_at_the_head_and_in_the_middle_matches_the_expected_digest()
 {
    let scenario = scenario(33, 50, 4, 2, 20, Some(600), 20, 200);
    let trace = trace_of(&scenario);
    assert_eq!(trace.lines().count(), 146);
    assert_eq!(kinds(&trace), [50, 16, 30, 16, 18, 16]);
    assert_eq!(fnv1a(&trace), 0x0a1eba38144e2f94);
}

// DESIGN.md gate run rules; expected trace computed independently from the simulation rules in docs/DESIGN.md.
#[test]
fn gate_run_more_tenants_than_requests_gives_each_ticket_its_own_tenant_matches_the_expected_trace()
{
    let expected = r##"{"t":28,"ev":"arrive","ticket":0,"tenant":0,"in_flight":0,"queued":0}
{"t":28,"ev":"grant","ticket":0,"tenant":0,"in_flight":1,"queued":0}
{"t":39,"ev":"arrive","ticket":1,"tenant":1,"in_flight":1,"queued":0}
{"t":39,"ev":"queue","ticket":1,"tenant":1,"in_flight":1,"queued":1}
{"t":50,"ev":"arrive","ticket":2,"tenant":2,"in_flight":1,"queued":1}
{"t":50,"ev":"queue","ticket":2,"tenant":2,"in_flight":1,"queued":2}
{"t":59,"ev":"release","ticket":0,"tenant":0,"in_flight":0,"queued":2}
{"t":59,"ev":"grant","ticket":1,"tenant":1,"in_flight":1,"queued":1}
{"t":76,"ev":"arrive","ticket":3,"tenant":3,"in_flight":1,"queued":1}
{"t":76,"ev":"queue","ticket":3,"tenant":3,"in_flight":1,"queued":2}
{"t":91,"ev":"release","ticket":1,"tenant":1,"in_flight":0,"queued":2}
{"t":91,"ev":"grant","ticket":2,"tenant":2,"in_flight":1,"queued":1}
{"t":96,"ev":"release","ticket":2,"tenant":2,"in_flight":0,"queued":1}
{"t":96,"ev":"grant","ticket":3,"tenant":3,"in_flight":1,"queued":0}
{"t":136,"ev":"release","ticket":3,"tenant":3,"in_flight":0,"queued":0}
"##;
    let scenario = scenario(9, 4, 100, 1, 2, None, 20, 20);
    assert_eq!(trace_of(&scenario), expected);
}

// DESIGN.md gate run rules; expected trace computed independently from the simulation rules in docs/DESIGN.md.
#[test]
fn gate_run_the_largest_tenant_count_gives_each_ticket_its_own_tenant_matches_the_expected_trace() {
    let expected = r##"{"t":28,"ev":"arrive","ticket":0,"tenant":0,"in_flight":0,"queued":0}
{"t":28,"ev":"grant","ticket":0,"tenant":0,"in_flight":1,"queued":0}
{"t":39,"ev":"arrive","ticket":1,"tenant":1,"in_flight":1,"queued":0}
{"t":39,"ev":"queue","ticket":1,"tenant":1,"in_flight":1,"queued":1}
{"t":50,"ev":"arrive","ticket":2,"tenant":2,"in_flight":1,"queued":1}
{"t":50,"ev":"queue","ticket":2,"tenant":2,"in_flight":1,"queued":2}
{"t":76,"ev":"arrive","ticket":3,"tenant":3,"in_flight":1,"queued":2}
{"t":76,"ev":"reject","ticket":3,"tenant":3,"in_flight":1,"queued":2,"reason":"queue_full"}
{"t":85,"ev":"arrive","ticket":4,"tenant":4,"in_flight":1,"queued":2}
{"t":85,"ev":"reject","ticket":4,"tenant":4,"in_flight":1,"queued":2,"reason":"queue_full"}
{"t":109,"ev":"arrive","ticket":5,"tenant":5,"in_flight":1,"queued":2}
{"t":109,"ev":"reject","ticket":5,"tenant":5,"in_flight":1,"queued":2,"reason":"queue_full"}
{"t":179,"ev":"release","ticket":0,"tenant":0,"in_flight":0,"queued":2}
{"t":179,"ev":"grant","ticket":1,"tenant":1,"in_flight":1,"queued":1}
{"t":336,"ev":"release","ticket":1,"tenant":1,"in_flight":0,"queued":1}
{"t":336,"ev":"grant","ticket":2,"tenant":2,"in_flight":1,"queued":0}
{"t":359,"ev":"release","ticket":2,"tenant":2,"in_flight":0,"queued":0}
"##;
    let scenario = scenario(9, 6, 18446744073709551615, 1, 2, Some(300), 20, 100);
    assert_eq!(trace_of(&scenario), expected);
}

// DESIGN.md gate run rules; expected trace computed independently from the simulation rules in docs/DESIGN.md.
#[test]
fn gate_run_times_that_saturate_at_the_largest_value_keep_the_documented_order_matches_the_expected_trace()
 {
    let expected = r##"{"t":9271579095833343350,"ev":"arrive","ticket":0,"tenant":0,"in_flight":0,"queued":0}
{"t":9271579095833343350,"ev":"grant","ticket":0,"tenant":0,"in_flight":1,"queued":0}
{"t":10514075541860043927,"ev":"arrive","ticket":1,"tenant":1,"in_flight":1,"queued":0}
{"t":10514075541860043927,"ev":"queue","ticket":1,"tenant":1,"in_flight":1,"queued":1}
{"t":18446744073709551615,"ev":"release","ticket":0,"tenant":0,"in_flight":0,"queued":1}
{"t":18446744073709551615,"ev":"grant","ticket":1,"tenant":1,"in_flight":1,"queued":0}
{"t":18446744073709551615,"ev":"release","ticket":1,"tenant":1,"in_flight":0,"queued":0}
{"t":18446744073709551615,"ev":"arrive","ticket":2,"tenant":2,"in_flight":0,"queued":0}
{"t":18446744073709551615,"ev":"grant","ticket":2,"tenant":2,"in_flight":1,"queued":0}
{"t":18446744073709551615,"ev":"release","ticket":2,"tenant":2,"in_flight":0,"queued":0}
"##;
    let scenario = scenario(
        1377,
        3,
        3,
        1,
        1,
        None,
        9223372036854775807,
        18446744073709551615,
    );
    assert_eq!(trace_of(&scenario), expected);
}

// DESIGN.md gate run rules; expected trace computed independently from the simulation rules in docs/DESIGN.md.
#[test]
fn gate_run_a_saturated_arrival_with_a_finite_timeout_is_rejected_as_expired_matches_the_expected_trace()
 {
    let expected = r##"{"t":10451216379200822464,"ev":"arrive","ticket":0,"tenant":0,"in_flight":0,"queued":0}
{"t":10451216379200822464,"ev":"grant","ticket":0,"tenant":0,"in_flight":1,"queued":0}
{"t":18446744073709551615,"ev":"release","ticket":0,"tenant":0,"in_flight":0,"queued":0}
{"t":18446744073709551615,"ev":"arrive","ticket":1,"tenant":1,"in_flight":0,"queued":0}
{"t":18446744073709551615,"ev":"reject","ticket":1,"tenant":1,"in_flight":0,"queued":0,"reason":"expired"}
{"t":18446744073709551615,"ev":"arrive","ticket":2,"tenant":2,"in_flight":0,"queued":0}
{"t":18446744073709551615,"ev":"reject","ticket":2,"tenant":2,"in_flight":0,"queued":0,"reason":"expired"}
"##;
    let scenario = scenario(
        1,
        3,
        3,
        1,
        1,
        Some(5),
        9223372036854775807,
        18446744073709551615,
    );
    assert_eq!(trace_of(&scenario), expected);
}

// DESIGN.md gate run rules; expected trace computed independently from the simulation rules in docs/DESIGN.md.
#[test]
fn gate_run_the_largest_timeout_saturates_every_deadline_matches_the_expected_trace() {
    let expected = r##"{"t":39,"ev":"arrive","ticket":0,"tenant":0,"in_flight":0,"queued":0}
{"t":39,"ev":"grant","ticket":0,"tenant":0,"in_flight":1,"queued":0}
{"t":63,"ev":"arrive","ticket":1,"tenant":1,"in_flight":1,"queued":0}
{"t":63,"ev":"queue","ticket":1,"tenant":1,"in_flight":1,"queued":1}
{"t":82,"ev":"arrive","ticket":2,"tenant":0,"in_flight":1,"queued":1}
{"t":82,"ev":"queue","ticket":2,"tenant":0,"in_flight":1,"queued":2}
{"t":181,"ev":"arrive","ticket":3,"tenant":1,"in_flight":1,"queued":2}
{"t":181,"ev":"queue","ticket":3,"tenant":1,"in_flight":1,"queued":3}
{"t":224,"ev":"arrive","ticket":4,"tenant":0,"in_flight":1,"queued":3}
{"t":224,"ev":"queue","ticket":4,"tenant":0,"in_flight":1,"queued":4}
{"t":270,"ev":"arrive","ticket":5,"tenant":1,"in_flight":1,"queued":4}
{"t":270,"ev":"queue","ticket":5,"tenant":1,"in_flight":1,"queued":5}
{"t":340,"ev":"release","ticket":0,"tenant":0,"in_flight":0,"queued":5}
{"t":340,"ev":"grant","ticket":1,"tenant":1,"in_flight":1,"queued":4}
{"t":358,"ev":"arrive","ticket":6,"tenant":0,"in_flight":1,"queued":4}
{"t":358,"ev":"queue","ticket":6,"tenant":0,"in_flight":1,"queued":5}
{"t":380,"ev":"release","ticket":1,"tenant":1,"in_flight":0,"queued":5}
{"t":380,"ev":"grant","ticket":2,"tenant":0,"in_flight":1,"queued":4}
{"t":454,"ev":"arrive","ticket":7,"tenant":1,"in_flight":1,"queued":4}
{"t":454,"ev":"queue","ticket":7,"tenant":1,"in_flight":1,"queued":5}
{"t":533,"ev":"release","ticket":2,"tenant":0,"in_flight":0,"queued":5}
{"t":533,"ev":"grant","ticket":3,"tenant":1,"in_flight":1,"queued":4}
{"t":538,"ev":"arrive","ticket":8,"tenant":0,"in_flight":1,"queued":4}
{"t":538,"ev":"queue","ticket":8,"tenant":0,"in_flight":1,"queued":5}
{"t":554,"ev":"arrive","ticket":9,"tenant":1,"in_flight":1,"queued":5}
{"t":554,"ev":"queue","ticket":9,"tenant":1,"in_flight":1,"queued":6}
{"t":566,"ev":"arrive","ticket":10,"tenant":0,"in_flight":1,"queued":6}
{"t":566,"ev":"reject","ticket":10,"tenant":0,"in_flight":1,"queued":6,"reason":"queue_full"}
{"t":585,"ev":"arrive","ticket":11,"tenant":1,"in_flight":1,"queued":6}
{"t":585,"ev":"reject","ticket":11,"tenant":1,"in_flight":1,"queued":6,"reason":"queue_full"}
{"t":738,"ev":"release","ticket":3,"tenant":1,"in_flight":0,"queued":6}
{"t":738,"ev":"grant","ticket":4,"tenant":0,"in_flight":1,"queued":5}
{"t":980,"ev":"release","ticket":4,"tenant":0,"in_flight":0,"queued":5}
{"t":980,"ev":"grant","ticket":5,"tenant":1,"in_flight":1,"queued":4}
{"t":1035,"ev":"release","ticket":5,"tenant":1,"in_flight":0,"queued":4}
{"t":1035,"ev":"grant","ticket":6,"tenant":0,"in_flight":1,"queued":3}
{"t":1217,"ev":"release","ticket":6,"tenant":0,"in_flight":0,"queued":3}
{"t":1217,"ev":"grant","ticket":7,"tenant":1,"in_flight":1,"queued":2}
{"t":1593,"ev":"release","ticket":7,"tenant":1,"in_flight":0,"queued":2}
{"t":1593,"ev":"grant","ticket":8,"tenant":0,"in_flight":1,"queued":1}
{"t":1778,"ev":"release","ticket":8,"tenant":0,"in_flight":0,"queued":1}
{"t":1778,"ev":"grant","ticket":9,"tenant":1,"in_flight":1,"queued":0}
{"t":1961,"ev":"release","ticket":9,"tenant":1,"in_flight":0,"queued":0}
"##;
    let scenario = scenario(5, 12, 2, 1, 6, Some(18446744073709551615), 50, 200);
    assert_eq!(trace_of(&scenario), expected);
}

// DESIGN.md gate run rules; expected trace computed independently from the simulation rules in docs/DESIGN.md.
#[test]
fn gate_run_saturated_service_times_let_queued_tickets_expire_while_a_request_runs_matches_the_expected_trace()
 {
    let expected = r##"{"t":9,"ev":"arrive","ticket":0,"tenant":0,"in_flight":0,"queued":0}
{"t":9,"ev":"grant","ticket":0,"tenant":0,"in_flight":1,"queued":0}
{"t":27,"ev":"arrive","ticket":1,"tenant":0,"in_flight":1,"queued":0}
{"t":27,"ev":"queue","ticket":1,"tenant":0,"in_flight":1,"queued":1}
{"t":35,"ev":"arrive","ticket":2,"tenant":0,"in_flight":1,"queued":1}
{"t":35,"ev":"queue","ticket":2,"tenant":0,"in_flight":1,"queued":2}
{"t":54,"ev":"arrive","ticket":3,"tenant":0,"in_flight":1,"queued":2}
{"t":54,"ev":"queue","ticket":3,"tenant":0,"in_flight":1,"queued":3}
{"t":58,"ev":"arrive","ticket":4,"tenant":0,"in_flight":1,"queued":3}
{"t":58,"ev":"reject","ticket":4,"tenant":0,"in_flight":1,"queued":3,"reason":"queue_full"}
{"t":74,"ev":"arrive","ticket":5,"tenant":0,"in_flight":1,"queued":3}
{"t":74,"ev":"reject","ticket":5,"tenant":0,"in_flight":1,"queued":3,"reason":"queue_full"}
{"t":85,"ev":"arrive","ticket":6,"tenant":0,"in_flight":1,"queued":3}
{"t":85,"ev":"reject","ticket":6,"tenant":0,"in_flight":1,"queued":3,"reason":"queue_full"}
{"t":103,"ev":"arrive","ticket":7,"tenant":0,"in_flight":1,"queued":3}
{"t":103,"ev":"reject","ticket":7,"tenant":0,"in_flight":1,"queued":3,"reason":"queue_full"}
{"t":527,"ev":"expire","ticket":1,"tenant":0,"in_flight":1,"queued":2}
{"t":535,"ev":"expire","ticket":2,"tenant":0,"in_flight":1,"queued":1}
{"t":554,"ev":"expire","ticket":3,"tenant":0,"in_flight":1,"queued":0}
{"t":16462000697783136313,"ev":"release","ticket":0,"tenant":0,"in_flight":0,"queued":0}
"##;
    let scenario = scenario(4, 8, 1, 1, 3, Some(500), 10, 18446744073709551615);
    assert_eq!(trace_of(&scenario), expected);
}

// DESIGN.md gate run rules; expected trace computed independently from the simulation rules in docs/DESIGN.md.
#[test]
fn gate_run_at_the_largest_time_a_release_drops_the_queued_tickets_whose_deadline_is_that_time_matches_the_expected_trace()
 {
    let expected = r##"{"t":177,"ev":"arrive","ticket":0,"tenant":0,"in_flight":0,"queued":0}
{"t":177,"ev":"grant","ticket":0,"tenant":0,"in_flight":1,"queued":0}
{"t":183,"ev":"arrive","ticket":1,"tenant":1,"in_flight":1,"queued":0}
{"t":183,"ev":"queue","ticket":1,"tenant":1,"in_flight":1,"queued":1}
{"t":205,"ev":"arrive","ticket":2,"tenant":0,"in_flight":1,"queued":1}
{"t":205,"ev":"queue","ticket":2,"tenant":0,"in_flight":1,"queued":2}
{"t":240,"ev":"arrive","ticket":3,"tenant":1,"in_flight":1,"queued":2}
{"t":240,"ev":"queue","ticket":3,"tenant":1,"in_flight":1,"queued":3}
{"t":290,"ev":"arrive","ticket":4,"tenant":0,"in_flight":1,"queued":3}
{"t":290,"ev":"reject","ticket":4,"tenant":0,"in_flight":1,"queued":3,"reason":"queue_full"}
{"t":7960286522194355877,"ev":"release","ticket":0,"tenant":0,"in_flight":0,"queued":3}
{"t":7960286522194355877,"ev":"grant","ticket":1,"tenant":1,"in_flight":1,"queued":2}
{"t":18446744073709551615,"ev":"release","ticket":1,"tenant":1,"in_flight":0,"queued":2}
{"t":18446744073709551615,"ev":"expire","ticket":2,"tenant":0,"in_flight":0,"queued":1}
{"t":18446744073709551615,"ev":"expire","ticket":3,"tenant":1,"in_flight":0,"queued":0}
"##;
    let scenario = scenario(
        0,
        5,
        2,
        1,
        3,
        Some(18446744073709551615),
        100,
        18446744073709551615,
    );
    assert_eq!(trace_of(&scenario), expected);
}

// DESIGN.md gate run rules; expected trace computed independently from the simulation rules in docs/DESIGN.md.
#[test]
fn gate_run_saturated_gaps_with_a_zero_timeout_reject_everything_as_expired_matches_the_expected_trace()
 {
    let expected = r##"{"t":10905525725756348110,"ev":"arrive","ticket":0,"tenant":0,"in_flight":0,"queued":0}
{"t":10905525725756348110,"ev":"reject","ticket":0,"tenant":0,"in_flight":0,"queued":0,"reason":"expired"}
{"t":18446744073709551615,"ev":"arrive","ticket":1,"tenant":0,"in_flight":0,"queued":0}
{"t":18446744073709551615,"ev":"reject","ticket":1,"tenant":0,"in_flight":0,"queued":0,"reason":"expired"}
{"t":18446744073709551615,"ev":"arrive","ticket":2,"tenant":0,"in_flight":0,"queued":0}
{"t":18446744073709551615,"ev":"reject","ticket":2,"tenant":0,"in_flight":0,"queued":0,"reason":"expired"}
{"t":18446744073709551615,"ev":"arrive","ticket":3,"tenant":0,"in_flight":0,"queued":0}
{"t":18446744073709551615,"ev":"reject","ticket":3,"tenant":0,"in_flight":0,"queued":0,"reason":"expired"}
{"t":18446744073709551615,"ev":"arrive","ticket":4,"tenant":0,"in_flight":0,"queued":0}
{"t":18446744073709551615,"ev":"reject","ticket":4,"tenant":0,"in_flight":0,"queued":0,"reason":"expired"}
"##;
    let scenario = scenario(2, 5, 1, 1, 2, Some(0), 18446744073709551615, 10);
    assert_eq!(trace_of(&scenario), expected);
}

// DESIGN.md gate run rules; expected trace computed independently from the simulation rules in docs/DESIGN.md.
#[test]
fn gate_run_a_zero_queue_and_a_zero_timeout_are_a_valid_scenario_matches_the_expected_trace() {
    let expected = r##"{"t":12,"ev":"arrive","ticket":0,"tenant":0,"in_flight":0,"queued":0}
{"t":12,"ev":"reject","ticket":0,"tenant":0,"in_flight":0,"queued":0,"reason":"expired"}
{"t":32,"ev":"arrive","ticket":1,"tenant":0,"in_flight":0,"queued":0}
{"t":32,"ev":"reject","ticket":1,"tenant":0,"in_flight":0,"queued":0,"reason":"expired"}
{"t":41,"ev":"arrive","ticket":2,"tenant":0,"in_flight":0,"queued":0}
{"t":41,"ev":"reject","ticket":2,"tenant":0,"in_flight":0,"queued":0,"reason":"expired"}
"##;
    let scenario = scenario(1, 3, 1, 1, 0, Some(0), 10, 10);
    assert_eq!(trace_of(&scenario), expected);
}

// DESIGN.md gate run rules; expected trace computed independently from the simulation rules in docs/DESIGN.md.
#[test]
fn gate_run_events_of_one_release_carry_the_state_after_each_single_event_matches_the_expected_trace()
 {
    let expected = r##"{"t":2,"ev":"arrive","ticket":0,"tenant":0,"in_flight":0,"queued":0}
{"t":2,"ev":"grant","ticket":0,"tenant":0,"in_flight":1,"queued":0}
{"t":4,"ev":"arrive","ticket":1,"tenant":1,"in_flight":1,"queued":0}
{"t":4,"ev":"queue","ticket":1,"tenant":1,"in_flight":1,"queued":1}
{"t":17,"ev":"release","ticket":0,"tenant":0,"in_flight":0,"queued":1}
{"t":17,"ev":"grant","ticket":1,"tenant":1,"in_flight":1,"queued":0}
{"t":28,"ev":"release","ticket":1,"tenant":1,"in_flight":0,"queued":0}
"##;
    let scenario = scenario(31, 2, 2, 1, 1, None, 1, 10);
    assert_eq!(trace_of(&scenario), expected);
}

// DESIGN.md gate run rules; expected trace computed independently from the simulation rules in docs/DESIGN.md.
#[test]
fn gate_run_arrivals_at_the_same_saturated_time_are_handled_one_per_round_matches_the_expected_trace()
 {
    let expected = r##"{"t":10451216379200822464,"ev":"arrive","ticket":0,"tenant":0,"in_flight":0,"queued":0}
{"t":10451216379200822464,"ev":"grant","ticket":0,"tenant":0,"in_flight":1,"queued":0}
{"t":18446744073709551615,"ev":"release","ticket":0,"tenant":0,"in_flight":0,"queued":0}
{"t":18446744073709551615,"ev":"arrive","ticket":1,"tenant":1,"in_flight":0,"queued":0}
{"t":18446744073709551615,"ev":"grant","ticket":1,"tenant":1,"in_flight":1,"queued":0}
{"t":18446744073709551615,"ev":"release","ticket":1,"tenant":1,"in_flight":0,"queued":0}
{"t":18446744073709551615,"ev":"arrive","ticket":2,"tenant":2,"in_flight":0,"queued":0}
{"t":18446744073709551615,"ev":"grant","ticket":2,"tenant":2,"in_flight":1,"queued":0}
{"t":18446744073709551615,"ev":"release","ticket":2,"tenant":2,"in_flight":0,"queued":0}
"##;
    let scenario = scenario(
        1,
        3,
        3,
        1,
        1,
        None,
        9223372036854775807,
        18446744073709551615,
    );
    assert_eq!(trace_of(&scenario), expected);
}

// DESIGN.md gate run rules; expected trace computed independently from the simulation rules in docs/DESIGN.md.
#[test]
fn gate_run_a_request_granted_from_the_queue_that_completes_at_once_is_released_before_the_arrival_matches_the_expected_trace()
 {
    let expected = r##"{"t":9271579095833343350,"ev":"arrive","ticket":0,"tenant":0,"in_flight":0,"queued":0}
{"t":9271579095833343350,"ev":"grant","ticket":0,"tenant":0,"in_flight":1,"queued":0}
{"t":10514075541860043927,"ev":"arrive","ticket":1,"tenant":1,"in_flight":1,"queued":0}
{"t":10514075541860043927,"ev":"queue","ticket":1,"tenant":1,"in_flight":1,"queued":1}
{"t":18446744073709551615,"ev":"release","ticket":0,"tenant":0,"in_flight":0,"queued":1}
{"t":18446744073709551615,"ev":"grant","ticket":1,"tenant":1,"in_flight":1,"queued":0}
{"t":18446744073709551615,"ev":"release","ticket":1,"tenant":1,"in_flight":0,"queued":0}
{"t":18446744073709551615,"ev":"arrive","ticket":2,"tenant":2,"in_flight":0,"queued":0}
{"t":18446744073709551615,"ev":"grant","ticket":2,"tenant":2,"in_flight":1,"queued":0}
{"t":18446744073709551615,"ev":"release","ticket":2,"tenant":2,"in_flight":0,"queued":0}
"##;
    let scenario = scenario(
        1377,
        3,
        3,
        1,
        1,
        None,
        9223372036854775807,
        18446744073709551615,
    );
    assert_eq!(trace_of(&scenario), expected);
}

// DESIGN.md gate run rules; digests and counts computed independently from the simulation rules in docs/DESIGN.md.
#[test]
fn random_gate_scenarios_match_the_expected_digests() {
    // (name, scenario, lines, [arrive, grant, queue, reject, expire, release], FNV-1a of the trace)
    #[allow(clippy::type_complexity)]
    let cases: Vec<(&str, GateScenario, usize, [usize; 6], u64)> = vec![
        (
            "random 0",
            scenario(31226, 11, 5, 3, 3, Some(1), 100, 100),
            33,
            [11, 10, 1, 0, 1, 10],
            0xf423211a9d429073,
        ),
        (
            "random 1",
            scenario(99532, 19, 5, 1, 3, None, 100, 250),
            60,
            [19, 12, 10, 7, 0, 12],
            0x09d4ca47f29fedab,
        ),
        (
            "random 2",
            scenario(7963, 19, 2, 2, 3, Some(50), 300, 9223372036854775807),
            57,
            [19, 2, 17, 0, 17, 2],
            0xe3ca6099dde714b3,
        ),
        (
            "random 3",
            scenario(466, 21, 1, 1, 5, None, 2, 1),
            68,
            [21, 21, 5, 0, 0, 21],
            0xb44300ffbcbd5914,
        ),
        (
            "random 4",
            scenario(66490, 30, 3, 1, 0, None, 2, 600),
            61,
            [30, 1, 0, 29, 0, 1],
            0x1c231c09cf5077ed,
        ),
        (
            "random 5",
            scenario(77924, 41, 2, 4, 1, Some(50), 100, 100),
            123,
            [41, 41, 0, 0, 0, 41],
            0xb004f9eec6ed2405,
        ),
        (
            "random 6",
            scenario(47898, 49, 3, 2, 0, Some(1), 50, 2),
            147,
            [49, 49, 0, 0, 0, 49],
            0x7718ba2de37660f0,
        ),
        (
            "random 7",
            scenario(26491, 56, 2, 4, 1, None, 10, 100),
            164,
            [56, 30, 22, 26, 0, 30],
            0x03665c10b9be0b57,
        ),
        (
            "random 8",
            scenario(5675, 19, 3, 1, 0, Some(1000), 100, 10),
            57,
            [19, 19, 0, 0, 0, 19],
            0x74c4feb2b404551f,
        ),
        (
            "random 9",
            scenario(95893, 34, 2, 4, 0, Some(0), 1, 9223372036854775807),
            68,
            [34, 0, 0, 34, 0, 0],
            0x5813744aaa8f1b4d,
        ),
        (
            "random 10",
            scenario(4946, 46, 2, 2, 3, Some(1), 1, 250),
            138,
            [46, 2, 44, 0, 44, 2],
            0xd7b3847c093dddce,
        ),
        (
            "random 11",
            scenario(68488, 13, 1, 2, 0, Some(0), 1, 18446744073709551615),
            26,
            [13, 0, 0, 13, 0, 0],
            0x69e456e2a6b5a45e,
        ),
        (
            "random 12",
            scenario(37390, 45, 1, 3, 1, Some(50), 10, 1),
            135,
            [45, 45, 0, 0, 0, 45],
            0x6a1ebdb4bed08967,
        ),
        (
            "random 13",
            scenario(81185, 8, 5, 1, 2, None, 1, 100),
            23,
            [8, 4, 3, 4, 0, 4],
            0x7a4b7403f939f614,
        ),
        (
            "random 14",
            scenario(98353, 7, 1, 1, 1, None, 100, 18446744073709551615),
            17,
            [7, 2, 1, 5, 0, 2],
            0xeb0ec5f3ca6d7f6a,
        ),
        (
            "random 15",
            scenario(94348, 9, 3, 3, 3, Some(1), 1, 100),
            27,
            [9, 3, 6, 0, 6, 3],
            0xe21d8285ab849374,
        ),
        (
            "random 16",
            scenario(50463, 20, 1, 3, 1, Some(5), 10, 600),
            55,
            [20, 3, 12, 5, 12, 3],
            0x0e4cac593318f9f9,
        ),
        (
            "random 17",
            scenario(37009, 43, 3, 4, 5, Some(1), 50, 2),
            129,
            [43, 43, 0, 0, 0, 43],
            0xb37edca855381f0a,
        ),
        (
            "random 18",
            scenario(95854, 55, 5, 3, 2, Some(300), 6148914691236517205, 1),
            112,
            [55, 2, 0, 53, 0, 2],
            0x47c42d163acce1d5,
        ),
        (
            "random 19",
            scenario(50865, 51, 1, 3, 3, Some(1), 2, 2),
            153,
            [51, 51, 0, 0, 0, 51],
            0x5a263ddc11be021d,
        ),
        (
            "random 20",
            scenario(57756, 42, 3, 1, 3, Some(0), 50, 2),
            84,
            [42, 0, 0, 42, 0, 0],
            0xdc1cc09c064ad8c9,
        ),
        (
            "random 21",
            scenario(89688, 38, 5, 4, 10, Some(0), 1, 10),
            76,
            [38, 0, 0, 38, 0, 0],
            0xf816c783c6f00a87,
        ),
        (
            "random 22",
            scenario(4380, 50, 3, 3, 10, None, 100, 600),
            164,
            [50, 34, 30, 16, 0, 34],
            0xd672f65e70c3e7d4,
        ),
        (
            "random 23",
            scenario(10769, 36, 5, 1, 10, Some(5), 2, 600),
            108,
            [36, 1, 35, 0, 35, 1],
            0xb8709592b24a075a,
        ),
        (
            "random 24",
            scenario(1654, 19, 5, 4, 3, None, 50, 10),
            57,
            [19, 19, 0, 0, 0, 19],
            0xbed60b1ec1d3259a,
        ),
        (
            "random 25",
            scenario(63427, 11, 5, 3, 3, Some(0), 100, 2),
            22,
            [11, 0, 0, 11, 0, 0],
            0x1b6e725035953fc2,
        ),
        (
            "random 26",
            scenario(7439, 10, 2, 1, 1, Some(5), 100, 18446744073709551615),
            30,
            [10, 1, 9, 0, 9, 1],
            0x596b0ce7a7f061d0,
        ),
        (
            "random 27",
            scenario(35138, 59, 1, 1, 3, Some(0), 10, 100),
            118,
            [59, 0, 0, 59, 0, 0],
            0x367e3f7807c609ae,
        ),
        (
            "random 28",
            scenario(46300, 49, 2, 2, 5, None, 2, 2),
            155,
            [49, 49, 8, 0, 0, 49],
            0x0331783987c1308d,
        ),
        (
            "random 29",
            scenario(99057, 32, 3, 4, 2, Some(1), 300, 100),
            96,
            [32, 32, 0, 0, 0, 32],
            0x1466f8abf0b1eb51,
        ),
        (
            "random 30",
            scenario(56884, 3, 3, 3, 2, Some(50), 2, 10),
            9,
            [3, 3, 0, 0, 0, 3],
            0x8b9760a06dc625ac,
        ),
        (
            "random 31",
            scenario(56832, 4, 5, 3, 2, Some(1000), 50, 600),
            12,
            [4, 4, 0, 0, 0, 4],
            0x6920ab7a6c6f316d,
        ),
        (
            "random 32",
            scenario(44012, 4, 3, 1, 2, Some(1), 300, 1),
            12,
            [4, 4, 0, 0, 0, 4],
            0x78dd065f9c83a4d7,
        ),
        (
            "random 33",
            scenario(40860, 12, 3, 2, 3, Some(5), 10, 100),
            36,
            [12, 4, 8, 0, 8, 4],
            0x4c3e2d3304532263,
        ),
        (
            "random 34",
            scenario(80336, 34, 5, 1, 3, Some(5), 300, 10),
            102,
            [34, 33, 1, 0, 1, 33],
            0x687ed1c170846249,
        ),
        (
            "random 35",
            scenario(5151, 44, 5, 2, 2, None, 100, 2),
            132,
            [44, 44, 0, 0, 0, 44],
            0x1cc2534478014bd0,
        ),
        (
            "random 36",
            scenario(71607, 40, 1, 1, 0, None, 9223372036854775807, 250),
            120,
            [40, 40, 0, 0, 0, 40],
            0xb8cce141bcbcd3f6,
        ),
        (
            "random 37",
            scenario(6687, 58, 1, 1, 3, Some(0), 300, 9223372036854775807),
            116,
            [58, 0, 0, 58, 0, 0],
            0x52fe9e820c4fb815,
        ),
        (
            "random 38",
            scenario(59883, 27, 2, 3, 1, Some(0), 300, 600),
            54,
            [27, 0, 0, 27, 0, 0],
            0xede8e535a422b6f0,
        ),
        (
            "random 39",
            scenario(88362, 50, 3, 1, 3, Some(0), 100, 100),
            100,
            [50, 0, 0, 50, 0, 0],
            0x5f0140d5ff809719,
        ),
        (
            "random 40",
            scenario(25626, 16, 2, 1, 10, Some(1000), 2, 600),
            44,
            [16, 2, 10, 5, 9, 2],
            0x4de742c086af14cf,
        ),
        (
            "random 41",
            scenario(70195, 5, 1, 1, 0, None, 10, 250),
            11,
            [5, 1, 0, 4, 0, 1],
            0x8fdecac29500285b,
        ),
        (
            "random 42",
            scenario(57648, 40, 2, 2, 2, None, 1, 10),
            108,
            [40, 15, 13, 25, 0, 15],
            0x0c45c004dcab0f86,
        ),
        (
            "random 43",
            scenario(30451, 17, 5, 3, 0, None, 2, 250),
            38,
            [17, 4, 0, 13, 0, 4],
            0x4dc849f76e747694,
        ),
        (
            "random 44",
            scenario(7230, 24, 5, 3, 3, Some(300), 10, 1),
            72,
            [24, 24, 0, 0, 0, 24],
            0x1fbc180a09bbafcb,
        ),
        (
            "random 45",
            scenario(55228, 54, 1, 1, 3, Some(50), 50, 250),
            167,
            [54, 12, 47, 0, 42, 12],
            0xf42a461d0cea4f88,
        ),
        (
            "random 46",
            scenario(1715, 44, 3, 2, 2, Some(5), 50, 250),
            132,
            [44, 17, 27, 0, 27, 17],
            0x3968de9b6c2750c3,
        ),
        (
            "random 47",
            scenario(71256, 17, 5, 3, 0, Some(300), 300, 600),
            48,
            [17, 14, 0, 3, 0, 14],
            0x0ad3b1ecc5735076,
        ),
        (
            "random 48",
            scenario(19108, 52, 2, 1, 10, Some(300), 50, 600),
            159,
            [52, 4, 51, 0, 48, 4],
            0x5e2d57077757bf2b,
        ),
        (
            "random 49",
            scenario(89212, 42, 5, 3, 5, Some(5), 100, 600),
            126,
            [42, 15, 27, 0, 27, 15],
            0x603741bca0b3dddb,
        ),
        (
            "random 50",
            scenario(86276, 22, 3, 3, 3, Some(300), 50, 2),
            66,
            [22, 22, 0, 0, 0, 22],
            0x5d1770e5765334ea,
        ),
        (
            "random 51",
            scenario(26436, 23, 2, 4, 0, Some(300), 300, 100),
            69,
            [23, 23, 0, 0, 0, 23],
            0x37b5360b099590ec,
        ),
        (
            "random 52",
            scenario(23302, 18, 1, 2, 2, Some(300), 300, 10),
            54,
            [18, 18, 0, 0, 0, 18],
            0x77ee263d39367d13,
        ),
        (
            "random 53",
            scenario(98426, 45, 3, 2, 0, Some(1000), 50, 9223372036854775807),
            92,
            [45, 2, 0, 43, 0, 2],
            0xc20b1d8a622a4c0b,
        ),
        (
            "random 54",
            scenario(89271, 59, 2, 4, 3, None, 50, 600),
            167,
            [59, 28, 21, 31, 0, 28],
            0xb36cde5395cbeee1,
        ),
        (
            "random 55",
            scenario(52105, 33, 3, 1, 10, Some(1), 300, 100),
            99,
            [33, 28, 5, 0, 5, 28],
            0xcbb95032a18bd62c,
        ),
        (
            "random 56",
            scenario(65557, 38, 1, 4, 0, Some(300), 300, 100),
            114,
            [38, 38, 0, 0, 0, 38],
            0x103a3864ffd1585f,
        ),
        (
            "random 57",
            scenario(2963, 16, 3, 2, 2, Some(0), 10, 250),
            32,
            [16, 0, 0, 16, 0, 0],
            0x38ddff4684dd2e17,
        ),
        (
            "random 58",
            scenario(56772, 36, 5, 4, 10, None, 2, 10),
            124,
            [36, 36, 16, 0, 0, 36],
            0x8bdd9ff60745f33a,
        ),
        (
            "random 59",
            scenario(95225, 13, 1, 3, 10, Some(0), 300, 18446744073709551615),
            26,
            [13, 0, 0, 13, 0, 0],
            0xb085352e4e830fe6,
        ),
        (
            "random 60",
            scenario(25480, 37, 2, 1, 3, None, 300, 250),
            133,
            [37, 37, 22, 0, 0, 37],
            0x748a1e04893bcdf5,
        ),
        (
            "random 61",
            scenario(88044, 22, 3, 1, 5, Some(1000), 300, 2),
            66,
            [22, 22, 0, 0, 0, 22],
            0x9ade2263524fc54b,
        ),
        (
            "random 62",
            scenario(21995, 44, 2, 3, 1, Some(0), 10, 600),
            88,
            [44, 0, 0, 44, 0, 0],
            0xf27862999dbf0b6d,
        ),
        (
            "random 63",
            scenario(51854, 58, 5, 4, 2, Some(1), 2, 100),
            174,
            [58, 9, 49, 0, 49, 9],
            0x5870048ea3aee3c6,
        ),
        (
            "random 64",
            scenario(27419, 46, 5, 2, 5, Some(300), 50, 600),
            134,
            [46, 9, 33, 11, 26, 9],
            0xff547a6ea6272f82,
        ),
        (
            "random 65",
            scenario(88338, 46, 1, 1, 3, Some(5), 6148914691236517205, 10),
            94,
            [46, 2, 0, 44, 0, 2],
            0xb216f36a66ee8f7f,
        ),
        (
            "random 66",
            scenario(61186, 39, 5, 1, 10, Some(300), 50, 2),
            119,
            [39, 39, 2, 0, 0, 39],
            0x7e073db839ba9361,
        ),
        (
            "random 67",
            scenario(55807, 13, 5, 2, 10, None, 2, 250),
            50,
            [13, 13, 11, 0, 0, 13],
            0x4afd28f9f6a6d9b8,
        ),
        (
            "random 68",
            scenario(67629, 43, 5, 1, 5, Some(0), 6148914691236517205, 600),
            86,
            [43, 0, 0, 43, 0, 0],
            0x40f88b2e5ae1f2cc,
        ),
        (
            "random 69",
            scenario(73176, 37, 5, 1, 5, Some(50), 1, 1),
            144,
            [37, 37, 33, 0, 0, 37],
            0x91374deab941355b,
        ),
        (
            "random 70",
            scenario(95533, 47, 5, 3, 0, Some(300), 50, 600),
            110,
            [47, 16, 0, 31, 0, 16],
            0x1561db3583baa1a8,
        ),
        (
            "random 71",
            scenario(34325, 26, 5, 1, 1, Some(0), 6148914691236517205, 100),
            52,
            [26, 0, 0, 26, 0, 0],
            0xa0062b6838722d81,
        ),
        (
            "random 72",
            scenario(59097, 24, 2, 4, 3, Some(5), 50, 10),
            72,
            [24, 24, 0, 0, 0, 24],
            0x8366d70e0b51e4a8,
        ),
        (
            "random 73",
            scenario(71411, 42, 1, 2, 10, Some(1), 100, 1),
            126,
            [42, 42, 0, 0, 0, 42],
            0x8e6cd3981a8b1e75,
        ),
        (
            "random 74",
            scenario(41081, 8, 3, 1, 3, Some(0), 300, 10),
            16,
            [8, 0, 0, 8, 0, 0],
            0x99ddd3a462e7d22d,
        ),
        (
            "random 75",
            scenario(66925, 6, 2, 4, 10, Some(50), 50, 10),
            18,
            [6, 6, 0, 0, 0, 6],
            0x814272457e4d3dbd,
        ),
        (
            "random 76",
            scenario(11502, 44, 1, 2, 3, None, 2, 100),
            100,
            [44, 7, 5, 37, 0, 7],
            0x57df3bea065051ef,
        ),
        (
            "random 77",
            scenario(81250, 14, 1, 4, 2, Some(1), 6148914691236517205, 100),
            32,
            [14, 4, 0, 10, 0, 4],
            0x4642c00a836b424f,
        ),
        (
            "random 78",
            scenario(76376, 22, 1, 3, 10, Some(5), 300, 250),
            66,
            [22, 22, 0, 0, 0, 22],
            0x61fbfcb6c66675c3,
        ),
        (
            "random 79",
            scenario(68108, 3, 1, 4, 0, Some(5), 50, 600),
            9,
            [3, 3, 0, 0, 0, 3],
            0x9983472a42488f32,
        ),
    ];
    let mut wrong = Vec::new();
    for (name, scenario, lines, counts, digest) in cases {
        let trace = trace_of(&scenario);
        if trace.lines().count() != lines || kinds(&trace) != counts || fnv1a(&trace) != digest {
            wrong.push(name);
        }
    }
    assert!(wrong.is_empty(), "traces differ for {wrong:?}");
}
