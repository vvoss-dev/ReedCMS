// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Performance profiler and analysis tools.
//!
//! ## Modules
//! - `core`: Span-based profiler
//! - `middleware`: Actix-Web profiling middleware
//! - `slow_queries`: Slow operation tracker
//! - `flamegraph`: Flame graph generator

pub mod core;
pub mod flamegraph;
pub mod middleware;
pub mod slow_queries;

#[cfg(test)]
mod core_test;
#[cfg(test)]
mod slow_queries_test;

// Re-exports for convenience
pub use core::{ProfileReport, Profiler, Span, SpanGuard};
pub use flamegraph::{generate_flamegraph_data, generate_svg};
pub use middleware::ProfilerMiddleware;
pub use slow_queries::{global_slow_tracker, SlowQuery, SlowQueryTracker};
