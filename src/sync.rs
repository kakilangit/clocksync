//! Marzullo's algorithm implementation and consensus orchestration.

use crate::{Interval, Sample, source::ClockSource};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Endpoint {
    value: u128,
    is_start: bool,
}

/// Apply Marzullo's algorithm to a set of samples.
///
/// Returns the tightest interval consistent with the maximum number of sources,
/// or `None` if the input is empty. All bounds are in nanoseconds.
#[must_use]
pub fn marzullo(samples: &[Sample]) -> Option<Interval> {
    if samples.is_empty() {
        return None;
    }

    let mut endpoints: Vec<Endpoint> = Vec::with_capacity(samples.len() * 2);

    for sample in samples {
        endpoints.push(Endpoint {
            value: sample.lower_bound(),
            is_start: true,
        });
        endpoints.push(Endpoint {
            value: sample.upper_bound(),
            is_start: false,
        });
    }

    // Sort: by value ascending; ties broken by start before end
    endpoints.sort_by_key(|e| (e.value, !e.is_start));

    let mut best_count: u32 = 0;
    let mut best_start: u128 = 0;
    let mut best_end: u128 = 0;
    let mut current_count: u32 = 0;

    for endpoint in &endpoints {
        if endpoint.is_start {
            current_count += 1;
            if current_count > best_count {
                best_count = current_count;
                best_start = endpoint.value;
            }
        } else {
            if current_count == best_count && best_count > 0 {
                best_end = endpoint.value;
            }
            current_count -= 1;
        }
    }

    if best_count == 0 {
        return None;
    }

    Some(Interval::new(
        best_start,
        best_end,
        best_count,
        samples.len() as u32,
    ))
}

/// Probe all addresses via `source` and compute a consensus interval.
///
/// Each successful probe returns a [`ProbeResponse`](crate::ProbeResponse)
/// which is converted into a [`Sample`] with nanosecond bounds. Addresses
/// that fail to probe are silently skipped. Returns `None` if no samples
/// could be collected.
pub async fn consensus<S: ClockSource>(addresses: &[&str], source: &S) -> Option<Interval> {
    let mut samples = Vec::with_capacity(addresses.len());

    for address in addresses {
        if let Ok(response) = source.probe(address).await {
            samples.push(Sample::from_response(response, *address));
        }
    }

    marzullo(&samples)
}

#[cfg(test)]
mod tests {
    use nulid::Nulid;

    use super::*;
    use crate::ProbeResponse;

    fn make_nulid(nanos: u128) -> Nulid {
        Nulid::from_nanos(nanos, 1)
    }

    fn make_sample(lower: u128, upper: u128, source: &str) -> Sample {
        let mid = lower + (upper - lower) / 2;
        Sample::new(lower, upper, make_nulid(mid), source)
    }

