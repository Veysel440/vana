//! Property tests for invariants I1 and I2 on `FixedLimiter`.

use std::collections::HashSet;

use proptest::prelude::*;
use vana_core::{FixedLimiter, Limiter, Nanos, Outcome, Permit};

/// One step of a random walk: acquire a permit, or release the held permit at
/// the given position (taken modulo the number of held permits).
#[derive(Clone, Debug)]
enum Op {
    Acquire,
    Release(usize, u8),
}

fn op() -> impl Strategy<Value = Op> {
    prop_oneof![
        3 => Just(Op::Acquire),
        2 => (any::<usize>(), 0u8..3).prop_map(|(i, o)| Op::Release(i, o)),
    ]
}

fn outcome(code: u8) -> Outcome {
    match code {
        0 => Outcome::Success,
        1 => Outcome::Error,
        _ => Outcome::Cancelled,
    }
}

/// Plays `ops` against a `FixedLimiter` and returns the first violation found.
/// Every held permit is released before returning, so a failing case never
/// trips the drop check.
fn play(limit: u32, ops: &[Op]) -> Result<(), String> {
    let mut limiter = FixedLimiter::new(limit).map_err(|e| e.to_string())?;
    let mut held: Vec<Permit> = Vec::new();
    let mut granted: u64 = 0;
    let mut released: u64 = 0;
    let mut problem: Option<String> = None;
    let mut now: u64 = 0;

    for (step, op) in ops.iter().enumerate() {
        now += 1;
        match op {
            Op::Acquire => {
                let before = limiter.in_flight();
                let got = limiter.try_acquire(Nanos::new(now));
                // I1: a permit is granted exactly while in_flight < limit.
                if got.is_some() != (before < limit) && problem.is_none() {
                    problem = Some(format!(
                        "step {step}: acquire with in_flight {before}, limit {limit}, granted {}",
                        got.is_some()
                    ));
                }
                if let Some(permit) = got {
                    if permit.acquired_at() != Nanos::new(now) && problem.is_none() {
                        problem = Some(format!("step {step}: wrong acquired_at"));
                    }
                    granted += 1;
                    held.push(permit);
                }
            }
            Op::Release(index, code) => {
                if !held.is_empty() {
                    let permit = held.swap_remove(index % held.len());
                    limiter.release(permit, Nanos::new(now), outcome(*code));
                    released += 1;
                }
            }
        }
        if problem.is_none() {
            let in_flight = limiter.in_flight();
            if in_flight > limit {
                problem = Some(format!(
                    "step {step}: in_flight {in_flight} > limit {limit}"
                ));
            } else if u64::from(in_flight) != held.len() as u64 {
                problem = Some(format!(
                    "step {step}: in_flight {in_flight} != held {}",
                    held.len()
                ));
            } else if granted != released + u64::from(in_flight) {
                problem = Some(format!(
                    "step {step}: granted {granted} != released {released} + in_flight {in_flight}"
                ));
            } else if limiter.limit() != limit {
                problem = Some(format!("step {step}: the limit changed"));
            }
        }
    }

    for permit in held.drain(..) {
        limiter.release(permit, Nanos::new(now + 1), Outcome::Success);
        released += 1;
    }
    if problem.is_none() && limiter.in_flight() != 0 {
        problem = Some(format!(
            "in_flight {} after releasing all",
            limiter.in_flight()
        ));
    }
    if problem.is_none() && granted != released {
        problem = Some(format!(
            "granted {granted} != released {released} at the end"
        ));
    }
    match problem {
        Some(message) => Err(message),
        None => Ok(()),
    }
}

/// Releases the held permit at `pick` (modulo the number held) and records its tag.
fn release_one(
    limiter: &mut FixedLimiter,
    held: &mut Vec<(u64, Permit)>,
    pick: usize,
    now: u64,
    released_tags: &mut HashSet<u64>,
    problem: &mut Option<String>,
) {
    let (tag, permit) = held.swap_remove(pick % held.len());
    limiter.release(permit, Nanos::new(now), Outcome::Success);
    if !released_tags.insert(tag) && problem.is_none() {
        *problem = Some(format!("permit {tag} released twice"));
    }
}

