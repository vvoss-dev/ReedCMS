# REED-19-01A: Metrics & Observability Infrastructure

## MANDATORY Development Standards

**CRITICAL**: Every implementation MUST follow these standards:

- **Language**: All documentation and code comments in BBC English
- **Principle**: KISS (Keep It Simple, Stupid) - minimal code and professional inline documentation
- **File Naming**: File name = Unique theme - crystal clear what single topic this file handles
- **Files**: One file = One responsibility
- **Functions**: One function = One distinctive job
- **Shared Functions**: Same patterns = One shared function
- **Testing**: Separate test files as `{name}_test.rs` - never inline `#[cfg(test)]` modules
- **Avoid**: Avoid Swiss Army knife functions
- **Avoid**: Generic file names like `handler.rs`, `utils.rs`
- **Templates**: See `_workbench/Tickets/templates/service-template.md` for complete implementation guide
- **Testing**: See `_workbench/Tickets/templates/service-template.test.md` for test structure

## Ticket Information
- **ID**: REED-19-01A
- **Title**: Metrics & Observability Infrastructure
- **Layer**: ReedBase Layer (REED-19)
- **Priority**: Critical
- **Status**: Open
- **Complexity**: Medium
- **Dependencies**: None (foundation for all tickets)
- **Estimated Time**: 6 hours

## Objective

Implement reusable, KISS-compliant metrics infrastructure that ALL ReedBase modules will use. ONE implementation, no copy-paste code across tickets.

## Requirements

### Core Principles

1. **Singleton Pattern**: ONE global MetricsCollector instance
2. **Thread-Safe**: RwLock for concurrent access
3. **Minimal API**: Modules call `metrics().record()` - nothing complex
4. **Zero Copy-Paste**: All tickets use THIS module, no duplicated code

### File Structure

```
src/reedcms/reedbase/metrics/
├── mod.rs                    # Public API exports
├── types.rs                  # Core types (Metric, MetricType, MetricUnit)
├── collector.rs              # MetricsCollector singleton
├── storage.rs                # CSV backend for persistence
└── aggregator.rs             # P50/P95/P99 calculation
```

### Storage Format

```
.reedbase/metrics/
├── {module}.csv              # Per-module raw metrics
├── aggregates/               # Pre-computed aggregates
│   ├── 5min/                 # 5-minute rolling windows
│   └── 1hour/                # Hourly aggregates
└── alerts.log                # Triggered alerts
```

## Implementation Files

### Primary Implementation

**`src/reedcms/reedbase/metrics/mod.rs`**

```rust
// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Metrics and observability infrastructure for ReedBase.
//!
//! Provides thread-safe, singleton-based metrics collection.

mod types;
mod collector;
mod storage;
mod aggregator;

pub use types::{Metric, MetricType, MetricUnit};
pub use collector::{global, MetricsCollector};

/// Get global metrics collector.
///
/// ## Example Usage
/// ```rust
/// use crate::reedbase::metrics::global as metrics;
///
/// metrics().record(Metric {
///     name: "table_read_latency".to_string(),
///     value: 123.45,
///     unit: MetricUnit::Microseconds,
///     tags: hashmap!{ "table" => "text" },
/// });
/// ```
pub fn global() -> &'static MetricsCollector {
    collector::global()
}
```

**`src/reedcms/reedbase/metrics/types.rs`**

```rust
// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Core metric types.

use std::collections::HashMap;

/// Single metric data point.
#[derive(Debug, Clone)]
pub struct Metric {
    pub name: String,
    pub value: f64,
    pub unit: MetricUnit,
    pub tags: HashMap<String, String>,
}

impl Metric {
    /// Create new metric with automatic timestamp.
    pub fn new(name: String, value: f64, unit: MetricUnit) -> Self {
        Self {
            name,
            value,
            unit,
            tags: HashMap::new(),
        }
    }
    
    /// Add tag to metric.
    pub fn with_tag(mut self, key: &str, value: &str) -> Self {
        self.tags.insert(key.to_string(), value.to_string());
        self
    }
}

/// Metric type (how to aggregate).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetricType {
    /// Monotonically increasing counter.
    Counter,
    
    /// Point-in-time value.
    Gauge,
    
    /// Distribution (P50/P95/P99).
    Histogram,
    
    /// Duration measurement.
    Timer,
}

/// Metric unit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetricUnit {
    Nanoseconds,
    Microseconds,
    Milliseconds,
    Seconds,
    Bytes,
    Kilobytes,
    Megabytes,
    Count,
    Percent,
}

