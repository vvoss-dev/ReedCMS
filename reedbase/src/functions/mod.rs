// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Function system with automatic memoization caching.
//!
//! Provides three categories of functions with automatic result caching:
//!
//! ## Function Categories
//!
//! ### 1. Computed Functions (`computed`)
//! Pure functions that derive values from inputs:
//! - `calculate_age(birthdate)` → Age in years
//! - `full_name(first, last)` → Combined name
//! - `days_since(date)` → Days elapsed
//! - `is_expired(date)` → Boolean check
//! - `format_date(date, format)` → Formatted date
//! - `calculate_discount(price, percentage)` → Discounted price
//!
//! ### 2. Aggregation Functions (`aggregations`)
//! Dataset-level operations with CSV scanning:
//! - `count(table)` → Row count
//! - `sum(table, column)` → Sum of numeric column
//! - `avg(table, column)` → Average of numeric column
//! - `min(table, column)` → Minimum value
//! - `max(table, column)` → Maximum value
//! - `group_by(table, column)` → Grouped counts as JSON
//!
//! ### 3. Transformation Functions (`transformations`)
//! String cleaning and formatting:
//! - `normalize_email(email)` → Cleaned email
//! - `trim(value)` → Whitespace removed
//! - `capitalize(value)` → First letter uppercase
//! - `slugify(value)` → URL-safe slug
//! - `truncate(value, length)` → Truncated string
//! - `replace(value, old, new)` → String replacement
//! - `uppercase(value)` → All uppercase
//! - `lowercase(value)` → All lowercase
//! - `remove_whitespace(value)` → All whitespace removed
//! - `pad_right(value, length)` → Right-padded with spaces
//! - `reverse(value)` → Reversed string
//!
//! ## Performance
//!
//! - **Cache hit**: < 100ns (instant)
//! - **Cache insert**: < 10μs
//! - **Computed functions**: < 1μs (first), < 100ns (cached)
//! - **Aggregations**: 2-10ms (first, 10k rows), < 100ns (cached)
//! - **Transformations**: < 2μs (first), < 100ns (cached)
//!
//! ## Example Usage
//!
//! ```rust
//! use reedbase::functions::{computed, aggregations, transformations};
//!
//! // Computed
//! let age = computed::calculate_age("1990-05-15")?; // "35"
//!
//! // Aggregation
//! let total = aggregations::count("users")?; // "1250"
//! let avg_age = aggregations::avg("users", "age")?; // "35.00"
//!
//! // Transformation
//! let email = transformations::normalize_email("John@Example.COM")?; // "john@example.com"
//! let slug = transformations::slugify("Hello World!")?; // "hello-world"
//! # Ok::<(), reedbase::ReedError>(())
//! ```
//!
//! ## Cache Management
//!
//! The global cache can be accessed for statistics and management:
//!
//! ```rust
//! use reedbase::functions::cache::get_cache;
//!
//! // Get cache statistics
//! let stats = get_cache().stats();
//! println!("Hit rate: {:.2}%", stats.hit_rate());
//! println!("Total entries: {}", get_cache().entry_count());
//!
//! // Clear cache
//! get_cache().clear();
//!
//! // Invalidate table-specific caches
//! get_cache().invalidate_table("users");
//! ```

pub mod aggregations;
pub mod cache;
pub mod computed;
pub mod transformations;

#[cfg(test)]
mod aggregations_test;
#[cfg(test)]
mod cache_test;
#[cfg(test)]
mod computed_test;
#[cfg(test)]
mod transformations_test;

// Re-export commonly used types
pub use cache::{get_cache, CacheKey, CacheStats, FunctionCache};
