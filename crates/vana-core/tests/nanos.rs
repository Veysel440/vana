//! Black-box tests for `Nanos` and the `Clock` trait.

use proptest::prelude::*;
use vana_core::{Clock, Nanos};

// DESIGN.md, time: Nanos saturates at MAX instead of wrapping.
#[test]
fn adding_one_to_max_stays_at_max() {
    assert_eq!(Nanos::MAX.saturating_add(Nanos::new(1)), Nanos::MAX);
}

// DESIGN.md, time: Nanos saturates at zero instead of wrapping.
#[test]
fn subtracting_one_from_zero_stays_at_zero() {
    assert_eq!(Nanos::ZERO.saturating_sub(Nanos::new(1)), Nanos::ZERO);
}

// DESIGN.md, time: ordinary additions are exact.
#[test]
fn adding_small_values_is_exact() {
    assert_eq!(Nanos::new(2).saturating_add(Nanos::new(3)), Nanos::new(5));
}

// DESIGN.md, time: ordinary subtractions are exact.
#[test]
fn subtracting_small_values_is_exact() {
    assert_eq!(Nanos::new(5).saturating_sub(Nanos::new(3)), Nanos::new(2));
}

// DESIGN.md, time: boundary just below the saturation point.
#[test]
fn adding_up_to_exactly_max_does_not_saturate_early() {
    let almost = Nanos::new(u64::MAX - 1);
    assert_eq!(almost.saturating_add(Nanos::new(1)), Nanos::MAX);
    assert_eq!(almost.saturating_add(Nanos::new(1)).get(), u64::MAX);
}

// DESIGN.md, time: a value minus itself is zero, not an underflow.
#[test]
fn subtracting_a_value_from_itself_gives_zero() {
    assert_eq!(Nanos::MAX.saturating_sub(Nanos::MAX), Nanos::ZERO);
    assert_eq!(Nanos::new(7).saturating_sub(Nanos::new(7)), Nanos::ZERO);
}

// DESIGN.md, time: subtracting a larger value saturates at zero.
#[test]
fn subtracting_a_larger_value_gives_zero() {
    assert_eq!(Nanos::new(3).saturating_sub(Nanos::new(5)), Nanos::ZERO);
    assert_eq!(Nanos::ZERO.saturating_sub(Nanos::MAX), Nanos::ZERO);
}

// DESIGN.md, time: adding zero changes nothing, including at MAX.
#[test]
fn adding_zero_changes_nothing() {
    assert_eq!(Nanos::MAX.saturating_add(Nanos::ZERO), Nanos::MAX);
    assert_eq!(Nanos::new(9).saturating_add(Nanos::ZERO), Nanos::new(9));
}

// DESIGN.md, time: ZERO, MAX, new and get agree with plain u64 values.
#[test]
fn constants_and_accessors_match_the_raw_value() {
    assert_eq!(Nanos::ZERO.get(), 0);
    assert_eq!(Nanos::MAX.get(), u64::MAX);
    assert_eq!(Nanos::new(123).get(), 123);
    assert_eq!(Nanos::new(0), Nanos::ZERO);
    assert_eq!(Nanos::new(u64::MAX), Nanos::MAX);
}

// DESIGN.md, time: new and get are const fns (usable in const context).
#[test]
fn new_and_get_work_in_const_context() {
    const VALUE: Nanos = Nanos::new(77);
    const RAW: u64 = VALUE.get();
    assert_eq!(RAW, 77);
}

// DESIGN.md, time: Nanos derives Default, and the default is zero.
#[test]
fn the_default_value_is_zero() {
    assert_eq!(Nanos::default(), Nanos::ZERO);
}

// DESIGN.md, time: Nanos is ordered by its raw value.
#[test]
fn values_are_ordered_by_their_raw_number() {
    assert!(Nanos::ZERO < Nanos::new(1));
    assert!(Nanos::new(1) < Nanos::MAX);
    assert_eq!(Nanos::new(4).max(Nanos::new(9)), Nanos::new(9));
    assert_eq!(Nanos::new(4).min(Nanos::new(9)), Nanos::new(4));
}

// DESIGN.md, time: Nanos derives Clone, Copy, Debug, Default, Eq, Ord, Hash.
#[test]
fn nanos_has_the_documented_derives() {
    fn assert_traits<T: Clone + Copy + core::fmt::Debug + Default + Eq + Ord + core::hash::Hash>() {
    }
    assert_traits::<Nanos>();
    let set: std::collections::HashSet<Nanos> = [Nanos::new(1), Nanos::new(1), Nanos::new(2)]
        .into_iter()
        .collect();
    assert_eq!(set.len(), 2);
}

// DESIGN.md, time: Clock is a one-method trait that implementations can supply.
#[test]
fn a_clock_can_be_implemented_and_used_through_a_trait_object() {
    struct Fixed(Nanos);
    impl Clock for Fixed {
        fn now(&self) -> Nanos {
            self.0
        }
    }
    let clock = Fixed(Nanos::new(42));
    let dynamic: &dyn Clock = &clock;
    assert_eq!(dynamic.now(), Nanos::new(42));
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    // DESIGN.md, time: saturating_add equals the u64 saturating sum.
    #[test]
    fn saturating_add_matches_u64_saturating_add(a in any::<u64>(), b in any::<u64>()) {
        prop_assert_eq!(
            Nanos::new(a).saturating_add(Nanos::new(b)).get(),
            a.saturating_add(b)
        );
    }

    // DESIGN.md, time: saturating_sub equals the u64 saturating difference.
    #[test]
    fn saturating_sub_matches_u64_saturating_sub(a in any::<u64>(), b in any::<u64>()) {
        prop_assert_eq!(
            Nanos::new(a).saturating_sub(Nanos::new(b)).get(),
            a.saturating_sub(b)
        );
    }

    // DESIGN.md, time: adding never makes a value smaller (no wrap-around).
    #[test]
    fn adding_never_decreases_a_value(a in any::<u64>(), b in any::<u64>()) {
        prop_assert!(Nanos::new(a).saturating_add(Nanos::new(b)) >= Nanos::new(a));
    }
}
