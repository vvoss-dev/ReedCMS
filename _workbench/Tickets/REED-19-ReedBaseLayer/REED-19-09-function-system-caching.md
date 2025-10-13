# REED-19-09: Function System & Caching

## MANDATORY Development Standards

**CRITICAL**: Every implementation MUST follow these standards:

- **Language**: All documentation and code comments in BBC English
- **Principle**: KISS (Keep It Simple, Stupid) - minimal code and professional inline documentation
- **File Naming**: File name = Unique theme - crystal clear what single topic this file handles
- **Files**: One file = One responsibility
- **Functions**: One function = One distinctive job
- **Shared Functions**: Same patterns = One shared function
- **Testing**: Separate test files as `{name}.test.rs` - never inline `#[cfg(test)]` modules
- **Avoid**: Avoid Swiss Army knife functions
- **Avoid**: Generic file names like `handler.rs`, `utils.rs`
- **Templates**: See `_workbench/Tickets/templates/service-template.md` for complete implementation guide
- **Testing**: See `_workbench/Tickets/templates/service-template.test.md` for test structure

## Ticket Information
- **ID**: REED-19-09
- **Title**: Function System & Caching
- **Layer**: ReedBase Layer (REED-19)
- **Priority**: Medium
- **Status**: Open
- **Complexity**: Medium
- **Dependencies**: REED-19-02 (Universal Table API)
- **Estimated Time**: 5 days

## Objective

Implement Rust-based function system with memoization cache. Provide computed columns, aggregations, and custom transformations with automatic result caching for performance.

## Requirements

### Function Types

**1. Computed Columns**
```rust
// Calculate age from birth_date
fn calculate_age(birth_date: u64) -> u32 {
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    ((now - birth_date) / 31536000) as u32
}
```

**2. Aggregations**
```rust
// Count rows matching condition
fn count_users_by_status(rows: &[CsvRow], status: &str) -> usize {
    rows.iter().filter(|r| r.values.get(2) == Some(&status.to_string())).count()
}
```

**3. Transformations**
```rust
// Normalize email to lowercase
fn normalize_email(email: &str) -> String {
    email.to_lowercase()
}
```

### Function Registry

```
.reed/functions/
├── mod.rs                  # Function registry
├── computed.rs             # Computed column functions
├── aggregations.rs         # Aggregation functions
├── transformations.rs      # Transformation functions
└── cache.rs                # Memoization cache
```

### Cache Structure

```rust
struct FunctionCache {
    cache: Arc<RwLock<HashMap<String, CachedResult>>>,
}

struct CachedResult {
    input_hash: u64,
    output: Value,
    timestamp: u64,
    hit_count: usize,
}
```

### Performance Targets

| Operation | Target | Notes |
|-----------|--------|-------|
| Function execution (cached) | < 100ns | O(1) HashMap lookup |
| Function execution (uncached) | Varies | Depends on function |
| Cache insert | < 10μs | HashMap insert + hash |
| Cache lookup | < 100ns | HashMap get |
| Aggregate 1000 rows | < 10ms | Single pass through data |

## Implementation Files

### Primary Implementation

**`reedbase/src/functions/cache.rs`**

One file = Function caching only. NO other responsibilities.

