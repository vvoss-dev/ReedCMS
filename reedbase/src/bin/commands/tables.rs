// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Tables command implementation.

use anyhow::{Context, Result};
use reedbase::Database;
use std::path::Path;

pub fn execute(
    path: &Path,
    create: Option<&str>,
    drop: Option<&str>,
    confirm: bool,
    verbose: bool,
) -> Result<()> {
    let db = Database::open(path)
        .with_context(|| format!("Failed to open database at {}", path.display()))?;

    // Create table
    if let Some(table_name) = create {
        db.create_table(table_name, None)
            .with_context(|| format!("Failed to create table '{}'", table_name))?;
        println!("Created table: {}", table_name);
        return Ok(());
    }

    // Drop table
    if let Some(table_name) = drop {
        if !confirm {
            eprintln!("Error: Dropping tables requires --confirm flag");
            eprintln!("This is a destructive operation!");
            return Ok(());
        }

        // TODO: Implement table deletion
        eprintln!("Table deletion not yet implemented");
        return Ok(());
    }

    // List tables
    let tables = db.list_tables()?;

    if tables.is_empty() {
        println!("No tables found");
        return Ok(());
    }

    println!("Tables:");
    for table in tables {
        if verbose {
            // TODO: Show table statistics
            println!("  - {} (stats not yet implemented)", table);
        } else {
            println!("  - {}", table);
        }
    }

    Ok(())
}
