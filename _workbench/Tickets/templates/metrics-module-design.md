# Reusable Metrics Module Design

## File Structure

```
src/reedcms/reedbase/metrics/
├── mod.rs                    # Public API
├── types.rs                  # Core types (Metric, MetricDefinition, AlertRule)
├── collector.rs              # MetricsCollector (singleton, thread-safe)
├── storage.rs                # CSV backend for persistence
├── aggregator.rs             # P50/P95/P99 calculation
└── provider.rs               # MetricsProvider trait
```

---

## Core Types (`types.rs`)

```rust
// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Core metric types for ReedBase observability.

use std::collections::HashMap;
use std::time::Duration;

/// Single metric data point.
#[derive(Debug, Clone)]
pub struct Metric {
    pub name: String,
    pub value: f64,
    pub unit: MetricUnit,
    pub tags: HashMap<String, String>,
    pub timestamp: i64,  // Unix timestamp
}

/// Metric type (how to aggregate).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetricType {
    Counter,      // Monotonically increasing
    Gauge,        // Point-in-time value
    Histogram,    // Distribution (P50/P95/P99)
    Timer,        // Duration measurement
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

/// Metric definition (what to collect).
#[derive(Debug, Clone)]
pub struct MetricDefinition {
    pub name: String,
    pub metric_type: MetricType,
    pub unit: MetricUnit,
    pub target: Option<f64>,
    pub alert_rules: Vec<AlertRule>,
}

/// Alert rule (when to trigger).
#[derive(Debug, Clone)]
pub struct AlertRule {
    pub condition: AlertCondition,
    pub severity: AlertSeverity,
    pub duration: Duration,
    pub cooldown: Duration,
}

/// Alert condition.
#[derive(Debug, Clone)]
pub enum AlertCondition {
    Exceeds(f64),
    Below(f64),
    P99Exceeds(f64),
    P95Exceeds(f64),
    RateExceeds(f64),  // Per second
}

/// Alert severity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlertSeverity {
    Warning,
    Critical,
}
```

---

## MetricsProvider Trait (`provider.rs`)

```rust
// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! MetricsProvider trait for declarative metrics.

use super::types::MetricDefinition;

/// Trait for modules that expose metrics.
///
/// Implement this trait to define what metrics your module provides.
/// The metrics collection is handled automatically by the framework.
pub trait MetricsProvider {
    /// Define metrics for this module.
    ///
    /// Return a list of metric definitions. The framework will
    /// automatically collect these metrics based on the definitions.
    ///
    /// ## Example
    /// ```rust
    /// impl MetricsProvider for Table {
    ///     fn metrics_spec() -> Vec<MetricDefinition> {
    ///         vec![
    ///             MetricDefinition {
    ///                 name: "table_read_latency".to_string(),
    ///                 metric_type: MetricType::Histogram,
    ///                 unit: MetricUnit::Microseconds,
    ///                 target: Some(1000.0),
    ///                 alert_rules: vec![/* ... */],
    ///             },
    ///         ]
    ///     }
    /// }
    /// ```
    fn metrics_spec() -> Vec<MetricDefinition>;
}
```

---

## MetricsCollector (`collector.rs`)

```rust
// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Global metrics collector (singleton, thread-safe).

use std::sync::{Arc, RwLock, OnceLock};
use std::collections::HashMap;
use super::types::{Metric, MetricUnit};

/// Global metrics collector instance.
static METRICS: OnceLock<MetricsCollector> = OnceLock::new();

/// Get global metrics collector.
pub fn global() -> &'static MetricsCollector {
    METRICS.get_or_init(|| MetricsCollector::new())
}

/// Thread-safe metrics collector.
pub struct MetricsCollector {
    buffer: Arc<RwLock<Vec<Metric>>>,
}

impl MetricsCollector {
    fn new() -> Self {
        Self {
            buffer: Arc::new(RwLock::new(Vec::new())),
        }
    }
    
    /// Record a metric value.
    pub fn record(&self, metric: Metric) {
        let mut buffer = self.buffer.write().unwrap();
        buffer.push(metric);
        
        // Flush if buffer too large
        if buffer.len() >= 1000 {
            self.flush_internal(&buffer);
            buffer.clear();
        }
    }
    
    /// Increment a counter.
    pub fn increment(&self, name: &str, tags: HashMap<String, String>) {
        self.record(Metric {
            name: name.to_string(),
            value: 1.0,
            unit: MetricUnit::Count,
            tags,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64,
        });
    }
    
