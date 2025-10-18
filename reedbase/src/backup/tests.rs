// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Comprehensive tests for backup and point-in-time recovery.

use crate::backup::{create_backup, list_backups, restore_point_in_time, RestoreReport};
use crate::registry::init_registry;
use crate::tables::Table;
use std::fs;
use std::thread;
use std::time::Duration;
use tempfile::TempDir;

/// Helper: Create test database with sample data.
fn setup_test_db() -> TempDir {
    let temp = TempDir::new().expect("Failed to create temp dir");
    let base_path = temp.path();

    // Initialize registry for user codes
    crate::registry::set_base_path(base_path.to_path_buf());
    init_registry(base_path).expect("Failed to init registry");
    crate::registry::reload_dictionaries().expect("Failed to reload dictionaries");

    // Create tables directory
    fs::create_dir_all(base_path.join("tables")).expect("Failed to create tables dir");

    // Create two tables with version history
    let table1 = Table::new(base_path, "users");
    table1
        .init(b"key|value\nuser:1|Alice\n", "test_user")
        .expect("Failed to init table1");

    thread::sleep(Duration::from_millis(100));

    table1
        .write(b"key|value\nuser:1|Alice\nuser:2|Bob\n", "test_user")
        .expect("Failed to write table1");

    thread::sleep(Duration::from_millis(100));

    let table2 = Table::new(base_path, "posts");
    table2
        .init(b"key|value\npost:1|Hello World\n", "test_user")
        .expect("Failed to init table2");

    temp
}

#[test]
fn test_create_backup_success() {
    let temp = setup_test_db();
    let base_path = temp.path();

    let result = create_backup(base_path);
    assert!(result.is_ok(), "Backup creation should succeed");

    let info = result.unwrap();
    assert!(info.timestamp > 0, "Timestamp should be positive");
    assert!(info.path.exists(), "Backup file should exist");
    assert!(info.size_bytes > 0, "Backup should have size");
    assert!(info.size_mb > 0.0, "Size in MB should be positive");

    // Verify backup is a valid tar.gz file
    assert!(info.path.extension().unwrap() == "gz", "Should be .gz file");
}

#[test]
fn test_create_backup_creates_directory() {
    let temp = TempDir::new().expect("Failed to create temp dir");
    let base_path = temp.path();

    // Create minimal database structure
    fs::create_dir_all(base_path.join("tables")).expect("Failed to create tables dir");

    // Backup directory doesn't exist yet
    assert!(!base_path.join("backups").exists());

    let result = create_backup(base_path);
    assert!(result.is_ok(), "Backup creation should succeed");

    // Backup directory should now exist
    assert!(base_path.join("backups").exists());
}

#[test]
fn test_list_backups_empty() {
    let temp = TempDir::new().expect("Failed to create temp dir");
    let base_path = temp.path();

    let result = list_backups(base_path);
    assert!(result.is_ok(), "List should succeed even with no backups");

    let backups = result.unwrap();
    assert_eq!(backups.len(), 0, "Should return empty list");
}

#[test]
fn test_list_backups_single() {
    let temp = setup_test_db();
    let base_path = temp.path();

    // Create one backup
    create_backup(base_path).expect("Failed to create backup");

    let result = list_backups(base_path);
    assert!(result.is_ok(), "List should succeed");

    let backups = result.unwrap();
    assert_eq!(backups.len(), 1, "Should find one backup");
    assert!(backups[0].timestamp > 0);
}

#[test]
fn test_list_backups_multiple_sorted() {
    let temp = setup_test_db();
    let base_path = temp.path();

    // Create multiple backups with delays
    create_backup(base_path).expect("Failed to create backup 1");
    thread::sleep(Duration::from_millis(1100));

    create_backup(base_path).expect("Failed to create backup 2");
    thread::sleep(Duration::from_millis(1100));

    create_backup(base_path).expect("Failed to create backup 3");

    let result = list_backups(base_path);
    assert!(result.is_ok(), "List should succeed");

    let backups = result.unwrap();
    assert_eq!(backups.len(), 3, "Should find three backups");

    // Verify sorted by timestamp (newest first)
    assert!(
        backups[0].timestamp > backups[1].timestamp,
        "Should be sorted newest first"
    );
    assert!(
        backups[1].timestamp > backups[2].timestamp,
        "Should be sorted newest first"
    );
}

