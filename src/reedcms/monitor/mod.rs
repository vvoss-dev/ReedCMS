// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! ReedMonitor - Performance monitoring and metrics collection.
//!
//! ## Modules
//! - `syslog`: FreeBSD-style syslog logger
//! - `log_manager`: Log file rotation and cleanup
//! - `core`: Main monitoring system
//! - `metrics`: Metrics storage and aggregation
//! - `middleware`: Actix-Web middleware integration
//! - `health`: Health check and metrics endpoints

pub mod core;
pub mod health;
pub mod log_manager;
pub mod metrics;
pub mod middleware;
pub mod syslog;

#[cfg(test)]
mod core_test;
#[cfg(test)]
mod syslog_test;

// Re-exports for convenience
pub use core::{global_monitor, ReedMonitor};
pub use health::{health_check, metrics_endpoint};
pub use log_manager::LogFileManager;
pub use metrics::{Health, HealthStatus, MetricsSnapshot};
pub use middleware::MonitorMiddleware;
pub use syslog::{LogLevel, OutputMode, SysLogger};
