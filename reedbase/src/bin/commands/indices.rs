// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Indices command implementation.

use anyhow::{Context, Result};
use reedbase::Database;
use std::path::Path;

pub fn execute(
    path: &Path,
    create: Option<&str>,
    drop: Option<&str>,
    rebuild: Option<&str>,
    verbose: bool,
) -> Result<()> {
    let db = Database::open(path)
        .with_context(|| format!("Failed to open database at {}", path.display()))?;

    // Create index
    if let Some(table_column) = create {
        let parts: Vec<&str> = table_column.split('.').collect();
        if parts.len() != 2 {
            anyhow::bail!("Invalid format. Use: table.column");
        }

        db.create_index(parts[0], parts[1])
            .with_context(|| format!("Failed to create index on {}", table_column))?;
        println!("Created index: {}", table_column);
        return Ok(());
    }

    // Drop index
    if let Some(table_column) = drop {
        // TODO: Implement index dropping
        eprintln!("Index dropping not yet implemented for: {}", table_column);
        return Ok(());
    }

    // Rebuild index
    if let Some(table_column) = rebuild {
        // TODO: Implement index rebuilding
        eprintln!("Index rebuilding not yet implemented for: {}", table_column);
        return Ok(());
    }

    // List indices
    let indices = db.list_indices();

    if indices.is_empty() {
        println!("No indices found");
        return Ok(());
    }

    println!("Indices:");
    for index in indices {
        if verbose {
            println!(
                "  - {}.{} ({}, {} entries, {} bytes)",
                index.table,
                index.column,
                index.index_type,
                index.entry_count,
                index.total_bytes()
            );
        } else {
            println!(
                "  - {}.{} ({})",
                index.table, index.column, index.index_type
            );
        }
    }

    Ok(())
}
