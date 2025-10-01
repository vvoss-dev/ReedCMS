// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! ReedBase Initialisation Service
//!
//! Provides init() function for loading CSV files into memory cache.

use crate::reedcms::csv::read_csv;
use crate::reedcms::reedstream::{current_timestamp, ReedRequest, ReedResponse, ReedResult};
use std::collections::HashMap;
use std::path::Path;

/// Initialises in-memory cache from a CSV file.
///
/// ## Input
/// - `request`: ReedRequest with context.value = path to CSV file
///
/// ## Output
/// - `ReedResult<ReedResponse<HashMap<String, String>>>`: HashMap with key-value pairs
///
/// ## Performance
/// - O(n) where n is number of CSV rows
/// - < 10ms for < 1000 rows
/// - Returns HashMap for O(1) lookups
///
/// ## Behaviour
/// - Reads CSV file using csv::read_csv()
/// - Builds HashMap with key → value mapping
/// - Skips records without description (optional field)
/// - Returns source path in response
///
/// ## Error Conditions
/// - Returns `ReedError::IoError` if file cannot be read
/// - Returns `ReedError::ParseError` if CSV format is invalid
/// - Returns `ReedError::ValidationError` if path is not provided
///
/// ## Example Usage
/// ```
/// let request = ReedRequest {
///     key: "".to_string(),
///     language: None,
///     environment: None,
///     context: Some("text".to_string()),
///     value: Some(".reed/text.csv".to_string()),
///     description: None,
/// };
/// let response = init(request)?;
/// let cache = response.data;
/// ```
pub fn init(request: ReedRequest) -> ReedResult<ReedResponse<HashMap<String, String>>> {
    // Validate path is provided
    let path = request.value.ok_or_else(|| {
        crate::reedcms::reedstream::validation_error(
            "value",
            "none",
            "CSV file path required in request.value",
        )
    })?;

    // Read CSV file
    let records = read_csv(Path::new(&path))?;

    // Build HashMap
    let mut cache = HashMap::new();
    for record in records {
        cache.insert(record.key, record.value);
    }

    // Build response
    Ok(ReedResponse {
        data: cache,
        source: path,
        cached: false, // Initial load, not from cache
        timestamp: current_timestamp(),
        metrics: None,
    })
}