#[test]
fn test_list_backups_ignores_non_backup_files() {
    let temp = setup_test_db();
    let base_path = temp.path();

    // Create backup directory with mixed files
    let backup_dir = base_path.join("backups");
    fs::create_dir_all(&backup_dir).expect("Failed to create backup dir");

    // Create valid backup
    create_backup(base_path).expect("Failed to create backup");

    // Create invalid files
    fs::write(backup_dir.join("readme.txt"), "test").expect("Failed to write txt");
    fs::write(backup_dir.join("1234567890.tar"), "test").expect("Failed to write tar");
    fs::write(backup_dir.join("not_a_timestamp.tar.gz"), "test")
        .expect("Failed to write invalid gz");

    let result = list_backups(base_path);
    assert!(result.is_ok(), "List should succeed");

    let backups = result.unwrap();
    assert_eq!(backups.len(), 1, "Should only find valid backups");
}

#[test]
fn test_restore_point_in_time_success() {
    let temp = setup_test_db();
    let base_path = temp.path();

    // Get current timestamp (after all writes)
    let current_time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64;

    // Restore to current time (should restore everything)
    let result = restore_point_in_time(base_path, current_time);
    assert!(result.is_ok(), "Restore should succeed");

    let report = result.unwrap();
    assert_eq!(report.target_timestamp, current_time);
    assert!(report.is_success(), "Restore should be successful");
    assert!(
        !report.tables_restored.is_empty(),
        "Should have restored tables"
    );
}

#[test]
fn test_restore_point_in_time_partial() {
    let temp = TempDir::new().expect("Failed to create temp dir");
    let base_path = temp.path();

    // Initialize registry
    crate::registry::set_base_path(base_path.to_path_buf());
    init_registry(base_path).expect("Failed to init registry");
    crate::registry::reload_dictionaries().expect("Failed to reload dictionaries");

    fs::create_dir_all(base_path.join("tables")).expect("Failed to create tables dir");

    // Create table1 at T1
    let table1 = Table::new(base_path, "users");
    table1
        .init(b"key|value\nuser:1|Alice\n", "test_user")
        .expect("Failed to init table1");

    let t1 = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64;

    // Wait
    thread::sleep(Duration::from_millis(1100));

    // Create table2 at T2 (after T1)
    let table2 = Table::new(base_path, "posts");
    table2
        .init(b"key|value\npost:1|Hello\n", "test_user")
        .expect("Failed to init table2");

    // Restore to T1 (before table2 existed)
    let result = restore_point_in_time(base_path, t1);
    assert!(result.is_ok(), "Restore should succeed");

    let report = result.unwrap();
    assert_eq!(
        report.tables_restored.len(),
        1,
        "Should restore table1 only"
    );
    assert_eq!(report.tables_skipped.len(), 1, "Should skip table2");
    assert!(report.is_success(), "Should be successful (no errors)");
}

#[test]
fn test_restore_point_in_time_no_tables() {
    let temp = TempDir::new().expect("Failed to create temp dir");
    let base_path = temp.path();

    // No tables directory
    let result = restore_point_in_time(base_path, 1234567890);
    assert!(result.is_err(), "Should fail with no tables");
}

#[test]
fn test_restore_point_in_time_before_all_data() {
    let temp = setup_test_db();
    let base_path = temp.path();

    // Restore to timestamp 1 (before any data existed)
    let result = restore_point_in_time(base_path, 1);
    assert!(result.is_ok(), "Restore should succeed");

    let report = result.unwrap();
    assert!(
        report.tables_restored.is_empty(),
        "Should not restore any tables"
    );
    assert!(!report.tables_skipped.is_empty(), "Should skip all tables");
    assert!(report.is_success(), "Should be successful (no errors)");
}

