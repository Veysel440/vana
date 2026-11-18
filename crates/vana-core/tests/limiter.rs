//! Black-box tests for `FixedLimiter`, `LimitError` and `Outcome`.

use vana_core::{FixedLimiter, LimitError, Limiter, Nanos, Outcome, Permit};

fn limiter(limit: u32) -> FixedLimiter {
    FixedLimiter::new(limit).expect("limit is at least 1")
}

fn acquire(limiter: &mut FixedLimiter, now: u64) -> Permit {
    limiter
        .try_acquire(Nanos::new(now))
        .expect("a permit should be available")
}

fn release(limiter: &mut FixedLimiter, permit: Permit, now: u64) {
    limiter.release(permit, Nanos::new(now), Outcome::Success);
}

// DESIGN.md, limiter: a limit of 0 is rejected with LimitError::Zero.
#[test]
fn a_limit_of_zero_is_rejected() {
    assert_eq!(FixedLimiter::new(0).unwrap_err(), LimitError::Zero);
}

// DESIGN.md, limiter: the error message is "limit must be at least 1".
#[test]
fn the_limit_error_message_is_exact() {
    assert_eq!(LimitError::Zero.to_string(), "limit must be at least 1");
    assert_eq!(
        FixedLimiter::new(0).unwrap_err().to_string(),
        "limit must be at least 1"
    );
}

// DESIGN.md, limiter: LimitError implements std::error::Error and Clone, Copy, Eq.
#[test]
fn the_limit_error_is_a_standard_error() {
    let boxed: Box<dyn std::error::Error> = Box::new(LimitError::Zero);
    assert_eq!(boxed.to_string(), "limit must be at least 1");
    let copy = LimitError::Zero;
    let again = copy;
    assert_eq!(copy, again);
    assert!(!format!("{copy:?}").is_empty());
}

// DESIGN.md, limiter: a new limiter reports its limit and holds no permits.
#[test]
fn a_new_limiter_has_nothing_in_flight() {
    let limiter = limiter(4);
    assert_eq!(limiter.limit(), 4);
    assert_eq!(limiter.in_flight(), 0);
}

// DESIGN.md, limiter: the largest limit is accepted.
#[test]
fn the_largest_limit_is_accepted() {
    let mut limiter = limiter(u32::MAX);
    assert_eq!(limiter.limit(), u32::MAX);
    let permit = acquire(&mut limiter, 0);
    assert_eq!(limiter.in_flight(), 1);
    release(&mut limiter, permit, 1);
}

// I1: with limit 1 a second permit is refused until the first is released.
#[test]
fn a_full_limiter_rejects_until_a_permit_is_released() {
    let mut limiter = limiter(1);
    let first = acquire(&mut limiter, 10);
    assert_eq!(limiter.in_flight(), 1);
    assert!(limiter.try_acquire(Nanos::new(11)).is_none());
    assert_eq!(limiter.in_flight(), 1);
    release(&mut limiter, first, 12);
    assert_eq!(limiter.in_flight(), 0);
    let second = acquire(&mut limiter, 13);
    assert_eq!(limiter.in_flight(), 1);
    release(&mut limiter, second, 14);
}

// I1: with limit 3 exactly three permits are granted, then one slot per release.
#[test]
fn a_limit_of_three_grants_exactly_three_permits() {
    let mut limiter = limiter(3);
    let a = acquire(&mut limiter, 1);
    let b = acquire(&mut limiter, 2);
    let c = acquire(&mut limiter, 3);
    assert_eq!(limiter.in_flight(), 3);
    assert!(limiter.try_acquire(Nanos::new(4)).is_none());
    release(&mut limiter, b, 5);
    assert_eq!(limiter.in_flight(), 2);
    let d = acquire(&mut limiter, 6);
    assert!(limiter.try_acquire(Nanos::new(7)).is_none());
    release(&mut limiter, a, 8);
    release(&mut limiter, c, 9);
    release(&mut limiter, d, 10);
    assert_eq!(limiter.in_flight(), 0);
}

// I1: a refused try_acquire does not change in_flight, however often it repeats.
#[test]
fn refusals_do_not_change_the_count() {
    let mut limiter = limiter(2);
    let a = acquire(&mut limiter, 0);
    let b = acquire(&mut limiter, 0);
    for now in 1..=5 {
        assert!(limiter.try_acquire(Nanos::new(now)).is_none());
        assert_eq!(limiter.in_flight(), 2);
    }
    release(&mut limiter, a, 6);
    release(&mut limiter, b, 6);
}

