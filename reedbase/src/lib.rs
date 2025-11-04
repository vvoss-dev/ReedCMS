// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! ReedBase - CSV-based versioned distributed database.
//!
//! A lightweight, file-based database with Git-like versioning and P2P distribution.
//!
//! ## Features
//!
//! - **Binary Delta Versioning**: Space-efficient versioning using bsdiff
//! - **Three Deployment Modes**: Global, local, and distributed
//! - **Frame-System**: Coordinated batch operations with shared timestamp
//! - **Concurrent Writes**: File locking with automatic conflict resolution
//! - **Smart Indices**: O(1) lookups via HashMap indexing
//! - **Metrics & Observability**: Built-in performance monitoring
//!
//! ## Quick Start
//!
//! ```rust
//! use reedbase::metrics::{MetricsCollector, Metric, MetricUnit};
//!
//! // Record a performance metric
//! let metric = Metric::new("query_duration", 1250.0, MetricUnit::Microseconds)
//!     .with_tag("table", "text")
//!     .with_tag("operation", "get");
//!
//! MetricsCollector::global().record(metric);
//! ```
//!
//! ## Architecture
//!
//! ReedBase is organized into modules:
//!
//! - **functions**: Computed functions, aggregations, and transformations with caching
//! - **metrics**: Performance monitoring and observability
//! - **registry**: Database registry and discovery (planned)
//! - **table**: Universal table API (planned)
//! - **versioning**: Binary delta versioning (planned)
//! - **concurrency**: Concurrent write handling (planned)
//! - **distribution**: P2P replication (planned)

pub mod backup;
pub mod btree;
pub mod concurrent;
pub mod conflict;
pub mod database;
pub mod error;
pub mod functions;
pub mod indices;
pub mod log;
pub mod merge;
pub mod metrics;
pub mod reedql;
pub mod registry;
pub mod schema;
pub mod tables;
pub mod version;

// Re-export commonly used types
pub use backup::{create_backup, list_backups, restore_point_in_time, BackupInfo, RestoreReport};
pub use btree::{BPlusTree, Index, Order};
pub use database::{AutoIndexConfig, Database, DatabaseStats, ExecuteResult, QueryMetrics};
pub use error::{ReedError, ReedResult};
pub use metrics::{Metric, MetricType, MetricUnit, MetricsCollector};
pub use reedql::QueryResult;
