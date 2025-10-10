// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Health check and metrics endpoints.
//!
//! ## Endpoints
//! - GET /health - Health status
//! - GET /metrics - Detailed metrics
//!
//! ## Usage
//! ```rust
//! use actix_web::web;
//! use reedcms::monitor::health;
//!
//! web::resource("/health").route(web::get().to(health::health_check))
//! ```

use super::core::global_monitor;
use actix_web::{web, HttpResponse, Responder};

/// Health check endpoint handler.
///
/// ## Endpoint
/// GET /health
///
/// ## Response
/// ```json
/// {
///   "status": "healthy",
///   "uptime": "3h 24m 15s",
///   "total_requests": 15234,
///   "error_rate": 0.023,
///   "avg_response_time": "45ms"
/// }
/// ```
///
/// ## Status Codes
/// - 200: Healthy or Degraded
/// - 503: Unhealthy
pub async fn health_check() -> impl Responder {
    let health = global_monitor().get_health();

    let status_code = match health.status {
        super::metrics::Health::Healthy | super::metrics::Health::Degraded => 200,
        super::metrics::Health::Unhealthy => 503,
    };

    HttpResponse::build(actix_web::http::StatusCode::from_u16(status_code).unwrap()).json(
        serde_json::json!({
            "status": format!("{:?}", health.status).to_lowercase(),
            "uptime": format_duration(health.uptime),
            "total_requests": health.total_requests,
            "error_rate": format!("{:.3}", health.error_rate),
            "avg_response_time": format!("{}ms", health.avg_response_time.as_millis())
        }),
    )
}

/// Metrics endpoint handler.
///
/// ## Endpoint
/// GET /metrics
///
/// ## Response
/// Detailed metrics snapshot in JSON format
pub async fn metrics_endpoint() -> impl Responder {
    let snapshot = global_monitor().get_snapshot();

    web::Json(serde_json::json!({
        "uptime": format_duration(snapshot.uptime),
        "total_requests": snapshot.total_requests,
        "avg_response_time_ms": snapshot.avg_response_time.as_millis(),
        "error_rate": snapshot.error_rate,
        "requests_by_path": snapshot.requests_by_path,
        "status_codes": snapshot.status_codes,
        "reedbase": {
            "hit_rate": snapshot.reedbase_hit_rate,
            "avg_time_us": snapshot.reedbase_avg_time.as_micros()
        },
        "templates": {
            "avg_time_ms": snapshot.template_avg_time.as_millis()
        },
        "system": {
            "memory_usage_mb": snapshot.memory_usage as f64 / 1024.0 / 1024.0
        }
    }))
}

/// Formats duration as human-readable string.
///
/// ## Examples
/// - "3h 24m 15s"
/// - "45m 30s"
/// - "12s"
fn format_duration(duration: std::time::Duration) -> String {
    let secs = duration.as_secs();
    let hours = secs / 3600;
    let minutes = (secs % 3600) / 60;
    let seconds = secs % 60;

    if hours > 0 {
        format!("{}h {}m {}s", hours, minutes, seconds)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, seconds)
    } else {
        format!("{}s", seconds)
    }
}
