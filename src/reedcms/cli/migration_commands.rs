// Copyright 2025 Vivian Voss. Licensed under the Apache Licence, Version 2.0.
// SPDX-Licence-Identifier: Apache-2.0

//! CLI migration commands.
//!
//! Implements bulk data migration commands for importing text and routes from CSV files.
//! Commands follow the pattern: reed migrate:action [args] [--flags]

use crate::reedcms::backup::create_backup;
use crate::reedcms::csv::{read_csv, write_csv};
use crate::reedcms::reedstream::{ReedError, ReedResponse, ReedResult};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

/// Migrates text content from component/layout CSV files to central .reed/text.csv.
///
/// ## Usage
/// reed migrate:text <path> [--recursive] [--dry-run] [--no-backup]
///
/// ## Arguments
/// - path: Path to .text.csv file or directory containing .text.csv files
///
/// ## Flags
/// - --recursive: Process directories recursively
/// - --dry-run: Preview changes without applying
/// - --no-backup: Skip backup creation (default: create backup)
///
/// ## CSV Format Expected
/// ```csv
/// key|value|comment
/// page-header.logo.title@de|Effektive Software-Architektur|Logo subtitle text
/// ```
///
/// ## Important
/// Keys MUST include full namespace and @language suffix.
/// No auto-prefixing is performed.
///
/// ## Example
/// ```bash
/// reed migrate:text templates/components/organisms/page-header/
/// reed migrate:text templates/components/ --recursive
/// reed migrate:text templates/layouts/knowledge/ --dry-run
/// ```
pub fn migrate_text(
    args: &[String],
    flags: &HashMap<String, String>,
) -> ReedResult<ReedResponse<String>> {
    let start = Instant::now();

    if args.is_empty() {
        return Err(ReedError::ValidationError {
            field: "path".to_string(),
            value: String::new(),
            constraint: "path to .text.csv file or directory required".to_string(),
        });
    }

    let path = &args[0];
    let recursive = flags.contains_key("recursive");
    let dry_run = flags.contains_key("dry-run");
    let create_backup_flag = !flags.contains_key("no-backup");

    let path_buf = PathBuf::from(path);
    if !path_buf.exists() {
        return Err(ReedError::NotFound {
            resource: format!("path: {}", path),
            context: Some("File or directory does not exist".to_string()),
        });
    }

    // Discover .text.csv files
    let csv_files = discover_text_csv_files(&path_buf, recursive)?;

    if csv_files.is_empty() {
        return Ok(ReedResponse {
            data: format!("No .text.csv files found in {}", path),
            source: "migration".to_string(),
            cached: false,
            timestamp: crate::reedcms::reedstream::current_timestamp(),
            metrics: None,
        });
    }

    let mut output = format!("📦 Migrating text from: {}\n", path);
    output.push_str(&format!(
        "✓ Found {} .text.csv file(s)\n\n",
        csv_files.len()
    ));

    // Load existing entries from .reed/text.csv
    let target_path = PathBuf::from(".reed/text.csv");
    let existing_entries = if target_path.exists() {
        read_csv(&target_path)?
    } else {
        Vec::new()
    };

    let existing_keys: HashSet<String> = existing_entries.iter().map(|e| e.key.clone()).collect();

    // Process each file
    let mut total_entries = 0;
    let mut imported_count = 0;
    let mut skipped_count = 0;
    let mut error_count = 0;
    let mut new_entries = Vec::new();

    for csv_file in &csv_files {
        output.push_str(&format!("Processing: {}\n", csv_file.display()));

        let entries = match read_csv(csv_file) {
            Ok(e) => e,
            Err(err) => {
                error_count += 1;
                output.push_str(&format!("  ⚠ Error reading file: {}\n", err));
                continue;
            }
        };

        total_entries += entries.len();

        // Validate entries
        for entry in entries {
            // Check if key has @lang suffix
            if !entry.key.contains('@') {
                error_count += 1;
                output.push_str(&format!("  ⚠ Key missing @lang suffix: {}\n", entry.key));
                continue;
            }

            // Check for duplicates
            if existing_keys.contains(&entry.key) {
                skipped_count += 1;
                output.push_str(&format!("  ⊘ Skipped duplicate: {}\n", entry.key));
                continue;
            }

            imported_count += 1;
            new_entries.push(entry);
        }
    }

    output.push_str("\nValidation:\n");
    output.push_str(&format!("✓ Total entries: {}\n", total_entries));
    output.push_str(&format!("✓ To import: {}\n", imported_count));
    output.push_str(&format!("⊘ Skipped (duplicates): {}\n", skipped_count));
    output.push_str(&format!("⚠ Errors: {}\n\n", error_count));

    if dry_run {
        output.push_str("🔍 Dry-run mode: No changes applied\n");
    } else if !new_entries.is_empty() {
        // Create backup
        if create_backup_flag && target_path.exists() {
            match create_backup(&target_path) {
                Ok(_) => output.push_str("✓ Backup created\n"),
                Err(e) => {
                    return Err(ReedError::IoError {
                        operation: "create_backup".to_string(),
                        path: target_path.display().to_string(),
                        reason: e.to_string(),
                    });
                }
            }
        }

        // Ensure .reed directory exists
        fs::create_dir_all(".reed").map_err(|e| ReedError::IoError {
            operation: "create_dir".to_string(),
            path: ".reed".to_string(),
            reason: e.to_string(),
        })?;

        // Merge with existing and write
        let mut all_entries = existing_entries;
        all_entries.extend(new_entries);
        write_csv(&target_path, &all_entries)?;
        output.push_str(&format!(
            "✓ Imported {} entries to .reed/text.csv\n",
            imported_count
        ));
    }

    let duration = start.elapsed();
    output.push_str(&format!("\nDuration: {:.2}s\n", duration.as_secs_f64()));

    Ok(ReedResponse {
        data: output,
        source: "migration".to_string(),
        cached: false,
        timestamp: crate::reedcms::reedstream::current_timestamp(),
        metrics: None,
    })
}

