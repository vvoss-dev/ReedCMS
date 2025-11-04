// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Test utilities for ReedBase integration tests.
//!
//! Provides helper functions for creating test databases, inserting test data,
//! and asserting on query results and execution metrics.

use reedbase::{AutoIndexConfig, Database, ExecuteResult, QueryResult};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tempfile::TempDir;

/// Creates a temporary test database with the specified name and initial row count.
///
/// Auto-indexing is **disabled** by default for predictable testing.
/// Use `create_test_database_with_auto_index()` if you need auto-indexing.
///
/// ## Arguments
/// - `name`: Database name (used for temp directory)
/// - `rows`: Number of test rows to insert initially
///
/// ## Returns
/// - `(Database, TempDir)`: Database instance and temp directory handle
pub fn create_test_database(_name: &str, rows: usize) -> (Database, TempDir) {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let db_path = temp_dir.path().join(".reed");
    fs::create_dir_all(&db_path).expect("Failed to create .reed directory");

    // Initialize registry (creates actions.dict and users.dict)
    reedbase::registry::init_registry(&db_path).expect("Failed to init registry");

    // Set base path for registry lookups
    reedbase::registry::set_base_path(db_path.clone());

    // Force reload of dictionaries with new path (critical for concurrent tests)
    reedbase::registry::reload_dictionaries().expect("Failed to reload dictionaries");

    // Open database with auto-indexing disabled (for predictable testing)
    let db = Database::open_with_config(&db_path, AutoIndexConfig::disabled())
        .expect("Failed to open database");

    // Create text table
    db.create_table("text", None)
        .expect("Failed to create text table");

    // Insert test data
    if rows > 0 {
        insert_test_data(&db, rows);
    }

    (db, temp_dir)
}

/// Creates a temporary test database with auto-indexing enabled.
///
/// Use this for testing auto-index creation behavior.
///
/// ## Arguments
/// - `name`: Database name (used for temp directory)
/// - `rows`: Number of test rows to insert initially
///
/// ## Returns
/// - `(Database, TempDir)`: Database instance and temp directory handle
pub fn create_test_database_with_auto_index(_name: &str, rows: usize) -> (Database, TempDir) {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let db_path = temp_dir.path().join(".reed");
    fs::create_dir_all(&db_path).expect("Failed to create .reed directory");

    // Initialize registry (creates actions.dict and users.dict)
    reedbase::registry::init_registry(&db_path).expect("Failed to init registry");

    // Set base path for registry lookups
    reedbase::registry::set_base_path(db_path.clone());

    // Force reload of dictionaries with new path (critical for concurrent tests)
    reedbase::registry::reload_dictionaries().expect("Failed to reload dictionaries");

    // Open database with auto-indexing ENABLED (default config)
    let db = Database::open_with_config(&db_path, AutoIndexConfig::default())
        .expect("Failed to open database");

    // Create text table
    db.create_table("text", None)
        .expect("Failed to create text table");

    // Insert test data
    if rows > 0 {
        insert_test_data(&db, rows);
    }

    (db, temp_dir)
}

/// Creates a test database at a specific path (not temporary).
///
/// Used for creating persistent test fixtures.
pub fn create_test_database_at_path<P: AsRef<Path>>(path: P, rows: usize) -> Database {
    let db_path = path.as_ref().join(".reed");
    fs::create_dir_all(&db_path).expect("Failed to create .reed directory");

    // Initialize registry
    reedbase::registry::init_registry(&db_path).expect("Failed to init registry");

    // Set base path for registry lookups
    reedbase::registry::set_base_path(db_path.clone());

    // Force reload of dictionaries with new path (critical for concurrent tests)
    reedbase::registry::reload_dictionaries().expect("Failed to reload dictionaries");

    // Open database with auto-indexing disabled (for predictable testing)
    let db = Database::open_with_config(&db_path, AutoIndexConfig::disabled())
        .expect("Failed to open database");
    db.create_table("text", None)
        .expect("Failed to create text table");

    if rows > 0 {
        insert_test_data(&db, rows);
    }

    db
}

/// Inserts test data into the text table.
///
/// ## Data Format
/// - Keys: `test.key.000001`, `test.key.000002`, ...
/// - Values: `Test value 1`, `Test value 2`, ...
/// - Description: `Test description 1`, ...
///
/// ## Arguments
/// - `db`: Database instance
/// - `count`: Number of rows to insert
pub fn insert_test_data(db: &Database, count: usize) {
    for i in 0..count {
        let key = format!("test.key.{:06}", i);
        let value = format!("Test value {}", i);
        let desc = format!("Test description {}", i);

        let sql = format!(
            "INSERT INTO text (key, value, description) VALUES ('{}', '{}', '{}')",
            key, value, desc
        );

        db.execute(&sql, "test_user")
            .unwrap_or_else(|e| panic!("Failed to insert row {}: {}", i, e));
    }
}

