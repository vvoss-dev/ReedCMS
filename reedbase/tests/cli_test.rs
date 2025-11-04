// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! CLI integration tests.
//!
//! Tests the `reedbase` command-line tool with real database operations.
//! Uses assert_cmd for running the binary and predicates for output assertions.

mod test_utils;

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;
use test_utils::*;

/// Helper to create a test database and return its path as a string.
fn setup_test_db(name: &str, rows: usize) -> (TempDir, String) {
    let temp = TempDir::new().expect("Failed to create temp dir");
    let db_path = temp.path().join(".reed");
    fs::create_dir_all(&db_path).expect("Failed to create .reed directory");

    let db = create_test_database_at_path(temp.path(), rows);
    drop(db); // Close database

    let path_str = temp.path().to_str().unwrap().to_string();
    (temp, path_str)
}

// ============================================================================
// Query Command
// ============================================================================

#[test]
fn test_cli_query_basic() {
    let (_temp, db_path) = setup_test_db("query_basic", 10);

    Command::cargo_bin("reedbase")
        .unwrap()
        .args(&["query", "SELECT * FROM text LIMIT 5", &db_path])
        .assert()
        .success()
        .stdout(predicate::str::contains("5 rows"));
}

#[test]
fn test_cli_query_json_format() {
    let (_temp, db_path) = setup_test_db("query_json", 5);

    Command::cargo_bin("reedbase")
        .unwrap()
        .args(&[
            "query",
            "SELECT * FROM text LIMIT 2",
            &db_path,
            "--format",
            "json",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("["))
        .stdout(predicate::str::contains("key"))
        .stdout(predicate::str::contains("value"));
}

#[test]
fn test_cli_query_csv_format() {
    let (_temp, db_path) = setup_test_db("query_csv", 5);

    Command::cargo_bin("reedbase")
        .unwrap()
        .args(&[
            "query",
            "SELECT * FROM text LIMIT 2",
            &db_path,
            "--format",
            "csv",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("key,value"));
}

#[test]
fn test_cli_query_to_file() {
    let (_temp, db_path) = setup_test_db("query_file", 5);
    let output_file = _temp.path().join("output.txt");
    let output_path = output_file.to_str().unwrap();

    Command::cargo_bin("reedbase")
        .unwrap()
        .args(&[
            "query",
            "SELECT * FROM text LIMIT 3",
            &db_path,
            "--output",
            output_path,
        ])
        .assert()
        .success();

    // Verify file was created
    assert!(output_file.exists(), "Output file should be created");

    let content = fs::read_to_string(output_file).expect("Failed to read output file");
    assert!(content.contains("test.key"));
}

#[test]
fn test_cli_query_no_header() {
    let (_temp, db_path) = setup_test_db("query_no_header", 5);

    let output = Command::cargo_bin("reedbase")
        .unwrap()
        .args(&[
            "query",
            "SELECT * FROM text LIMIT 2",
            &db_path,
            "--format",
            "csv",
            "--no-header",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let output_str = String::from_utf8(output).unwrap();

    // Should NOT contain header line
    assert!(!output_str
        .lines()
        .next()
        .unwrap_or("")
        .starts_with("key,value"));
}

#[test]
fn test_cli_query_with_where() {
    let (_temp, db_path) = setup_test_db("query_where", 20);

    Command::cargo_bin("reedbase")
        .unwrap()
        .args(&[
            "query",
            "SELECT * FROM text WHERE key = 'test.key.000010'",
            &db_path,
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("1 row"))
        .stdout(predicate::str::contains("test.key.000010"));
}

// ============================================================================
// Exec Command
// ============================================================================

#[test]
fn test_cli_exec_insert() {
    let (_temp, db_path) = setup_test_db("exec_insert", 0);

    Command::cargo_bin("reedbase")
        .unwrap()
        .args(&[
            "exec",
            "INSERT INTO text (key, value) VALUES ('new.key', 'new value')",
            &db_path,
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("1 row affected"));

    // Verify insertion
    Command::cargo_bin("reedbase")
        .unwrap()
        .args(&[
            "query",
            "SELECT * FROM text WHERE key = 'new.key'",
            &db_path,
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("new.key"));
}

#[test]
fn test_cli_exec_update() {
    let (_temp, db_path) = setup_test_db("exec_update", 10);

    Command::cargo_bin("reedbase")
        .unwrap()
        .args(&[
            "exec",
            "UPDATE text SET value = 'updated' WHERE key = 'test.key.000005'",
            &db_path,
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("1 row affected"));

    // Verify update
    Command::cargo_bin("reedbase")
        .unwrap()
        .args(&[
            "query",
            "SELECT value FROM text WHERE key = 'test.key.000005'",
            &db_path,
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("updated"));
}

#[test]
fn test_cli_exec_delete() {
    let (_temp, db_path) = setup_test_db("exec_delete", 10);

    Command::cargo_bin("reedbase")
        .unwrap()
        .args(&[
            "exec",
            "DELETE FROM text WHERE key = 'test.key.000007'",
            &db_path,
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("1 row affected"));

    // Verify deletion
    Command::cargo_bin("reedbase")
        .unwrap()
        .args(&[
            "query",
            "SELECT * FROM text WHERE key = 'test.key.000007'",
            &db_path,
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("0 rows"));
}

#[test]
fn test_cli_exec_with_user() {
    let (_temp, db_path) = setup_test_db("exec_user", 0);

    Command::cargo_bin("reedbase")
        .unwrap()
        .args(&[
            "exec",
            "INSERT INTO text (key, value) VALUES ('test', 'value')",
            &db_path,
            "--user",
            "alice",
        ])
        .assert()
        .success();
}

#[test]
fn test_cli_exec_quiet_mode() {
    let (_temp, db_path) = setup_test_db("exec_quiet", 0);

    let output = Command::cargo_bin("reedbase")
        .unwrap()
        .args(&[
            "exec",
            "INSERT INTO text (key, value) VALUES ('test', 'value')",
            &db_path,
            "--quiet",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    // Quiet mode should produce no output
    assert_eq!(output.len(), 0, "Quiet mode should produce no output");
}

// ============================================================================
// Tables Command
// ============================================================================

#[test]
fn test_cli_tables_list() {
    let (_temp, db_path) = setup_test_db("tables_list", 5);

    Command::cargo_bin("reedbase")
        .unwrap()
        .args(&["tables", &db_path])
        .assert()
        .success()
        .stdout(predicate::str::contains("text"));
}

#[test]
fn test_cli_tables_create() {
    let (_temp, db_path) = setup_test_db("tables_create", 0);

    Command::cargo_bin("reedbase")
        .unwrap()
        .args(&["tables", &db_path, "--create", "routes"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Created table: routes"));

    // Verify table exists
    Command::cargo_bin("reedbase")
        .unwrap()
        .args(&["tables", &db_path])
        .assert()
        .success()
        .stdout(predicate::str::contains("routes"));
}

#[test]
fn test_cli_tables_drop() {
    let (_temp, db_path) = setup_test_db("tables_drop", 5);

    Command::cargo_bin("reedbase")
        .unwrap()
        .args(&["tables", &db_path, "--drop", "text", "--confirm"])
        .assert()
        .success();

    // Verify table is gone
    Command::cargo_bin("reedbase")
        .unwrap()
        .args(&["tables", &db_path])
        .assert()
        .success()
        .stdout(predicate::str::contains("text").not());
}

#[test]
fn test_cli_tables_verbose() {
    let (_temp, db_path) = setup_test_db("tables_verbose", 10);

    Command::cargo_bin("reedbase")
        .unwrap()
        .args(&["tables", &db_path, "--verbose"])
        .assert()
        .success()
        .stdout(predicate::str::contains("rows"));
}

// ============================================================================
// Indices Command
// ============================================================================

#[test]
fn test_cli_indices_list() {
    let (_temp, db_path) = setup_test_db("indices_list", 10);

    // Initially no indices
    Command::cargo_bin("reedbase")
        .unwrap()
        .args(&["indices", &db_path])
        .assert()
        .success();
}

#[test]
fn test_cli_indices_create() {
    let (_temp, db_path) = setup_test_db("indices_create", 10);

    Command::cargo_bin("reedbase")
        .unwrap()
        .args(&["indices", &db_path, "--create", "text.key"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Created index: text.key"));

    // Verify index exists
    Command::cargo_bin("reedbase")
        .unwrap()
        .args(&["indices", &db_path])
        .assert()
        .success()
        .stdout(predicate::str::contains("text.key"));
}

#[test]
fn test_cli_indices_drop() {
    let (_temp, db_path) = setup_test_db("indices_drop", 10);

    // Create index first
    Command::cargo_bin("reedbase")
        .unwrap()
        .args(&["indices", &db_path, "--create", "text.key"])
        .assert()
        .success();

    // Drop it
    Command::cargo_bin("reedbase")
        .unwrap()
        .args(&["indices", &db_path, "--drop", "text.key"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Dropped index: text.key"));
}

#[test]
fn test_cli_indices_verbose() {
    let (_temp, db_path) = setup_test_db("indices_verbose", 10);

    // Create index
    Command::cargo_bin("reedbase")
        .unwrap()
        .args(&["indices", &db_path, "--create", "text.key"])
        .assert()
        .success();

    // List with verbose
    Command::cargo_bin("reedbase")
        .unwrap()
        .args(&["indices", &db_path, "--verbose"])
        .assert()
        .success()
        .stdout(predicate::str::contains("entries"));
}

// ============================================================================
// Stats Command
// ============================================================================

#[test]
fn test_cli_stats_display() {
    let (_temp, db_path) = setup_test_db("stats_display", 50);

    Command::cargo_bin("reedbase")
        .unwrap()
        .args(&["stats", &db_path])
        .assert()
        .success()
        .stdout(predicate::str::contains("Database Statistics"))
        .stdout(predicate::str::contains("Tables:"))
        .stdout(predicate::str::contains("Total Rows:"));
}

#[test]
fn test_cli_stats_json() {
    let (_temp, db_path) = setup_test_db("stats_json", 50);

    Command::cargo_bin("reedbase")
        .unwrap()
        .args(&["stats", &db_path, "--format", "json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("{"))
        .stdout(predicate::str::contains("table_count"));
}

// ============================================================================
// Explain Command
// ============================================================================

#[test]
fn test_cli_explain_query() {
    let (_temp, db_path) = setup_test_db("explain_query", 100);

    Command::cargo_bin("reedbase")
        .unwrap()
        .args(&[
            "explain",
            "SELECT * FROM text WHERE key = 'test.key.000050'",
            &db_path,
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Query Plan"));
}

#[test]
fn test_cli_explain_verbose() {
    let (_temp, db_path) = setup_test_db("explain_verbose", 100);

    Command::cargo_bin("reedbase")
        .unwrap()
        .args(&[
            "explain",
            "SELECT * FROM text WHERE key LIKE 'test%'",
            &db_path,
            "--verbose",
        ])
        .assert()
        .success();
}

// ============================================================================
// Error Cases
// ============================================================================

#[test]
fn test_cli_invalid_path() {
    Command::cargo_bin("reedbase")
        .unwrap()
        .args(&["query", "SELECT * FROM text", "/nonexistent/path"])
        .assert()
        .failure();
}

#[test]
fn test_cli_invalid_sql() {
    let (_temp, db_path) = setup_test_db("error_sql", 10);

    Command::cargo_bin("reedbase")
        .unwrap()
        .args(&["query", "SELECT FORM text", &db_path])
        .assert()
        .failure();
}

#[test]
fn test_cli_missing_arguments() {
    Command::cargo_bin("reedbase")
        .unwrap()
        .args(&["query"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}

#[test]
fn test_cli_help() {
    Command::cargo_bin("reedbase")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage:"))
        .stdout(predicate::str::contains("query"))
        .stdout(predicate::str::contains("exec"));
}

#[test]
fn test_cli_version() {
    Command::cargo_bin("reedbase")
        .unwrap()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("0.1.0"));
}

// ============================================================================
// Shell Command (Limited Testing)
// ============================================================================

// Note: Interactive shell testing is limited without PTY.
// We can only test that it starts successfully with --help or invalid input.

#[test]
fn test_cli_shell_help() {
    Command::cargo_bin("reedbase")
        .unwrap()
        .args(&["shell", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Interactive ReedBase shell"));
}