```rust
// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Function result caching (memoization).
//!
//! Caches function results for identical inputs.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use serde_json::Value;
use crate::types::{ReedResult, ReedError};

/// Function cache (memoization).
pub struct FunctionCache {
    cache: Arc<RwLock<HashMap<u64, CachedResult>>>,
}

/// Cached function result.
#[derive(Debug, Clone)]
struct CachedResult {
    output: Value,
    timestamp: u64,
    hit_count: usize,
}

impl FunctionCache {
    /// Create new function cache.
    ///
    /// ## Output
    /// - `FunctionCache`: New cache instance
    ///
    /// ## Performance
    /// - O(1) operation
    /// - < 1μs
    ///
    /// ## Example Usage
    /// ```rust
    /// let cache = FunctionCache::new();
    /// ```
    pub fn new() -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Get cached result if available.
    ///
    /// ## Input
    /// - `function_name`: Function name
    /// - `args`: Function arguments
    ///
    /// ## Output
    /// - `Option<Value>`: Cached result or None
    ///
    /// ## Performance
    /// - < 100ns cache hit
    /// - O(1) HashMap lookup
    ///
    /// ## Example Usage
    /// ```rust
    /// if let Some(result) = cache.get("calculate_age", &args) {
    ///     return Ok(result);
    /// }
    /// ```
    pub fn get(&self, function_name: &str, args: &[Value]) -> Option<Value> {
        let input_hash = hash_input(function_name, args);
        
        let mut cache = self.cache.write().unwrap();
        
        if let Some(cached) = cache.get_mut(&input_hash) {
            cached.hit_count += 1;
            Some(cached.output.clone())
        } else {
            None
        }
    }
    
    /// Store result in cache.
    ///
    /// ## Input
    /// - `function_name`: Function name
    /// - `args`: Function arguments
    /// - `result`: Function result
    ///
    /// ## Output
    /// - None
    ///
    /// ## Performance
    /// - < 10μs typical
    /// - O(1) HashMap insert
    ///
    /// ## Example Usage
    /// ```rust
    /// let result = calculate_age(birth_date);
    /// cache.set("calculate_age", &args, result);
    /// ```
    pub fn set(&self, function_name: &str, args: &[Value], result: Value) {
        let input_hash = hash_input(function_name, args);
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let mut cache = self.cache.write().unwrap();
        
        cache.insert(input_hash, CachedResult {
            output: result,
            timestamp,
            hit_count: 0,
        });
    }
    
    /// Clear all cached results.
    ///
    /// ## Output
    /// - None
    ///
    /// ## Performance
    /// - O(1) operation (drops HashMap)
    /// - < 1ms for large caches
    ///
    /// ## Example Usage
    /// ```rust
    /// cache.clear();
    /// ```
    pub fn clear(&self) {
        let mut cache = self.cache.write().unwrap();
        cache.clear();
    }
    
    /// Get cache statistics.
    ///
    /// ## Output
    /// - `CacheStats`: Statistics (size, hit rate, etc.)
    ///
    /// ## Performance
    /// - O(n) where n = cache size
    /// - < 1ms for typical caches (< 1000 entries)
    ///
    /// ## Example Usage
    /// ```rust
    /// let stats = cache.stats();
    /// println!("Cache size: {}, total hits: {}", stats.size, stats.total_hits);
    /// ```
    pub fn stats(&self) -> CacheStats {
        let cache = self.cache.read().unwrap();
        
        let size = cache.len();
        let total_hits: usize = cache.values().map(|c| c.hit_count).sum();
        
        CacheStats {
            size,
            total_hits,
            hit_rate: if size > 0 {
                (total_hits as f64 / size as f64) * 100.0
            } else {
                0.0
            },
        }
    }
    
    /// Evict old entries (LRU-like).
    ///
    /// ## Input
    /// - `max_age_secs`: Maximum age in seconds
    ///
    /// ## Output
    /// - `usize`: Number of entries evicted
    ///
    /// ## Performance
    /// - O(n) where n = cache size
    /// - < 5ms for 1000 entries
    ///
    /// ## Example Usage
    /// ```rust
    /// let evicted = cache.evict_old(3600); // Remove entries older than 1 hour
    /// println!("Evicted {} old entries", evicted);
    /// ```
    pub fn evict_old(&self, max_age_secs: u64) -> usize {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let mut cache = self.cache.write().unwrap();
        
        let before = cache.len();
        cache.retain(|_, cached| now - cached.timestamp <= max_age_secs);
        let after = cache.len();
        
        before - after
    }
}

/// Hash function inputs for cache key.
///
/// ## Input
/// - `function_name`: Function name
/// - `args`: Function arguments
///
/// ## Output
/// - `u64`: Hash value
///
/// ## Performance
/// - < 1μs typical
///
/// ## Example Usage
/// ```rust
/// let hash = hash_input("calculate_age", &args);
/// ```
fn hash_input(function_name: &str, args: &[Value]) -> u64 {
    let mut hasher = DefaultHasher::new();
    function_name.hash(&mut hasher);
    
    for arg in args {
        // Hash JSON representation of argument
        let json = serde_json::to_string(arg).unwrap_or_default();
        json.hash(&mut hasher);
    }
    
    hasher.finish()
}

/// Cache statistics.
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub size: usize,
    pub total_hits: usize,
    pub hit_rate: f64,
}

impl Default for FunctionCache {
    fn default() -> Self {
        Self::new()
    }
}
```

**`reedbase/src/functions/computed.rs`**

One file = Computed column functions only. NO other responsibilities.

```rust
// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Computed column functions.
//!
//! Functions that compute values from existing columns.

