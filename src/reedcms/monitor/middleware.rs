// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Monitoring middleware for Actix-Web integration.
//!
//! ## Features
//! - Automatic request timing
//! - Status code tracking
//! - Path-based metrics
//! - Low overhead (< 10μs)

use super::core::global_monitor;
use actix_web::dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::Error;
use futures_util::future::LocalBoxFuture;
use std::future::{ready, Ready};
use std::time::Instant;

/// Monitoring middleware for Actix-Web.
///
/// ## Usage
/// ```rust
/// use actix_web::App;
/// use reedcms::monitor::middleware::MonitorMiddleware;
///
/// let app = App::new()
///     .wrap(MonitorMiddleware);
/// ```
///
/// ## Performance Overhead
/// - < 10μs per request
/// - Negligible impact on throughput
pub struct MonitorMiddleware;

impl<S, B> Transform<S, ServiceRequest> for MonitorMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = MonitorMiddlewareService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(MonitorMiddlewareService { service }))
    }
}

pub struct MonitorMiddlewareService<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for MonitorMiddlewareService<S>
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
        let start_time = Instant::now();
        let method = req.method().to_string();
        let path = req.path().to_string();

        let fut = self.service.call(req);

        Box::pin(async move {
            let res = fut.await?;
            let duration = start_time.elapsed();
            let status = res.status().as_u16();

            // Record metrics
            global_monitor().record_request(&method, &path, status, duration);

            Ok(res)
        })
    }
}
