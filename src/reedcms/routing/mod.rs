// Copyright 2025 Vivian Voss. Licensed under the Apache Licence, Version 2.0.
// SPDX-Licence-Identifier: Apache-2.0

//! URL routing services for ReedCMS.
//!
//! Provides URL → Layout + Language resolution via .reed/routes.csv.

pub mod language;
pub mod patterns;
pub mod resolver;

pub use resolver::{resolve_url, RouteInfo};
