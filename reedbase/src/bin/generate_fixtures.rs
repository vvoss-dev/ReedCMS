// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Test fixture generator for ReedBase.
//!
//! Creates standardised test databases with varying sizes and characteristics.
//!
//! ## Fixtures Generated
//!
//! - `test_data/small/.reed` - 100 rows
//! - `test_data/medium/.reed` - 10,000 rows
//! - `test_data/large/.reed` - 100,000 rows
//! - `test_data/versioned/.reed` - 50 versions
//!
//! ## Usage
//!
//! ```bash
//! cargo run --bin generate_fixtures
//! ```

use reedbase::Database;
use std::fs;
use std::path::{Path, PathBuf};

fn main() -> anyhow::Result<()> {
    println!("ReedBase Test Fixture Generator");
    println!("================================\n");

    // Parse command line arguments
    let args: Vec<String> = std::env::args().collect();
    let fixtures_to_generate = if args.len() > 1 {
        args[1..].to_vec()
    } else {
        vec!["small".to_string(), "versioned".to_string()]
    };

    // Create test_data directory
    let base_path = PathBuf::from("test_data");
    fs::create_dir_all(&base_path)?;

    // Generate requested fixtures
    for fixture in &fixtures_to_generate {
        match fixture.as_str() {
            "small" => generate_small_fixture(&base_path)?,
            "medium" => generate_medium_fixture(&base_path)?,
            "large" => generate_large_fixture(&base_path)?,
            "versioned" => generate_versioned_fixture(&base_path)?,
            "all" => {
                generate_small_fixture(&base_path)?;
                generate_medium_fixture(&base_path)?;
                generate_large_fixture(&base_path)?;
                generate_versioned_fixture(&base_path)?;
            }
            _ => {
                eprintln!("Unknown fixture: {}", fixture);
                eprintln!("Available: small, medium, large, versioned, all");
                std::process::exit(1);
            }
        }
    }

    println!("\n✓ Fixture(s) generated successfully!");
    println!("Location: {}", base_path.display());

    Ok(())
}

/// Generates small fixture (100 rows).
fn generate_small_fixture(base_path: &Path) -> anyhow::Result<()> {
    println!("Generating small fixture (100 rows)...");

    let fixture_path = base_path.join("small");

    // Remove existing fixture if present
    if fixture_path.exists() {
        fs::remove_dir_all(&fixture_path)?;
    }

    fs::create_dir_all(&fixture_path)?;

    let db_path = fixture_path.join(".reed");
    create_database_with_rows(&db_path, 100)?;

    println!("  ✓ Small fixture created: {}", fixture_path.display());
    Ok(())
}

/// Generates medium fixture (10,000 rows).
fn generate_medium_fixture(base_path: &Path) -> anyhow::Result<()> {
    println!("Generating medium fixture (10,000 rows)...");

    let fixture_path = base_path.join("medium");

    // Remove existing fixture if present
    if fixture_path.exists() {
        fs::remove_dir_all(&fixture_path)?;
    }

    fs::create_dir_all(&fixture_path)?;

    let db_path = fixture_path.join(".reed");
    create_database_with_rows(&db_path, 10_000)?;

    println!("  ✓ Medium fixture created: {}", fixture_path.display());
    Ok(())
}

/// Generates large fixture (50,000 rows).
///
/// Note: Reduced from 100k to 50k for faster generation and smaller repository size.
fn generate_large_fixture(base_path: &Path) -> anyhow::Result<()> {
    println!("Generating large fixture (50,000 rows)...");
    println!("  (This may take a while...)");

    let fixture_path = base_path.join("large");

    // Remove existing fixture if present
    if fixture_path.exists() {
        fs::remove_dir_all(&fixture_path)?;
    }

    fs::create_dir_all(&fixture_path)?;

    let db_path = fixture_path.join(".reed");
    create_database_with_rows(&db_path, 50_000)?;

    println!("  ✓ Large fixture created: {}", fixture_path.display());
    Ok(())
}

/// Generates versioned fixture (50 versions).
fn generate_versioned_fixture(base_path: &Path) -> anyhow::Result<()> {
    println!("Generating versioned fixture (50 versions)...");

    let fixture_path = base_path.join("versioned");

    // Remove existing fixture if present
    if fixture_path.exists() {
        fs::remove_dir_all(&fixture_path)?;
    }

    fs::create_dir_all(&fixture_path)?;

    let db_path = fixture_path.join(".reed");

    // Initialize registry
    reedbase::registry::init_registry(&db_path)?;
    reedbase::registry::set_base_path(db_path.clone());

    // Create database
    let db = Database::open(&db_path)?;

    // Create text table
    db.create_table("text", None)?;

    // Insert initial data (10 rows)
    for i in 0..10 {
        let sql = format!(
            "INSERT INTO text (key, value) VALUES ('test.key.{:06}', 'initial value {}')",
            i, i
        );
        db.execute(&sql, "system")?;
    }

    // Create 50 versions by updating rows
    for version in 0..50 {
        let row_idx = version % 10; // Cycle through 10 rows
        let sql = format!(
            "UPDATE text SET value = 'version {} value' WHERE key = 'test.key.{:06}'",
            version, row_idx
        );
        db.execute(&sql, &format!("user{}", version % 5))?;

        if (version + 1) % 10 == 0 {
            print!("  {} versions created...\r", version + 1);
            use std::io::Write;
            std::io::stdout().flush()?;
        }
    }

    println!("  ✓ Versioned fixture created: {} versions", 50);
    Ok(())
}

/// Creates a database with specified number of rows.
fn create_database_with_rows(db_path: &Path, rows: usize) -> anyhow::Result<()> {
    // Initialize registry
    reedbase::registry::init_registry(db_path)?;
    reedbase::registry::set_base_path(db_path.to_path_buf());

    // Create database
    let db = Database::open(db_path)?;

    // Create text table
    db.create_table("text", None)?;

    // Insert rows in batches
    let batch_size = 1000;
    for batch_start in (0..rows).step_by(batch_size) {
        let batch_end = (batch_start + batch_size).min(rows);

        for i in batch_start..batch_end {
            let sql = format!(
                "INSERT INTO text (key, value) VALUES ('test.key.{:06}', 'test value {}')",
                i, i
            );
            db.execute(&sql, "system")?;
        }

        if batch_end % 10_000 == 0 || batch_end == rows {
            print!("  {} rows inserted...\r", batch_end);
            use std::io::Write;
            std::io::stdout().flush()?;
        }
    }

    println!("  {} rows inserted    ", rows);
    Ok(())
}
