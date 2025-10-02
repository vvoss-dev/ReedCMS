// Copyright 2025 Vivian Voss. Licensed under the Apache Licence, Version 2.0.
// SPDX-Licence-Identifier: Apache-2.0

//! Response Module
//!
//! HTTP response building with template rendering, variant detection, and caching.

pub mod builder;
pub mod cache;
pub mod content_type;
pub mod errors;

pub use builder::build_response;
pub use cache::cache_control_header;
pub use content_type::{negotiate_content_type, ContentType};
pub use errors::{build_404_response, build_500_response};
