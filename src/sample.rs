//! Clock sample from a time source.

use std::fmt;

use nulid::Nulid;

use crate::ProbeResponse;

/// A clock sample representing an interval of possible true time in nanoseconds.
///
/// Built from a [`ProbeResponse`] by extracting the NULID's nanosecond
/// timestamp and applying RTT-based uncertainty bounds:
///
/// ```text
/// lower = peer_nanos - margin
/// upper = peer_nanos + margin
/// ```
///
/// where `margin = rtt_nanos / 2 + uncertainty_nanos`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sample {
    lower_bound: u128,
    upper_bound: u128,
    remote_id: Nulid,
    source: String,
}

impl Sample {
    /// Create a sample from a [`ProbeResponse`] and a source identifier.
    ///
    /// The bounds are computed from the NULID's nanosecond timestamp and the
    /// response's RTT / uncertainty margins.
    #[must_use]
    pub fn from_response(response: ProbeResponse, source: impl Into<String>) -> Self {
        let peer_nanos = response.remote_nanos();
        let margin = response.margin_nanos();

        Self {
            lower_bound: peer_nanos.saturating_sub(margin),
            upper_bound: peer_nanos.saturating_add(margin),
            remote_id: response.remote_id(),
            source: source.into(),
        }
    }

    /// Create a sample with explicit nanosecond bounds.
    ///
    /// Useful when you already have computed bounds (e.g. from
    /// [`marzullo`](crate::marzullo) tests or manual construction).
    #[must_use]
    pub fn new(
        lower_bound: u128,
        upper_bound: u128,
        remote_id: Nulid,
        source: impl Into<String>,
    ) -> Self {
        Self {
            lower_bound,
            upper_bound: upper_bound.max(lower_bound),
            remote_id,
            source: source.into(),
        }
    }

    /// Lower bound of the time interval in nanoseconds.
    #[must_use]
    pub const fn lower_bound(&self) -> u128 {
        self.lower_bound
    }

    /// Upper bound of the time interval in nanoseconds.
    #[must_use]
    pub const fn upper_bound(&self) -> u128 {
        self.upper_bound
    }

    /// The NULID received from the remote peer.
    #[must_use]
    pub const fn remote_id(&self) -> Nulid {
        self.remote_id
    }

    /// Identifier of the source that produced this sample.
    #[must_use]
    pub fn source(&self) -> &str {
        &self.source
    }

    /// Width of the interval in nanoseconds.
    #[must_use]
    pub const fn width(&self) -> u128 {
        self.upper_bound.saturating_sub(self.lower_bound)
    }

    /// Midpoint of the interval in nanoseconds.
    #[must_use]
    pub const fn midpoint(&self) -> u128 {
        self.lower_bound + self.width() / 2
    }
}

impl fmt::Display for Sample {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Sample([{}..{}] ns, {}, {})",
            self.lower_bound, self.upper_bound, self.remote_id, self.source
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_nulid(nanos: u128) -> Nulid {
        Nulid::from_nanos(nanos, 1)
    }

    #[test]
    fn test_sample_from_response() {
        let nanos: u128 = 1_700_000_000_000_000_000;
        let id = make_nulid(nanos);
        let resp = ProbeResponse::new(id, 10_000_000, 1_000_000);
        let s = Sample::from_response(resp, "peer1");

        // margin = 10_000_000 / 2 + 1_000_000 = 6_000_000
        assert_eq!(s.lower_bound(), nanos - 6_000_000);
        assert_eq!(s.upper_bound(), nanos + 6_000_000);
        assert_eq!(s.remote_id(), id);
        assert_eq!(s.source(), "peer1");
    }

    #[test]
    fn test_sample_from_response_zero_rtt() {
        let nanos: u128 = 1_000_000_000_000_000_000;
        let id = make_nulid(nanos);
        let resp = ProbeResponse::new(id, 0, 0);
        let s = Sample::from_response(resp, "peer1");

        assert_eq!(s.lower_bound(), nanos);
        assert_eq!(s.upper_bound(), nanos);
        assert_eq!(s.width(), 0);
    }

    #[test]
    fn test_sample_new_explicit() {
        let id = make_nulid(500);
        let s = Sample::new(100, 200, id, "peer1");
        assert_eq!(s.lower_bound(), 100);
        assert_eq!(s.upper_bound(), 200);
    }

    #[test]
    fn test_sample_bounds_normalization() {
        let id = make_nulid(500);
        let s = Sample::new(200, 100, id, "peer1");
        assert_eq!(s.lower_bound(), 200);
        assert_eq!(s.upper_bound(), 200);
    }

    #[test]
    fn test_sample_width() {
        let id = make_nulid(500);
        let s = Sample::new(100, 150, id, "peer1");
        assert_eq!(s.width(), 50);
    }

    #[test]
    fn test_sample_midpoint() {
        let id = make_nulid(500);
        let s = Sample::new(100, 200, id, "peer1");
        assert_eq!(s.midpoint(), 150);
    }

    #[test]
    fn test_sample_saturating_lower_bound() {
        let id = make_nulid(1000);
        let resp = ProbeResponse::new(id, 10_000, 0);
        let s = Sample::from_response(resp, "peer1");
        // peer_nanos = 1000, margin = 5000
        // lower would be negative without saturation
        assert_eq!(s.lower_bound(), 0);
        assert_eq!(s.upper_bound(), 6000);
    }
}
