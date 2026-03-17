# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.0] - 2026-03-17

### Added

- **`wasm` feature flag** - Enables WASM/Cloudflare Workers compatibility
  - Passes through `nulid/wasm` to activate `getrandom/wasm_js` and `web-time`
  - Core algorithm (`marzullo`, `consensus`, all types) works unchanged on WASM targets

### Changed

- **`ClockSource::probe()` no longer requires `Send` on the returned future** (**breaking**)
  - Was: `fn probe(&self, address: &str) -> impl Future<...> + Send`
  - Now: `fn probe(&self, address: &str) -> impl Future<...>`
  - This enables implementing `ClockSource` in single-threaded environments
    (e.g. Cloudflare Workers) where futures from platform APIs are `!Send`
  - Callers using tokio multi-threaded runtime are unaffected if the
    implementing type's future is already `Send` (RPITIT captures auto-traits)
- **`nulid` dependency** changed to `default-features = false, features = ["std"]`
  - Prevents pulling in the `quanta` native clock backend by default
  - Native builds retain full functionality via `std` feature
  - WASM builds activate `web-time` backend via the `wasm` feature

## [0.1.0] - 2026-03-17

### Added

- **Marzullo's algorithm** - Sweep-line interval consensus implementation
  - O(n log n) complexity for finding the tightest interval consistent with the maximum number of sources
  - Returns consensus interval with confidence ratio (overlapping / total sources)
  - Handles edge cases: empty input, single sample, no overlap, full overlap

- **NULID-centric design** - All time values in nanoseconds via NULID
  - `ProbeResponse` type for probe results containing remote peer's `Nulid`, RTT, and uncertainty
  - `Sample` type with `u128` nanosecond bounds built from NULID timestamps + RTT margins
  - `Interval` type with `nulid()` method to generate a NULID at consensus midpoint time
  - Bounds computation: `peer_nanos +/- (rtt/2 + uncertainty)`

- **Async `ClockSource` trait** - User-implementable trait for remote clock probing
  - `probe(&self, address: &str) -> Result<ProbeResponse, Self::Error>`
  - Transport-agnostic: works with HTTP, gRPC, UDP, or any async protocol

- **`consensus()` async function** - Orchestrates probing and Marzullo execution
  - Probes all addresses, silently skips failures
  - Converts `ProbeResponse` into `Sample` with nanosecond bounds
  - Returns `Option<Interval>` (None if no samples collected)

- **Strict code quality**
  - No `panic!`, `unwrap()`, or `expect()` in production code
  - Pedantic clippy linting enforced
  - 30 unit tests covering all types and edge cases

- **CI/CD** - GitHub Actions workflows
  - CI: format check, clippy, tests, doc tests
  - Release: version verification, crates.io publishing, GitHub release creation

[Unreleased]: https://github.com/kakilangit/clocksync/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/kakilangit/clocksync/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/kakilangit/clocksync/releases/tag/v0.1.0
