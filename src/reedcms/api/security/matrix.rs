// Copyright 2025 Vivian Voss. Licensed under the Apache Licence, Version 2.0.
// SPDX-Licence-Identifier: Apache-2.0

//! Security Matrix for API Access Control
//!
//! Provides resource and operation-level security with role and permission checks.
//!
//! ## Security Model
//! - Resource-based access control (text, route, meta, config, batch, list)
//! - Operation-level permissions (read, write)
//! - Role-based requirements (user, editor, admin)
//! - Permission-based requirements (text.read, text.write, etc.)
//! - Rate limiting per resource-operation pair
//!
//! ## Configuration
//! - Security rules loaded from `.reed/api.security.csv`
//! - Format: `resource|operation|required_permission|required_role|rate_limit`
//! - Example: `text|read|text.read|user|100/min`
//!
//! ## Performance
//! - Security check: O(1) HashMap lookup
//! - < 1ms per check (cached rules)
//!
//! ## Example Usage
//! ```rust
//! let matrix = SecurityMatrix::load()?;
//! matrix.check_access("text", "read", &user)?;
//! ```

use crate::reedcms::auth::verification::AuthenticatedUser;
use crate::reedcms::csv::read_csv;
use crate::reedcms::reedstream::{ReedError, ReedResult};
use std::collections::HashMap;

/// Security matrix for API access control.
///
/// ## Fields
/// - `rules`: HashMap of security rules (key: "resource.operation")
#[derive(Debug, Clone)]
pub struct SecurityMatrix {
    rules: HashMap<String, SecurityRule>,
}

impl SecurityMatrix {
    /// Loads security matrix from `.reed/api.security.csv`.
    ///
    /// ## Output
    /// - `ReedResult<SecurityMatrix>`: Loaded matrix or error
    ///
    /// ## Performance
    /// - O(n) where n is number of rules
    /// - < 10ms for < 100 rules
    ///
    /// ## Error Conditions
    /// - Returns default rules if CSV file missing
    /// - Returns error if CSV format invalid
    ///
    /// ## Example Usage
    /// ```rust
    /// let matrix = SecurityMatrix::load()?;
    /// ```
    pub fn load() -> ReedResult<Self> {
        let csv_path = ".reed/api.security.csv";

        // If file doesn't exist, use defaults
        if !std::path::Path::new(csv_path).exists() {
            return Ok(Self {
                rules: default_security_rules(),
            });
        }

        // Read CSV using existing csv module
        let records = read_csv(csv_path)?;
        let mut rules = HashMap::new();

        for record in records {
            // Parse CSV record (format: resource|operation|permission|role|rate_limit)
            let parts: Vec<&str> = record.key.split('|').collect();
            if parts.len() < 5 {
                continue; // Skip invalid rows
            }

            let resource = parts[0].to_string();
            let operation = parts[1].to_string();
            let required_permission = if parts[2].is_empty() {
                None
            } else {
                Some(parts[2].to_string())
            };
            let required_role = if parts[3].is_empty() {
                None
            } else {
                Some(parts[3].to_string())
            };
            let rate_limit = if parts[4].is_empty() {
                None
            } else {
                Some(RateLimit::parse(parts[4])?)
            };

            let rule_key = format!("{}.{}", resource, operation);
            rules.insert(
                rule_key,
                SecurityRule {
                    resource,
                    operation,
                    required_permission,
                    required_role,
                    rate_limit,
                },
            );
        }

        Ok(Self { rules })
    }

    /// Checks if user has access to resource operation.
    ///
    /// ## Arguments
    /// - `resource`: Resource type (text, route, meta, config, batch, list)
    /// - `operation`: Operation type (read, write)
    /// - `user`: Authenticated user from AuthMiddleware
    ///
    /// ## Returns
    /// - `Ok(())` if access granted
    /// - `Err(ReedError::AuthError)` if access denied
    ///
    /// ## Performance
    /// - O(1) HashMap lookup
    /// - < 100μs typical
    ///
    /// ## Example Usage
    /// ```rust
    /// matrix.check_access("text", "read", &user)?;
    /// ```
    pub fn check_access(
        &self,
        resource: &str,
        operation: &str,
        user: &AuthenticatedUser,
    ) -> ReedResult<()> {
        let rule_key = format!("{}.{}", resource, operation);

        let rule = self
            .rules
            .get(&rule_key)
            .ok_or_else(|| ReedError::AuthError {
                user: Some(user.username.clone()),
                action: rule_key.clone(),
                reason: format!("No security rule for {}", rule_key),
            })?;

        // Check role requirement
        if let Some(required_role) = &rule.required_role {
            if !user.roles.contains(required_role) {
                return Err(ReedError::AuthError {
                    user: Some(user.username.clone()),
                    action: rule_key,
                    reason: format!("Required role: {}", required_role),
                });
            }
        }

        // Check permission requirement using existing security module
        if let Some(required_permission) = &rule.required_permission {
            if !user.has_permission(required_permission) {
                return Err(ReedError::AuthError {
                    user: Some(user.username.clone()),
                    action: rule_key,
                    reason: format!("Required permission: {}", required_permission),
                });
            }
        }

        Ok(())
    }

