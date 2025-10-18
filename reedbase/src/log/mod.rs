// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Encoded log system for version history.
//!
//! Provides efficient log encoding/decoding using integer codes and CRC32 validation.

pub mod decoder;
pub mod encoder;
pub mod types;
pub mod validator;

#[cfg(test)]
mod decoder_test;
#[cfg(test)]
mod encoder_test;
#[cfg(test)]
mod validator_test;

// Re-export public API
pub use decoder::{
    decode_log_entries, decode_log_entry, filter_by_action, filter_by_time_range, filter_by_user,
};
pub use encoder::{calculate_size_savings, encode_log_entries, encode_log_entry};
pub use types::{LogEntry, ValidationReport};
pub use validator::{validate_and_truncate_log, validate_log};
