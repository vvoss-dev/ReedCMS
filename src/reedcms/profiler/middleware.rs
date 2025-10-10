// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Profiling middleware for Actix-Web.
//!
//! ## Features
//! - Automatic request profiling
//! - Environment-controlled (REED_PROFILE)
//! - Bottleneck detection
//! - Profile report logging

use super::core::Profiler;
use actix_web::dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::Error;
use futures_util::future::LocalBoxFuture;
use std::future::{ready, Ready};

/// Profiling middleware for Actix-Web.
///
/// ## Usage
/// Enable via environment variable:
/// ```bash
/// REED_PROFILE=true reed server:start
/// ```
///
/// ## Output
/// Profile data logged to stdout with bottleneck detection.
pub struct ProfilerMiddleware {
    enabled: bool,
}

impl ProfilerMiddleware {
    /// Creates new profiler middleware.
    ///
    /// Reads REED_PROFILE environment variable (true/1 to enable).
    pub fn new() -> Self {
        let enabled = std::env::var("REED_PROFILE")
            .map(|v| v == "true" || v == "1")
            .unwrap_or(false);

        Self { enabled }
    }
}

impl Default for ProfilerMiddleware {
    fn default() -> Self {
        Self::new()
    }
}

impl<S, B> Transform<S, ServiceRequest> for ProfilerMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = ProfilerMiddlewareService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(ProfilerMiddlewareService {
            service,
            enabled: self.enabled,
        }))
    }
}

pub struct ProfilerMiddlewareService<S> {
    service: S,
    enabled: bool,
}

impl<S, B> Service<ServiceRequest> for ProfilerMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        if !self.enabled {
            return Box::pin(self.service.call(req));
        }

        let path = req.path().to_string();
        let profiler = Profiler::start(&format!("request_{}", path));

        let fut = self.service.call(req);

        Box::pin(async move {
            let res = fut.await?;

            // Generate and log profile report
            let report = profiler.finish();
            println!("{}", report.format());

            // Check for bottlenecks
            let bottlenecks = report.bottlenecks();
            if !bottlenecks.is_empty() {
                eprintln!("⚠ Performance bottlenecks detected:");
                for span in bottlenecks {
                    eprintln!(
                        "  - {}: {:.1}ms",
                        span.name,
                        span.duration.as_secs_f64() * 1000.0
                    );
                }
            }

            Ok(res)
        })
    }
}
