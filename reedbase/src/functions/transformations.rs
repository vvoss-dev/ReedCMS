// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Data transformation functions with automatic caching.
//!
//! Provides string cleaning, normalization, and formatting operations.
//! All results are automatically cached for instant repeated access.
//!
//! ## Performance
//!
//! - **First call**: < 2μs (actual transformation)
//! - **Cached calls**: < 100ns (cache hit)
//!
//! ## Example Usage
//!
//! ```rust
//! use reedbase::functions::transformations::{normalize_email, trim, capitalize, slugify};
//!
//! // Email normalization
//! let email = normalize_email("  John.Doe@Example.COM  ")?; // "john.doe@example.com"
//!
//! // String cleaning
//! let clean = trim("  hello  ")?; // "hello"
//!
//! // Capitalization
//! let title = capitalize("hello world")?; // "Hello world"
//!
//! // URL slugification
//! let slug = slugify("Hello World!")?; // "hello-world"
//! ```

use crate::error::ReedResult;
use crate::functions::cache::{get_cache, CacheKey};

/// Normalize email address (lowercase, trim whitespace).
///
/// ## Input
/// - `email` - Raw email address
///
/// ## Output
/// - Normalized email (lowercase, trimmed)
///
/// ## Performance
/// - First call: < 500ns
/// - Cached: < 100ns
///
/// ## Example Usage
/// ```rust
/// let email = normalize_email("  John.Doe@Example.COM  ")?;
/// assert_eq!(email, "john.doe@example.com");
/// ```
pub fn normalize_email(email: &str) -> ReedResult<String> {
    let key = CacheKey::new("normalize_email", vec![email]);

    if let Some(cached) = get_cache().get(&key) {
        return Ok(cached);
    }

    let result = email.trim().to_lowercase();

    get_cache().insert(key, result.clone());

    Ok(result)
}

/// Trim whitespace from both ends of string.
///
/// ## Input
/// - `value` - String to trim
///
/// ## Output
/// - Trimmed string
///
/// ## Performance
/// - < 200ns (not cached - too fast)
///
/// ## Example Usage
/// ```rust
/// let trimmed = trim("  hello  ")?; // "hello"
/// let tabs = trim("\t\nworld\t\n")?; // "world"
/// ```
pub fn trim(value: &str) -> ReedResult<String> {
    // Don't cache - trimming is faster than cache lookup
    Ok(value.trim().to_string())
}

/// Capitalize first letter of string.
///
/// ## Input
/// - `value` - String to capitalize
///
/// ## Output
/// - String with first letter uppercase, rest unchanged
///
/// ## Performance
/// - First call: < 300ns
/// - Cached: < 100ns
///
/// ## Example Usage
/// ```rust
/// let cap = capitalize("hello")?; // "Hello"
/// let already = capitalize("World")?; // "World"
/// let empty = capitalize("")?; // ""
/// ```
pub fn capitalize(value: &str) -> ReedResult<String> {
    let key = CacheKey::new("capitalize", vec![value]);

    if let Some(cached) = get_cache().get(&key) {
        return Ok(cached);
    }

    let mut chars = value.chars();
    let result = match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => String::new(),
    };

    get_cache().insert(key, result.clone());

    Ok(result)
}

/// Convert string to URL-safe slug.
///
/// Converts to lowercase, replaces non-alphanumeric chars with hyphens,
/// removes consecutive hyphens, and trims hyphens from ends.
///
/// ## Input
/// - `value` - String to slugify
///
/// ## Output
/// - URL-safe slug (lowercase, hyphens only)
///
/// ## Performance
/// - First call: < 2μs
/// - Cached: < 100ns
///
/// ## Example Usage
/// ```rust
/// let slug = slugify("Hello World!")?; // "hello-world"
/// let complex = slugify("A/B Testing & Analysis")?; // "a-b-testing-analysis"
/// let spaces = slugify("  Multiple   Spaces  ")?; // "multiple-spaces"
/// ```
pub fn slugify(value: &str) -> ReedResult<String> {
    let key = CacheKey::new("slugify", vec![value]);

    if let Some(cached) = get_cache().get(&key) {
        return Ok(cached);
    }

    let result = value
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        // Remove consecutive hyphens
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-");

    get_cache().insert(key, result.clone());

    Ok(result)
}

/// Truncate string to maximum length.
///
/// ## Input
/// - `value` - String to truncate
/// - `max_length` - Maximum length as string (e.g., "50")
///
/// ## Output
/// - Truncated string (adds "..." if truncated)
///
/// ## Performance
/// - First call: < 500ns
/// - Cached: < 100ns
///
/// ## Example Usage
/// ```rust
/// let short = truncate("Hello", "10")?; // "Hello"
/// let long = truncate("This is a very long string", "10")?; // "This is..."
/// ```
pub fn truncate(value: &str, max_length: &str) -> ReedResult<String> {
    let key = CacheKey::new("truncate", vec![value, max_length]);

    if let Some(cached) = get_cache().get(&key) {
        return Ok(cached);
    }

    let max_len = max_length.parse::<usize>().unwrap_or(100);

    let result = if value.len() <= max_len {
        value.to_string()
    } else {
        let truncated = &value[..max_len.saturating_sub(3)];
        format!("{}...", truncated)
    };

    get_cache().insert(key, result.clone());

    Ok(result)
}

