//! SplitMix64 reference values.

use vana_sim::SplitMix64;

fn first_outputs(seed: u64, count: usize) -> Vec<u64> {
    let mut rng = SplitMix64::new(seed);
    (0..count).map(|_| rng.next_u64()).collect()
}

fn below_outputs(seed: u64, n: u64, count: usize) -> Vec<u64> {
    let mut rng = SplitMix64::new(seed);
    (0..count).map(|_| rng.next_below(n)).collect()
}

// DESIGN.md, SplitMix64 (published algorithm); expected values computed independently.
#[test]
fn seed_zero_gives_the_published_sequence() {
    let expected: [u64; 5] = [
        0xe220a8397b1dcdaf,
        0x6e789e6aa1b965f4,
        0x06c45d188009454f,
        0xf88bb8a8724c81ec,
        0x1b39896a51a8749b,
    ];
    assert_eq!(first_outputs(0, 5), expected);
}

// DESIGN.md, SplitMix64 (published algorithm); expected values computed independently.
#[test]
fn seed_42_gives_the_expected_sequence() {
    let expected: [u64; 5] = [
        0xbdd732262feb6e95,
        0x28efe333b266f103,
        0x47526757130f9f52,
        0x581ce1ff0e4ae394,
        0x09bc585a244823f2,
    ];
    assert_eq!(first_outputs(42, 5), expected);
}

// DESIGN.md, SplitMix64 (published algorithm); expected values computed independently.
#[test]
fn seed_12345_gives_the_published_sequence() {
    let expected: [u64; 3] = [0x22118258a9d111a0, 0x346edce5f713f8ed, 0x1e9a57bc80e6721d];
    assert_eq!(first_outputs(12345, 3), expected);
}

// DESIGN.md, SplitMix64 (published algorithm); expected values computed independently.
#[test]
fn seed_1234567_gives_the_published_sequence() {
    let expected: [u64; 5] = [
        0x599ed017fb08fc85,
        0x2c73f08458540fa5,
        0x883ebce5a3f27c77,
        0x3fbef740e9177b3f,
        0xe3b8346708cb5ecd,
    ];
    assert_eq!(first_outputs(1234567, 5), expected);
}

// DESIGN.md, SplitMix64 (published algorithm); expected values computed independently.
#[test]
fn the_largest_seed_wraps_without_panicking() {
    let expected: [u64; 3] = [0xe4d971771b652c20, 0xe99ff867dbf682c9, 0x382ff84cb27281e9];
    assert_eq!(first_outputs(18446744073709551615, 3), expected);
}

// DESIGN.md, SplitMix64 next_below = high 64 bits of next_u64 * n; expected values computed independently.
#[test]
fn seed_7_next_below_10_gives_the_expected_values() {
    let expected: [u64; 10] = [3, 0, 9, 5, 4, 2, 4, 3, 1, 4];
    assert_eq!(below_outputs(7, 10, 10), expected);
}

// DESIGN.md, SplitMix64 next_below = high 64 bits of next_u64 * n; expected values computed independently.
#[test]
fn next_below_one_is_always_zero() {
    let expected: [u64; 5] = [0, 0, 0, 0, 0];
    assert_eq!(below_outputs(7, 1, 5), expected);
}

// DESIGN.md, SplitMix64 next_below = high 64 bits of next_u64 * n; expected values computed independently.
#[test]
fn next_below_the_largest_bound_gives_the_expected_values() {
    let expected: [u64; 5] = [
        2092789425003139052,
        12918135221727111560,
        11307387092600937728,
        1344154044715485646,
        3992596847233833365,
    ];
    assert_eq!(below_outputs(3, 18446744073709551615, 5), expected);
}

// DESIGN.md, SplitMix64 next_below = high 64 bits of next_u64 * n; expected values computed independently.
#[test]
fn next_below_two_to_the_63_gives_the_expected_values() {
    let expected: [u64; 5] = [
        7090103820010046847,
        3031610771954683960,
        5837035732972272228,
        2689419059127622713,
        7531591203003333553,
    ];
    assert_eq!(below_outputs(13, 9223372036854775808, 5), expected);
}
