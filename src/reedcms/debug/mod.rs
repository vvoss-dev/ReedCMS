// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Debug tools and development utilities.
//!
//! ## Modules
//! - `request_inspector`: Request debugging
//! - `cache_viewer`: ReedBase cache inspection
//! - `route_tester`: Route testing
//! - `config_inspector`: Configuration debugging

pub mod cache_viewer;
pub mod config_inspector;
pub mod request_inspector;
pub mod route_tester;

// Re-exports for convenience
pub use cache_viewer::{view_cache, CacheView};
pub use config_inspector::{inspect_config, ConfigInspection};
pub use request_inspector::{inspect_request, RequestInspection};
pub use route_tester::{test_route, RouteTest};