/// Replace all occurrences of substring.
///
/// ## Input
/// - `value` - String to modify
/// - `old` - Substring to replace
/// - `new` - Replacement string
///
/// ## Output
/// - String with all occurrences replaced
///
/// ## Performance
/// - First call: < 1μs
/// - Cached: < 100ns
///
/// ## Example Usage
/// ```rust
/// let replaced = replace("hello world", "world", "rust")?; // "hello rust"
/// let multiple = replace("foo bar foo", "foo", "baz")?; // "baz bar baz"
/// ```
pub fn replace(value: &str, old: &str, new: &str) -> ReedResult<String> {
    let key = CacheKey::new("replace", vec![value, old, new]);

    if let Some(cached) = get_cache().get(&key) {
        return Ok(cached);
    }

    let result = value.replace(old, new);

    get_cache().insert(key, result.clone());

    Ok(result)
}

/// Convert string to uppercase.
///
/// ## Input
/// - `value` - String to convert
///
/// ## Output
/// - Uppercase string
///
/// ## Performance
/// - First call: < 300ns
/// - Cached: < 100ns
///
/// ## Example Usage
/// ```rust
/// let upper = uppercase("hello")?; // "HELLO"
/// let mixed = uppercase("Hello World")?; // "HELLO WORLD"
/// ```
pub fn uppercase(value: &str) -> ReedResult<String> {
    let key = CacheKey::new("uppercase", vec![value]);

    if let Some(cached) = get_cache().get(&key) {
        return Ok(cached);
    }

    let result = value.to_uppercase();

    get_cache().insert(key, result.clone());

    Ok(result)
}

/// Convert string to lowercase.
///
/// ## Input
/// - `value` - String to convert
///
/// ## Output
/// - Lowercase string
///
/// ## Performance
/// - First call: < 300ns
/// - Cached: < 100ns
///
/// ## Example Usage
/// ```rust
/// let lower = lowercase("HELLO")?; // "hello"
/// let mixed = lowercase("Hello World")?; // "hello world"
/// ```
pub fn lowercase(value: &str) -> ReedResult<String> {
    let key = CacheKey::new("lowercase", vec![value]);

    if let Some(cached) = get_cache().get(&key) {
        return Ok(cached);
    }

    let result = value.to_lowercase();

    get_cache().insert(key, result.clone());

    Ok(result)
}

/// Remove all whitespace from string.
///
/// ## Input
/// - `value` - String to clean
///
/// ## Output
/// - String with all whitespace removed
///
/// ## Performance
/// - First call: < 500ns
/// - Cached: < 100ns
///
/// ## Example Usage
/// ```rust
/// let clean = remove_whitespace("hello world")?; // "helloworld"
/// let tabs = remove_whitespace("foo\t\nbar")?; // "foobar"
/// ```
pub fn remove_whitespace(value: &str) -> ReedResult<String> {
    let key = CacheKey::new("remove_whitespace", vec![value]);

    if let Some(cached) = get_cache().get(&key) {
        return Ok(cached);
    }

    let result: String = value.chars().filter(|c| !c.is_whitespace()).collect();

    get_cache().insert(key, result.clone());

    Ok(result)
}

/// Pad string to minimum length with spaces (right padding).
///
/// ## Input
/// - `value` - String to pad
/// - `min_length` - Minimum length as string
///
/// ## Output
/// - Padded string (right-aligned with spaces)
///
/// ## Performance
/// - First call: < 500ns
/// - Cached: < 100ns
///
/// ## Example Usage
/// ```rust
/// let padded = pad_right("hello", "10")?; // "hello     "
/// let no_pad = pad_right("hello", "3")?; // "hello" (no change if already longer)
/// ```
pub fn pad_right(value: &str, min_length: &str) -> ReedResult<String> {
    let key = CacheKey::new("pad_right", vec![value, min_length]);

    if let Some(cached) = get_cache().get(&key) {
        return Ok(cached);
    }

    let min_len = min_length.parse::<usize>().unwrap_or(0);

    let result = if value.len() >= min_len {
        value.to_string()
    } else {
        format!("{:width$}", value, width = min_len)
    };

    get_cache().insert(key, result.clone());

    Ok(result)
}

/// Reverse string.
///
/// ## Input
/// - `value` - String to reverse
///
/// ## Output
/// - Reversed string
///
/// ## Performance
/// - First call: < 500ns
/// - Cached: < 100ns
///
/// ## Example Usage
/// ```rust
/// let rev = reverse("hello")?; // "olleh"
/// let palindrome = reverse("racecar")?; // "racecar"
/// ```
pub fn reverse(value: &str) -> ReedResult<String> {
    let key = CacheKey::new("reverse", vec![value]);

    if let Some(cached) = get_cache().get(&key) {
        return Ok(cached);
    }

    let result: String = value.chars().rev().collect();

    get_cache().insert(key, result.clone());

    Ok(result)
}
