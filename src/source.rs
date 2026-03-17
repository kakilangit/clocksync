//! Clock source trait for abstracting remote time probing.

use std::future::Future;

use crate::ProbeResponse;

/// Trait for probing remote endpoints to obtain clock readings.
///
/// Implementors define how to reach a remote address and return a
/// [`ProbeResponse`] containing the peer's [`Nulid`](nulid::Nulid) and
/// round-trip timing information. The crate internally converts this into
/// a [`Sample`](crate::Sample) with nanosecond-precision bounds.
///
/// # Example
///
/// ```ignore
/// use clocksync::{ClockSource, ProbeResponse};
/// use nulid::Nulid;
/// use std::time::Instant;
///
/// struct MyClient;
///
/// impl ClockSource for MyClient {
///     type Error = std::io::Error;
///
///     async fn probe(&self, address: &str) -> Result<ProbeResponse, Self::Error> {
///         let start = Instant::now();
///         // ... send request to `address`, receive peer's Nulid ...
///         let rtt = start.elapsed().as_nanos();
///         let peer_id: Nulid = todo!("parse from response");
///         Ok(ProbeResponse::new(peer_id, rtt, 0))
///     }
/// }
/// ```
pub trait ClockSource {
    type Error;

    fn probe(&self, address: &str) -> impl Future<Output = Result<ProbeResponse, Self::Error>>;
}
