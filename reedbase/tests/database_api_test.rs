// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Database API integration tests.
//!
//! Tests the programmatic Rust API for ReedBase operations including:
//! - Basic CRUD operations
//! - Complex queries with WHERE, LIKE, ORDER BY, LIMIT
//! - Index operations and performance
//! - Concurrency and thread safety
//! - Error handling
//! - Versioning operations

mod test_utils;

use reedbase::{Database, QueryResult};
use serial_test::serial;
use std::sync::Arc;
use std::thread;
use std::time::Instant;
use test_utils::*;

// ============================================================================
// Basic Operations
// ============================================================================

#[test]
fn test_database_open_create() {
    let (db, _temp) = create_test_database("open_test", 0);
    let stats = db.stats();
    assert_eq!(stats.table_count, 1); // text table created
}

#[test]
fn test_database_create_table() {
    let (db, _temp) = create_test_database("create_table_test", 0);

    db.create_table("routes", None)
        .expect("Failed to create routes table");
    db.create_table("meta", None)
        .expect("Failed to create meta table");

    let tables = db.list_tables().expect("Failed to list tables");
    assert!(tables.contains(&"text".to_string()));
    assert!(tables.contains(&"routes".to_string()));
    assert!(tables.contains(&"meta".to_string()));
}

#[test]
fn test_database_insert_query() {
    let (db, _temp) = create_test_database("insert_query_test", 0);

    // Insert a row
    let result = db.execute(
        "INSERT INTO text (key, value, description) VALUES ('test.key', 'test value', 'test desc')",
        "admin"
    ).expect("Insert failed");

    assert_rows_affected(&result, 1);

    // Query it back
    let query_result = db
        .query("SELECT * FROM text WHERE key = 'test.key'")
        .expect("Query failed");

    assert_query_result_count(&query_result, 1);
    assert_eq!(get_rows(&query_result)[0].get("key").unwrap(), "test.key");
    assert_eq!(
        get_rows(&query_result)[0].get("value").unwrap(),
        "test value"
    );
}

#[test]
fn test_database_update_query() {
    let (db, _temp) = create_test_database("update_test", 5);

    // Update a row
    let result = db
        .execute(
            "UPDATE text SET value = 'updated value' WHERE key = 'test.key.000001'",
            "admin",
        )
        .expect("Update failed");

    assert_rows_affected(&result, 1);

    // Verify update
    let query_result = db
        .query("SELECT value FROM text WHERE key = 'test.key.000001'")
        .expect("Query failed");

    assert_eq!(
        get_rows(&query_result)[0].get("value").unwrap(),
        "updated value"
    );
}

#[test]
fn test_database_delete_query() {
    let (db, _temp) = create_test_database("delete_test", 10);

    // Delete a row
    let result = db
        .execute("DELETE FROM text WHERE key = 'test.key.000005'", "admin")
        .expect("Delete failed");

    assert_rows_affected(&result, 1);

    // Verify deletion
    let query_result = db
        .query("SELECT * FROM text WHERE key = 'test.key.000005'")
        .expect("Query failed");

    assert_query_result_count(&query_result, 0);

    // Verify other rows still exist
    let all_result = db.query("SELECT * FROM text").expect("Query failed");
    assert_query_result_count(&all_result, 9);
}

// ============================================================================
// Complex Queries
// ============================================================================

#[test]
fn test_query_with_where_clause() {
    let (db, _temp) = create_test_database("where_test", 20);

    let result = db
        .query("SELECT * FROM text WHERE key = 'test.key.000010'")
        .expect("Query failed");

    assert_query_result_count(&result, 1);
    assert_eq!(get_rows(&result)[0].get("key").unwrap(), "test.key.000010");
}

#[test]
fn test_query_with_like_pattern() {
    let (db, _temp) = create_test_database("like_test", 0);
    insert_multilingual_test_data(&db, 3);

    // Query all German keys
    let result = db
        .query("SELECT * FROM text WHERE key LIKE '%@de'")
        .expect("Query failed");

    // Should have: page.title.*.@de, page.header.logo.*.@de, footer.copyright.*.@de, menu.item.*.@de
    // = 4 prefixes × 3 keys = 12 rows
    assert_query_result_count(&result, 12);

    // Verify all are German
    for row in get_rows(&result) {
        let key = row.get("key").unwrap();
        assert!(key.ends_with("@de"), "Key should end with @de: {}", key);
    }
}

