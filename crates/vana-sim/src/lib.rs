//! Virtual-time simulator for vana.
//!
//! This crate drives the `vana-core` decision engine on virtual time. Today
//! it provides a [`ManualClock`], the [`SplitMix64`] generator and
//! [`run_fixed`], which runs synthetic traffic against a fixed limit and
//! returns a trace of [`TraceEvent`]s. The same scenario always produces the
//! same trace, byte for byte.

mod clock;
mod rng;
mod scenario;

pub use clock::ManualClock;
pub use rng::SplitMix64;
pub use scenario::{FixedScenario, ScenarioError, TraceEvent, run_fixed, to_json_lines};