#[test]
fn test_backup_info_size_calculation() {
    let temp = setup_test_db();
    let base_path = temp.path();

    let info = create_backup(base_path).expect("Failed to create backup");

    // Verify size calculations
    assert_eq!(
        info.size_mb,
        info.size_bytes as f64 / 1_048_576.0,
        "MB calculation should be correct"
    );
}

#[test]
fn test_restore_report_status_methods() {
    // Success: no errors
    let mut report = RestoreReport::new(1234567890);
    report
        .tables_restored
        .push(("users".to_string(), 1234567890));
    assert!(report.is_success());
    assert!(!report.is_partial());
    assert!(!report.is_failure());

    // Partial: errors + restored tables
    let mut report = RestoreReport::new(1234567890);
    report
        .tables_restored
        .push(("users".to_string(), 1234567890));
    report.errors.push((
        "posts".to_string(),
        crate::error::ReedError::TableNotFound {
            name: "posts".to_string(),
        },
    ));
    assert!(!report.is_success());
    assert!(report.is_partial());
    assert!(!report.is_failure());

    // Failure: errors + no restored tables
    let mut report = RestoreReport::new(1234567890);
    report.errors.push((
        "users".to_string(),
        crate::error::ReedError::TableNotFound {
            name: "users".to_string(),
        },
    ));
    assert!(!report.is_success());
    assert!(!report.is_partial());
    assert!(report.is_failure());
}

#[test]
fn test_backup_path_format() {
    let temp = setup_test_db();
    let base_path = temp.path();

    let info = create_backup(base_path).expect("Failed to create backup");

    // Verify path format: {base_path}/backups/{timestamp}.tar.gz
    let expected_dir = base_path.join("backups");
    assert_eq!(
        info.path.parent().unwrap(),
        expected_dir,
        "Backup should be in backups/ directory"
    );

    let filename = info.path.file_name().unwrap().to_str().unwrap();
    assert!(
        filename.ends_with(".tar.gz"),
        "Filename should end with .tar.gz"
    );

    // Verify timestamp in filename matches info.timestamp
    let stem = info
        .path
        .file_stem()
        .unwrap()
        .to_str()
        .unwrap()
        .strip_suffix(".tar")
        .unwrap();
    let parsed_timestamp: u64 = stem.parse().expect("Failed to parse timestamp");
    assert_eq!(
        parsed_timestamp, info.timestamp,
        "Filename timestamp should match info"
    );
}

#[test]
fn test_create_multiple_backups_unique_timestamps() {
    let temp = setup_test_db();
    let base_path = temp.path();

    let info1 = create_backup(base_path).expect("Failed to create backup 1");
    thread::sleep(Duration::from_millis(1100)); // Ensure different timestamp

    let info2 = create_backup(base_path).expect("Failed to create backup 2");

    assert_ne!(
        info1.timestamp, info2.timestamp,
        "Backups should have unique timestamps"
    );
    assert_ne!(info1.path, info2.path, "Backups should have unique paths");
}

#[test]
fn test_restore_preserves_table_data() {
    let temp = TempDir::new().expect("Failed to create temp dir");
    let base_path = temp.path();

    // Initialize registry
    crate::registry::set_base_path(base_path.to_path_buf());
    init_registry(base_path).expect("Failed to init registry");
    crate::registry::reload_dictionaries().expect("Failed to reload dictionaries");

    fs::create_dir_all(base_path.join("tables")).expect("Failed to create tables dir");

    // Create data
    let table = Table::new(base_path, "users");
    table
        .init(b"key|value\nuser:1|Alice\n", "test_user")
        .expect("Failed to init table");

    let restore_time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64;

    thread::sleep(Duration::from_millis(100));

    // Modify data (after restore point)
    table
        .write(b"key|value\nuser:1|Bob\n", "test_user")
        .expect("Failed to update table");

    // Restore to before modification
    let result = restore_point_in_time(base_path, restore_time);
    assert!(result.is_ok(), "Restore should succeed");

    // Verify data is restored
    let content = table.read_current().expect("Failed to read table");
    let content_str = String::from_utf8_lossy(&content);
    assert!(
        content_str.contains("Alice"),
        "Data should be restored to original value"
    );
    assert!(
        !content_str.contains("Bob"),
        "Modified data should not be present"
    );
}
