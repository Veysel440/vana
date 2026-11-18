//! Hand-made limiter sequences replayed against `FixedLimiter`.

use std::collections::HashMap;

use vana_core::{FixedLimiter, Limiter, Nanos, Outcome, Permit};

/// One step of a sequence. `acquired_in_step` is the index of the step whose
/// permit is released.
enum Step {
    Acquire {
        now: u64,
        granted_at: Option<u64>,
        in_flight: u32,
    },
    Release {
        acquired_in_step: usize,
        now: u64,
        outcome: Outcome,
        in_flight: u32,
    },
}

struct Sequence {
    name: &'static str,
    limit: u32,
    steps: Vec<Step>,
    final_in_flight: u32,
}

fn sequences() -> Vec<Sequence> {
    vec![
        Sequence {
            name: "lim_limit1",
            limit: 1,
            steps: vec![
                Step::Acquire {
                    now: 0,
                    granted_at: Some(0),
                    in_flight: 1,
                },
                Step::Acquire {
                    now: 1,
                    granted_at: None,
                    in_flight: 1,
                },
                Step::Release {
                    acquired_in_step: 0,
                    now: 2,
                    outcome: Outcome::Success,
                    in_flight: 0,
                },
                Step::Acquire {
                    now: 3,
                    granted_at: Some(3),
                    in_flight: 1,
                },
                Step::Acquire {
                    now: 4,
                    granted_at: None,
                    in_flight: 1,
                },
                Step::Release {
                    acquired_in_step: 3,
                    now: 5,
                    outcome: Outcome::Success,
                    in_flight: 0,
                },
            ],
            final_in_flight: 0,
        },
        Sequence {
            name: "lim_limit3",
            limit: 3,
            steps: vec![
                Step::Acquire {
                    now: 0,
                    granted_at: Some(0),
                    in_flight: 1,
                },
                Step::Acquire {
                    now: 0,
                    granted_at: Some(0),
                    in_flight: 2,
                },
                Step::Acquire {
                    now: 0,
                    granted_at: Some(0),
                    in_flight: 3,
                },
                Step::Acquire {
                    now: 1,
                    granted_at: None,
                    in_flight: 3,
                },
                Step::Release {
                    acquired_in_step: 1,
                    now: 2,
                    outcome: Outcome::Success,
                    in_flight: 2,
                },
                Step::Acquire {
                    now: 3,
                    granted_at: Some(3),
                    in_flight: 3,
                },
                Step::Acquire {
                    now: 3,
                    granted_at: None,
                    in_flight: 3,
                },
                Step::Release {
                    acquired_in_step: 0,
                    now: 4,
                    outcome: Outcome::Success,
                    in_flight: 2,
                },
                Step::Release {
                    acquired_in_step: 2,
                    now: 4,
                    outcome: Outcome::Success,
                    in_flight: 1,
                },
                Step::Release {
                    acquired_in_step: 5,
                    now: 5,
                    outcome: Outcome::Success,
                    in_flight: 0,
                },
            ],
            final_in_flight: 0,
        },
        Sequence {
            name: "lim_full_none",
            limit: 2,
            steps: vec![
                Step::Acquire {
                    now: 0,
                    granted_at: Some(0),
                    in_flight: 1,
                },
                Step::Acquire {
                    now: 0,
                    granted_at: Some(0),
                    in_flight: 2,
                },
                Step::Acquire {
                    now: 0,
                    granted_at: None,
                    in_flight: 2,
                },
                Step::Acquire {
                    now: 0,
                    granted_at: None,
                    in_flight: 2,
                },
                Step::Acquire {
                    now: 0,
                    granted_at: None,
                    in_flight: 2,
                },
            ],
            final_in_flight: 2,
        },
        Sequence {
            name: "lim_regrant_after_release",
            limit: 2,
            steps: vec![
                Step::Acquire {
                    now: 10,
                    granted_at: Some(10),
                    in_flight: 1,
                },
                Step::Acquire {
                    now: 11,
                    granted_at: Some(11),
                    in_flight: 2,
                },
                Step::Acquire {
                    now: 12,
                    granted_at: None,
                    in_flight: 2,
                },
                Step::Release {
                    acquired_in_step: 0,
                    now: 13,
                    outcome: Outcome::Success,
                    in_flight: 1,
                },
                Step::Acquire {
                    now: 14,
                    granted_at: Some(14),
                    in_flight: 2,
                },
                Step::Acquire {
                    now: 15,
                    granted_at: None,
                    in_flight: 2,
                },
                Step::Release {
                    acquired_in_step: 1,
                    now: 16,
                    outcome: Outcome::Success,
                    in_flight: 1,
                },
                Step::Release {
                    acquired_in_step: 4,
                    now: 17,
                    outcome: Outcome::Success,
                    in_flight: 0,
                },
                Step::Acquire {
                    now: 18,
                    granted_at: Some(18),
                    in_flight: 1,
                },
            ],
            final_in_flight: 1,
        },
        Sequence {
            name: "lim_outcome_ignored",
            limit: 3,
            steps: vec![
                Step::Acquire {
                    now: 0,
                    granted_at: Some(0),
                    in_flight: 1,
                },
                Step::Acquire {
                    now: 0,
                    granted_at: Some(0),
                    in_flight: 2,
                },
                Step::Acquire {
                    now: 0,
                    granted_at: Some(0),
                    in_flight: 3,
                },
                Step::Release {
                    acquired_in_step: 0,
                    now: 1,
                    outcome: Outcome::Error,
                    in_flight: 2,
                },
                Step::Release {
                    acquired_in_step: 1,
                    now: 1,
                    outcome: Outcome::Cancelled,
                    in_flight: 1,
                },
                Step::Release {
                    acquired_in_step: 2,
                    now: 1,
                    outcome: Outcome::Success,
                    in_flight: 0,
                },
                Step::Acquire {
                    now: 2,
                    granted_at: Some(2),
                    in_flight: 1,
                },
            ],
            final_in_flight: 1,
        },
        Sequence {
            name: "lim_empty_ops",
            limit: 5,
            steps: vec![],
            final_in_flight: 0,
        },
        Sequence {
            name: "lim_u32max",
            limit: 4294967295,
            steps: vec![
                Step::Acquire {
                    now: 18446744073709551615,
                    granted_at: Some(18446744073709551615),
                    in_flight: 1,
                },
                Step::Release {
                    acquired_in_step: 0,
                    now: 18446744073709551615,
                    outcome: Outcome::Success,
                    in_flight: 0,
                },
            ],
            final_in_flight: 0,
        },
        Sequence {
            name: "lim_acquired_at",
            limit: 2,
            steps: vec![
                Step::Acquire {
                    now: 5,
                    granted_at: Some(5),
                    in_flight: 1,
                },
                Step::Acquire {
                    now: 9,
                    granted_at: Some(9),
                    in_flight: 2,
                },
                Step::Release {
                    acquired_in_step: 0,
                    now: 20,
                    outcome: Outcome::Success,
                    in_flight: 1,
                },
                Step::Release {
                    acquired_in_step: 1,
                    now: 20,
                    outcome: Outcome::Success,
                    in_flight: 0,
                },
            ],
            final_in_flight: 0,
        },
    ]
}

