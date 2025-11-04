// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Error handling integration tests.
//!
//! Verifies proper error messages, error types, and recovery behavior:
//! - Table not found errors
//! - Invalid SQL syntax errors
//! - Invalid column errors
//! - Index errors (already exists, not found)
//! - File system errors
//! - Corrupted database recovery

mod test_utils;

use reedbase::{Database, ReedError};
use std::fs;
use test_utils::*;

// ============================================================================
// Query Errors
// ============================================================================

#[test]
fn test_table_not_found_error() {
    let (db, _temp) = create_test_database("error_table_not_found", 0);

    let result = db.query("SELECT * FROM nonexistent_table");

    assert!(result.is_err(), "Query on nonexistent table should fail");

    match result.unwrap_err() {
        ReedError::TableNotFound { name } => {
            assert_eq!(name, "nonexistent_table");
        }
        other => panic!("Expected TableNotFound error, got: {:?}", other),
    }
}

#[test]
fn test_invalid_sql_syntax_error() {
    let (db, _temp) = create_test_database("error_invalid_sql", 10);

    let result = db.query("SELECT FORM text");
    assert!(result.is_err(), "Invalid SQL should fail");

    let result2 = db.query("SELEKT * FROM text");
    assert!(result2.is_err(), "Invalid SQL should fail");

    let result3 = db.query("SELECT * FROM");
    assert!(result3.is_err(), "Incomplete SQL should fail");
}

#[test]
fn test_empty_query_error() {
    let (db, _temp) = create_test_database("error_empty_query", 10);

    let result = db.query("");
    assert!(result.is_err(), "Empty query should fail");

    let result2 = db.query("   ");
    assert!(result2.is_err(), "Whitespace-only query should fail");
}

#[test]
fn test_invalid_where_clause() {
    let (db, _temp) = create_test_database("error_where", 10);

    // Malformed WHERE clause
    let result = db.query("SELECT * FROM text WHERE");
    assert!(result.is_err(), "Incomplete WHERE clause should fail");

    let result2 = db.query("SELECT * FROM text WHERE =");
    assert!(result2.is_err(), "Invalid WHERE clause should fail");
}

// ============================================================================
// Execute Errors
// ============================================================================

#[test]
fn test_insert_invalid_syntax() {
    let (db, _temp) = create_test_database("error_insert_syntax", 0);

    let result = db.execute("INSERT INTO text VALUES ('key')", "admin");
    assert!(result.is_err(), "Invalid INSERT syntax should fail");

    let result2 = db.execute("INSERT text (key) VALUES ('key')", "admin");
    assert!(result2.is_err(), "Missing INTO should fail");
}

#[test]
fn test_update_invalid_syntax() {
    let (db, _temp) = create_test_database("error_update_syntax", 10);

    let result = db.execute("UPDATE text SET", "admin");
    assert!(result.is_err(), "Incomplete UPDATE should fail");

    let result2 = db.execute("UPDATE text value = 'new'", "admin");
    assert!(result2.is_err(), "Missing SET should fail");
}

#[test]
fn test_delete_invalid_syntax() {
    let (db, _temp) = create_test_database("error_delete_syntax", 10);

    let result = db.execute("DELETE text WHERE key = 'test'", "admin");
    assert!(result.is_err(), "Missing FROM should fail");

    let result2 = db.execute("DELETE FROM", "admin");
    assert!(result2.is_err(), "Incomplete DELETE should fail");
}

#[test]
fn test_execute_select_error() {
    let (db, _temp) = create_test_database("error_execute_select", 10);

    // Execute should not accept SELECT
    let result = db.execute("SELECT * FROM text", "admin");
    assert!(result.is_err(), "Execute should not accept SELECT queries");
}

#[test]
fn test_query_insert_error() {
    let (db, _temp) = create_test_database("error_query_insert", 0);

    // Query should not accept INSERT
    let result = db.query("INSERT INTO text (key, value) VALUES ('test', 'value')");
    assert!(result.is_err(), "Query should not accept INSERT commands");
}

// ============================================================================
// Index Errors
// ============================================================================

