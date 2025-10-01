// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! CSV Handler System for ReedCMS
//!
//! Provides universal CSV operations with pipe delimiter (|) format:
//! - Type-safe parsing via CsvRecord
//! - Atomic writes using temp file + rename pattern
//! - Performance: < 1ms for < 1000 rows
//! - Thread-safe operations

mod read;
mod record;
mod write;

pub use read::read_csv;
pub use record::{create_row, parse_row, CsvRecord};
pub use write::write_csv;

#[cfg(test)]
mod read_test;
#[cfg(test)]
mod record_test;
#[cfg(test)]
mod write_test;
