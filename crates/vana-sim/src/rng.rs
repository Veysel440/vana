//! A small deterministic pseudo-random number generator.

/// The SplitMix64 generator.
///
/// This is the algorithm of `splitmix64.c`, published by Sebastiano Vigna
/// (<https://prng.di.unimi.it/splitmix.c>; the old `splitmix64.c` address
/// redirects there): the state advances by a fixed
/// odd increment and each output is a mix of the new state. Every seed,
/// including 0, is valid. The same seed always gives the same sequence.
#[derive(Clone, Debug)]
pub struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    /// Creates a generator whose state starts at `seed`.
    pub fn new(seed: u64) -> SplitMix64 {
        SplitMix64 { state: seed }
    }

    /// Returns the next 64-bit output.
    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }

    /// Returns a value in `0..n`, computed as the high 64 bits of
    /// `next_u64() * n` with one draw and no rejection.
    ///
    /// The result is not exactly uniform: each value in `0..n` is hit by
    /// either `floor(2^64 / n)` or `ceil(2^64 / n)` of the 2^64 possible
    /// draws. For `n` up to 2^32 the difference between two values'
    /// probabilities is below 2^-32 of either; as `n` approaches 2^64 some
    /// values become up to twice as likely as others.
    ///
    /// # Panics
    ///
    /// Panics if `n` is 0.
    pub fn next_below(&mut self, n: u64) -> u64 {
        assert!(n != 0, "next_below: n must be at least 1");
        ((u128::from(self.next_u64()) * u128::from(n)) >> 64) as u64
    }
}