    #[test]
    fn test_marzullo_basic() {
        let samples = vec![
            make_sample(100, 110, "peer1"),
            make_sample(105, 115, "peer2"),
            make_sample(108, 118, "peer3"),
        ];

        let result = marzullo(&samples);
        assert!(result.is_some());
        let result = result.unwrap_or_else(|| Interval::new(0, 0, 0, 0));
        assert_eq!(result.lower(), 108);
        assert_eq!(result.upper(), 110);
        assert!((result.confidence() - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_marzullo_partial_overlap() {
        let samples = vec![
            make_sample(100, 105, "peer1"),
            make_sample(104, 110, "peer2"),
            make_sample(200, 210, "peer3"),
        ];

        let result = marzullo(&samples);
        assert!(result.is_some());
        let result = result.unwrap_or_else(|| Interval::new(0, 0, 0, 0));
        assert_eq!(result.lower(), 104);
        assert_eq!(result.upper(), 105);
        assert!((result.confidence() - (2.0 / 3.0)).abs() < 0.01);
    }

    #[test]
    fn test_marzullo_no_overlap() {
        let samples = vec![
            make_sample(100, 110, "peer1"),
            make_sample(200, 210, "peer2"),
            make_sample(300, 310, "peer3"),
        ];

        let result = marzullo(&samples);
        assert!(result.is_some());
        let result = result.unwrap_or_else(|| Interval::new(0, 0, 0, 0));
        assert_eq!(result.overlapping_sources(), 1);
    }

    #[test]
    fn test_marzullo_empty() {
        let samples: Vec<Sample> = vec![];
        assert!(marzullo(&samples).is_none());
    }

    #[test]
    fn test_marzullo_single_sample() {
        let samples = vec![make_sample(100, 200, "peer1")];

        let result = marzullo(&samples);
        assert!(result.is_some());
        let result = result.unwrap_or_else(|| Interval::new(0, 0, 0, 0));
        assert_eq!(result.lower(), 100);
        assert_eq!(result.upper(), 200);
        assert!((result.confidence() - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_marzullo_full_overlap() {
        let samples = vec![
            make_sample(100, 120, "peer1"),
            make_sample(100, 120, "peer2"),
            make_sample(100, 120, "peer3"),
        ];

        let result = marzullo(&samples);
        assert!(result.is_some());
        let result = result.unwrap_or_else(|| Interval::new(0, 0, 0, 0));
        assert_eq!(result.lower(), 100);
        assert_eq!(result.upper(), 120);
        assert!((result.confidence() - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_marzullo_nanosecond_precision() {
        // Realistic nanosecond values
        let base: u128 = 1_700_000_000_000_000_000; // ~2023 in nanos
        let samples = vec![
            make_sample(base, base + 10_000_000, "peer1"), // 10ms window
            make_sample(base + 5_000_000, base + 15_000_000, "peer2"), // 10ms window, offset 5ms
            make_sample(base + 8_000_000, base + 18_000_000, "peer3"), // 10ms window, offset 8ms
        ];

        let result = marzullo(&samples);
        assert!(result.is_some());
        let result = result.unwrap_or_else(|| Interval::new(0, 0, 0, 0));
        assert_eq!(result.lower(), base + 8_000_000);
        assert_eq!(result.upper(), base + 10_000_000);
        assert!((result.confidence() - 1.0).abs() < 0.01);
    }

    struct FakeSource;

    impl ClockSource for FakeSource {
        type Error = std::io::Error;

        async fn probe(&self, address: &str) -> Result<ProbeResponse, Self::Error> {
            let base: u128 = 1_700_000_000_000_000_000;
            match address {
                "peer1" => {
                    let id = make_nulid(base + 105_000_000);
                    Ok(ProbeResponse::new(id, 10_000_000, 0))
                    // margin = 5_000_000, bounds = [100M, 110M] offset from base
                }
                "peer2" => {
                    let id = make_nulid(base + 110_000_000);
                    Ok(ProbeResponse::new(id, 10_000_000, 0))
                    // margin = 5_000_000, bounds = [105M, 115M] offset from base
                }
                "peer3" => {
                    let id = make_nulid(base + 113_000_000);
                    Ok(ProbeResponse::new(id, 10_000_000, 0))
                    // margin = 5_000_000, bounds = [108M, 118M] offset from base
                }
                "bad" => Err(std::io::Error::new(
                    std::io::ErrorKind::ConnectionRefused,
                    "unreachable",
                )),
                _ => {
                    let id = make_nulid(base);
                    Ok(ProbeResponse::new(id, 0, 0))
                }
            }
        }
    }

    #[tokio::test]
    async fn test_consensus_all_succeed() {
        let base: u128 = 1_700_000_000_000_000_000;
        let source = FakeSource;
        let addrs = vec!["peer1", "peer2", "peer3"];

        let result = consensus(&addrs, &source).await;
        assert!(result.is_some());
        let result = result.unwrap_or_else(|| Interval::new(0, 0, 0, 0));
        // peer1: [base+100M, base+110M]
        // peer2: [base+105M, base+115M]
        // peer3: [base+108M, base+118M]
        // overlap: [base+108M, base+110M]
        assert_eq!(result.lower(), base + 108_000_000);
        assert_eq!(result.upper(), base + 110_000_000);
        assert!((result.confidence() - 1.0).abs() < 0.01);
    }

    #[tokio::test]
    async fn test_consensus_with_failures() {
        let base: u128 = 1_700_000_000_000_000_000;
        let source = FakeSource;
        let addrs = vec!["peer1", "bad", "peer2"];

        let result = consensus(&addrs, &source).await;
        assert!(result.is_some());
        let result = result.unwrap_or_else(|| Interval::new(0, 0, 0, 0));
        // peer1: [base+100M, base+110M], peer2: [base+105M, base+115M]
        // overlap: [base+105M, base+110M]
        assert_eq!(result.lower(), base + 105_000_000);
        assert_eq!(result.upper(), base + 110_000_000);
        assert_eq!(result.overlapping_sources(), 2);
        assert_eq!(result.total_sources(), 2);
    }

    #[tokio::test]
    async fn test_consensus_all_fail() {
        let source = FakeSource;
        let addrs = vec!["bad", "bad"];

        assert!(consensus(&addrs, &source).await.is_none());
    }

    #[tokio::test]
    async fn test_consensus_empty_addresses() {
        let source = FakeSource;
        let addrs: Vec<&str> = vec![];

        assert!(consensus(&addrs, &source).await.is_none());
    }
}
