// Copyright 2025 Vivian Voss. Licensed under the Apache Licence, Version 2.0.
// SPDX-Licence-Identifier: Apache-2.0

//! Reed.toml configuration system.
//!
//! Provides TOML-based configuration management with automatic sync to CSV files.

pub mod toml_parser;
pub mod toml_to_csv;

pub use toml_parser::{parse_reed_toml, validate_config, ReedConfig};
pub use toml_to_csv::sync_toml_to_csv;