use crate::types::{ReedResult, ReedError};
use serde_json::Value;
use std::time::{SystemTime, UNIX_EPOCH};

/// Calculate age from birth date (Unix timestamp).
///
/// ## Input
/// - `birth_date`: Birth date as Unix timestamp
///
/// ## Output
/// - `u32`: Age in years
///
/// ## Performance
/// - < 1μs typical
///
/// ## Example Usage
/// ```rust
/// let age = calculate_age(946684800); // Born 2000-01-01
/// assert_eq!(age, 25); // Assuming current year is 2025
/// ```
pub fn calculate_age(birth_date: u64) -> u32 {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    ((now - birth_date) / 31536000) as u32
}

/// Calculate full name from first and last name.
///
/// ## Input
/// - `first_name`: First name
/// - `last_name`: Last name
///
/// ## Output
/// - `String`: Full name
///
/// ## Performance
/// - < 1μs typical
///
/// ## Example Usage
/// ```rust
/// let full = full_name("Alice", "Smith");
/// assert_eq!(full, "Alice Smith");
/// ```
pub fn full_name(first_name: &str, last_name: &str) -> String {
    format!("{} {}", first_name, last_name)
}

/// Calculate days since date.
///
/// ## Input
/// - `date`: Date as Unix timestamp
///
/// ## Output
/// - `u64`: Days elapsed
///
/// ## Performance
/// - < 1μs typical
///
/// ## Example Usage
/// ```rust
/// let days = days_since(1736860800);
/// ```
pub fn days_since(date: u64) -> u64 {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    (now - date) / 86400
}

/// Calculate percentage.
///
/// ## Input
/// - `part`: Part value
/// - `total`: Total value
///
/// ## Output
/// - `ReedResult<f64>`: Percentage (0.0 to 100.0)
///
/// ## Performance
/// - < 1μs typical
///
/// ## Error Conditions
/// - DivisionByZero: Total is zero
///
/// ## Example Usage
/// ```rust
/// let pct = calculate_percentage(25.0, 100.0)?;
/// assert_eq!(pct, 25.0);
/// ```
pub fn calculate_percentage(part: f64, total: f64) -> ReedResult<f64> {
    if total == 0.0 {
        return Err(ReedError::DivisionByZero {
            context: "calculate_percentage".to_string(),
        });
    }
    
    Ok((part / total) * 100.0)
}

/// Check if date is in past.
///
/// ## Input
/// - `date`: Date as Unix timestamp
///
/// ## Output
/// - `bool`: True if date is in past
///
/// ## Performance
/// - < 1μs typical
///
/// ## Example Usage
/// ```rust
/// if is_past(some_date) {
///     println!("Date has passed");
/// }
/// ```
pub fn is_past(date: u64) -> bool {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    date < now
}
```

**`reedbase/src/functions/aggregations.rs`**

One file = Aggregation functions only. NO other responsibilities.

```rust
// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Aggregation functions.
//!
//! Functions that aggregate data across multiple rows.

use crate::types::{ReedResult, ReedError, CsvRow};

/// Count rows.
///
/// ## Input
/// - `rows`: Rows to count
///
/// ## Output
/// - `usize`: Row count
///
/// ## Performance
/// - O(1) operation (len())
/// - < 1μs typical
///
/// ## Example Usage
/// ```rust
/// let count = count_rows(&rows);
/// ```
pub fn count_rows(rows: &[CsvRow]) -> usize {
    rows.len()
}

/// Count rows matching predicate.
///
/// ## Input
/// - `rows`: Rows to filter
/// - `column_index`: Column index to check
/// - `value`: Value to match
///
/// ## Output
/// - `usize`: Count of matching rows
///
/// ## Performance
/// - O(n) where n = number of rows
/// - < 5ms for 1000 rows
///
/// ## Example Usage
/// ```rust
/// let active_users = count_where(&rows, 2, "active");
/// ```
pub fn count_where(rows: &[CsvRow], column_index: usize, value: &str) -> usize {
    rows.iter()
        .filter(|row| {
            row.values.get(column_index)
                .map(|v| v == value)
                .unwrap_or(false)
        })
        .count()
}

