// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Tests for metrics aggregator module.

#[cfg(test)]
mod tests {
    use crate::metrics::aggregator::{
        calculate_stats, max, mean, min, p50, p95, p99, percentile, stddev,
    };

    #[test]
    fn test_calculate_stats() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let stats = calculate_stats(&values).unwrap();

        assert_eq!(stats.count, 5);
        assert_eq!(stats.sum, 15.0);
        assert_eq!(stats.mean, 3.0);
        assert_eq!(stats.min, 1.0);
        assert_eq!(stats.max, 5.0);
        assert_eq!(stats.p50, 3.0);
    }

    #[test]
    fn test_calculate_stats_empty() {
        let values: Vec<f64> = vec![];
        assert!(calculate_stats(&values).is_none());
    }

    #[test]
    fn test_percentile() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];

        assert_eq!(percentile(&values, 0.0), 1.0);
        assert_eq!(percentile(&values, 50.0), 3.0);
        assert_eq!(percentile(&values, 100.0), 5.0);
    }

    #[test]
    fn test_percentile_interpolation() {
        let values = vec![1.0, 2.0, 3.0, 4.0];

        // P50 should be between 2.0 and 3.0
        let p50_value = percentile(&values, 50.0);
        assert!(p50_value >= 2.0 && p50_value <= 3.0);
    }

    #[test]
    fn test_p50_p95_p99() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];

        assert_eq!(p50(&values), 5.5);
        assert!(p95(&values) >= 9.0);
        assert!(p99(&values) >= 9.5);
    }

    #[test]
    fn test_mean() {
        assert_eq!(mean(&[1.0, 2.0, 3.0, 4.0, 5.0]), 3.0);
        assert_eq!(mean(&[10.0, 20.0]), 15.0);
        assert_eq!(mean(&[]), 0.0);
    }

    #[test]
    fn test_stddev() {
        // Standard deviation of [1, 2, 3, 4, 5] is sqrt(2) ≈ 1.414
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let sd = stddev(&values);

        assert!((sd - 1.414).abs() < 0.01);
    }

    #[test]
    fn test_min_max() {
        let values = vec![3.0, 1.0, 4.0, 1.0, 5.0, 9.0, 2.0];

        assert_eq!(min(&values), 1.0);
        assert_eq!(max(&values), 9.0);
    }

    #[test]
    fn test_min_max_empty() {
        let values: Vec<f64> = vec![];

        assert_eq!(min(&values), 0.0);
        assert_eq!(max(&values), 0.0);
    }
}