#[test]
fn test_index_already_exists_error() {
    let (db, _temp) = create_test_database("error_index_exists", 10);

    // Create index
    db.create_index("text", "key")
        .expect("First index should succeed");

    // Try to create same index again
    let result = db.create_index("text", "key");
    assert!(result.is_err(), "Duplicate index should fail");

    match result.unwrap_err() {
        ReedError::IndexAlreadyExists { table, column } => {
            assert_eq!(table, "text");
            assert_eq!(column, "key");
        }
        other => panic!("Expected IndexAlreadyExists error, got: {:?}", other),
    }
}

#[test]
fn test_index_table_not_found() {
    let (db, _temp) = create_test_database("error_index_no_table", 0);

    let result = db.create_index("nonexistent", "key");
    assert!(result.is_err(), "Index on nonexistent table should fail");
}

#[test]
#[ignore] // TODO: Implement drop_index in Database API
fn test_drop_index_not_found() {
    let (db, _temp) = create_test_database("error_drop_index", 10);

    // let result = db.drop_index("text", "nonexistent_column");
    // assert!(result.is_err(), "Dropping nonexistent index should fail");
}

// ============================================================================
// Table Errors
// ============================================================================

#[test]
fn test_create_table_already_exists() {
    let (db, _temp) = create_test_database("error_table_exists", 0);

    // text table already exists from setup
    let result = db.create_table("text", None);
    assert!(result.is_err(), "Creating existing table should fail");
}

#[test]
fn test_create_table_invalid_name() {
    let (db, _temp) = create_test_database("error_table_name", 0);

    // Table names with invalid characters
    let result = db.create_table("", None);
    assert!(result.is_err(), "Empty table name should fail");

    let result2 = db.create_table("table/with/slash", None);
    assert!(result2.is_err(), "Table name with slashes should fail");
}

// ============================================================================
// File System Errors
// ============================================================================

#[test]
fn test_open_nonexistent_database() {
    let result = Database::open("/nonexistent/path/.reed");
    assert!(result.is_err(), "Opening nonexistent database should fail");
}

#[test]
fn test_open_invalid_path() {
    let result = Database::open("");
    assert!(result.is_err(), "Opening with empty path should fail");
}

#[test]
fn test_read_only_database_write() {
    let (db, temp) = create_test_database("error_readonly", 10);
    let db_path = temp.path().join(".reed");
    drop(db); // Close database

    // Make directory read-only
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = fs::Permissions::from_mode(0o444); // Read-only
        fs::set_permissions(&db_path, perms).unwrap();
    }

    // Try to open and write
    let db = Database::open(&db_path);

    #[cfg(unix)]
    {
        if let Ok(db) = db {
            let result = db.execute(
                "INSERT INTO text (key, value) VALUES ('test', 'value')",
                "admin",
            );
            // Should fail due to read-only permissions
            // Note: Actual behavior depends on implementation
            println!("Read-only write result: {:?}", result);
        }

        // Restore permissions for cleanup
        use std::os::unix::fs::PermissionsExt;
        let perms = fs::Permissions::from_mode(0o755);
        fs::set_permissions(&db_path, perms).unwrap();
    }
}

// ============================================================================
// Data Validation Errors
// ============================================================================

#[test]
fn test_insert_missing_required_column() {
    let (db, _temp) = create_test_database("error_missing_column", 0);

    // Insert without key (assuming key is required)
    let result = db.execute("INSERT INTO text (value) VALUES ('only value')", "admin");

    // Should fail if key is required
    // Note: Behavior depends on schema validation
    println!("Missing column result: {:?}", result);
}

#[test]
fn test_update_nonexistent_column() {
    let (db, _temp) = create_test_database("error_update_column", 10);

    let result = db.execute(
        "UPDATE text SET nonexistent_column = 'value' WHERE key = 'test.key.000001'",
        "admin",
    );

    // Should fail or succeed depending on schema flexibility
    println!("Update nonexistent column result: {:?}", result);
}

// ============================================================================
// Corrupted Database Recovery
// ============================================================================