/// Plays `ops` with every permit tagged by its grant number, releasing the
/// permit picked by each `Release` op, then drains the rest in the order given
/// by `drain_order`. Checks that each tag is released exactly once.
fn play_any_order(limit: u32, ops: &[Op], drain_order: &[usize]) -> Result<(), String> {
    let mut limiter = FixedLimiter::new(limit).map_err(|e| e.to_string())?;
    let mut held: Vec<(u64, Permit)> = Vec::new();
    let mut released_tags: HashSet<u64> = HashSet::new();
    let mut granted: u64 = 0;
    let mut problem: Option<String> = None;

    let mut now: u64 = 0;
    for (step, op) in ops.iter().enumerate() {
        now += 1;
        match op {
            Op::Acquire => {
                if let Some(permit) = limiter.try_acquire(Nanos::new(now)) {
                    held.push((granted, permit));
                    granted += 1;
                }
            }
            Op::Release(pick, _) => {
                if !held.is_empty() {
                    release_one(
                        &mut limiter,
                        &mut held,
                        *pick,
                        now,
                        &mut released_tags,
                        &mut problem,
                    );
                }
            }
        }
        let released = released_tags.len() as u64;
        let in_flight = u64::from(limiter.in_flight());
        if granted != released + in_flight && problem.is_none() {
            problem = Some(format!(
                "step {step}: granted {granted} != released {released} + in_flight {in_flight}"
            ));
        }
    }

    let mut picks = drain_order.iter().copied().cycle();
    while !held.is_empty() {
        now += 1;
        let pick = picks.next().unwrap_or(0);
        release_one(
            &mut limiter,
            &mut held,
            pick,
            now,
            &mut released_tags,
            &mut problem,
        );
    }

    if problem.is_none() && limiter.in_flight() != 0 {
        problem = Some(format!(
            "in_flight {} after releasing all",
            limiter.in_flight()
        ));
    }
    if problem.is_none() && released_tags.len() as u64 != granted {
        problem = Some(format!(
            "{} distinct releases for {granted} grants",
            released_tags.len()
        ));
    }
    match problem {
        Some(message) => Err(message),
        None => Ok(()),
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    // I1: in_flight never exceeds the limit and a permit is granted exactly
    // while in_flight < limit, over random acquire and release sequences.
    #[test]
    fn i1_never_grants_beyond_the_limit(
        limit in 1u32..=8,
        ops in proptest::collection::vec(op(), 0..200),
    ) {
        let result = play(limit, &ops);
        prop_assert!(result.is_ok(), "{:?}", result);
    }

    // I2: every granted permit is released exactly once, in an arbitrary
    // order: granted equals released plus in flight after every step, and
    // in_flight is 0 once all are released.
    #[test]
    fn i2_every_granted_permit_is_released_exactly_once_in_any_order(
        limit in 1u32..=8,
        ops in proptest::collection::vec(op(), 0..200),
        drain_order in proptest::collection::vec(any::<usize>(), 0..16),
    ) {
        let result = play_any_order(limit, &ops, &drain_order);
        prop_assert!(result.is_ok(), "{:?}", result);
    }

    // I1: limit 1 is the tightest case: a second permit is never granted
    // while one is held, however long the sequence.
    #[test]
    fn i1_a_limit_of_one_never_has_two_in_flight(
        ops in proptest::collection::vec(op(), 0..200),
    ) {
        let result = play(1, &ops);
        prop_assert!(result.is_ok(), "{:?}", result);
    }
}

// I1: a fully loaded limiter at every limit from 1 to 16 refuses exactly the
// permit after the last one.
#[test]
fn i1_filling_a_limiter_to_its_limit_and_one_more_is_refused() {
    for limit in 1u32..=16 {
        let mut limiter = FixedLimiter::new(limit).expect("limit is at least 1");
        let mut held = Vec::new();
        for i in 0..limit {
            held.push(
                limiter
                    .try_acquire(Nanos::new(u64::from(i)))
                    .expect("free slot"),
            );
        }
        assert_eq!(limiter.in_flight(), limit);
        assert!(limiter.try_acquire(Nanos::new(100)).is_none());
        for permit in held {
            limiter.release(permit, Nanos::new(200), Outcome::Success);
        }
        assert_eq!(limiter.in_flight(), 0);
    }
}
