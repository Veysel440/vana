//! Black-box tests for `Permit`.

use vana_core::{FixedLimiter, Limiter, Nanos, Outcome, Permit};

fn limiter(limit: u32) -> FixedLimiter {
    FixedLimiter::new(limit).expect("limit is at least 1")
}

// I2: Permit is neither Clone nor Copy, so one permit cannot be released twice.
// The impls below overlap for any type that is Clone or Copy, which would make
// this file fail to compile.
#[test]
fn a_permit_is_neither_clone_nor_copy() {
    trait AmbiguousIfClone<A> {
        fn check() {}
    }
    impl<T: ?Sized> AmbiguousIfClone<()> for T {}
    impl<T: Clone> AmbiguousIfClone<u8> for T {}

    trait AmbiguousIfCopy<A> {
        fn check() {}
    }
    impl<T: ?Sized> AmbiguousIfCopy<()> for T {}
    impl<T: Copy> AmbiguousIfCopy<u8> for T {}

    <Permit as AmbiguousIfClone<_>>::check();
    <Permit as AmbiguousIfCopy<_>>::check();
}

// I2: Permit implements Debug.
#[test]
fn a_permit_can_be_formatted_with_debug() {
    let mut limiter = limiter(1);
    let permit = limiter.try_acquire(Nanos::new(5)).expect("free slot");
    assert!(!format!("{permit:?}").is_empty());
    limiter.release(permit, Nanos::new(6), Outcome::Success);
}

// I2: a released permit does not trigger the drop check.
#[test]
fn a_released_permit_does_not_panic() {
    let mut limiter = limiter(1);
    let permit = limiter.try_acquire(Nanos::new(1)).expect("free slot");
    limiter.release(permit, Nanos::new(2), Outcome::Success);
    assert_eq!(limiter.in_flight(), 0);
}

// I2: dropping a permit without releasing it panics in debug builds.
#[cfg(debug_assertions)]
#[test]
#[should_panic(expected = "permit dropped without release")]
fn a_permit_dropped_without_release_panics_in_debug_builds() {
    let mut limiter = limiter(1);
    let permit = limiter.try_acquire(Nanos::new(1)).expect("free slot");
    drop(permit);
}

// I2: the drop check also fires when the permit simply goes out of scope.
#[cfg(debug_assertions)]
#[test]
#[should_panic(expected = "permit dropped without release")]
fn a_permit_that_goes_out_of_scope_panics_in_debug_builds() {
    let mut limiter = limiter(1);
    let _permit = limiter.try_acquire(Nanos::new(1)).expect("free slot");
}

// I2: while the thread is already panicking the drop check stays quiet, so the
// original panic is the one reported and the process does not abort.
#[cfg(debug_assertions)]
#[test]
fn a_permit_dropped_while_panicking_does_not_abort() {
    use std::panic::{AssertUnwindSafe, catch_unwind};
    let mut limiter = limiter(1);
    let result = catch_unwind(AssertUnwindSafe(|| {
        let _permit = limiter.try_acquire(Nanos::new(1)).expect("free slot");
        panic!("the original failure");
    }));
    let payload = result.expect_err("the closure panics");
    let message = payload
        .downcast_ref::<&str>()
        .copied()
        .expect("the payload is the original message");
    assert_eq!(message, "the original failure");
}

// I2: in release builds an unreleased permit is dropped silently.
#[cfg(not(debug_assertions))]
#[test]
fn a_permit_dropped_without_release_is_silent_in_release_builds() {
    let mut limiter = limiter(1);
    let permit = limiter.try_acquire(Nanos::new(1)).expect("free slot");
    drop(permit);
    // The slot stays taken: nothing released it.
    assert_eq!(limiter.in_flight(), 1);
}
