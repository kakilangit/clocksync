//! Probe response from a remote clock source.

use nulid::Nulid;

/// Response from probing a remote time source.
///
/// Contains the remote peer's [`Nulid`] (whose nanosecond timestamp represents
/// the peer's current time) along with round-trip timing information used to
/// compute uncertainty bounds.
///
/// The crate converts a `ProbeResponse` into a [`Sample`](crate::Sample) by
/// extracting the NULID's nanosecond timestamp and applying RTT-based
/// uncertainty:
///
/// ```text
/// lower = peer_nanos - rtt_nanos / 2 - uncertainty_nanos
/// upper = peer_nanos + rtt_nanos / 2 + uncertainty_nanos
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProbeResponse {
    remote_id: Nulid,
    rtt_nanos: u128,
    uncertainty_nanos: u128,
}

impl ProbeResponse {
    /// Create a new probe response.
    ///
    /// # Arguments
    ///
    /// * `remote_id` — the NULID received from the remote peer (its timestamp
    ///   component is the peer's clock reading in nanoseconds).
    /// * `rtt_nanos` — round-trip time of the probe exchange, in nanoseconds.
    /// * `uncertainty_nanos` — additional uncertainty reported by the peer, in
    ///   nanoseconds (pass `0` if the peer does not report uncertainty).
    #[must_use]
    pub const fn new(remote_id: Nulid, rtt_nanos: u128, uncertainty_nanos: u128) -> Self {
        Self {
            remote_id,
            rtt_nanos,
            uncertainty_nanos,
        }
    }

    /// The NULID received from the remote peer.
    #[must_use]
    pub const fn remote_id(&self) -> Nulid {
        self.remote_id
    }

    /// Nanosecond timestamp extracted from the remote NULID.
    #[must_use]
    pub fn remote_nanos(&self) -> u128 {
        self.remote_id.nanos()
    }

    /// Round-trip time in nanoseconds.
    #[must_use]
    pub const fn rtt_nanos(&self) -> u128 {
        self.rtt_nanos
    }

    /// Additional uncertainty in nanoseconds.
    #[must_use]
    pub const fn uncertainty_nanos(&self) -> u128 {
        self.uncertainty_nanos
    }

    /// Total one-sided error margin: `rtt / 2 + uncertainty`.
    #[must_use]
    pub const fn margin_nanos(&self) -> u128 {
        self.rtt_nanos / 2 + self.uncertainty_nanos
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_nulid(nanos: u128) -> Nulid {
        Nulid::from_nanos(nanos, 1)
    }

    #[test]
    fn test_probe_response_creation() {
        let nanos: u128 = 1_700_000_000_000_000_000;
        let id = make_nulid(nanos);
        let resp = ProbeResponse::new(id, 2_000_000, 500_000);
        assert_eq!(resp.remote_id(), id);
        assert_eq!(resp.rtt_nanos(), 2_000_000);
        assert_eq!(resp.uncertainty_nanos(), 500_000);
    }

    #[test]
    fn test_probe_response_remote_nanos() {
        let nanos: u128 = 1_700_000_000_000_000_000;
        let id = Nulid::from_nanos(nanos, 42);
        let resp = ProbeResponse::new(id, 0, 0);
        assert_eq!(resp.remote_nanos(), nanos);
    }

    #[test]
    fn test_probe_response_margin() {
        let id = make_nulid(1_000_000_000_000_000_000);
        let resp = ProbeResponse::new(id, 10_000_000, 1_000_000);
        // margin = 10_000_000 / 2 + 1_000_000 = 6_000_000
        assert_eq!(resp.margin_nanos(), 6_000_000);
    }

    #[test]
    fn test_probe_response_zero_uncertainty() {
        let id = make_nulid(1_000_000_000_000_000_000);
        let resp = ProbeResponse::new(id, 4_000_000, 0);
        assert_eq!(resp.margin_nanos(), 2_000_000);
    }
}
