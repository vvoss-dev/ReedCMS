// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! ReedBase cache viewer for debugging.
//!
//! ## Features
//! - View cache contents by type
//! - Search cache entries
//! - Display cache statistics
//!
//! ## CLI Usage
//! ```bash
//! reed debug:cache
//! reed debug:cache text
//! reed debug:cache --search "knowledge"
//! ```

use crate::reedcms::monitor::global_monitor;
use std::collections::HashMap;

/// Cache view structure for debugging.
#[derive(Debug, Clone)]
pub struct CacheView {
    pub total_entries: usize,
    pub cache_type: Option<String>,
    pub search_term: Option<String>,
    pub entries: Vec<CacheEntry>,
    pub statistics: CacheStatistics,
}

/// Single cache entry.
#[derive(Debug, Clone)]
pub struct CacheEntry {
    pub key: String,
    pub value: String,
    pub cache_type: String,
}

/// Cache statistics.
#[derive(Debug, Clone)]
pub struct CacheStatistics {
    pub total_entries: usize,
    pub text_entries: usize,
    pub route_entries: usize,
    pub meta_entries: usize,
    pub hit_rate: f64,
    pub avg_lookup_time_us: u64,
}

impl CacheView {
    /// Creates new cache view.
    pub fn new() -> Self {
        Self {
            total_entries: 0,
            cache_type: None,
            search_term: None,
            entries: Vec::new(),
            statistics: CacheStatistics::default(),
        }
    }

    /// Loads cache statistics from monitor.
    pub fn load_statistics(&mut self) {
        let snapshot = global_monitor().get_snapshot();

        self.statistics = CacheStatistics {
            total_entries: 0, // Would need access to actual cache
            text_entries: 0,
            route_entries: 0,
            meta_entries: 0,
            hit_rate: snapshot.reedbase_hit_rate,
            avg_lookup_time_us: snapshot.reedbase_avg_time.as_micros() as u64,
        };
    }

    /// Formats cache view as string.
    pub fn format(&self) -> String {
        let mut output = String::from("📦 ReedBase Cache Viewer\n\n");

        if let Some(ref cache_type) = self.cache_type {
            output.push_str(&format!("Cache Type: {}\n", cache_type));
        }

        if let Some(ref search) = self.search_term {
            output.push_str(&format!("Search: \"{}\"\n", search));
        }

        output.push_str("\nCache Statistics:\n");
        output.push_str(&format!(
            "  Hit Rate: {:.1}%\n",
            self.statistics.hit_rate * 100.0
        ));
        output.push_str(&format!(
            "  Avg Lookup Time: {}μs\n",
            self.statistics.avg_lookup_time_us
        ));

        if !self.entries.is_empty() {
            output.push_str(&format!("\nEntries ({}):\n", self.entries.len()));
            for (i, entry) in self.entries.iter().take(20).enumerate() {
                output.push_str(&format!(
                    "  {}. [{}] {} = \"{}\"\n",
                    i + 1,
                    entry.cache_type,
                    entry.key,
                    entry.value
                ));
            }

            if self.entries.len() > 20 {
                output.push_str(&format!("  ... and {} more\n", self.entries.len() - 20));
            }
        } else {
            output.push_str("\nNo entries to display.\n");
            output.push_str("Note: Direct cache access requires runtime integration.\n");
        }

        output
    }
}

impl Default for CacheView {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for CacheStatistics {
    fn default() -> Self {
        Self {
            total_entries: 0,
            text_entries: 0,
            route_entries: 0,
            meta_entries: 0,
            hit_rate: 0.0,
            avg_lookup_time_us: 0,
        }
    }
}

/// Views cache contents.
///
/// ## Arguments
/// - `cache_type`: Optional cache type filter (text, route, meta)
/// - `search`: Optional search term
///
/// ## Returns
/// Cache view with statistics
pub fn view_cache(cache_type: Option<&str>, search: Option<&str>) -> CacheView {
    let mut view = CacheView::new();
    view.cache_type = cache_type.map(|s| s.to_string());
    view.search_term = search.map(|s| s.to_string());

    // Load statistics from monitor
    view.load_statistics();

    // Note: Actual cache data would be loaded from ReedBase here
    // For now, we show statistics only

    view
}