// I1: when in_flight equals the limit, one release makes exactly one grant possible.
#[test]
fn one_release_frees_exactly_one_slot() {
    let mut limiter = limiter(2);
    let a = acquire(&mut limiter, 0);
    let b = acquire(&mut limiter, 0);
    assert_eq!(limiter.in_flight(), limiter.limit());
    release(&mut limiter, a, 1);
    let c = acquire(&mut limiter, 2);
    assert!(limiter.try_acquire(Nanos::new(3)).is_none());
    release(&mut limiter, b, 4);
    release(&mut limiter, c, 4);
}

// DESIGN.md, limiter: a permit remembers the time it was granted.
#[test]
fn a_permit_records_when_it_was_acquired() {
    let mut limiter = limiter(2);
    let first = acquire(&mut limiter, 0);
    let second = acquire(&mut limiter, u64::MAX);
    assert_eq!(first.acquired_at(), Nanos::ZERO);
    assert_eq!(second.acquired_at(), Nanos::MAX);
    release(&mut limiter, first, 1);
    release(&mut limiter, second, 1);
}

// DESIGN.md, limiter: release ignores the outcome; each one frees a slot.
#[test]
fn every_outcome_frees_one_slot() {
    for outcome in [Outcome::Success, Outcome::Error, Outcome::Cancelled] {
        let mut limiter = limiter(1);
        let permit = acquire(&mut limiter, 0);
        limiter.release(permit, Nanos::new(1), outcome);
        assert_eq!(limiter.in_flight(), 0, "outcome {outcome:?}");
        let again = acquire(&mut limiter, 2);
        release(&mut limiter, again, 3);
    }
}

// DESIGN.md, limiter: the release time does not matter, even earlier than the grant.
#[test]
fn the_release_time_does_not_change_the_count() {
    let mut limiter = limiter(1);
    let permit = acquire(&mut limiter, 100);
    limiter.release(permit, Nanos::ZERO, Outcome::Success);
    assert_eq!(limiter.in_flight(), 0);
}

// DESIGN.md, limiter: Outcome derives Clone, Copy, Debug, PartialEq, Eq.
#[test]
fn outcomes_compare_by_variant() {
    fn assert_traits<T: Clone + Copy + core::fmt::Debug + PartialEq + Eq>() {}
    assert_traits::<Outcome>();
    assert_eq!(Outcome::Success, Outcome::Success);
    assert_ne!(Outcome::Success, Outcome::Error);
    assert_ne!(Outcome::Error, Outcome::Cancelled);
    assert_ne!(Outcome::Success, Outcome::Cancelled);
}

// DESIGN.md, limiter: FixedLimiter works through the Limiter trait, also as a trait object.
#[test]
fn the_limiter_trait_works_as_a_trait_object() {
    let mut fixed = limiter(1);
    let dynamic: &mut dyn Limiter = &mut fixed;
    assert_eq!(dynamic.limit(), 1);
    let permit = dynamic.try_acquire(Nanos::new(1)).expect("free slot");
    assert_eq!(dynamic.in_flight(), 1);
    assert!(dynamic.try_acquire(Nanos::new(2)).is_none());
    dynamic.release(permit, Nanos::new(3), Outcome::Error);
    assert_eq!(dynamic.in_flight(), 0);
}

// I2: releasing at in_flight 0 is a bug and panics in debug builds.
#[cfg(debug_assertions)]
#[test]
#[should_panic(expected = "release called on a limiter with no permit in flight")]
fn releasing_to_an_empty_limiter_panics_in_debug_builds() {
    let mut owner = limiter(1);
    let permit = acquire(&mut owner, 0);
    let mut other = limiter(1);
    other.release(permit, Nanos::new(1), Outcome::Success);
}

// I2: in release builds a release at in_flight 0 leaves in_flight at 0.
#[cfg(not(debug_assertions))]
#[test]
fn releasing_to_an_empty_limiter_keeps_zero_in_release_builds() {
    let mut owner = limiter(1);
    let permit = acquire(&mut owner, 0);
    let mut other = limiter(1);
    other.release(permit, Nanos::new(1), Outcome::Success);
    assert_eq!(other.in_flight(), 0);
}

// I2: a debug panic from a release at in_flight 0 leaves the other limiter's state intact.
#[cfg(debug_assertions)]
#[test]
fn a_foreign_release_does_not_corrupt_the_other_limiter() {
    use std::panic::{AssertUnwindSafe, catch_unwind};
    let mut owner = limiter(1);
    let permit = acquire(&mut owner, 0);
    let mut other = limiter(2);
    let result = catch_unwind(AssertUnwindSafe(|| {
        other.release(permit, Nanos::new(1), Outcome::Success);
    }));
    assert!(result.is_err());
    assert_eq!(other.in_flight(), 0);
    assert_eq!(other.limit(), 2);
}