    /// Set a gauge value.
    pub fn gauge(&self, name: &str, value: f64, tags: HashMap<String, String>) {
        self.record(Metric {
            name: name.to_string(),
            value,
            unit: MetricUnit::Count,
            tags,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64,
        });
    }
    
    fn flush_internal(&self, buffer: &[Metric]) {
        // Write to CSV (implemented in storage.rs)
        let _ = crate::reedbase::metrics::storage::write_metrics(buffer);
    }
}
```

---

## Usage in Modules (Clean, Simple)

### Example 1: Table Module

```rust
// src/reedbase/tables/table.rs

use crate::reedbase::metrics::{MetricsProvider, global as metrics};
use std::time::Instant;

impl Table {
    pub fn read_current(&self) -> ReedResult<Vec<u8>> {
        let start = Instant::now();
        
        let result = self.read_current_inner()?;
        
        // ONE line to record metric
        metrics().record(Metric {
            name: "table_read_latency".to_string(),
            value: start.elapsed().as_micros() as f64,
            unit: MetricUnit::Microseconds,
            tags: hashmap!{
                "table" => self.name.clone(),
            },
        });
        
        Ok(result)
    }
}
```

### Example 2: Concurrent Write System

```rust
// src/reedbase/concurrent/queue.rs

pub fn queue_write(table_name: &str, operation: PendingWrite) -> ReedResult<String> {
    let depth = count_pending(table_name)?;
    
    // ONE line to record gauge
    metrics().gauge("queue_depth", depth as f64, hashmap!{
        "table" => table_name.to_string(),
    });
    
    if depth >= 100 {
        // ONE line to increment counter
        metrics().increment("queue_full_rejections", hashmap!{
            "table" => table_name.to_string(),
        });
        return Err(ReedError::QueueFull { table: table_name.to_string(), max_size: 100 });
    }
    
    // ... rest of implementation
}
```

---

## Benefits of This Design

### 1. KISS - Keep It Simple
- ✅ Modules only call `metrics().record()` - ONE function
- ✅ No complex instrumentation code
- ✅ No trait implementations needed in every module

### 2. DRY - Don't Repeat Yourself
- ✅ MetricsCollector implemented ONCE
- ✅ CSV storage logic ONCE
- ✅ Alert evaluation ONCE

### 3. Performance
- ✅ Thread-safe with RwLock
- ✅ Buffered writes (flush at 1000 metrics)
- ✅ Minimal overhead (<100ns per metric)

### 4. Testability
- ✅ Easy to mock in tests
- ✅ Can query buffer state
- ✅ Can disable metrics in tests

---

## Standard Template for Tickets

```markdown
## Metrics & Observability

### Performance Metrics

| Metric | Type | Unit | Target | P99 Alert | Collection Point |
|--------|------|------|--------|-----------|------------------|
| {module}_{operation}_latency | Histogram | μs | <100 | >200 | {file}.rs:{function}() |

### Alert Rules

**CRITICAL Alerts:**
- `{metric} {condition}` for {duration} → "{message}"

**WARNING Alerts:**
- `{metric} {condition}` for {duration} → "{message}"

### Implementation

```rust
use crate::reedbase::metrics::global as metrics;

pub fn {operation}(&self) -> ReedResult<T> {
    let start = Instant::now();
    
    let result = self.{operation}_inner()?;
    
    metrics().record(Metric {
        name: "{module}_{operation}_latency".to_string(),
        value: start.elapsed().as_micros() as f64,
        unit: MetricUnit::Microseconds,
        tags: hashmap!{ "module" => "{module}" },
    });
    
    Ok(result)
}
```

### Collection Strategy

- **Sampling**: All operations
- **Aggregation**: 1-minute rolling window
- **Storage**: `.reedbase/metrics/{module}.csv`
- **Retention**: 7 days raw, 90 days aggregated

### Why These Metrics Matter

**{Primary Metric}**: {1-2 sentence explanation}
- {Key point 1}
- {Key point 2}
- {Impact on system}
```

---

## Implementation Priority

1. **Create `src/reedcms/reedbase/metrics/` module** (types, collector, storage)
2. **Add to REED-19-00** as reference for all tickets
3. **Update existing tickets** (01, 02, 05, 03-08) to use this pattern
4. **Complete remaining tickets** (09-19) with this pattern

**Result:** Consistent, maintainable, KISS-compliant metrics across entire ReedBase.
