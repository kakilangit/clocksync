//! Fault-tolerant clock synchronization using Marzullo's algorithm with
//! nanosecond-precision [`Nulid`](nulid::Nulid) timestamps.
//!
//! This crate provides a decentralized clock consensus mechanism based on
//! Marzullo's algorithm, which finds the smallest interval consistent with
//! the majority of time sources. All time values are in **nanoseconds** and
//! the [`Nulid`](nulid::Nulid) type is the core data currency throughout.
//!
//! ## Usage
//!
//! Implement [`ClockSource`] to define how your application probes remote
//! endpoints. Each probe returns a [`ProbeResponse`] containing the peer's
//! [`Nulid`](nulid::Nulid) and round-trip timing. The crate converts this
//! into nanosecond-precision [`Sample`] bounds and runs Marzullo's algorithm
//! to produce a consensus [`Interval`].
//!
//! ```ignore
//! use clocksync::{ClockSource, ProbeResponse, consensus};
//! use nulid::Nulid;
//! use std::time::Instant;
//!
//! struct MyClient;
//!
//! impl ClockSource for MyClient {
//!     type Error = std::io::Error;
//!
//!     async fn probe(&self, address: &str) -> Result<ProbeResponse, Self::Error> {
//!         let start = Instant::now();
//!         // ... send request to `address`, receive peer's Nulid ...
//!         let rtt = start.elapsed().as_nanos();
//!         let peer_id: Nulid = todo!("parse from response");
//!         Ok(ProbeResponse::new(peer_id, rtt, 0))
//!     }
//! }
//!
//! let source = MyClient;
//! let addrs = vec!["10.0.0.1:8080", "10.0.0.2:8080", "10.0.0.3:8080"];
//! let interval = consensus(&addrs, &source).await.expect("consensus found");
//!
//! // Generate a NULID from consensus time
//! let id = interval.nulid();
//! ```
//!
//! Or use [`marzullo`] directly if you already have samples:
//!
//! ```
//! use clocksync::{Sample, marzullo};
//! use nulid::Nulid;
//!
//! let id = Nulid::from_nanos(1_000_000_000, 1);
//! let samples = vec![
//!     Sample::new(100, 110, id, "peer1"),
//!     Sample::new(105, 115, id, "peer2"),
//!     Sample::new(108, 118, id, "peer3"),
//! ];
//!
//! let interval = marzullo(&samples).expect("consensus found");
//! assert_eq!(interval.lower(), 108);
//! assert_eq!(interval.upper(), 110);
//!
//! // Generate a NULID at the consensus midpoint
//! let consensus_id = interval.nulid();
//! ```

mod interval;
mod response;
mod sample;
mod source;
mod sync;

pub use interval::Interval;
pub use response::ProbeResponse;
pub use sample::Sample;
pub use source::ClockSource;
pub use sync::{consensus, marzullo};