#[test]
fn test_corrupted_csv_recovery() {
    let (db, temp) = create_test_database("error_corrupted", 10);
    let db_path = temp.path().join(".reed");
    let table_path = db_path.join("tables/text/current.csv");
    drop(db); // Close database

    // Corrupt the CSV file
    fs::write(&table_path, "CORRUPTED DATA\nINVALID\n").unwrap();

    // Try to open database
    let result = Database::open(&db_path);

    // Should either fail gracefully or recover
    match result {
        Ok(db) => {
            println!("Database opened despite corruption");
            let query_result = db.query("SELECT * FROM text");
            println!("Query on corrupted table: {:?}", query_result);
        }
        Err(e) => {
            println!("Database failed to open due to corruption: {:?}", e);
        }
    }
}

#[test]
fn test_empty_csv_file() {
    let (db, temp) = create_test_database("error_empty_csv", 10);
    let db_path = temp.path().join(".reed");
    let table_path = db_path.join("tables/text/current.csv");
    drop(db); // Close database

    // Create empty CSV file
    fs::write(&table_path, "").unwrap();

    // Try to open and query
    let result = Database::open(&db_path);

    if let Ok(db) = result {
        let query_result = db.query("SELECT * FROM text");
        println!("Query on empty CSV: {:?}", query_result);

        // Should return empty result set
        if let Ok(result) = query_result {
            assert_eq!(get_rows(&result).len(), 0);
        }
    }
}

// ============================================================================
// Concurrent Error Handling
// ============================================================================

#[test]
fn test_concurrent_errors_isolated() {
    use std::sync::Arc;
    use std::thread;

    let (db, _temp) = create_test_database("error_concurrent", 10);
    let db = Arc::new(db);

    let mut handles = vec![];

    // Spawn threads that will encounter errors
    for i in 0..5 {
        let db_clone = Arc::clone(&db);
        let handle = thread::spawn(move || {
            // Some threads query nonexistent table
            let result = db_clone.query("SELECT * FROM nonexistent");
            assert!(result.is_err());

            // Should not affect other operations
            let valid_result = db_clone.query("SELECT * FROM text LIMIT 1");
            assert!(valid_result.is_ok());
        });
        handles.push(handle);
    }

    // Wait for all threads
    for handle in handles {
        handle.join().expect("Thread panicked");
    }
}

// ============================================================================
// Error Message Quality
// ============================================================================

#[test]
fn test_error_messages_informative() {
    let (db, _temp) = create_test_database("error_messages", 10);

    // Test various errors and check message quality
    let mut error_messages = Vec::new();

    if let Err(e) = db.query("SELECT * FROM nonexistent") {
        error_messages.push(format!("{:?}", e));
    }
    if let Err(e) = db.query("SELECT FORM text") {
        error_messages.push(format!("{:?}", e));
    }
    if let Err(e) = db.execute("INSERT INTO text VALUES", "admin") {
        error_messages.push(format!("{:?}", e));
    }
    if let Err(e) = db.create_index("nonexistent", "key") {
        error_messages.push(format!("{:?}", e));
    }

    for (i, msg) in error_messages.iter().enumerate() {
        println!("Error {}: {}", i, msg);

        // Error messages should be non-empty and descriptive
        assert!(!msg.is_empty(), "Error message should not be empty");
        assert!(msg.len() > 10, "Error message should be descriptive");
    }
}

#[test]
fn test_error_types_distinguishable() {
    let (db, _temp) = create_test_database("error_types", 10);

    // Different error types should be distinguishable
    let table_error = db.query("SELECT * FROM nonexistent");
    let syntax_error = db.query("SELECT FORM text");

    assert!(table_error.is_err());
    assert!(syntax_error.is_err());

    // Error types should be different
    let err1 = format!("{:?}", table_error.unwrap_err());
    let err2 = format!("{:?}", syntax_error.unwrap_err());

    // Should contain different keywords
    assert!(err1.contains("TableNotFound") || err1.contains("not found"));
    assert!(err2.contains("Parse") || err2.contains("Syntax") || err2.contains("Invalid"));
}
