// Copyright 2025 Vivian Voss. Licensed under the Apache Licence, Version 2.0.
// SPDX-Licence-Identifier: Apache-2.0

//! Matrix CSV Handler System
//!
//! Provides support for complex CSV structures with 4 value types:
//! - Type 1: Single values (simple text)
//! - Type 2: Lists (comma-separated)
//! - Type 3: Values with modifiers (value[modifier])
//! - Type 4: Lists with modifiers (item1[mod1],item2[mod2])

mod parse;
mod read;
mod record;
mod write;

pub use parse::{parse_matrix_value, parse_modifiers};
pub use read::read_matrix_csv;
pub use record::{MatrixRecord, MatrixValue};
pub use write::write_matrix_csv;

#[cfg(test)]
mod parse_test;
#[cfg(test)]
mod read_test;
#[cfg(test)]
mod record_test;
#[cfg(test)]
mod write_test;
