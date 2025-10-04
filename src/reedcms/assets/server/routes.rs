// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Actix-Web route configuration for static asset serving.
//!
//! This module provides route handlers and configuration for serving
//! static assets with compression and caching.

use crate::reedcms::assets::server::static_server::serve_static_asset;
use actix_web::{web, HttpRequest, HttpResponse};

/// Handler for serving CSS bundles.
///
/// ## Input
/// - `req`: HTTP request
/// - `path`: Path parameter from route
///
/// ## Output
/// - `HttpResponse`: Asset response or error
///
/// ## Example Route
/// - `/static/css/{filename:.+}`
pub async fn serve_css(req: HttpRequest, path: web::Path<String>) -> HttpResponse {
    let filename = path.into_inner();
    match serve_static_asset(&req, &filename, "public/css").await {
        Ok(response) => response,
        Err(e) => HttpResponse::NotFound().body(format!("CSS not found: {}", e)),
    }
}

/// Handler for serving JS bundles.
///
/// ## Input
/// - `req`: HTTP request
/// - `path`: Path parameter from route
///
/// ## Output
/// - `HttpResponse`: Asset response or error
///
/// ## Example Route
/// - `/static/js/{filename:.+}`
pub async fn serve_js(req: HttpRequest, path: web::Path<String>) -> HttpResponse {
    let filename = path.into_inner();
    match serve_static_asset(&req, &filename, "public/js").await {
        Ok(response) => response,
        Err(e) => HttpResponse::NotFound().body(format!("JS not found: {}", e)),
    }
}

/// Handler for serving images.
///
/// ## Input
/// - `req`: HTTP request
/// - `path`: Path parameter from route
///
/// ## Output
/// - `HttpResponse`: Asset response or error
///
/// ## Example Route
/// - `/static/images/{filename:.+}`
pub async fn serve_image(req: HttpRequest, path: web::Path<String>) -> HttpResponse {
    let filename = path.into_inner();
    match serve_static_asset(&req, &filename, "public/images").await {
        Ok(response) => response,
        Err(e) => HttpResponse::NotFound().body(format!("Image not found: {}", e)),
    }
}

/// Handler for serving fonts.
///
/// ## Input
/// - `req`: HTTP request
/// - `path`: Path parameter from route
///
/// ## Output
/// - `HttpResponse`: Asset response or error
///
/// ## Example Route
/// - `/static/fonts/{filename:.+}`
pub async fn serve_font(req: HttpRequest, path: web::Path<String>) -> HttpResponse {
    let filename = path.into_inner();
    match serve_static_asset(&req, &filename, "public/fonts").await {
        Ok(response) => response,
        Err(e) => HttpResponse::NotFound().body(format!("Font not found: {}", e)),
    }
}

/// Handler for serving source maps.
///
/// ## Input
/// - `req`: HTTP request
/// - `path`: Path parameter from route
///
/// ## Output
/// - `HttpResponse`: Asset response or error
///
/// ## Example Route
/// - `/static/maps/{filename:.+}`
pub async fn serve_source_map(req: HttpRequest, path: web::Path<String>) -> HttpResponse {
    let filename = path.into_inner();
    match serve_static_asset(&req, &filename, "public/maps").await {
        Ok(response) => response,
        Err(e) => HttpResponse::NotFound().body(format!("Source map not found: {}", e)),
    }
}

/// Configures all static asset routes for Actix-Web application.
///
/// ## Input
/// - `cfg`: Actix-Web service configuration
///
/// ## Example Usage
/// ```rust
/// HttpServer::new(|| {
///     App::new()
///         .configure(configure_static_routes)
/// })
/// ```
pub fn configure_static_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/static")
            .route("/css/{filename:.+}", web::get().to(serve_css))
            .route("/js/{filename:.+}", web::get().to(serve_js))
            .route("/images/{filename:.+}", web::get().to(serve_image))
            .route("/fonts/{filename:.+}", web::get().to(serve_font))
            .route("/maps/{filename:.+}", web::get().to(serve_source_map)),
    );
}