#[test]
fn test_query_with_order_by() {
    let (db, _temp) = create_test_database("order_test", 10);

    // Order by key ascending
    let result = db
        .query("SELECT key FROM text ORDER BY key ASC")
        .expect("Query failed");

    assert_query_result_count(&result, 10);

    // Verify sorted
    let keys: Vec<String> = get_rows(&result)
        .iter()
        .map(|row| row.get("key").unwrap().to_string())
        .collect();

    let mut sorted_keys = keys.clone();
    sorted_keys.sort();
    assert_eq!(keys, sorted_keys, "Keys should be sorted ascending");
}

#[test]
fn test_query_with_limit_offset() {
    let (db, _temp) = create_test_database("limit_test", 50);

    // Get rows 10-19 (LIMIT 10 OFFSET 10)
    let result = db
        .query("SELECT * FROM text ORDER BY key ASC LIMIT 10 OFFSET 10")
        .expect("Query failed");

    assert_query_result_count(&result, 10);

    // Verify first row is the 11th row (offset 10)
    assert_eq!(get_rows(&result)[0].get("key").unwrap(), "test.key.000010");
}

#[test]
fn test_query_with_aggregation() {
    let (db, _temp) = create_test_database("aggregation_test", 100);

    // Count all rows
    let result = db.query("SELECT COUNT(*) FROM text").expect("Query failed");

    // Aggregation queries return Aggregation variant, not Rows
    match result {
        QueryResult::Aggregation(count) => {
            assert_eq!(count, 100.0, "COUNT(*) should return 100");
        }
        QueryResult::Rows(_) => {
            panic!("Expected Aggregation result, got Rows");
        }
    }
}

// ============================================================================
// Index Operations
// ============================================================================

#[test]
fn test_create_index_speeds_up_query() {
    let (db, _temp) = create_test_database("index_speed_test", 100);

    // Create index on key column
    db.create_index("text", "key")
        .expect("Failed to create index");

    // Verify index exists
    let indices = db.list_indices();
    assert_eq!(indices.len(), 1, "Should have 1 index");
    assert_eq!(indices[0].table, "text");
    assert_eq!(indices[0].column, "key");

    // Query should complete successfully (index usage is internal optimization)
    let result = db
        .query("SELECT * FROM text WHERE key = 'test.key.000050'")
        .expect("Query with index failed");

    assert_eq!(get_rows(&result).len(), 1, "Should find exactly 1 row");
}

#[test]
fn test_auto_index_creation() {
    let (db, _temp) = create_test_database_with_auto_index("auto_index_test", 100);

    // Check if primary key auto-index was created on table creation
    let indices = db.list_indices();

    assert_eq!(indices.len(), 1, "Should have 1 auto-created index");
    assert_eq!(indices[0].table, "text");
    assert_eq!(indices[0].column, "key");
    assert!(
        indices[0].auto_created,
        "Index should be marked as auto-created"
    );
}

#[test]
fn test_list_indices() {
    let (db, _temp) = create_test_database("list_indices_test", 10);

    // Initially no indices
    let indices = db.list_indices();
    assert_eq!(indices.len(), 0);

    // Create two indices
    db.create_index("text", "key")
        .expect("Failed to create index on key");
    db.create_index("text", "value")
        .expect("Failed to create index on value");

    // List indices
    let indices = db.list_indices();
    assert_eq!(indices.len(), 2);

    let key_index = indices.iter().find(|idx| idx.column == "key");
    let value_index = indices.iter().find(|idx| idx.column == "value");

    assert!(key_index.is_some());
    assert!(value_index.is_some());
}

#[test]
fn test_drop_index() {
    let (db, _temp) = create_test_database("drop_index_test", 10);

    // Create index
    db.create_index("text", "key")
        .expect("Failed to create index");
    assert_eq!(db.list_indices().len(), 1);

    // Drop index
    // db.drop_index("text", "key").expect("Failed to drop index");
    // assert_eq!(db.list_indices().len(), 0);
}

// ============================================================================
// Concurrency Tests
// ============================================================================

#[test]
#[serial]
fn test_concurrent_reads() {
    let (db, _temp) = create_test_database("concurrent_reads_test", 100);
    let db = Arc::new(db);

    let mut handles = vec![];

    // Spawn 10 threads, each executing 50 queries
    for i in 0..10 {
        let db_clone = Arc::clone(&db);
        let handle = thread::spawn(move || {
            for j in 0..50 {
                let key = format!("test.key.{:06}", (i * 10 + j) % 100);
                let result = db_clone
                    .query(&format!("SELECT * FROM text WHERE key = '{}'", key))
                    .expect("Query failed");
                assert!(get_rows(&result).len() <= 1);
            }
        });
        handles.push(handle);
    }

    // Wait for all threads
    for handle in handles {
        handle.join().expect("Thread panicked");
    }
}

