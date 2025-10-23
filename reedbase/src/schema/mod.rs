// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Schema validation module for ReedBase.
//!
//! Provides comprehensive schema validation for ReedBase:
//! - **RBKS v2 Key Validation** - Structured key format enforcement
//! - **Column Schema Validation** - Type and constraint enforcement
//!
//! ## Key Validation (RBKS v2)
//!
//! Enforces structured key format: `<namespace>.<hierarchy>[<modifier,modifier>]`
//!
//! ### Key Structure Rules
//!
//! - Lowercase only
//! - Dots for hierarchy
//! - Angle brackets for modifiers
//! - Comma-separated modifiers
//! - Order-independent modifiers
//! - Depth 2-8 levels
//!
//! ### Modifier Categories
//!
//! - **Language**: ISO 639-1 codes (de, en, fr, etc.) - max 1
//! - **Environment**: dev/prod/staging/test - max 1
//! - **Season**: christmas/easter/summer/winter - max 1
//! - **Variant**: mobile/desktop/tablet - max 1
//! - **Custom**: Any other identifier - multiple allowed
//!
//! ## Column Schema Validation
//!
//! TOML-based schema files with type and constraint validation:
//!
//! ### Column Types
//!
//! - **string**: Text data with length and pattern constraints
//! - **integer**: Whole numbers with min/max range
//! - **float**: Decimal numbers
//! - **boolean**: True/false values
//! - **timestamp**: Unix timestamps
//!
//! ### Constraints
//!
//! - **required**: Cannot be empty
//! - **unique**: No duplicate values
//! - **primary_key**: Required + unique
//! - **min/max**: Range constraints for integer/float
//! - **min_length/max_length**: Length constraints for string
//! - **pattern**: Regex validation for string
//!
//! ## Example Usage
//!
//! ```rust
//! use reedbase::schema::{
//!     validate_key, parse_key, normalize_key,
//!     Schema, ColumnDef, validate_row, CsvRow,
//! };
//!
//! // Key validation
//! validate_key("page.header.title<de,prod>")?;
//!
//! // Column validation
//! let schema = Schema::new("2.0".to_string(), true, vec![
//!     ColumnDef::primary_key("id".to_string(), "integer".to_string()),
//!     ColumnDef::new("name".to_string(), "string".to_string()).required(),
//! ]);
//!
//! let row = CsvRow::new("1".to_string(), vec!["1".to_string(), "Alice".to_string()]);
//! validate_row(&row, &schema)?;
//! ```
//!
//! ## Performance
//!
//! - Key validation: < 20μs
//! - Row validation: < 1ms
//! - Schema load: < 5ms
//! - Total overhead: < 30μs per write
//!
//! ## Benefits
//!
//! - **Type-safe data** with column validation
//! - **Self-documenting schemas** in TOML
//! - **Catch errors early** at write time
//! - **Enables O(1) queries** via Smart Indices

pub mod loader;
pub mod rbks;
pub mod types;
pub mod validation;

#[cfg(test)]
mod loader_test;
#[cfg(test)]
mod rbks_test;
#[cfg(test)]
mod validation_test;

// Re-export commonly used types

// RBKS v2 (key validation)
pub use rbks::{
    normalize_key, parse_key, validate_key, Modifiers, ParsedKey, KNOWN_ENVIRONMENTS,
    KNOWN_LANGUAGES, KNOWN_SEASONS, KNOWN_VARIANTS, RBKS_V2_PATTERN,
};

// Column schema validation
pub use loader::{create_default_schema, delete_schema, load_schema, save_schema, schema_exists};
pub use types::{ColumnDef, Schema};
pub use validation::{validate_row, validate_rows, validate_uniqueness, CsvRow};
