// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Request inspector for debugging.
//!
//! ## Features
//! - Simulate request analysis
//! - Show timing breakdown
//! - Display response headers
//!
//! ## CLI Usage
//! ```bash
//! reed debug:request /knowledge
//! ```

use std::collections::HashMap;
use std::time::Duration;

/// Request inspection data.
#[derive(Debug, Clone)]
pub struct RequestInspection {
    pub url: String,
    pub method: String,
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub response_headers: HashMap<String, String>,
    pub timing: HashMap<String, Duration>,
}

impl RequestInspection {
    /// Creates new request inspection.
    pub fn new(url: &str) -> Self {
        let mut headers = HashMap::new();
        headers.insert("User-Agent".to_string(), "ReedCMS-Debug/1.0".to_string());
        headers.insert("Accept".to_string(), "*/*".to_string());
        headers.insert("Host".to_string(), "localhost:8333".to_string());

        let mut response_headers = HashMap::new();
        response_headers.insert(
            "Content-Type".to_string(),
            "text/html; charset=utf-8".to_string(),
        );
        response_headers.insert(
            "Cache-Control".to_string(),
            "public, max-age=3600".to_string(),
        );

        Self {
            url: url.to_string(),
            method: "GET".to_string(),
            status: 200,
            headers,
            response_headers,
            timing: HashMap::new(),
        }
    }

    /// Simulates request timing.
    pub fn simulate_timing(&mut self) {
        self.timing
            .insert("routing".to_string(), Duration::from_millis(2));
        self.timing
            .insert("reedbase".to_string(), Duration::from_millis(8));
        self.timing
            .insert("rendering".to_string(), Duration::from_millis(32));
        self.timing
            .insert("total".to_string(), Duration::from_millis(42));
    }

    /// Formats inspection report.
    pub fn format(&self) -> String {
        let mut output = format!("🔍 Request Inspector: {}\n\n", self.url);

        output.push_str("Request:\n");
        output.push_str(&format!("  Method: {}\n", self.method));
        output.push_str(&format!("  URL: {}\n", self.url));
        output.push_str("\nRequest Headers:\n");
        for (key, value) in &self.headers {
            output.push_str(&format!("  {}: {}\n", key, value));
        }

        output.push_str("\nResponse:\n");
        output.push_str(&format!("  Status: {} OK\n", self.status));
        output.push_str("\nResponse Headers:\n");
        for (key, value) in &self.response_headers {
            output.push_str(&format!("  {}: {}\n", key, value));
        }

        output.push_str("\nTiming Breakdown:\n");
        for (key, duration) in &self.timing {
            if key != "total" {
                output.push_str(&format!(
                    "  {}: {:.1}ms\n",
                    key,
                    duration.as_secs_f64() * 1000.0
                ));
            }
        }
        if let Some(total) = self.timing.get("total") {
            output.push_str(&format!("  Total: {:.1}ms\n", total.as_secs_f64() * 1000.0));
        }

        output.push_str("\nNote: This is a simulated inspection.\n");
        output.push_str("For live inspection, use profiler middleware during server runtime.\n");

        output
    }
}

/// Inspects request.
///
/// ## Arguments
/// - `url`: URL to inspect
///
/// ## Returns
/// Request inspection with timing data
pub fn inspect_request(url: &str) -> RequestInspection {
    let mut inspection = RequestInspection::new(url);
    inspection.simulate_timing();
    inspection
}