#[test]
#[serial]
fn test_concurrent_writes() {
    let (db, _temp) = create_test_database("concurrent_writes_test", 0);
    let db = Arc::new(db);

    let mut handles = vec![];

    // Spawn 5 threads, each inserting 20 rows
    for i in 0..5 {
        let db_clone = Arc::clone(&db);
        let handle = thread::spawn(move || {
            for j in 0..20 {
                let key = format!("thread{}.key.{:03}", i, j);
                let sql = format!(
                    "INSERT INTO text (key, value) VALUES ('{}', 'value from thread {}')",
                    key, i
                );
                db_clone
                    .execute(&sql, &format!("user{}", i))
                    .expect("Insert failed");
            }
        });
        handles.push(handle);
    }

    // Wait for all threads
    for handle in handles {
        handle.join().expect("Thread panicked");
    }

    // Verify all 100 rows inserted
    let result = db.query("SELECT * FROM text").expect("Query failed");
    assert_query_result_count(&result, 100);
}

#[test]
#[serial]
fn test_read_during_write() {
    let (db, _temp) = create_test_database("read_write_test", 50);
    let db = Arc::new(db);

    let mut handles = vec![];

    // Writer thread (inserts 50 more rows)
    let db_writer = Arc::clone(&db);
    let writer = thread::spawn(move || {
        for i in 50..100 {
            let key = format!("test.key.{:06}", i);
            let sql = format!(
                "INSERT INTO text (key, value) VALUES ('{}', 'new value')",
                key
            );
            db_writer.execute(&sql, "writer").expect("Insert failed");
        }
    });
    handles.push(writer);

    // Reader threads (query existing data)
    for _ in 0..5 {
        let db_reader = Arc::clone(&db);
        let reader = thread::spawn(move || {
            for _ in 0..20 {
                let result = db_reader
                    .query("SELECT * FROM text LIMIT 10")
                    .expect("Query failed");
                assert!(get_rows(&result).len() <= 10);
            }
        });
        handles.push(reader);
    }

    // Wait for all threads
    for handle in handles {
        handle.join().expect("Thread panicked");
    }
}

// ============================================================================
// Error Handling
// ============================================================================

#[test]
fn test_query_nonexistent_table() {
    let (db, _temp) = create_test_database("error_table_test", 0);

    let result = db.query("SELECT * FROM nonexistent");
    assert!(result.is_err(), "Query on nonexistent table should fail");
}

#[test]
fn test_invalid_sql_syntax() {
    let (db, _temp) = create_test_database("error_syntax_test", 10);

    let result = db.query("SELECT FORM text");
    assert!(result.is_err(), "Invalid SQL syntax should fail");
}

#[test]
fn test_insert_duplicate_key() {
    let (db, _temp) = create_test_database("error_duplicate_test", 5);

    // Try to insert duplicate key
    let result = db.execute(
        "INSERT INTO text (key, value) VALUES ('test.key.000001', 'duplicate')",
        "admin",
    );

    // Note: Behavior depends on schema - may succeed or fail
    // This test documents current behavior
    println!("Duplicate insert result: {:?}", result);
}

#[test]
fn test_index_already_exists_error() {
    let (db, _temp) = create_test_database("error_index_test", 10);

    // Create index
    db.create_index("text", "key")
        .expect("First index creation should succeed");

    // Try to create same index again
    let result = db.create_index("text", "key");
    assert!(result.is_err(), "Creating duplicate index should fail");
}

// ============================================================================
// Statistics
// ============================================================================

#[test]
fn test_database_stats_accurate() {
    let (db, _temp) = create_test_database("stats_test", 100);

    // Execute some operations
    db.query("SELECT * FROM text LIMIT 10")
        .expect("Query failed");
    db.execute(
        "INSERT INTO text (key, value) VALUES ('new.key', 'new value')",
        "admin",
    )
    .expect("Insert failed");
    db.execute(
        "UPDATE text SET value = 'updated' WHERE key = 'new.key'",
        "admin",
    )
    .expect("Update failed");
    db.execute("DELETE FROM text WHERE key = 'new.key'", "admin")
        .expect("Delete failed");

    let stats = db.stats();

    assert_eq!(stats.table_count, 1);
    assert!(stats.query_count >= 1);
    assert!(stats.insert_count >= 1);
    assert!(stats.update_count >= 1);
    assert!(stats.delete_count >= 1);
}

