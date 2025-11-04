// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Stats command implementation.

use anyhow::{Context, Result};
use reedbase::Database;
use std::path::Path;

pub fn execute(path: &Path, format: &str) -> Result<()> {
    let db = Database::open(path)
        .with_context(|| format!("Failed to open database at {}", path.display()))?;

    let stats = db.stats();

    match format {
        "json" => {
            // JSON output
            println!("{{");
            println!("  \"tables\": {},", stats.table_count);
            println!("  \"total_rows\": {},", stats.total_rows);
            println!("  \"indices\": {},", stats.index_count);
            println!("  \"query_count\": {},", stats.query_count);
            println!("  \"insert_count\": {},", stats.insert_count);
            println!("  \"update_count\": {},", stats.update_count);
            println!("  \"delete_count\": {},", stats.delete_count);
            println!("  \"avg_query_time_us\": {}", stats.avg_query_time_us);
            println!("}}");
        }
        _ => {
            // Table output
            println!("Database Statistics:");
            println!("  Tables:           {}", stats.table_count);
            println!("  Total Rows:       {}", stats.total_rows);
            println!("  Indices:          {}", stats.index_count);
            println!("  Total Queries:    {}", stats.query_count);
            println!("  Total Inserts:    {}", stats.insert_count);
            println!("  Total Updates:    {}", stats.update_count);
            println!("  Total Deletes:    {}", stats.delete_count);
            println!(
                "  Avg Query Time:   {:.2}ms",
                stats.avg_query_time_us as f64 / 1000.0
            );
        }
    }

    Ok(())
}