impl MetricUnit {
    /// Get unit suffix for display.
    pub fn suffix(&self) -> &'static str {
        match self {
            Self::Nanoseconds => "ns",
            Self::Microseconds => "μs",
            Self::Milliseconds => "ms",
            Self::Seconds => "s",
            Self::Bytes => "B",
            Self::Kilobytes => "KB",
            Self::Megabytes => "MB",
            Self::Count => "",
            Self::Percent => "%",
        }
    }
}
```

**`src/reedcms/reedbase/metrics/collector.rs`**

```rust
// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Global metrics collector (singleton).

use std::sync::{Arc, RwLock, OnceLock};
use std::collections::HashMap;
use super::types::{Metric, MetricUnit};
use super::storage;

/// Global metrics collector instance.
static COLLECTOR: OnceLock<MetricsCollector> = OnceLock::new();

/// Get global metrics collector.
pub fn global() -> &'static MetricsCollector {
    COLLECTOR.get_or_init(|| MetricsCollector::new())
}

/// Thread-safe metrics collector.
pub struct MetricsCollector {
    buffer: Arc<RwLock<Vec<Metric>>>,
}

impl MetricsCollector {
    fn new() -> Self {
        Self {
            buffer: Arc::new(RwLock::new(Vec::with_capacity(1000))),
        }
    }
    
    /// Record a metric value.
    ///
    /// ## Input
    /// - `metric`: Metric to record
    ///
    /// ## Performance
    /// - < 100ns typical (in-memory append)
    /// - Automatic flush at 1000 metrics
    ///
    /// ## Example Usage
    /// ```rust
    /// use crate::reedbase::metrics::global as metrics;
    ///
    /// metrics().record(Metric {
    ///     name: "table_read_latency".to_string(),
    ///     value: 123.45,
    ///     unit: MetricUnit::Microseconds,
    ///     tags: hashmap!{ "table" => "text" },
    /// });
    /// ```
    pub fn record(&self, metric: Metric) {
        let mut buffer = self.buffer.write().unwrap();
        buffer.push(metric);
        
        // Flush if buffer full
        if buffer.len() >= 1000 {
            let _ = storage::write_metrics(&buffer);
            buffer.clear();
        }
    }
    
    /// Increment a counter by 1.
    ///
    /// ## Input
    /// - `name`: Metric name
    /// - `tags`: Optional tags
    ///
    /// ## Example Usage
    /// ```rust
    /// metrics().increment("lock_timeout_count", hashmap!{
    ///     "table" => "users"
    /// });
    /// ```
    pub fn increment(&self, name: &str, tags: HashMap<String, String>) {
        self.record(Metric {
            name: name.to_string(),
            value: 1.0,
            unit: MetricUnit::Count,
            tags,
        });
    }
    
    /// Set a gauge value.
    ///
    /// ## Input
    /// - `name`: Metric name
    /// - `value`: Gauge value
    /// - `tags`: Optional tags
    ///
    /// ## Example Usage
    /// ```rust
    /// metrics().gauge("queue_depth", 42.0, hashmap!{
    ///     "table" => "routes"
    /// });
    /// ```
    pub fn gauge(&self, name: &str, value: f64, tags: HashMap<String, String>) {
        self.record(Metric {
            name: name.to_string(),
            value,
            unit: MetricUnit::Count,
            tags,
        });
    }
    
    /// Force flush buffer to disk.
    ///
    /// Called automatically at 1000 metrics, but can be called manually.
    ///
    /// ## Performance
    /// - < 10ms typical
    pub fn flush(&self) -> std::io::Result<()> {
        let mut buffer = self.buffer.write().unwrap();
        storage::write_metrics(&buffer)?;
        buffer.clear();
        Ok(())
    }
}
```

**`src/reedcms/reedbase/metrics/storage.rs`**

```rust
// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! CSV storage backend for metrics.

use std::fs::{OpenOptions, create_dir_all};
use std::io::{Write, Result};
use std::path::{Path, PathBuf};
use super::types::Metric;

/// Get metrics directory path.
fn metrics_dir() -> PathBuf {
    PathBuf::from(".reedbase/metrics")
}

