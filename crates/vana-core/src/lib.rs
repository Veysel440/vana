//! Decision engine of vana.
//!
//! This crate will decide what happens to each request on its way to a PHP
//! upstream: whether it is admitted, when it leaves the queue, and how many
//! requests may be in flight at once.
//!
//! Rule: no I/O, no async runtime. Both the proxy (`vana`) and the simulator
//! (`vana-sim`) will drive this crate, so it must stay pure logic.