    /// Gets rate limit for resource operation.
    ///
    /// ## Arguments
    /// - `resource`: Resource type
    /// - `operation`: Operation type
    ///
    /// ## Returns
    /// - `Option<RateLimit>`: Rate limit if configured
    ///
    /// ## Example Usage
    /// ```rust
    /// if let Some(limit) = matrix.get_rate_limit("text", "read") {
    ///     println!("Limit: {} per {:?}", limit.requests, limit.period);
    /// }
    /// ```
    pub fn get_rate_limit(&self, resource: &str, operation: &str) -> Option<RateLimit> {
        let rule_key = format!("{}.{}", resource, operation);
        self.rules.get(&rule_key).and_then(|r| r.rate_limit.clone())
    }
}

/// Security rule structure.
///
/// ## Fields
/// - `resource`: Resource type (text, route, meta, etc.)
/// - `operation`: Operation type (read, write)
/// - `required_permission`: Optional permission requirement
/// - `required_role`: Optional role requirement
/// - `rate_limit`: Optional rate limit
#[derive(Debug, Clone)]
pub struct SecurityRule {
    pub resource: String,
    pub operation: String,
    pub required_permission: Option<String>,
    pub required_role: Option<String>,
    pub rate_limit: Option<RateLimit>,
}

/// Rate limit configuration.
///
/// ## Fields
/// - `requests`: Number of allowed requests
/// - `period`: Time period for rate limit
#[derive(Debug, Clone)]
pub struct RateLimit {
    pub requests: u32,
    pub period: RateLimitPeriod,
}

impl RateLimit {
    /// Parses rate limit from string (e.g., "100/min").
    ///
    /// ## Arguments
    /// - `s`: Rate limit string
    ///
    /// ## Returns
    /// - `ReedResult<RateLimit>`: Parsed rate limit or error
    ///
    /// ## Error Conditions
    /// - Invalid format (not "number/period")
    /// - Invalid number
    /// - Invalid period
    ///
    /// ## Example Usage
    /// ```rust
    /// let limit = RateLimit::parse("100/min")?;
    /// assert_eq!(limit.requests, 100);
    /// ```
    pub fn parse(s: &str) -> ReedResult<Self> {
        let parts: Vec<&str> = s.split('/').collect();
        if parts.len() != 2 {
            return Err(ReedError::ConfigError {
                component: "rate_limit".to_string(),
                reason: format!("Invalid rate limit format: {}", s),
            });
        }

        let requests = parts[0].parse().map_err(|_| ReedError::ConfigError {
            component: "rate_limit".to_string(),
            reason: format!("Invalid request count: {}", parts[0]),
        })?;

        let period = match parts[1] {
            "sec" | "second" => RateLimitPeriod::Second,
            "min" | "minute" => RateLimitPeriod::Minute,
            "hour" => RateLimitPeriod::Hour,
            "day" => RateLimitPeriod::Day,
            _ => {
                return Err(ReedError::ConfigError {
                    component: "rate_limit".to_string(),
                    reason: format!("Invalid period: {}", parts[1]),
                })
            }
        };

        Ok(Self { requests, period })
    }
}

/// Rate limit period.
#[derive(Debug, Clone, Copy)]
pub enum RateLimitPeriod {
    Second,
    Minute,
    Hour,
    Day,
}

impl RateLimitPeriod {
    /// Returns duration in seconds.
    ///
    /// ## Output
    /// - Duration in seconds
    ///
    /// ## Example Usage
    /// ```rust
    /// let duration = RateLimitPeriod::Minute.duration();
    /// assert_eq!(duration, 60);
    /// ```
    pub fn duration(&self) -> u64 {
        match self {
            RateLimitPeriod::Second => 1,
            RateLimitPeriod::Minute => 60,
            RateLimitPeriod::Hour => 3600,
            RateLimitPeriod::Day => 86400,
        }
    }
}