/// Sum numeric column.
///
/// ## Input
/// - `rows`: Rows to sum
/// - `column_index`: Column index to sum
///
/// ## Output
/// - `ReedResult<f64>`: Sum of values
///
/// ## Performance
/// - O(n) where n = number of rows
/// - < 10ms for 1000 rows
///
/// ## Error Conditions
/// - ParseError: Non-numeric value encountered
///
/// ## Example Usage
/// ```rust
/// let total = sum_column(&rows, 3)?;
/// ```
pub fn sum_column(rows: &[CsvRow], column_index: usize) -> ReedResult<f64> {
    let mut sum = 0.0;
    
    for row in rows {
        if let Some(value) = row.values.get(column_index) {
            let num = value.parse::<f64>()
                .map_err(|_| ReedError::ParseError {
                    reason: format!("Cannot parse '{}' as number", value),
                })?;
            sum += num;
        }
    }
    
    Ok(sum)
}

/// Calculate average of numeric column.
///
/// ## Input
/// - `rows`: Rows to average
/// - `column_index`: Column index to average
///
/// ## Output
/// - `ReedResult<f64>`: Average value
///
/// ## Performance
/// - O(n) where n = number of rows
/// - < 10ms for 1000 rows
///
/// ## Error Conditions
/// - ParseError: Non-numeric value encountered
/// - DivisionByZero: No rows provided
///
/// ## Example Usage
/// ```rust
/// let avg_age = average_column(&rows, 2)?;
/// ```
pub fn average_column(rows: &[CsvRow], column_index: usize) -> ReedResult<f64> {
    if rows.is_empty() {
        return Err(ReedError::DivisionByZero {
            context: "average_column".to_string(),
        });
    }
    
    let sum = sum_column(rows, column_index)?;
    Ok(sum / rows.len() as f64)
}

/// Find minimum value in numeric column.
///
/// ## Input
/// - `rows`: Rows to search
/// - `column_index`: Column index to search
///
/// ## Output
/// - `ReedResult<f64>`: Minimum value
///
/// ## Performance
/// - O(n) where n = number of rows
/// - < 10ms for 1000 rows
///
/// ## Error Conditions
/// - ParseError: Non-numeric value encountered
/// - EmptySet: No rows provided
///
/// ## Example Usage
/// ```rust
/// let min_age = min_column(&rows, 2)?;
/// ```
pub fn min_column(rows: &[CsvRow], column_index: usize) -> ReedResult<f64> {
    if rows.is_empty() {
        return Err(ReedError::EmptySet {
            context: "min_column".to_string(),
        });
    }
    
    let mut min = f64::MAX;
    
    for row in rows {
        if let Some(value) = row.values.get(column_index) {
            let num = value.parse::<f64>()
                .map_err(|_| ReedError::ParseError {
                    reason: format!("Cannot parse '{}' as number", value),
                })?;
            if num < min {
                min = num;
            }
        }
    }
    
    Ok(min)
}

/// Find maximum value in numeric column.
///
/// ## Input
/// - `rows`: Rows to search
/// - `column_index`: Column index to search
///
/// ## Output
/// - `ReedResult<f64>`: Maximum value
///
/// ## Performance
/// - O(n) where n = number of rows
/// - < 10ms for 1000 rows
///
/// ## Error Conditions
/// - ParseError: Non-numeric value encountered
/// - EmptySet: No rows provided
///
/// ## Example Usage
/// ```rust
/// let max_age = max_column(&rows, 2)?;
/// ```
pub fn max_column(rows: &[CsvRow], column_index: usize) -> ReedResult<f64> {
    if rows.is_empty() {
        return Err(ReedError::EmptySet {
            context: "max_column".to_string(),
        });
    }
    
    let mut max = f64::MIN;
    
    for row in rows {
        if let Some(value) = row.values.get(column_index) {
            let num = value.parse::<f64>()
                .map_err(|_| ReedError::ParseError {
                    reason: format!("Cannot parse '{}' as number", value),
                })?;
            if num > max {
                max = num;
            }
        }
    }
    
    Ok(max)
}

