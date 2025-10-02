// Copyright 2025 Vivian Voss. Licensed under the Apache Licence, Version 2.0.
// SPDX-Licence-Identifier: Apache-2.0

//! ReedBase Set Service
//!
//! Provides set() function for updating key-value pairs with CSV persistence.

use crate::reedcms::backup::create_backup;
use crate::reedcms::csv::{write_csv, CsvRecord};
use crate::reedcms::reedstream::{current_timestamp, ReedRequest, ReedResponse, ReedResult};
use std::collections::HashMap;
use std::path::Path;

/// Updates a key-value pair in cache and persists to CSV.
///
/// ## Input
/// - `request`: ReedRequest with key, value, and optional description
/// - `cache`: Mutable reference to HashMap cache
/// - `csv_path`: Path to CSV file for persistence
///
/// ## Output
/// - `ReedResult<ReedResponse<String>>`: Confirmation with updated value
///
/// ## Performance
/// - O(1) cache update
/// - O(n) CSV write where n is total records
/// - < 10ms for < 1000 records
///
/// ## Behaviour
/// - Creates backup before modification
/// - Updates value in HashMap cache
/// - Writes entire cache to CSV file (atomic operation)
/// - Returns updated value in response
///
/// ## Error Conditions
/// - Returns `ReedError::ValidationError` if value not provided
/// - Returns `ReedError::IoError` if backup creation fails
/// - Returns `ReedError::IoError` if CSV write fails
///
/// ## Example Usage
/// ```
/// let request = ReedRequest {
///     key: "page.title@en".to_string(),
///     language: None,
///     environment: None,
///     context: Some("text".to_string()),
///     value: Some("New Title".to_string()),
///     description: Some("Homepage title".to_string()),
/// };
/// let mut cache = HashMap::new();
/// let response = set(request, &mut cache, ".reed/text.csv")?;
/// ```
pub fn set(
    request: ReedRequest,
    cache: &mut HashMap<String, String>,
    csv_path: &str,
) -> ReedResult<ReedResponse<String>> {
    // Validate value is provided
    let value = request.value.clone().ok_or_else(|| {
        crate::reedcms::reedstream::validation_error(
            "value",
            "none",
            "Value required for set operation",
        )
    })?;

    // Create backup before modification
    create_backup(Path::new(csv_path))?;

    // Update cache
    cache.insert(request.key.clone(), value.clone());

    // Convert cache to CsvRecord vector
    let mut records: Vec<CsvRecord> = cache
        .iter()
        .map(|(k, v)| CsvRecord {
            key: k.clone(),
            value: v.clone(),
            description: None, // Description not stored in cache
        })
        .collect();

    // Sort for deterministic output
    records.sort_by(|a, b| a.key.cmp(&b.key));

    // Write to CSV
    write_csv(Path::new(csv_path), &records)?;

    // Build response
    Ok(ReedResponse {
        data: value,
        source: csv_path.to_string(),
        cached: false, // Just wrote to disk
        timestamp: current_timestamp(),
        metrics: None,
    })
}