/// Default security rules if config file missing.
///
/// ## Output
/// - Default rules for all API resources
///
/// ## Default Rules
/// - text.read: user role, text.read permission, 100/min
/// - text.write: editor role, text.write permission, 50/min
/// - All standard API resources covered
fn default_security_rules() -> HashMap<String, SecurityRule> {
    let mut rules = HashMap::new();

    // Text operations
    rules.insert(
        "text.read".to_string(),
        SecurityRule {
            resource: "text".to_string(),
            operation: "read".to_string(),
            required_permission: Some("text.read".to_string()),
            required_role: Some("user".to_string()),
            rate_limit: Some(RateLimit {
                requests: 100,
                period: RateLimitPeriod::Minute,
            }),
        },
    );

    rules.insert(
        "text.write".to_string(),
        SecurityRule {
            resource: "text".to_string(),
            operation: "write".to_string(),
            required_permission: Some("text.write".to_string()),
            required_role: Some("editor".to_string()),
            rate_limit: Some(RateLimit {
                requests: 50,
                period: RateLimitPeriod::Minute,
            }),
        },
    );

    // Route operations
    rules.insert(
        "route.read".to_string(),
        SecurityRule {
            resource: "route".to_string(),
            operation: "read".to_string(),
            required_permission: Some("route.read".to_string()),
            required_role: Some("user".to_string()),
            rate_limit: Some(RateLimit {
                requests: 100,
                period: RateLimitPeriod::Minute,
            }),
        },
    );

    rules.insert(
        "route.write".to_string(),
        SecurityRule {
            resource: "route".to_string(),
            operation: "write".to_string(),
            required_permission: Some("route.write".to_string()),
            required_role: Some("admin".to_string()),
            rate_limit: Some(RateLimit {
                requests: 20,
                period: RateLimitPeriod::Minute,
            }),
        },
    );

    // Meta operations
    rules.insert(
        "meta.read".to_string(),
        SecurityRule {
            resource: "meta".to_string(),
            operation: "read".to_string(),
            required_permission: Some("meta.read".to_string()),
            required_role: Some("user".to_string()),
            rate_limit: Some(RateLimit {
                requests: 100,
                period: RateLimitPeriod::Minute,
            }),
        },
    );

    rules.insert(
        "meta.write".to_string(),
        SecurityRule {
            resource: "meta".to_string(),
            operation: "write".to_string(),
            required_permission: Some("meta.write".to_string()),
            required_role: Some("admin".to_string()),
            rate_limit: Some(RateLimit {
                requests: 20,
                period: RateLimitPeriod::Minute,
            }),
        },
    );

    // Config operations
    rules.insert(
        "config.read".to_string(),
        SecurityRule {
            resource: "config".to_string(),
            operation: "read".to_string(),
            required_permission: Some("config.read".to_string()),
            required_role: Some("admin".to_string()),
            rate_limit: Some(RateLimit {
                requests: 50,
                period: RateLimitPeriod::Minute,
            }),
        },
    );

    rules.insert(
        "config.write".to_string(),
        SecurityRule {
            resource: "config".to_string(),
            operation: "write".to_string(),
            required_permission: Some("config.write".to_string()),
            required_role: Some("admin".to_string()),
            rate_limit: Some(RateLimit {
                requests: 10,
                period: RateLimitPeriod::Minute,
            }),
        },
    );

    // Batch operations
    rules.insert(
        "batch.read".to_string(),
        SecurityRule {
            resource: "batch".to_string(),
            operation: "read".to_string(),
            required_permission: Some("batch.read".to_string()),
            required_role: Some("user".to_string()),
            rate_limit: Some(RateLimit {
                requests: 10,
                period: RateLimitPeriod::Minute,
            }),
        },
    );

    rules.insert(
        "batch.write".to_string(),
        SecurityRule {
            resource: "batch".to_string(),
            operation: "write".to_string(),
            required_permission: Some("batch.write".to_string()),
            required_role: Some("editor".to_string()),
            rate_limit: Some(RateLimit {
                requests: 5,
                period: RateLimitPeriod::Minute,
            }),
        },
    );

    // List operations
    rules.insert(
        "list.read".to_string(),
        SecurityRule {
            resource: "list".to_string(),
            operation: "read".to_string(),
            required_permission: Some("list.read".to_string()),
            required_role: Some("user".to_string()),
            rate_limit: Some(RateLimit {
                requests: 50,
                period: RateLimitPeriod::Minute,
            }),
        },
    );

    rules
}
