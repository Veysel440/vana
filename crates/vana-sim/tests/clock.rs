//! Black-box tests for `ManualClock`.

use proptest::prelude::*;
use vana_core::{Clock, Nanos};
use vana_sim::ManualClock;

// DESIGN.md, simulator: a manual clock reads the time it was created with.
#[test]
fn a_new_clock_reads_its_start_time() {
    assert_eq!(ManualClock::new(Nanos::new(17)).now(), Nanos::new(17));
    assert_eq!(ManualClock::new(Nanos::ZERO).now(), Nanos::ZERO);
    assert_eq!(ManualClock::new(Nanos::MAX).now(), Nanos::MAX);
}

// DESIGN.md, simulator: time moves only when advance is called.
#[test]
fn the_clock_does_not_move_by_itself() {
    let clock = ManualClock::new(Nanos::new(5));
    assert_eq!(clock.now(), Nanos::new(5));
    assert_eq!(clock.now(), Nanos::new(5));
}

// DESIGN.md, simulator: advance adds to the current time.
#[test]
fn advance_moves_the_clock_forward() {
    let mut clock = ManualClock::new(Nanos::new(10));
    clock.advance(Nanos::new(5));
    assert_eq!(clock.now(), Nanos::new(15));
    clock.advance(Nanos::new(100));
    assert_eq!(clock.now(), Nanos::new(115));
}

// DESIGN.md, simulator: advancing by zero changes nothing.
#[test]
fn advancing_by_zero_changes_nothing() {
    let mut clock = ManualClock::new(Nanos::new(10));
    clock.advance(Nanos::ZERO);
    assert_eq!(clock.now(), Nanos::new(10));
}

// DESIGN.md, time: advance saturates at the largest value instead of wrapping.
#[test]
fn advance_saturates_at_the_largest_value() {
    let mut clock = ManualClock::new(Nanos::new(u64::MAX - 1));
    clock.advance(Nanos::new(5));
    assert_eq!(clock.now(), Nanos::MAX);
    clock.advance(Nanos::new(1));
    assert_eq!(clock.now(), Nanos::MAX);
}

// DESIGN.md, time: advancing a clock at the largest value stays there.
#[test]
fn advancing_a_clock_at_the_largest_value_stays_there() {
    let mut clock = ManualClock::new(Nanos::MAX);
    clock.advance(Nanos::MAX);
    assert_eq!(clock.now(), Nanos::MAX);
}

// DESIGN.md, simulator: ManualClock implements Clock, also as a trait object.
#[test]
fn the_manual_clock_works_as_a_clock_trait_object() {
    let mut clock = ManualClock::new(Nanos::new(1));
    clock.advance(Nanos::new(2));
    let dynamic: &dyn Clock = &clock;
    assert_eq!(dynamic.now(), Nanos::new(3));
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    // DESIGN.md, time: a Clock is monotonic; a manual clock never goes back.
    #[test]
    fn the_clock_never_goes_backwards(
        start in any::<u64>(),
        steps in proptest::collection::vec(any::<u64>(), 0..50),
    ) {
        let mut clock = ManualClock::new(Nanos::new(start));
        let mut last = clock.now();
        for step in steps {
            clock.advance(Nanos::new(step));
            prop_assert!(clock.now() >= last);
            prop_assert_eq!(clock.now().get(), last.get().saturating_add(step));
            last = clock.now();
        }
    }
}