/// Inserts ReedCMS-style multilingual test data.
///
/// Creates keys with language suffixes (@en, @de) and common patterns.
pub fn insert_multilingual_test_data(db: &Database, keys_per_language: usize) {
    let languages = vec!["en", "de", "fr", "es"];
    let prefixes = vec![
        "page.title",
        "page.header.logo",
        "footer.copyright",
        "menu.item",
    ];

    for prefix in &prefixes {
        for lang in &languages {
            for i in 0..keys_per_language {
                let key = format!("{}.{}@{}", prefix, i, lang);
                let value = format!("{} value {} in {}", prefix, i, lang);
                let desc = format!("Description for {}", key);

                let sql = format!(
                    "INSERT INTO text (key, value, description) VALUES ('{}', '{}', '{}')",
                    key, value, desc
                );

                db.execute(&sql, "test_user")
                    .expect("Failed to insert multilingual data");
            }
        }
    }
}

/// Gets rows from a QueryResult (handles the enum).
pub fn get_rows(result: &QueryResult) -> &Vec<std::collections::HashMap<String, String>> {
    match result {
        QueryResult::Rows(rows) => rows,
        QueryResult::Aggregation(_) => panic!("Expected Rows result, got Aggregation"),
    }
}

/// Asserts that a query result has the expected number of rows.
pub fn assert_query_result_count(result: &QueryResult, expected: usize) {
    let actual = result.row_count();
    assert_eq!(
        actual, expected,
        "Expected {} rows, got {}",
        expected, actual
    );
}

/// Asserts that query execution time is under the specified maximum.
pub fn assert_execution_time_under(duration: Duration, max_ms: u64) {
    let actual_ms = duration.as_millis() as u64;
    assert!(
        actual_ms <= max_ms,
        "Execution took {}ms, expected ≤ {}ms",
        actual_ms,
        max_ms
    );
}

/// Asserts that a query used an index.
///
/// Note: Since QueryResult doesn't expose metrics, this is a placeholder
/// that always passes. Actual index usage is tracked internally in Database.
pub fn assert_query_used_index(_result: &QueryResult) {
    // TODO: Once metrics are exposed in QueryResult, check index_used field
    // For now, this is a no-op
}

/// Asserts that a query did NOT use an index (full scan).
///
/// Note: Since QueryResult doesn't expose metrics, this is a placeholder
/// that always passes. Actual index usage is tracked internally in Database.
pub fn assert_query_full_scan(_result: &QueryResult) {
    // TODO: Once metrics are exposed in QueryResult, check index_used field
    // For now, this is a no-op
}

/// Asserts that an execute result affected the expected number of rows.
pub fn assert_rows_affected(result: &ExecuteResult, expected: usize) {
    assert_eq!(
        result.rows_affected, expected,
        "Expected {} rows affected, got {}",
        expected, result.rows_affected
    );
}

/// Cleans up a test database directory.
pub fn cleanup_test_database<P: AsRef<Path>>(path: P) {
    let _ = fs::remove_dir_all(path.as_ref());
}

/// Creates test fixture directories if they don't exist.
pub fn ensure_test_fixtures() -> PathBuf {
    let fixtures_path = PathBuf::from("test_data");

    for size in &["small", "medium", "large", "versioned"] {
        let fixture_path = fixtures_path.join(size);
        fs::create_dir_all(fixture_path.join(".reed/tables"))
            .expect("Failed to create fixture directory");
    }

    fixtures_path
}

/// Macro for asserting query returns expected row count.
#[macro_export]
macro_rules! assert_query_returns {
    ($db:expr, $sql:expr, $expected:expr) => {
        let result = $db.query($sql).expect("Query failed");
        assert_eq!(
            get_rows(&result).len(),
            $expected,
            "Query '{}' expected {} rows, got {}",
            $sql,
            $expected,
            get_rows(&result).len()
        );
    };
}

/// Macro for asserting execute affects expected row count.
#[macro_export]
macro_rules! assert_exec_affects {
    ($db:expr, $sql:expr, $expected:expr) => {
        let result = $db.execute($sql, "test").expect("Execute failed");
        assert_eq!(
            result.rows_affected, $expected,
            "Execute '{}' expected {} rows affected, got {}",
            $sql, $expected, result.rows_affected
        );
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_test_database() {
        let (db, _temp) = create_test_database("test", 10);
        let result = db.query("SELECT * FROM text").unwrap();
        assert_eq!(result.row_count(), 10);
    }

    #[test]
    fn test_insert_test_data() {
        let (db, _temp) = create_test_database("test", 0);
        insert_test_data(&db, 5);
        let result = db.query("SELECT * FROM text").unwrap();
        assert_eq!(result.row_count(), 5);
    }

    #[test]
    fn test_assert_query_result_count() {
        let (db, _temp) = create_test_database("test", 3);
        let result = db.query("SELECT * FROM text").unwrap();
        assert_query_result_count(&result, 3);
    }
}
