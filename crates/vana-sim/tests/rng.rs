//! Black-box tests for `SplitMix64` properties.

use proptest::prelude::*;
use vana_sim::SplitMix64;

// I7: the same seed gives the same sequence.
#[test]
fn the_same_seed_gives_the_same_sequence() {
    let mut a = SplitMix64::new(99);
    let mut b = SplitMix64::new(99);
    for _ in 0..100 {
        assert_eq!(a.next_u64(), b.next_u64());
    }
}

// I7: different seeds give different first outputs (0 and 12345 are known to differ).
#[test]
fn different_seeds_give_different_sequences() {
    let mut a = SplitMix64::new(0);
    let mut b = SplitMix64::new(12345);
    assert_ne!(a.next_u64(), b.next_u64());
}

// I7: a clone continues with exactly the same outputs as the original.
#[test]
fn a_clone_continues_with_the_same_outputs() {
    let mut original = SplitMix64::new(7);
    for _ in 0..3 {
        original.next_u64();
    }
    let mut copy = original.clone();
    for _ in 0..20 {
        assert_eq!(original.next_u64(), copy.next_u64());
    }
}

// DESIGN.md, SplitMix64: a bound of 1 only allows 0.
#[test]
fn next_below_one_is_zero_for_many_draws() {
    let mut rng = SplitMix64::new(1);
    for _ in 0..200 {
        assert_eq!(rng.next_below(1), 0);
    }
}

// DESIGN.md, SplitMix64: next_below(0) panics (documented).
#[test]
#[should_panic]
fn next_below_zero_panics() {
    SplitMix64::new(1).next_below(0);
}

// DESIGN.md, SplitMix64: next_below is the high 64 bits of next_u64 * n, one draw per call.
#[test]
fn next_below_is_the_high_half_of_the_product_and_uses_one_draw() {
    for n in [1u64, 2, 3, 10, 1000, u64::MAX / 3, 1 << 63, u64::MAX] {
        let mut direct = SplitMix64::new(31);
        let mut bounded = SplitMix64::new(31);
        for _ in 0..20 {
            let x = direct.next_u64();
            let expected = ((u128::from(x) * u128::from(n)) >> 64) as u64;
            assert_eq!(bounded.next_below(n), expected, "n = {n}");
        }
    }
}

// DESIGN.md, SplitMix64: each next_below consumes exactly one next_u64.
#[test]
fn next_below_consumes_exactly_one_draw() {
    let mut a = SplitMix64::new(5);
    let mut b = SplitMix64::new(5);
    a.next_below(10);
    b.next_u64();
    assert_eq!(a.next_u64(), b.next_u64());
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    // DESIGN.md, SplitMix64: next_below(n) is always below n.
    #[test]
    fn next_below_stays_below_the_bound(seed in any::<u64>(), n in 1u64..=u64::MAX) {
        let mut rng = SplitMix64::new(seed);
        for _ in 0..10 {
            prop_assert!(rng.next_below(n) < n);
        }
    }

    // I7: any seed, including the largest, gives a reproducible sequence without panicking.
    #[test]
    fn any_seed_is_reproducible(seed in any::<u64>()) {
        let mut a = SplitMix64::new(seed);
        let mut b = SplitMix64::new(seed);
        for _ in 0..20 {
            prop_assert_eq!(a.next_u64(), b.next_u64());
        }
    }
}
