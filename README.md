[![Crates.io](https://img.shields.io/crates/v/clocksync.svg)](https://crates.io/crates/clocksync)
[![Documentation](https://docs.rs/clocksync/badge.svg)](https://docs.rs/clocksync)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/kakilangit/clocksync/blob/main/LICENSE)
[![Rust Version](https://img.shields.io/badge/rust-1.88%2B-blue.svg)](https://www.rust-lang.org)
<a href="https://github.com/kakilangit/clocksync"><img alt="github" src="https://img.shields.io/badge/github-kakilangit/clocksync-37a8e0?style=for-the-badge&labelColor=555555&logo=github" height="20"></a>

# clocksync

**Fault-tolerant clock synchronization using Marzullo's algorithm with NULID nanosecond timestamps.**

`clocksync` provides a decentralized clock consensus mechanism based on Marzullo's algorithm. It finds the smallest time interval consistent with the majority of time sources, producing nanosecond-precision consensus timestamps as [NULID](https://github.com/kakilangit/nulid) identifiers.

## Features

- **Marzullo's algorithm** - Sweep-line interval consensus with O(n log n) complexity
- **Nanosecond precision** - All time values in nanoseconds via NULID timestamps
- **NULID-centric** - Probes exchange NULIDs, samples are built from NULID timestamps, consensus produces NULIDs
- **Async `ClockSource` trait** - Implement your own probing logic for any transport (HTTP, gRPC, UDP, etc.)
- **WASM ready** - Optional `wasm` feature for Cloudflare Workers and browser targets
- **Confidence tracking** - Know how many sources agree on the consensus interval
- **Memory safe** - Zero unsafe code, panic-free production paths
- **Pedantic clippy** - Strict linting enforced

---

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
clocksync = "0.2"
```

### WASM / Cloudflare Workers

Enable the `wasm` feature for WASM targets:

```toml
[dependencies]
clocksync = { version = "0.2", features = ["wasm"] }
```

This activates `nulid/wasm` under the hood, which provides `getrandom/wasm_js`
and `web-time` for timestamp generation in browser and Workers environments.

---

## Quick Start

### Implement `ClockSource`

Define how your application probes remote peers. Each probe returns a `ProbeResponse` containing the peer's NULID and round-trip timing:

```rust
use clocksync::{ClockSource, ProbeResponse};
use nulid::Nulid;
use std::time::Instant;

struct MyClient;

impl ClockSource for MyClient {
    type Error = std::io::Error;

    async fn probe(&self, address: &str) -> Result<ProbeResponse, Self::Error> {
        let start = Instant::now();
        // ... send request to `address`, receive peer's Nulid ...
        let rtt = start.elapsed().as_nanos();
        let peer_id: Nulid = todo!("parse from response");
        Ok(ProbeResponse::new(peer_id, rtt, 0))
    }
}
```

### Run Consensus

```rust
use clocksync::consensus;

let source = MyClient;
let addrs = vec!["10.0.0.1:8080", "10.0.0.2:8080", "10.0.0.3:8080"];
let interval = consensus(&addrs, &source).await;

if let Some(interval) = interval {
    // Generate a NULID at the consensus time
    let id = interval.nulid();
    println!("Consensus: [{} .. {}] ns", interval.lower(), interval.upper());
    println!("Confidence: {:.0}%", interval.confidence() * 100.0);
    println!("NULID: {}", id);
}
```

### Use `marzullo` Directly

If you already have samples:

```rust
use clocksync::{Sample, marzullo};
use nulid::Nulid;

let id = Nulid::from_nanos(1_000_000_000, 1);
let samples = vec![
    Sample::new(100, 110, id, "peer1"),
    Sample::new(105, 115, id, "peer2"),
    Sample::new(108, 118, id, "peer3"),
];

let interval = marzullo(&samples).expect("consensus found");
assert_eq!(interval.lower(), 108);
assert_eq!(interval.upper(), 110);
```

---

## How It Works

### Data Flow

```text
User implements ClockSource::probe()
    → ProbeResponse { Nulid, rtt_nanos, uncertainty_nanos }
        → Sample { lower: nanos - margin, upper: nanos + margin }
            → marzullo() sweep-line algorithm
                → Interval { lower, upper, confidence }
                    → interval.nulid()
```

### Marzullo's Algorithm

The algorithm finds the tightest interval supported by the maximum number of sources:

1. Each clock sample provides an interval `[lower, upper]` representing possible true time
2. The algorithm creates a sorted list of interval endpoints
3. A sweep-line pass finds the region where the most intervals overlap
4. The result is the consensus interval with a confidence ratio

### Bounds Computation

When a `ProbeResponse` is converted to a `Sample`, the crate computes:

```text
margin = rtt_nanos / 2 + uncertainty_nanos
lower  = peer_nanos - margin
upper  = peer_nanos + margin
```

Where `peer_nanos` is extracted from the remote peer's NULID via `nulid.nanos()`.

---

## API Reference

### Core Types

| Type | Description |
|------|-------------|
| `ProbeResponse` | Response from probing a remote peer (NULID + RTT + uncertainty) |
| `Sample` | Clock sample with nanosecond interval bounds and source identifier |
| `Interval` | Consensus result with bounds, confidence, and NULID generation |
| `ClockSource` | Async trait for implementing remote clock probing |

### Functions

| Function | Description |
|----------|-------------|
| `marzullo(&[Sample])` | Run Marzullo's algorithm on pre-collected samples |
| `consensus(&[&str], &impl ClockSource)` | Probe addresses and compute consensus |

### `ProbeResponse`

```rust
// Create from probe results
let response = ProbeResponse::new(remote_nulid, rtt_nanos, uncertainty_nanos);

response.remote_id()         // Nulid: the peer's NULID
response.remote_nanos()      // u128: peer's nanosecond timestamp
response.rtt_nanos()         // u128: round-trip time in nanoseconds
response.uncertainty_nanos() // u128: additional uncertainty
response.margin_nanos()      // u128: rtt/2 + uncertainty
```

### `Sample`

```rust
// From a probe response
let sample = Sample::from_response(response, "peer1");

// Or with explicit bounds
let sample = Sample::new(lower, upper, nulid, "peer1");

sample.lower_bound()  // u128: lower bound in nanoseconds
sample.upper_bound()  // u128: upper bound in nanoseconds
sample.remote_id()    // Nulid: the remote peer's NULID
sample.source()       // &str: source identifier
sample.width()        // u128: interval width
sample.midpoint()     // u128: interval midpoint
```

### `Interval`

```rust
interval.lower()              // u128: consensus lower bound (nanos)
interval.upper()              // u128: consensus upper bound (nanos)
interval.width()              // u128: interval width (nanos)
interval.midpoint()           // u128: best estimate of true time (nanos)
interval.confidence()         // f64: overlapping / total sources
interval.overlapping_sources() // u32: number of agreeing sources
interval.total_sources()      // u32: total sources sampled
interval.nulid()              // Nulid: NULID at consensus midpoint
```

---

## Development

### Building

```sh
make build
```

### Testing

```sh
make test
```

### Linting

```sh
make clippy
```

### All CI Checks

```sh
make ci
```

---

## License

Licensed under the MIT License. See [LICENSE](https://github.com/kakilangit/clocksync/blob/main/LICENSE) for details.