/// Group by column value and count.
///
/// ## Input
/// - `rows`: Rows to group
/// - `column_index`: Column index to group by
///
/// ## Output
/// - `HashMap<String, usize>`: Value → count mapping
///
/// ## Performance
/// - O(n) where n = number of rows
/// - < 10ms for 1000 rows
///
/// ## Example Usage
/// ```rust
/// let status_counts = group_by_count(&rows, 2);
/// for (status, count) in status_counts {
///     println!("{}: {}", status, count);
/// }
/// ```
pub fn group_by_count(rows: &[CsvRow], column_index: usize) -> std::collections::HashMap<String, usize> {
    let mut counts = std::collections::HashMap::new();
    
    for row in rows {
        if let Some(value) = row.values.get(column_index) {
            *counts.entry(value.clone()).or_insert(0) += 1;
        }
    }
    
    counts
}
```

**`reedbase/src/functions/transformations.rs`**

One file = Transformation functions only. NO other responsibilities.

```rust
// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Transformation functions.
//!
//! Functions that transform data values.

/// Normalize email to lowercase.
///
/// ## Input
/// - `email`: Email address
///
/// ## Output
/// - `String`: Normalized email
///
/// ## Performance
/// - < 1μs typical
///
/// ## Example Usage
/// ```rust
/// let normalized = normalize_email("Alice@EXAMPLE.COM");
/// assert_eq!(normalized, "alice@example.com");
/// ```
pub fn normalize_email(email: &str) -> String {
    email.to_lowercase()
}

/// Trim whitespace from both ends.
///
/// ## Input
/// - `text`: Text to trim
///
/// ## Output
/// - `String`: Trimmed text
///
/// ## Performance
/// - < 1μs typical
///
/// ## Example Usage
/// ```rust
/// let trimmed = trim_text("  hello  ");
/// assert_eq!(trimmed, "hello");
/// ```
pub fn trim_text(text: &str) -> String {
    text.trim().to_string()
}

/// Capitalize first letter.
///
/// ## Input
/// - `text`: Text to capitalize
///
/// ## Output
/// - `String`: Capitalized text
///
/// ## Performance
/// - < 1μs typical
///
/// ## Example Usage
/// ```rust
/// let capitalized = capitalize("alice");
/// assert_eq!(capitalized, "Alice");
/// ```
pub fn capitalize(text: &str) -> String {
    let mut chars = text.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

/// Replace substring.
///
/// ## Input
/// - `text`: Text to search
/// - `from`: Substring to replace
/// - `to`: Replacement string
///
/// ## Output
/// - `String`: Text with replacements
///
/// ## Performance
/// - O(n) where n = text length
/// - < 10μs typical
///
/// ## Example Usage
/// ```rust
/// let replaced = replace_text("Hello World", "World", "Rust");
/// assert_eq!(replaced, "Hello Rust");
/// ```
pub fn replace_text(text: &str, from: &str, to: &str) -> String {
    text.replace(from, to)
}

/// Truncate text to maximum length.
///
/// ## Input
/// - `text`: Text to truncate
/// - `max_length`: Maximum length
///
/// ## Output
/// - `String`: Truncated text (with "..." if truncated)
///
/// ## Performance
/// - O(n) where n = max_length
/// - < 5μs typical
///
/// ## Example Usage
/// ```rust
/// let truncated = truncate_text("Hello World", 8);
/// assert_eq!(truncated, "Hello...");
/// ```
pub fn truncate_text(text: &str, max_length: usize) -> String {
    if text.len() <= max_length {
        text.to_string()
    } else {
        format!("{}...", &text[..max_length.saturating_sub(3)])
    }
}

/// Pad text to minimum length.
///
/// ## Input
/// - `text`: Text to pad
/// - `min_length`: Minimum length
/// - `pad_char`: Character to pad with
///
/// ## Output
/// - `String`: Padded text
///
/// ## Performance
/// - O(n) where n = min_length
/// - < 5μs typical
///
/// ## Example Usage
/// ```rust
/// let padded = pad_text("42", 5, '0');
/// assert_eq!(padded, "00042");
/// ```
pub fn pad_text(text: &str, min_length: usize, pad_char: char) -> String {
    if text.len() >= min_length {
        text.to_string()
    } else {
        format!("{}{}", pad_char.to_string().repeat(min_length - text.len()), text)
    }
}
```

**`reedbase/src/types.rs`** (additions)

```rust
/// Additional ReedBase errors.
#[derive(Error, Debug)]
pub enum ReedError {
    // ... (existing errors)
    
    #[error("Division by zero in {context}")]
    DivisionByZero {
        context: String,
    },
    
    #[error("Empty set in {context}")]
    EmptySet {
        context: String,
    },
}
```

### Test Files

**`reedbase/src/functions/cache.test.rs`**

```rust
// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    
    #[test]
    fn test_cache_get_set() {
        let cache = FunctionCache::new();
        let args = vec![json!(1736860800)];
        
        assert!(cache.get("calculate_age", &args).is_none());
        
        cache.set("calculate_age", &args, json!(25));
        
        let result = cache.get("calculate_age", &args);
        assert_eq!(result, Some(json!(25)));
    }
    
    #[test]
    fn test_cache_clear() {
        let cache = FunctionCache::new();
        let args = vec![json!(1736860800)];
        
        cache.set("calculate_age", &args, json!(25));
        assert!(cache.get("calculate_age", &args).is_some());
        
        cache.clear();
        assert!(cache.get("calculate_age", &args).is_none());
    }
    
    #[test]
    fn test_cache_stats() {
        let cache = FunctionCache::new();
        let args1 = vec![json!(1)];
        let args2 = vec![json!(2)];
        
        cache.set("func", &args1, json!(10));
        cache.set("func", &args2, json!(20));
        
        let _ = cache.get("func", &args1);
        let _ = cache.get("func", &args1);
        let _ = cache.get("func", &args2);
        
        let stats = cache.stats();
        assert_eq!(stats.size, 2);
        assert_eq!(stats.total_hits, 3);
    }
    
    #[test]
    fn test_evict_old() {
        let cache = FunctionCache::new();
        let args = vec![json!(1)];
        
        cache.set("func", &args, json!(10));
        
        std::thread::sleep(std::time::Duration::from_secs(2));
        
        let evicted = cache.evict_old(1); // Evict entries older than 1 second
        assert_eq!(evicted, 1);
        assert!(cache.get("func", &args).is_none());
    }
}
```

**`reedbase/src/functions/computed.test.rs`**

```rust
// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_calculate_age() {
        let birth_date = 946684800; // 2000-01-01
        let age = calculate_age(birth_date);
        assert!(age >= 24 && age <= 26); // Rough check
    }
    
    #[test]
    fn test_full_name() {
        let name = full_name("Alice", "Smith");
        assert_eq!(name, "Alice Smith");
    }
    
    #[test]
    fn test_calculate_percentage() {
        let pct = calculate_percentage(25.0, 100.0).unwrap();
        assert_eq!(pct, 25.0);
    }
    
    #[test]
    fn test_calculate_percentage_zero() {
        let result = calculate_percentage(25.0, 0.0);
        assert!(matches!(result, Err(ReedError::DivisionByZero { .. })));
    }
    
    #[test]
    fn test_is_past() {
        let old_date = 946684800; // 2000-01-01
        assert!(is_past(old_date));
        
        let future_date = 2000000000; // Far future
        assert!(!is_past(future_date));
    }
}
```

**`reedbase/src/functions/aggregations.test.rs`**

```rust
// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

