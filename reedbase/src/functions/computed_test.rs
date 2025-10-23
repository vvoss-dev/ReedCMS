// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Tests for computed functions.

#[cfg(test)]
mod tests {
    use crate::functions::cache::get_cache;
    use crate::functions::computed::*;

    #[test]
    fn test_calculate_age_valid() {
        get_cache().clear(); // Clear global cache

        let age = calculate_age("1990-05-15").unwrap();
        // Assuming test runs in 2025
        assert_eq!(age, "35");
    }

    #[test]
    fn test_calculate_age_not_birthday_yet() {
        get_cache().clear();

        // Future birthday this year (hasn't happened yet)
        let age = calculate_age("1990-12-31").unwrap();
        // If today is before Dec 31, age should be 34, otherwise 35
        assert!(age == "34" || age == "35");
    }

    #[test]
    fn test_calculate_age_invalid_date() {
        get_cache().clear();

        let result = calculate_age("invalid-date");
        assert!(result.is_err());
    }

    #[test]
    fn test_calculate_age_cached() {
        get_cache().clear();

        // First call
        let age1 = calculate_age("1990-05-15").unwrap();

        // Second call should be cached
        let age2 = calculate_age("1990-05-15").unwrap();

        assert_eq!(age1, age2);

        let stats = get_cache().stats();
        assert!(stats.hits >= 1); // At least one cache hit
    }

    #[test]
    fn test_full_name() {
        get_cache().clear();

        let name = full_name("John", "Doe").unwrap();
        assert_eq!(name, "John Doe");
    }

    #[test]
    fn test_full_name_with_whitespace() {
        get_cache().clear();

        let name = full_name("  Jane  ", "  Smith  ").unwrap();
        assert_eq!(name, "Jane Smith");
    }

    #[test]
    fn test_full_name_cached() {
        get_cache().clear();

        let name1 = full_name("John", "Doe").unwrap();
        let name2 = full_name("John", "Doe").unwrap();

        assert_eq!(name1, name2);

        let stats = get_cache().stats();
        assert!(stats.hits >= 1);
    }

    #[test]
    fn test_days_since_past() {
        get_cache().clear();

        // 100 days ago from today would be negative in the result
        let days = days_since("2025-01-01").unwrap();

        // Should be positive (days since January 1st)
        let days_num: i64 = days.parse().unwrap();
        assert!(days_num > 0);
    }

    #[test]
    fn test_days_since_future() {
        get_cache().clear();

        let days = days_since("2026-01-01").unwrap();

        // Should be negative (future date)
        let days_num: i64 = days.parse().unwrap();
        assert!(days_num < 0);
    }

    #[test]
    fn test_days_since_invalid_date() {
        get_cache().clear();

        let result = days_since("not-a-date");
        assert!(result.is_err());
    }

    #[test]
    fn test_is_expired_past_date() {
        get_cache().clear();

        let expired = is_expired("2020-01-01").unwrap();
        assert_eq!(expired, "true");
    }

    #[test]
    fn test_is_expired_future_date() {
        get_cache().clear();

        let expired = is_expired("2030-01-01").unwrap();
        assert_eq!(expired, "false");
    }

    #[test]
    fn test_is_expired_invalid_date() {
        get_cache().clear();

        let result = is_expired("invalid");
        assert!(result.is_err());
    }

    #[test]
    fn test_format_date_uk_style() {
        get_cache().clear();

        let formatted = format_date("2025-10-23", "%d/%m/%Y").unwrap();
        assert_eq!(formatted, "23/10/2025");
    }

    #[test]
    fn test_format_date_us_style() {
        get_cache().clear();

        let formatted = format_date("2025-10-23", "%m/%d/%Y").unwrap();
        assert_eq!(formatted, "10/23/2025");
    }

    #[test]
    fn test_format_date_long_format() {
        get_cache().clear();

        let formatted = format_date("2025-10-23", "%B %d, %Y").unwrap();
        assert_eq!(formatted, "October 23, 2025");
    }

    #[test]
    fn test_format_date_invalid() {
        get_cache().clear();

        let result = format_date("not-a-date", "%Y-%m-%d");
        assert!(result.is_err());
    }

    #[test]
    fn test_calculate_discount() {
        get_cache().clear();

        let discounted = calculate_discount("100.00", "20").unwrap();
        assert_eq!(discounted, "80.00");
    }

    #[test]
    fn test_calculate_discount_fifty_percent() {
        get_cache().clear();

        let discounted = calculate_discount("50.00", "50").unwrap();
        assert_eq!(discounted, "25.00");
    }

    #[test]
    fn test_calculate_discount_rounding() {
        get_cache().clear();

        let discounted = calculate_discount("99.99", "15").unwrap();
        assert_eq!(discounted, "84.99");
    }

    #[test]
    fn test_calculate_discount_invalid_price() {
        get_cache().clear();

        let result = calculate_discount("not-a-number", "20");
        assert!(result.is_err());
    }

    #[test]
    fn test_calculate_discount_invalid_percentage() {
        get_cache().clear();

        let result = calculate_discount("100.00", "not-a-number");
        assert!(result.is_err());
    }

    #[test]
    fn test_calculate_discount_cached() {
        get_cache().clear();

        let price1 = calculate_discount("100.00", "20").unwrap();
        let price2 = calculate_discount("100.00", "20").unwrap();

        assert_eq!(price1, price2);

        let stats = get_cache().stats();
        assert!(stats.hits >= 1);
    }
}
