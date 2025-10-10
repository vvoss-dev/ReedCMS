// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Tests for FreeBSD-style syslog logger.

#[cfg(test)]
mod tests {
    use crate::reedcms::monitor::syslog::{LogLevel, OutputMode, SysLogger};

    #[test]
    fn test_log_levels() {
        // Test all 8 RFC 5424 log levels
        assert_eq!(LogLevel::EMERG as u8, 0);
        assert_eq!(LogLevel::ALERT as u8, 1);
        assert_eq!(LogLevel::CRIT as u8, 2);
        assert_eq!(LogLevel::ERROR as u8, 3);
        assert_eq!(LogLevel::WARN as u8, 4);
        assert_eq!(LogLevel::NOTICE as u8, 5);
        assert_eq!(LogLevel::INFO as u8, 6);
        assert_eq!(LogLevel::DEBUG as u8, 7);
    }

    #[test]
    fn test_log_level_strings() {
        assert_eq!(LogLevel::EMERG.as_str(), "EMERG");
        assert_eq!(LogLevel::ALERT.as_str(), "ALERT");
        assert_eq!(LogLevel::CRIT.as_str(), "CRIT");
        assert_eq!(LogLevel::ERROR.as_str(), "ERROR");
        assert_eq!(LogLevel::WARN.as_str(), "WARN");
        assert_eq!(LogLevel::NOTICE.as_str(), "NOTICE");
        assert_eq!(LogLevel::INFO.as_str(), "INFO");
        assert_eq!(LogLevel::DEBUG.as_str(), "DEBUG");
    }

    #[test]
    fn test_silent_mode() {
        let mut logger = SysLogger::new(OutputMode::Silent, LogLevel::INFO).unwrap();

        // Should not panic or fail
        logger.log(LogLevel::INFO, "Test message");
        logger.log_metric("counter", "test", "42");
    }

    #[test]
    fn test_log_level_filtering() {
        let mut logger = SysLogger::new(OutputMode::Silent, LogLevel::WARN).unwrap();

        // These should be filtered out (higher than WARN)
        logger.log(LogLevel::NOTICE, "Should be filtered");
        logger.log(LogLevel::INFO, "Should be filtered");
        logger.log(LogLevel::DEBUG, "Should be filtered");

        // These should pass through
        logger.log(LogLevel::EMERG, "Should pass");
        logger.log(LogLevel::ALERT, "Should pass");
        logger.log(LogLevel::CRIT, "Should pass");
        logger.log(LogLevel::ERROR, "Should pass");
        logger.log(LogLevel::WARN, "Should pass");
    }

    #[test]
    fn test_metric_logging() {
        let mut logger = SysLogger::new(OutputMode::Silent, LogLevel::INFO).unwrap();

        logger.log_metric("counter", "requests_total", "42");
        logger.log_metric("gauge", "memory_usage_mb", "128.5");
        logger.log_metric("histogram", "response_time_ms", "45.2");
    }

    #[test]
    fn test_log_file_creation() {
        // Test that log file mode creates the directory
        let result = SysLogger::new(OutputMode::Log, LogLevel::INFO);
        assert!(result.is_ok());

        // Verify .reed/flow directory exists
        assert!(std::path::Path::new(".reed/flow").exists());
    }
}