#[cfg(test)]
mod tests {
    use super::*;
    
    fn create_test_rows() -> Vec<CsvRow> {
        vec![
            CsvRow {
                key: "1".to_string(),
                values: vec!["Alice".to_string(), "30".to_string(), "active".to_string()],
            },
            CsvRow {
                key: "2".to_string(),
                values: vec!["Bob".to_string(), "25".to_string(), "active".to_string()],
            },
            CsvRow {
                key: "3".to_string(),
                values: vec!["Charlie".to_string(), "35".to_string(), "inactive".to_string()],
            },
        ]
    }
    
    #[test]
    fn test_count_rows() {
        let rows = create_test_rows();
        assert_eq!(count_rows(&rows), 3);
    }
    
    #[test]
    fn test_count_where() {
        let rows = create_test_rows();
        let active_count = count_where(&rows, 2, "active");
        assert_eq!(active_count, 2);
    }
    
    #[test]
    fn test_sum_column() {
        let rows = create_test_rows();
        let total_age = sum_column(&rows, 1).unwrap();
        assert_eq!(total_age, 90.0);
    }
    
    #[test]
    fn test_average_column() {
        let rows = create_test_rows();
        let avg_age = average_column(&rows, 1).unwrap();
        assert_eq!(avg_age, 30.0);
    }
    
    #[test]
    fn test_min_column() {
        let rows = create_test_rows();
        let min_age = min_column(&rows, 1).unwrap();
        assert_eq!(min_age, 25.0);
    }
    
