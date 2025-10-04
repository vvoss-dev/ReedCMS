// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0
//
// == AI CODING GUIDELINES ==
// MANDATORY: Follow KISS principle - One test = One assertion
// MANDATORY: BBC English for all test names and documentation
// MANDATORY: Test all error paths explicitly
// MANDATORY: Performance assertions for all operations
//
// == FILE PURPOSE ==
// This file: Tests for rate_limit.rs API rate limiting operations
// Architecture: Separate test file following KISS principle
// Performance: All tests must complete within defined time limits
// Test Scope: Unit tests for sliding window rate limiting

#[cfg(test)]
mod tests {
    use crate::reedcms::api::security::rate_limit::{
        check_rate_limit, cleanup_rate_limits, RateLimit, RateLimitPeriod,
    };
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_rate_limit_period_duration_minute() {
        let period = RateLimitPeriod::Minute;
        assert_eq!(period.duration(), 60);
    }

    #[test]
    fn test_rate_limit_period_duration_hour() {
        let period = RateLimitPeriod::Hour;
        assert_eq!(period.duration(), 3600);
    }

    #[test]
    fn test_rate_limit_period_duration_day() {
        let period = RateLimitPeriod::Day;
        assert_eq!(period.duration(), 86400);
    }

    #[test]
    fn test_rate_limit_first_request_allowed() {
        let limit = RateLimit {
            requests: 5,
            period: RateLimitPeriod::Minute,
        };

        let result = check_rate_limit("user1", "text:read", &limit);
        assert!(result.is_ok());
    }

    #[test]
    fn test_rate_limit_multiple_requests_within_limit() {
        let limit = RateLimit {
            requests: 5,
            period: RateLimitPeriod::Minute,
        };

        for i in 0..5 {
            let result = check_rate_limit(&format!("user_multi_{}", i), "text:read", &limit);
            assert!(result.is_ok(), "Request {} should be allowed", i);
        }
    }

    #[test]
    fn test_rate_limit_exceeds_limit() {
        let limit = RateLimit {
            requests: 3,
            period: RateLimitPeriod::Minute,
        };

        // Make 3 requests (should all succeed)
        for _ in 0..3 {
            let result = check_rate_limit("user_exceed", "text:read", &limit);
            assert!(result.is_ok());
        }

        // 4th request should fail
        let result = check_rate_limit("user_exceed", "text:read", &limit);
        assert!(result.is_err());
    }

    #[test]
    fn test_rate_limit_different_users_independent() {
        let limit = RateLimit {
            requests: 2,
            period: RateLimitPeriod::Minute,
        };

        // User 1 makes 2 requests
        assert!(check_rate_limit("user_a", "text:read", &limit).is_ok());
        assert!(check_rate_limit("user_a", "text:read", &limit).is_ok());

        // User 2 should still be able to make 2 requests
        assert!(check_rate_limit("user_b", "text:read", &limit).is_ok());
        assert!(check_rate_limit("user_b", "text:read", &limit).is_ok());
    }

    #[test]
    fn test_rate_limit_different_operations_independent() {
        let limit = RateLimit {
            requests: 2,
            period: RateLimitPeriod::Minute,
        };

        // Make 2 requests for text:read
        assert!(check_rate_limit("user_ops", "text:read", &limit).is_ok());
        assert!(check_rate_limit("user_ops", "text:read", &limit).is_ok());

        // Should still be able to make 2 requests for text:write
        assert!(check_rate_limit("user_ops", "text:write", &limit).is_ok());
        assert!(check_rate_limit("user_ops", "text:write", &limit).is_ok());
    }

    #[test]
    fn test_rate_limit_sliding_window() {
        let limit = RateLimit {
            requests: 2,
            period: RateLimitPeriod::Minute, // 60 seconds
        };

        // Make 2 requests (fills limit)
        assert!(check_rate_limit("user_sliding", "text:read", &limit).is_ok());
        assert!(check_rate_limit("user_sliding", "text:read", &limit).is_ok());

        // 3rd request should fail immediately
        assert!(check_rate_limit("user_sliding", "text:read", &limit).is_err());

        // Wait 1 second (still within window)
        thread::sleep(Duration::from_secs(1));

        // Should still fail (requests still in window)
        assert!(check_rate_limit("user_sliding", "text:read", &limit).is_err());
    }

    #[test]
    fn test_cleanup_rate_limits() {
        let limit = RateLimit {
            requests: 5,
            period: RateLimitPeriod::Minute,
        };

        // Make some requests
        for i in 0..3 {
            check_rate_limit(&format!("cleanup_user_{}", i), "text:read", &limit).ok();
        }

        // Run cleanup (should remove expired entries)
        cleanup_rate_limits();

        // Cleanup should complete without errors
        // Note: Cannot easily verify internal state without exposing internals
    }

    #[test]
    fn test_rate_limit_zero_requests() {
        let limit = RateLimit {
            requests: 0,
            period: RateLimitPeriod::Minute,
        };

        // First request should fail (limit is 0)
        let result = check_rate_limit("user_zero", "text:read", &limit);
        assert!(result.is_err());
    }

    #[test]
    fn test_rate_limit_high_volume() {
        let limit = RateLimit {
            requests: 1000,
            period: RateLimitPeriod::Minute,
        };

        // Make 1000 requests (should all succeed)
        for _ in 0..1000 {
            let result = check_rate_limit("user_volume", "text:read", &limit);
            assert!(result.is_ok());
        }

        // 1001st request should fail
        let result = check_rate_limit("user_volume", "text:read", &limit);
        assert!(result.is_err());
    }

    #[test]
    fn test_rate_limit_concurrent_users() {
        let limit = RateLimit {
            requests: 10,
            period: RateLimitPeriod::Minute,
        };

        // Simulate 10 different users each making 10 requests
        for user_id in 0..10 {
            for _ in 0..10 {
                let result =
                    check_rate_limit(&format!("concurrent_user_{}", user_id), "text:read", &limit);
                assert!(result.is_ok());
            }
        }
    }
}
