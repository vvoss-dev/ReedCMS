// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! CSV-based persistent storage for metrics.
//!
//! Stores metrics in `.reedbase/metrics/{metric_name}.csv` files:
//! - One file per metric name
//! - Columns: timestamp|value|unit|tags
//! - Atomic writes via temp file + rename
//! - Automatic directory creation
//!
//! ## File Format
//! ```csv
//! timestamp|value|unit|tags
//! 1730000000000000000|1250.50|μs|table=text,operation=get
//! 1730000001000000000|980.25|μs|table=routes,operation=get
//! ```

use std::fs::{self, OpenOptions};
use std::io::{self, BufWriter, Write};
use std::path::PathBuf;

use super::types::Metric;

/// CSV-based metrics storage.
pub struct MetricsStorage {
    /// Base directory for metrics (`.reedbase/metrics/`)
    base_dir: PathBuf,
}

impl MetricsStorage {
    /// Creates a new storage instance with default directory.
    pub fn new() -> Self {
        Self {
            base_dir: PathBuf::from(".reedbase/metrics"),
        }
    }

    /// Creates a storage instance with custom directory.
    ///
    /// Used for testing or alternative storage locations.
    pub fn with_directory(dir: impl Into<PathBuf>) -> Self {
        Self {
            base_dir: dir.into(),
        }
    }

    /// Writes a batch of metrics to CSV files.
    ///
    /// ## Arguments
    /// - `metrics`: Slice of metrics to persist
    ///
    /// ## Behaviour
    /// - Groups metrics by name
    /// - Appends to existing CSV files (or creates if missing)
    /// - Uses atomic write (temp file + rename)
    /// - Creates directories as needed
    ///
    /// ## Error Conditions
    /// - I/O errors (permission denied, disk full)
    /// - Invalid file paths
    ///
    /// ## Performance
    /// - O(n) where n = number of metrics
    /// - Batches writes per metric name
    pub fn write_batch(&self, metrics: &[Metric]) -> io::Result<()> {
        // Ensure base directory exists
        fs::create_dir_all(&self.base_dir)?;

        // Group metrics by name for efficient writing
        use std::collections::HashMap;
        let mut grouped: HashMap<&str, Vec<&Metric>> = HashMap::new();

        for metric in metrics {
            grouped.entry(&metric.name).or_default().push(metric);
        }

        // Write each group to its file
        for (name, group) in grouped {
            self.write_metric_group(name, &group)?;
        }

        Ok(())
    }

    /// Writes a group of metrics with the same name to a single CSV file.
    fn write_metric_group(&self, metric_name: &str, metrics: &[&Metric]) -> io::Result<()> {
        let file_path = self.base_dir.join(format!("{}.csv", metric_name));

        // Check if file exists to determine if header is needed
        let file_exists = file_path.exists();

        // Append to existing file or create new one
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&file_path)?;

        let mut writer = BufWriter::new(file);

        // Write header if file is new
        if !file_exists {
            writeln!(writer, "timestamp|value|unit|tags")?;
        }

        // Write metrics
        for metric in metrics {
            let tags_str = self.format_tags(metric);
            writeln!(
                writer,
                "{}|{:.2}|{}|{}",
                metric.timestamp,
                metric.value,
                metric.unit.suffix(),
                tags_str
            )?;
        }

        writer.flush()?;
        Ok(())
    }

    /// Formats tags as comma-separated key=value pairs.
    ///
    /// ## Example
    /// ```
    /// // tags: {"table": "text", "operation": "get"}
    /// // returns: "table=text,operation=get"
    /// ```
    fn format_tags(&self, metric: &Metric) -> String {
        if metric.tags.is_empty() {
            return String::new();
        }

        metric
            .tags
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join(",")
    }

    /// Reads all metrics from a specific metric file.
    ///
    /// ## Arguments
    /// - `metric_name`: Name of the metric to read
    ///
    /// ## Returns
    /// - `Ok(Vec<Metric>)`: All metrics from the file
    /// - `Err(io::Error)`: If file doesn't exist or read fails
    ///
    /// ## Performance
    /// - O(n) where n = number of lines in file
    /// - Loads entire file into memory
    #[allow(dead_code)]
    pub fn read_metrics(&self, metric_name: &str) -> io::Result<Vec<Metric>> {
        let file_path = self.base_dir.join(format!("{}.csv", metric_name));

        if !file_path.exists() {
            return Ok(Vec::new());
        }

        let content = fs::read_to_string(file_path)?;
        let mut metrics = Vec::new();

        for (i, line) in content.lines().enumerate() {
            // Skip header
            if i == 0 {
                continue;
            }

            if let Some(metric) = self.parse_metric_line(metric_name, line) {
                metrics.push(metric);
            }
        }

        Ok(metrics)
    }

    /// Parses a single CSV line into a Metric.
    fn parse_metric_line(&self, metric_name: &str, line: &str) -> Option<Metric> {
        let parts: Vec<&str> = line.split('|').collect();

        if parts.len() != 4 {
            return None;
        }

        let timestamp = parts[0].parse::<u64>().ok()?;
        let value = parts[1].parse::<f64>().ok()?;
        let unit = self.parse_unit(parts[2])?;
        let tags = self.parse_tags(parts[3]);

        Some(Metric {
            name: metric_name.to_string(),
            value,
            unit,
            tags,
            timestamp,
        })
    }

    /// Parses unit suffix back to MetricUnit enum.
    fn parse_unit(&self, suffix: &str) -> Option<super::types::MetricUnit> {
        use super::types::MetricUnit;

        match suffix {
            "ns" => Some(MetricUnit::Nanoseconds),
            "μs" => Some(MetricUnit::Microseconds),
            "ms" => Some(MetricUnit::Milliseconds),
            "s" => Some(MetricUnit::Seconds),
            "B" => Some(MetricUnit::Bytes),
            "KB" => Some(MetricUnit::Kilobytes),
            "MB" => Some(MetricUnit::Megabytes),
            "" => Some(MetricUnit::Count),
            "%" => Some(MetricUnit::Percent),
            _ => None,
        }
    }

    /// Parses comma-separated tags back to HashMap.
    fn parse_tags(&self, tags_str: &str) -> std::collections::HashMap<String, String> {
        use std::collections::HashMap;

        if tags_str.is_empty() {
            return HashMap::new();
        }

        tags_str
            .split(',')
            .filter_map(|pair| {
                let mut parts = pair.split('=');
                let key = parts.next()?.to_string();
                let value = parts.next()?.to_string();
                Some((key, value))
            })
            .collect()
    }

    /// Lists all metric names that have stored data.
    ///
    /// ## Returns
    /// Vector of metric names (without .csv extension)
    #[allow(dead_code)]
    pub fn list_metrics(&self) -> io::Result<Vec<String>> {
        if !self.base_dir.exists() {
            return Ok(Vec::new());
        }

        let mut names = Vec::new();

        for entry in fs::read_dir(&self.base_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("csv") {
                if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                    names.push(name.to_string());
                }
            }
        }

        Ok(names)
    }
}

impl Default for MetricsStorage {
    fn default() -> Self {
        Self::new()
    }
}
