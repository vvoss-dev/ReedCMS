// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Exec command implementation.

use anyhow::{Context, Result};
use reedbase::Database;
use std::path::Path;

pub fn execute(sql: &str, path: &Path, user: &str, quiet: bool) -> Result<()> {
    // Open database
    let db = Database::open(path)
        .with_context(|| format!("Failed to open database at {}", path.display()))?;

    // Execute command
    let result = db
        .execute(sql, user)
        .with_context(|| format!("Command failed: {}", sql))?;

    // Print result
    if !quiet {
        println!(
            "{} row{} affected ({:.2}ms)",
            result.rows_affected,
            if result.rows_affected == 1 { "" } else { "s" },
            result.execution_time_us as f64 / 1000.0
        );
    }

    Ok(())
}