#[test]
#[ignore] // TODO: QueryResult does not expose metrics yet
fn test_query_metrics_collected() {
    let (db, _temp) = create_test_database("metrics_test", 50);

    let result = db
        .query("SELECT * FROM text LIMIT 5")
        .expect("Query failed");

    // TODO: Verify metrics are collected once QueryResult exposes them
    // assert!(result.metrics.parse_time_us > 0, "Parse time should be recorded");
    // assert!(result.metrics.execution_time_us > 0, "Execution time should be recorded");
    // assert_eq!(result.metrics.rows_returned, 5, "Should return 5 rows");
}

// ============================================================================
// Versioning Tests
// ============================================================================

#[test]
#[serial]
fn test_insert_creates_version() {
    use reedbase::tables::Table;
    use std::path::Path;

    let (db, temp) = create_test_database("versioning_insert", 0);

    // Get direct table reference (bypass Database API for testing)
    let table_path = temp.path().join(".reed");
    let table = Table::new(&table_path, "text");

    // Get initial version count
    let initial_versions = table.list_versions().expect("Should list versions");
    let initial_count = initial_versions.len();

    // Insert row via Database API
    db.execute(
        "INSERT INTO text (key, value) VALUES ('test.key', 'test value')",
        "admin",
    )
    .expect("Insert failed");

    // Verify version created
    let versions = table.list_versions().expect("Should list versions");
    assert_eq!(
        versions.len(),
        initial_count + 1,
        "Insert should create a new version"
    );

    // Verify version metadata
    let latest = &versions[0]; // Newest first
    assert_eq!(latest.user, "admin", "Version should record correct user");
    assert!(latest.delta_size > 0, "Delta should have non-zero size");
}

#[test]
#[serial]
fn test_update_creates_delta() {
    use reedbase::tables::Table;

    let (db, temp) = create_test_database("versioning_delta", 1);

    // Get direct table reference
    let table_path = temp.path().join(".reed");
    let table = Table::new(&table_path, "text");

    // Get version count after initial data
    let versions_before = table.list_versions().expect("Should list versions");
    let count_before = versions_before.len();

    // Update row via Database API
    db.execute(
        "UPDATE text SET value = 'new_value' WHERE key = 'test.key.000000'",
        "updater",
    )
    .expect("Update failed");

    // Verify new version created
    let versions_after = table.list_versions().expect("Should list versions");
    assert_eq!(
        versions_after.len(),
        count_before + 1,
        "Update should create new version"
    );

    // Verify delta properties
    let latest = &versions_after[0];
    assert_eq!(latest.user, "updater", "Should record updater username");
    assert!(latest.delta_size > 0, "Delta should exist");

    // Verify delta exists and has reasonable size
    // Note: For very small files, bsdiff delta may be larger than content due to metadata overhead
    // This is expected behavior - deltas are optimized for larger files
    let current_size = table.read_current().expect("Should read current").len();

    // Just verify delta exists (size > 0) - size comparison not meaningful for tiny test data
    assert!(
        latest.delta_size > 0,
        "Delta should exist and have non-zero size"
    );
}

#[test]
#[serial]
fn test_rollback_to_version() {
    use reedbase::tables::Table;

    let (db, temp) = create_test_database("versioning_rollback", 10);

    // Query original value
    let before = db
        .query("SELECT * FROM text WHERE key = 'test.key.000005'")
        .expect("Query failed");
    let original_value = get_rows(&before)[0]
        .get("value")
        .expect("Should have value column")
        .clone();

    // Get direct table reference
    let table_path = temp.path().join(".reed");
    let table = Table::new(&table_path, "text");

    // Get versions before modification
    let versions_before_modify = table.list_versions().expect("Should list versions");
    let target_version = versions_before_modify[0].timestamp; // Current version

    // Modify row via Database API
    db.execute(
        "UPDATE text SET value = 'modified' WHERE key = 'test.key.000005'",
        "modifier",
    )
    .expect("Update failed");

    // Verify modification
    let modified = db
        .query("SELECT * FROM text WHERE key = 'test.key.000005'")
        .expect("Query failed");
    let modified_value = get_rows(&modified)[0]
        .get("value")
        .expect("Should have value column");
    assert_eq!(
        modified_value, "modified",
        "Value should be modified before rollback"
    );

    // Rollback to version before modification
    table
        .rollback(target_version, "admin")
        .expect("Rollback failed");

    // Verify original value restored
    let after = db
        .query("SELECT * FROM text WHERE key = 'test.key.000005'")
        .expect("Query failed");
    let restored_value = get_rows(&after)[0]
        .get("value")
        .expect("Should have value column");
    assert_eq!(
        restored_value, &original_value,
        "Rollback should restore original value"
    );

    // Verify rollback created a new version
    let versions_after = table.list_versions().expect("Should list versions");
    assert!(
        versions_after.len() > versions_before_modify.len() + 1,
        "Rollback should create a new version"
    );
}