/// Write metrics to CSV.
///
/// ## Input
/// - `metrics`: Metrics to write
///
/// ## Output
/// - `Result<()>`: Success or IO error
///
/// ## Performance
/// - < 10ms for 1000 metrics
///
/// ## Format
/// ```csv
/// timestamp|metric_name|value|unit|tags
/// 1697123456|table_read_latency|123.45|μs|table=text,cache_hit=true
/// ```
pub fn write_metrics(metrics: &[Metric]) -> Result<()> {
    if metrics.is_empty() {
        return Ok(());
    }
    
    let dir = metrics_dir();
    create_dir_all(&dir)?;
    
    // Group by metric name (one CSV per metric)
    let mut groups: std::collections::HashMap<String, Vec<&Metric>> = 
        std::collections::HashMap::new();
    
    for metric in metrics {
        groups.entry(metric.name.clone())
            .or_insert_with(Vec::new)
            .push(metric);
    }
    
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    // Write each group to separate CSV
    for (name, group) in groups {
        let path = dir.join(format!("{}.csv", name));
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)?;
        
        for metric in group {
            let tags_str = metric.tags.iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect::<Vec<_>>()
                .join(",");
            
            writeln!(
                file,
                "{}|{}|{}|{}|{}",
                timestamp,
                metric.name,
                metric.value,
                metric.unit.suffix(),
                tags_str
            )?;
        }
    }
    
    Ok(())
}
```

**`src/reedcms/reedbase/metrics/aggregator.rs`**

```rust
// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Percentile and aggregation calculations.

/// Calculate percentiles from values.
///
/// ## Input
/// - `values`: Sorted values
/// - `percentile`: 0.0 to 1.0 (e.g., 0.99 for P99)
///
/// ## Output
/// - `f64`: Percentile value
///
/// ## Performance
/// - O(1) if values already sorted
pub fn percentile(values: &[f64], percentile: f64) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    
    let index = ((values.len() - 1) as f64 * percentile) as usize;
    values[index]
}

/// Calculate P50, P95, P99 from values.
///
/// ## Input
/// - `values`: Unsorted values
///
/// ## Output
/// - `(f64, f64, f64)`: (P50, P95, P99)
pub fn percentiles(values: &mut [f64]) -> (f64, f64, f64) {
    if values.is_empty() {
        return (0.0, 0.0, 0.0);
    }
    
    values.sort_by(|a, b| a.partial_cmp(b).unwrap());
    
    (
        percentile(values, 0.50),
        percentile(values, 0.95),
        percentile(values, 0.99),
    )
}
```

### Test Files

**`src/reedcms/reedbase/metrics/collector_test.rs`**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_record_metric() {
        // Test metric recording
    }
    
    #[test]
    fn test_increment_counter() {
        // Test counter increment
    }
    
    #[test]
    fn test_gauge_value() {
        // Test gauge value
    }
    
    #[test]
    fn test_buffer_flush() {
        // Test automatic flush at 1000 metrics
    }
    
    #[test]
    fn test_concurrent_recording() {
        // Test thread safety
    }
}
```

## Performance Requirements

- Record metric: < 100ns (in-memory append)
- Flush buffer: < 10ms (1000 metrics to CSV)
- Percentile calculation: < 1ms (1000 values)
- Memory: < 1MB for buffer
- Disk: ~100 bytes per metric (CSV format)

## Error Conditions

- **IoError**: Cannot write metrics to disk (non-fatal, logs warning)
- **LockPoisoned**: RwLock poisoned (should never happen)

## Acceptance Criteria

- [ ] MetricsCollector singleton implemented
- [ ] `record()`, `increment()`, `gauge()` methods work
- [ ] Automatic buffer flush at 1000 metrics
- [ ] CSV storage writes to `.reedbase/metrics/{metric}.csv`
- [ ] Thread-safe concurrent recording
- [ ] Percentile calculation works (P50/P95/P99)
- [ ] All tests pass with 100% coverage
- [ ] Performance benchmarks meet targets
- [ ] All code in BBC English
- [ ] Each file has one clear responsibility
- [ ] All functions have proper documentation

## Dependencies
- **Requires**: None (foundation ticket)

## Blocks
- REED-19-01 through REED-19-19 (all tickets use this infrastructure)

## References
- Service Template: `_workbench/Tickets/templates/service-template.md`
- Metrics Design: `_workbench/Tickets/templates/metrics-module-design.md`
- REED-19-00: Layer Overview

## Notes

This ticket implements the REUSABLE metrics infrastructure that all other tickets will use.

**Key Principles:**
- **ONE implementation**: No copy-paste code across tickets
- **KISS**: Simple API - just call `metrics().record()`
- **DRY**: All logic centralized in this module
- **Thread-Safe**: Works with concurrent operations

**After this ticket:**
- All other tickets can add metrics with ONE line of code
- No duplicate MetricsCollector implementations
- Consistent storage format across all modules
- Centralized alerting and aggregation

**Implementation Order:**
1. Implement this ticket FIRST (REED-19-01A)
2. Then add metrics sections to all other tickets (REED-19-01 through REED-19-19)
3. All tickets use `use crate::reedbase::metrics::global as metrics;`
