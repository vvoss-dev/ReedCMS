// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Percentile and statistical aggregation for metrics.
//!
//! Provides functions for calculating:
//! - P50 (median), P95, P99 percentiles
//! - Min, max, mean, standard deviation
//! - Count and sum
//!
//! ## Performance
//! - P50/P95/P99: O(n log n) due to sorting
//! - Mean/sum: O(n) single pass
//! - All functions accept `&[f64]` for zero-copy operation

/// Statistical summary of a metric.
#[derive(Debug, Clone, PartialEq)]
pub struct MetricStats {
    pub count: usize,
    pub sum: f64,
    pub mean: f64,
    pub min: f64,
    pub max: f64,
    pub p50: f64,
    pub p95: f64,
    pub p99: f64,
    pub stddev: f64,
}

/// Calculates complete statistics for a set of values.
///
/// ## Arguments
/// - `values`: Slice of metric values
///
/// ## Returns
/// - `Some(MetricStats)`: Statistical summary
/// - `None`: If values is empty
///
/// ## Example
/// ```
/// use reedbase::metrics::aggregator::calculate_stats;
///
/// let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
/// let stats = calculate_stats(&values).unwrap();
///
/// assert_eq!(stats.mean, 3.0);
/// assert_eq!(stats.p50, 3.0);
/// ```
pub fn calculate_stats(values: &[f64]) -> Option<MetricStats> {
    if values.is_empty() {
        return None;
    }

    let count = values.len();
    let sum: f64 = values.iter().sum();
    let mean = sum / count as f64;

    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let min = sorted[0];
    let max = sorted[count - 1];
    let p50 = percentile(&sorted, 50.0);
    let p95 = percentile(&sorted, 95.0);
    let p99 = percentile(&sorted, 99.0);

    // Calculate standard deviation
    let variance: f64 = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / count as f64;
    let stddev = variance.sqrt();

    Some(MetricStats {
        count,
        sum,
        mean,
        min,
        max,
        p50,
        p95,
        p99,
        stddev,
    })
}

/// Calculates a specific percentile from sorted values.
///
/// ## Arguments
/// - `sorted_values`: Pre-sorted slice of values (ascending order)
/// - `percentile`: Target percentile (0.0 - 100.0)
///
/// ## Returns
/// Interpolated percentile value
///
/// ## Algorithm
/// Uses linear interpolation between nearest values.
///
/// ## Example
/// ```
/// use reedbase::metrics::aggregator::percentile;
///
/// let mut values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
/// values.sort_by(|a, b| a.partial_cmp(b).unwrap());
///
/// assert_eq!(percentile(&values, 50.0), 3.0); // Median
/// ```
pub fn percentile(sorted_values: &[f64], percentile: f64) -> f64 {
    if sorted_values.is_empty() {
        return 0.0;
    }

    if sorted_values.len() == 1 {
        return sorted_values[0];
    }

    let index = (percentile / 100.0) * (sorted_values.len() - 1) as f64;
    let lower_index = index.floor() as usize;
    let upper_index = index.ceil() as usize;

    if lower_index == upper_index {
        sorted_values[lower_index]
    } else {
        // Linear interpolation
        let lower_value = sorted_values[lower_index];
        let upper_value = sorted_values[upper_index];
        let fraction = index - lower_index as f64;

        lower_value + (upper_value - lower_value) * fraction
    }
}

/// Calculates P50 (median) percentile.
///
/// ## Performance
/// - O(n log n) due to sorting
/// - Allocates sorted copy of input
pub fn p50(values: &[f64]) -> f64 {
    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    percentile(&sorted, 50.0)
}

/// Calculates P95 percentile.
///
/// ## Performance
/// - O(n log n) due to sorting
/// - Allocates sorted copy of input
pub fn p95(values: &[f64]) -> f64 {
    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    percentile(&sorted, 95.0)
}

/// Calculates P99 percentile.
///
/// ## Performance
/// - O(n log n) due to sorting
/// - Allocates sorted copy of input
pub fn p99(values: &[f64]) -> f64 {
    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    percentile(&sorted, 99.0)
}

/// Calculates mean (average) of values.
///
/// ## Performance
/// - O(n) single pass
/// - No allocation
pub fn mean(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }

    let sum: f64 = values.iter().sum();
    sum / values.len() as f64
}

/// Calculates standard deviation of values.
///
/// ## Performance
/// - O(n) two passes (one for mean, one for variance)
/// - No allocation
pub fn stddev(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }

    let mean_value = mean(values);
    let variance: f64 =
        values.iter().map(|v| (v - mean_value).powi(2)).sum::<f64>() / values.len() as f64;

    variance.sqrt()
}

/// Finds minimum value.
///
/// ## Performance
/// - O(n) single pass
/// - No allocation
pub fn min(values: &[f64]) -> f64 {
    values
        .iter()
        .copied()
        .min_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap_or(0.0)
}

/// Finds maximum value.
///
/// ## Performance
/// - O(n) single pass
/// - No allocation
pub fn max(values: &[f64]) -> f64 {
    values
        .iter()
        .copied()
        .max_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap_or(0.0)
}
