// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Query command implementation.

use anyhow::{Context, Result};
use reedbase::Database;
use std::path::Path;

use crate::formatters;

pub fn execute(
    sql: &str,
    path: &Path,
    format: &str,
    output: Option<&Path>,
    no_header: bool,
) -> Result<()> {
    // Open database
    let db = Database::open(path)
        .with_context(|| format!("Failed to open database at {}", path.display()))?;

    // Execute query
    let result = db
        .query(sql)
        .with_context(|| format!("Query failed: {}", sql))?;

    // Format output
    let output_str = match format {
        "json" => formatters::format_json(&result),
        "csv" => formatters::format_csv(&result, !no_header),
        _ => formatters::format_table(&result),
    };

    // Write to file or stdout
    if let Some(output_path) = output {
        std::fs::write(output_path, &output_str)
            .with_context(|| format!("Failed to write to {}", output_path.display()))?;
        println!("Output written to {}", output_path.display());
    } else {
        print!("{}", output_str);
    }

    Ok(())
}
