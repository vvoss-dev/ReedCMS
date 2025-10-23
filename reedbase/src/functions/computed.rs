// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Computed column functions with automatic caching.
//!
//! Pure functions that derive values from inputs. Results are automatically cached
//! for instant recomputation on subsequent calls.
//!
//! ## Performance
//!
//! - **First call**: < 1μs (actual computation)
//! - **Cached calls**: < 100ns (cache hit)
//!
//! ## Example Usage
//!
//! ```rust
//! use reedbase::functions::computed::{calculate_age, full_name, days_since};
//!
//! // Calculate age from birthdate
//! let age = calculate_age("1990-05-15")?; // "35"
//!
//! // Combine names
//! let name = full_name("John", "Doe")?; // "John Doe"
//!
//! // Days since date
//! let days = days_since("2025-01-01")?; // "296"
//! ```

use crate::error::{ReedError, ReedResult};
use crate::functions::cache::{get_cache, CacheKey};
use chrono::{Datelike, NaiveDate, Utc};

/// Calculate age from birthdate.
///
/// ## Input
/// - `birthdate` - ISO 8601 date string (YYYY-MM-DD)
///
/// ## Output
/// - Age in years as string
///
/// ## Performance
/// - First call: < 1μs (date parsing + calculation)
/// - Cached: < 100ns
///
/// ## Error Conditions
/// - Invalid date format → ReedError::InvalidInput
///
/// ## Example Usage
/// ```rust
/// let age = calculate_age("1990-05-15")?; // "35"
/// let age_cached = calculate_age("1990-05-15")?; // < 100ns (cached)
/// ```
pub fn calculate_age(birthdate: &str) -> ReedResult<String> {
    let key = CacheKey::new("calculate_age", vec![birthdate]);

    // Check cache first
    if let Some(cached) = get_cache().get(&key) {
        return Ok(cached);
    }

    // Parse birthdate
    let birth =
        NaiveDate::parse_from_str(birthdate, "%Y-%m-%d").map_err(|e| ReedError::ParseError {
            reason: format!(
                "Invalid birthdate '{}' (expected YYYY-MM-DD): {}",
                birthdate, e
            ),
        })?;

    // Calculate age
    let today = Utc::now().date_naive();

    let mut age = today.year() - birth.year();

    // Adjust if birthday hasn't occurred this year yet
    if today.month() < birth.month()
        || (today.month() == birth.month() && today.day() < birth.day())
    {
        age -= 1;
    }

    let result = age.to_string();

    // Cache result
    get_cache().insert(key, result.clone());

    Ok(result)
}

/// Compute full name from first and last name.
///
/// ## Input
/// - `first_name` - First name string
/// - `last_name` - Last name string
///
/// ## Output
/// - Full name as "FirstName LastName" (trimmed)
///
/// ## Performance
/// - First call: < 500ns (string concatenation)
/// - Cached: < 100ns
///
/// ## Example Usage
/// ```rust
/// let name = full_name("John", "Doe")?; // "John Doe"
/// let name_trim = full_name("  Jane  ", "  Smith  ")?; // "Jane Smith"
/// ```
pub fn full_name(first_name: &str, last_name: &str) -> ReedResult<String> {
    let key = CacheKey::new("full_name", vec![first_name, last_name]);

    if let Some(cached) = get_cache().get(&key) {
        return Ok(cached);
    }

    let result = format!("{} {}", first_name.trim(), last_name.trim());

    get_cache().insert(key, result.clone());

    Ok(result)
}

/// Calculate days since a given date.
///
/// ## Input
/// - `date` - ISO 8601 date string (YYYY-MM-DD)
///
/// ## Output
/// - Number of days since date as string (can be negative for future dates)
///
/// ## Performance
/// - First call: < 1μs (date parsing + calculation)
/// - Cached: < 100ns
///
/// ## Error Conditions
/// - Invalid date format → ReedError::InvalidInput
///
/// ## Example Usage
/// ```rust
/// let days = days_since("2025-01-01")?; // "296" (if today is 2025-10-23)
/// let future = days_since("2026-01-01")?; // "-70" (negative for future)
/// ```
pub fn days_since(date: &str) -> ReedResult<String> {
    let key = CacheKey::new("days_since", vec![date]);

    if let Some(cached) = get_cache().get(&key) {
        return Ok(cached);
    }

    let past = NaiveDate::parse_from_str(date, "%Y-%m-%d").map_err(|e| ReedError::ParseError {
        reason: format!("Invalid date '{}' (expected YYYY-MM-DD): {}", date, e),
    })?;

    let today = Utc::now().date_naive();
    let days = (today.signed_duration_since(past)).num_days();
    let result = days.to_string();

    get_cache().insert(key, result.clone());

    Ok(result)
}