/// Plays one sequence and returns the first mismatch.
fn replay(sequence: &Sequence) -> Result<(), String> {
    let mut limiter = FixedLimiter::new(sequence.limit).map_err(|e| e.to_string())?;
    if limiter.limit() != sequence.limit {
        return Err(format!("limit {} != {}", limiter.limit(), sequence.limit));
    }
    let mut held: HashMap<usize, Permit> = HashMap::new();
    let mut problem = None;
    for (index, step) in sequence.steps.iter().enumerate() {
        match step {
            Step::Acquire {
                now,
                granted_at,
                in_flight,
            } => {
                let got = limiter.try_acquire(Nanos::new(*now));
                let got_at = got.as_ref().map(|p| p.acquired_at().get());
                if got_at != *granted_at && problem.is_none() {
                    problem = Some(format!(
                        "step {index}: acquired_at {got_at:?}, expected {granted_at:?}"
                    ));
                }
                if let Some(permit) = got {
                    held.insert(index, permit);
                }
                if limiter.in_flight() != *in_flight && problem.is_none() {
                    problem = Some(format!(
                        "step {index}: in_flight {}, expected {in_flight}",
                        limiter.in_flight()
                    ));
                }
            }
            Step::Release {
                acquired_in_step,
                now,
                outcome,
                in_flight,
            } => {
                let Some(permit) = held.remove(acquired_in_step) else {
                    problem.get_or_insert(format!(
                        "step {index}: no permit from step {acquired_in_step}"
                    ));
                    continue;
                };
                limiter.release(permit, Nanos::new(*now), *outcome);
                if limiter.in_flight() != *in_flight && problem.is_none() {
                    problem = Some(format!(
                        "step {index}: in_flight {}, expected {in_flight}",
                        limiter.in_flight()
                    ));
                }
            }
        }
    }
    let final_in_flight = limiter.in_flight();
    let leftover = held.len();
    // Release what is left so the drop check never fires inside this helper.
    for (_, permit) in held.drain() {
        limiter.release(permit, Nanos::ZERO, Outcome::Success);
    }
    if let Some(message) = problem {
        return Err(message);
    }
    if final_in_flight != sequence.final_in_flight {
        return Err(format!(
            "final in_flight {final_in_flight}, expected {}",
            sequence.final_in_flight
        ));
    }
    if leftover as u32 != sequence.final_in_flight {
        return Err(format!(
            "{leftover} permits left, expected {}",
            sequence.final_in_flight
        ));
    }
    Ok(())
}

