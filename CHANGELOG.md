# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased]

### Added

- Project skeleton, CI and design notes.
- `vana-core`: `Nanos` (saturating time) and the `Clock` trait.
- `vana-core`: `TenantId`, `TicketId` and `Ticket` with an optional deadline.
- `vana-core`: `Permit`, released explicitly; dropping one without release
  panics in debug builds.
- `vana-core`: the `Limiter` trait and `FixedLimiter`, which grants a permit
  only while in_flight is below the limit.
- `vana-sim`: `ManualClock`, the `SplitMix64` generator and `run_fixed`, which
  runs synthetic traffic against a fixed limit and returns a trace that can be
  printed as JSON lines. The same scenario gives the same trace.
- Property tests with `proptest` (dev-dependency only) and golden traces.

### Changed

- CI runs clippy and tests with `--locked` and builds the documentation with
  warnings denied.
- Design notes describe the built API, the simulation rules and invariants
  I1 and I2 as implemented.
