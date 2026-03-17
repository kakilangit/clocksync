//! Consensus interval result from Marzullo's algorithm.

use std::fmt;

use nulid::Nulid;
use rand::Rng;

/// The consensus time interval produced by Marzullo's algorithm.
///
/// All time values are in **nanoseconds**. Use [`nulid()`](Self::nulid) to
/// generate a [`Nulid`] from the consensus midpoint.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Interval {
    lower: u128,
    upper: u128,
    confidence: u32,
    total_sources: u32,
}

impl Interval {
    #[must_use]
    pub const fn new(lower: u128, upper: u128, confidence: u32, total_sources: u32) -> Self {
        // const fn cannot call .max(), so use a ternary-style branch
        let upper = if upper > lower { upper } else { lower };
        Self {
            lower,
            upper,
            confidence,
            total_sources,
        }
    }

    /// Lower bound of the consensus interval in nanoseconds.
    #[must_use]
    pub const fn lower(&self) -> u128 {
        self.lower
    }

    /// Upper bound of the consensus interval in nanoseconds.
    #[must_use]
    pub const fn upper(&self) -> u128 {
        self.upper
    }

    /// Width of the interval in nanoseconds.
    #[must_use]
    pub const fn width(&self) -> u128 {
        self.upper.saturating_sub(self.lower)
    }

    /// Midpoint (best estimate of true time) in nanoseconds.
    #[must_use]
    pub const fn midpoint(&self) -> u128 {
        self.lower + self.width() / 2
    }

    /// Confidence ratio: overlapping sources / total sources.
    #[must_use]
    pub fn confidence(&self) -> f64 {
        if self.total_sources == 0 {
            return 0.0;
        }
        f64::from(self.confidence) / f64::from(self.total_sources)
    }

    /// Number of sources whose intervals overlap at this consensus region.
    #[must_use]
    pub const fn overlapping_sources(&self) -> u32 {
        self.confidence
    }

    /// Total number of sources that contributed samples.
    #[must_use]
    pub const fn total_sources(&self) -> u32 {
        self.total_sources
    }

    /// Generate a [`Nulid`] using the consensus midpoint as the timestamp.
    ///
    /// The midpoint of the interval (best estimate of true time in nanoseconds)
    /// becomes the NULID's timestamp component, with cryptographically secure
    /// random bits filling the remaining 60 bits.
    #[must_use]
    pub fn nulid(&self) -> Nulid {
        let nanos = self.midpoint();
        let random = rand::rng().random::<u64>() & ((1u64 << 60) - 1);
        Nulid::from_nanos(nanos, random)
    }
}

impl fmt::Display for Interval {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Interval([{}..{}] ns, {:.2}%)",
            self.lower,
            self.upper,
            self.confidence() * 100.0
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interval_creation() {
        let i = Interval::new(100, 200, 3, 5);
        assert_eq!(i.lower(), 100);
        assert_eq!(i.upper(), 200);
    }

    #[test]
    fn test_interval_width() {
        let i = Interval::new(100, 200, 3, 5);
        assert_eq!(i.width(), 100);
    }

    #[test]
    fn test_interval_midpoint() {
        let i = Interval::new(100, 200, 3, 5);
        assert_eq!(i.midpoint(), 150);
    }

    #[test]
    #[allow(clippy::float_cmp)]
    fn test_interval_confidence() {
        let i = Interval::new(100, 200, 3, 5);
        assert!((i.confidence() - 0.6).abs() < 0.01);
    }

    #[test]
    fn test_interval_confidence_zero_sources() {
        let i = Interval::new(100, 200, 0, 0);
        assert!(i.confidence().abs() < f64::EPSILON);
    }

    #[test]
    fn test_interval_nulid() {
        let lower: u128 = 1_234_567_880_000_000_000;
        let upper: u128 = 1_234_567_900_000_000_000;
        let i = Interval::new(lower, upper, 3, 3);
        let id = i.nulid();
        assert_eq!(id.nanos(), i.midpoint());
    }

    #[test]
    fn test_interval_nulid_uniqueness() {
        let i = Interval::new(1_000_000_000_000_000, 2_000_000_000_000_000, 3, 3);
        let a = i.nulid();
        let b = i.nulid();
        // Same timestamp, different random bits
        assert_eq!(a.nanos(), b.nanos());
        assert_ne!(a, b);
    }

    #[test]
    fn test_interval_bounds_normalization() {
        let i = Interval::new(200, 100, 1, 1);
        assert_eq!(i.lower(), 200);
        assert_eq!(i.upper(), 200);
    }
}