    #[test]
    fn test_max_column() {
        let rows = create_test_rows();
        let max_age = max_column(&rows, 1).unwrap();
        assert_eq!(max_age, 35.0);
    }
    
    #[test]
    fn test_group_by_count() {
        let rows = create_test_rows();
        let counts = group_by_count(&rows, 2);
        
        assert_eq!(counts.get("active"), Some(&2));
        assert_eq!(counts.get("inactive"), Some(&1));
    }
}
```

**`reedbase/src/functions/transformations.test.rs`**

```rust
// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_normalize_email() {
        let normalized = normalize_email("Alice@EXAMPLE.COM");
        assert_eq!(normalized, "alice@example.com");
    }
    
    #[test]
    fn test_trim_text() {
        let trimmed = trim_text("  hello  ");
        assert_eq!(trimmed, "hello");
    }
    
    #[test]
    fn test_capitalize() {
        let capitalized = capitalize("alice");
        assert_eq!(capitalized, "Alice");
    }
    
    #[test]
    fn test_replace_text() {
        let replaced = replace_text("Hello World", "World", "Rust");
        assert_eq!(replaced, "Hello Rust");
    }
    
    #[test]
    fn test_truncate_text() {
        let truncated = truncate_text("Hello World", 8);
        assert_eq!(truncated, "Hello...");
        
        let not_truncated = truncate_text("Hello", 10);
        assert_eq!(not_truncated, "Hello");
    }
    
    #[test]
    fn test_pad_text() {
        let padded = pad_text("42", 5, '0');
        assert_eq!(padded, "00042");
        
        let not_padded = pad_text("12345", 3, '0');
        assert_eq!(not_padded, "12345");
    }
}
```

## Performance Requirements

| Operation | Target |
|-----------|--------|
| Function execution (cached) | < 100ns |
| Cache insert | < 10μs |
| Cache lookup | < 100ns |
| Computed function | < 1μs |
| Aggregation (1000 rows) | < 10ms |
| Transformation | < 10μs |

## Error Conditions

- **DivisionByZero**: Division by zero in aggregation
- **EmptySet**: Aggregation on empty dataset
- **ParseError**: Cannot parse value as number

## CLI Commands

```bash
# Execute function
reed function:exec calculate_age 946684800
# Output: 25

# Show function cache stats
reed function:cache-stats
# Output:
# Cache size: 45 entries
# Total hits: 1250
# Hit rate: 96.5%

# Clear function cache
reed function:clear-cache
# Output: ✓ Cache cleared
```

## Acceptance Criteria

- [ ] Function result caching with memoization
- [ ] Cache get/set operations
- [ ] Cache statistics (size, hit rate)
- [ ] Cache eviction (by age)
- [ ] Computed column functions (age, full name, etc.)
- [ ] Aggregation functions (count, sum, avg, min, max)
- [ ] Transformation functions (normalize, trim, capitalize, etc.)
- [ ] Group by with count
- [ ] All tests pass with 100% coverage
- [ ] Performance targets met
- [ ] All code in BBC English
- [ ] All functions have complete documentation
- [ ] Separate test files for each module

## Dependencies

**Requires**: 
- REED-19-02 (Universal Table API - for CsvRow type)

**Blocks**: None

## References
- Service Template: `_workbench/Tickets/templates/service-template.md`
- Service Test Template: `_workbench/Tickets/templates/service-template.test.md`
- REED-19-00: Layer Overview

## Notes

**Function System Philosophy:**
- **Rust functions, not Lua**: Type-safe, compiled, no runtime overhead
- **Pure functions**: No side effects, deterministic results
- **Memoization**: Automatic caching for expensive computations
- **Composable**: Functions can call other functions

**Cache Strategy:**
- **Key**: Hash of (function_name + arguments)
- **Value**: Cached result + metadata (timestamp, hit count)
- **Eviction**: LRU-like based on timestamp
- **Size**: Unlimited (bounded by available memory, can add max_size limit)

**Trade-offs:**
- **Pro**: Rust performance (vs Lua/Python)
- **Pro**: Type safety (compile-time checking)
- **Pro**: Zero overhead (compiled code)
- **Con**: Functions must be compiled (cannot add at runtime)
- **Con**: Cache uses memory (mitigated by eviction policy)

**Future Enhancements:**
- Hot-reload functions (via dynamic library loading)
- Python/Lua bindings for scripting
- Window functions (LEAD, LAG, etc.)
- Recursive functions with cycle detection