fn check(name: &str) {
    let all = sequences();
    let sequence = all
        .iter()
        .find(|s| s.name == name)
        .unwrap_or_else(|| panic!("no sequence named {name}"));
    if let Err(message) = replay(sequence) {
        panic!("{}: {message}", sequence.name);
    }
}

// I1; a limit of 1 refuses while one permit is held; expected sequence computed independently from the limiter rules in docs/DESIGN.md.
#[test]
fn limit_one_refuses_the_second_permit_and_regrants_after_release() {
    check("lim_limit1");
}

// I1; limit 3, release frees exactly one slot; expected sequence computed independently from the limiter rules in docs/DESIGN.md.
#[test]
fn limit_three_refuses_the_fourth_permit_and_frees_one_slot_per_release() {
    check("lim_limit3");
}

// I1; a full limiter stays full; expected sequence computed independently from the limiter rules in docs/DESIGN.md.
#[test]
fn a_full_limiter_keeps_returning_none() {
    check("lim_full_none");
}

// I1; regrant after release; expected sequence computed independently from the limiter rules in docs/DESIGN.md.
#[test]
fn a_released_slot_is_granted_again() {
    check("lim_regrant_after_release");
}

// DESIGN.md, limiter: release ignores the outcome; expected sequence computed independently from the limiter rules in docs/DESIGN.md.
#[test]
fn every_outcome_releases_one_slot_in_the_sequence() {
    check("lim_outcome_ignored");
}

// DESIGN.md, limiter: new limiter state; expected sequence computed independently from the limiter rules in docs/DESIGN.md.
#[test]
fn a_fresh_limiter_has_the_given_limit_and_nothing_in_flight() {
    check("lim_empty_ops");
}

// DESIGN.md, limiter: boundary values; expected sequence computed independently from the limiter rules in docs/DESIGN.md.
#[test]
fn the_largest_limit_and_the_largest_time_are_kept() {
    check("lim_u32max");
}

// DESIGN.md, limiter: Permit::acquired_at; expected sequence computed independently from the limiter rules in docs/DESIGN.md.
#[test]
fn a_permit_reports_the_time_it_was_granted() {
    check("lim_acquired_at");
}