/// Migrates route definitions from CSV.
///
/// ## Usage
/// reed migrate:routes <path> [--dry-run] [--no-backup] [--force]
///
/// ## Arguments
/// - path: Path to routes CSV file
///
/// ## Flags
/// - --dry-run: Preview changes without applying
/// - --no-backup: Skip backup creation
/// - --force: Import even if conflicts exist
///
/// ## CSV Format Expected
/// ```csv
/// route|layout|language|description
/// wissen|knowledge|de|German route for knowledge
/// knowledge|knowledge|en|English route for knowledge
/// ```
///
/// ## Example
/// ```bash
/// reed migrate:routes data/routes.csv
/// reed migrate:routes data/routes.csv --dry-run
/// ```
pub fn migrate_routes(
    args: &[String],
    flags: &HashMap<String, String>,
) -> ReedResult<ReedResponse<String>> {
    let start = Instant::now();

    if args.is_empty() {
        return Err(ReedError::ValidationError {
            field: "path".to_string(),
            value: String::new(),
            constraint: "path to routes.csv file required".to_string(),
        });
    }

    let path = &args[0];
    let dry_run = flags.contains_key("dry-run");
    let create_backup_flag = !flags.contains_key("no-backup");
    let force = flags.contains_key("force");

    let source_path = PathBuf::from(path);
    if !source_path.exists() {
        return Err(ReedError::NotFound {
            resource: format!("path: {}", path),
            context: Some("File does not exist".to_string()),
        });
    }

    let mut output = format!("📦 Migrating routes from: {}\n\n", path);

    // Load source routes
    let source_entries = read_csv(&source_path)?;
    output.push_str(&format!(
        "✓ Found {} route(s) in source file\n\n",
        source_entries.len()
    ));

    // Load existing routes
    let target_path = PathBuf::from(".reed/routes.csv");
    let existing_entries = if target_path.exists() {
        read_csv(&target_path)?
    } else {
        Vec::new()
    };

    // Build conflict map: route@language -> layout
    let mut existing_routes: HashMap<String, String> = HashMap::new();
    for entry in &existing_entries {
        let route_key = entry.key.clone();
        existing_routes.insert(route_key, entry.value.clone());
    }

    // Validate new routes
    let mut conflicts = Vec::new();
    let mut new_entries = Vec::new();
    let mut skipped_count = 0;

    for entry in source_entries {
        let route_key = entry.key.clone();

        if let Some(existing_layout) = existing_routes.get(&route_key) {
            if existing_layout != &entry.value {
                conflicts.push(format!(
                    "  - {} → existing: {}, new: {}",
                    route_key, existing_layout, entry.value
                ));
            }
            skipped_count += 1;
        } else {
            new_entries.push(entry);
        }
    }

    if !conflicts.is_empty() {
        output.push_str(&format!("⚠ Found {} route conflict(s):\n", conflicts.len()));
        for conflict in &conflicts {
            output.push_str(&format!("{}\n", conflict));
        }
        output.push('\n');

        if !force {
            output.push_str("❌ Migration aborted. Use --force to override conflicts.\n");
            return Ok(ReedResponse {
                data: output,
                source: "migration".to_string(),
                cached: false,
                timestamp: crate::reedcms::reedstream::current_timestamp(),
                metrics: None,
            });
        } else {
            output.push_str("⚠ Proceeding with --force flag\n\n");
        }
    }

    output.push_str("Validation:\n");
    output.push_str(&format!("✓ To import: {}\n", new_entries.len()));
    output.push_str(&format!("⊘ Skipped (duplicates): {}\n\n", skipped_count));

    if dry_run {
        output.push_str("🔍 Dry-run mode: No changes applied\n");
    } else if !new_entries.is_empty() {
        // Create backup
        if create_backup_flag && target_path.exists() {
            match create_backup(&target_path) {
                Ok(_) => output.push_str("✓ Backup created\n"),
                Err(e) => {
                    return Err(ReedError::IoError {
                        operation: "create_backup".to_string(),
                        path: target_path.display().to_string(),
                        reason: e.to_string(),
                    });
                }
            }
        }

        // Ensure .reed directory exists
        fs::create_dir_all(".reed").map_err(|e| ReedError::IoError {
            operation: "create_dir".to_string(),
            path: ".reed".to_string(),
            reason: e.to_string(),
        })?;

        // Merge with existing and write
        let mut all_routes = existing_entries;
        all_routes.extend(new_entries.clone());
        write_csv(&target_path, &all_routes)?;
        output.push_str(&format!(
            "✓ Imported {} route(s) to .reed/routes.csv\n",
            new_entries.len()
        ));
    }

    let duration = start.elapsed();
    output.push_str(&format!("\nDuration: {:.2}s\n", duration.as_secs_f64()));

    Ok(ReedResponse {
        data: output,
        source: "migration".to_string(),
        cached: false,
        timestamp: crate::reedcms::reedstream::current_timestamp(),
        metrics: None,
    })
}

// Helper functions

/// Discovers .text.csv files in path.
fn discover_text_csv_files(path: &Path, recursive: bool) -> ReedResult<Vec<PathBuf>> {
    let mut csv_files = Vec::new();

    if path.is_file() {
        if path.file_name().and_then(|n| n.to_str()) == Some("text.csv")
            || path
                .file_name()
                .and_then(|n| n.to_str())
                .map(|n| n.ends_with(".text.csv"))
                .unwrap_or(false)
        {
            csv_files.push(path.to_path_buf());
        }
        return Ok(csv_files);
    }

    if path.is_dir() {
        discover_in_directory(path, &mut csv_files, recursive)?;
    }

    Ok(csv_files)
}

/// Recursively discovers .text.csv files in directory.
fn discover_in_directory(
    dir: &Path,
    csv_files: &mut Vec<PathBuf>,
    recursive: bool,
) -> ReedResult<()> {
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

        if path.is_file() {
            if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                if filename == "text.csv" || filename.ends_with(".text.csv") {
                    csv_files.push(path);
                }
            }
        } else if path.is_dir() && recursive {
            discover_in_directory(&path, csv_files, recursive)?;
        }
    }

    Ok(())
}
