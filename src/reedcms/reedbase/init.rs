// Copyright 2025 Vivian Voss. Licensed under the Apache Licence, Version 2.0.
// SPDX-Licence-Identifier: Apache-2.0

//! ReedBase Initialisation Service
//!
//! Provides init() function for loading CSV files into memory cache and
//! aggregate_text_csv() for collecting component-local .text.csv files.

use crate::reedcms::csv::{read_csv, write_csv};
use crate::reedcms::reedstream::{
    current_timestamp, ReedError, ReedRequest, ReedResponse, ReedResult,
};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

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
#[allow(dead_code)]
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

/// Aggregates all component-local .text.csv files into .reed/text.csv
///
/// ## Process
/// 1. Scans templates/ directory recursively
/// 2. Finds all *.text.csv files
/// 3. Reads and collects all records
/// 4. Writes aggregated file to .reed/text.csv
///
/// ## Performance
/// - O(n) where n is total CSV records
/// - < 50ms for typical component structure
///
/// ## Error Conditions
/// - Returns `ReedError::IoError` if templates/ directory not found
/// - Returns `ReedError::IoError` if .reed/text.csv cannot be written
///
/// ## Example Usage
/// ```
/// aggregate_text_csv()?;
/// ```
pub fn aggregate_text_csv() -> ReedResult<ReedResponse<String>> {
    let templates_dir = Path::new("templates");

    if !templates_dir.exists() {
        return Err(ReedError::IoError {
            operation: "aggregate_text_csv".to_string(),
            path: "templates".to_string(),
            reason: "templates/ directory not found".to_string(),
        });
    }

    // Collect all .text.csv files
    let text_csv_files = discover_text_csv_files(templates_dir)?;
    let file_count = text_csv_files.len();

    // Aggregate all records
    let mut all_records = Vec::new();

    for csv_file in text_csv_files {
        match read_csv(&csv_file) {
            Ok(records) => {
                all_records.extend(records);
            }
            Err(e) => {
                eprintln!("Warning: Failed to read {}: {}", csv_file.display(), e);
            }
        }
    }

    // Write aggregated file
    let output_path = Path::new(".reed/text.csv");
    write_csv(output_path, &all_records)?;

    Ok(ReedResponse {
        data: format!(
            "Aggregated {} entries from {} component files to .reed/text.csv",
            all_records.len(),
            file_count
        ),
        source: "reedbase::init::aggregate_text_csv".to_string(),
        cached: false,
        timestamp: current_timestamp(),
        metrics: None,
    })
}

/// Recursively discovers all .text.csv files in a directory
fn discover_text_csv_files(dir: &Path) -> ReedResult<Vec<PathBuf>> {
    let mut csv_files = Vec::new();

    let entries = fs::read_dir(dir).map_err(|e| ReedError::IoError {
        operation: "read_dir".to_string(),
        path: dir.display().to_string(),
        reason: e.to_string(),
    })?;

    for entry in entries {
        let entry = entry.map_err(|e| ReedError::IoError {
            operation: "read_dir_entry".to_string(),
            path: dir.display().to_string(),
            reason: e.to_string(),
        })?;

        let path = entry.path();

        if path.is_dir() {
            // Recurse into subdirectories
            let sub_files = discover_text_csv_files(&path)?;
            csv_files.extend(sub_files);
        } else if path.is_file() {
            // Check if file ends with .text.csv
            if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                if file_name.ends_with(".text.csv") {
                    csv_files.push(path);
                }
            }
        }
    }

    Ok(csv_files)
}