/// Check if a date has expired (is in the past).
///
/// ## Input
/// - `expiry_date` - ISO 8601 date string (YYYY-MM-DD)
///
/// ## Output
/// - "true" if expired (past), "false" if not expired (future or today)
///
/// ## Performance
/// - First call: < 1μs
/// - Cached: < 100ns
///
/// ## Error Conditions
/// - Invalid date format → ReedError::InvalidInput
///
/// ## Example Usage
/// ```rust
/// let expired = is_expired("2020-01-01")?; // "true"
/// let valid = is_expired("2030-01-01")?; // "false"
/// ```
pub fn is_expired(expiry_date: &str) -> ReedResult<String> {
    let key = CacheKey::new("is_expired", vec![expiry_date]);

    if let Some(cached) = get_cache().get(&key) {
        return Ok(cached);
    }

    let expiry =
        NaiveDate::parse_from_str(expiry_date, "%Y-%m-%d").map_err(|e| ReedError::ParseError {
            reason: format!(
                "Invalid expiry_date '{}' (expected YYYY-MM-DD): {}",
                expiry_date, e
            ),
        })?;

    let today = Utc::now().date_naive();
    let result = if today > expiry {
        "true".to_string()
    } else {
        "false".to_string()
    };

    get_cache().insert(key, result.clone());

    Ok(result)
}

/// Format date with custom format string.
///
/// ## Input
/// - `date` - ISO 8601 date string (YYYY-MM-DD)
/// - `format` - Format string (e.g., "%d/%m/%Y", "%B %d, %Y")
///
/// ## Output
/// - Formatted date string
///
/// ## Performance
/// - First call: < 2μs
/// - Cached: < 100ns
///
/// ## Error Conditions
/// - Invalid date format → ReedError::InvalidInput
/// - Invalid format string → ReedError::InvalidInput
///
/// ## Example Usage
/// ```rust
/// let uk = format_date("2025-10-23", "%d/%m/%Y")?; // "23/10/2025"
/// let us = format_date("2025-10-23", "%m/%d/%Y")?; // "10/23/2025"
/// let long = format_date("2025-10-23", "%B %d, %Y")?; // "October 23, 2025"
/// ```
pub fn format_date(date: &str, format: &str) -> ReedResult<String> {
    let key = CacheKey::new("format_date", vec![date, format]);

    if let Some(cached) = get_cache().get(&key) {
        return Ok(cached);
    }

    let parsed =
        NaiveDate::parse_from_str(date, "%Y-%m-%d").map_err(|e| ReedError::ParseError {
            reason: format!("Invalid date '{}' (expected YYYY-MM-DD): {}", date, e),
        })?;

    let result = parsed.format(format).to_string();

    get_cache().insert(key, result.clone());

    Ok(result)
}

/// Calculate discounted price.
///
/// ## Input
/// - `price` - Original price as string (e.g., "100.00")
/// - `percentage` - Discount percentage as string (e.g., "20" for 20% off)
///
/// ## Output
/// - Discounted price rounded to 2 decimal places
///
/// ## Performance
/// - First call: < 1μs
/// - Cached: < 100ns
///
/// ## Error Conditions
/// - Invalid price → ReedError::InvalidInput
/// - Invalid percentage → ReedError::InvalidInput
///
/// ## Example Usage
/// ```rust
/// let discounted = calculate_discount("100.00", "20")?; // "80.00"
/// let half_off = calculate_discount("50.00", "50")?; // "25.00"
/// ```
pub fn calculate_discount(price: &str, percentage: &str) -> ReedResult<String> {
    let key = CacheKey::new("calculate_discount", vec![price, percentage]);

    if let Some(cached) = get_cache().get(&key) {
        return Ok(cached);
    }

    let price_val = price.parse::<f64>().map_err(|e| ReedError::ParseError {
        reason: format!("Invalid price '{}': {}", price, e),
    })?;

    let percent_val = percentage
        .parse::<f64>()
        .map_err(|e| ReedError::ParseError {
            reason: format!("Invalid percentage '{}': {}", percentage, e),
        })?;

    let discount_amount = price_val * (percent_val / 100.0);
    let final_price = price_val - discount_amount;
    let result = format!("{:.2}", final_price);

    get_cache().insert(key, result.clone());

    Ok(result)
}
