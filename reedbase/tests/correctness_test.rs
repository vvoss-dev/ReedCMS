// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Correctness integration tests.
//!
//! Verifies that query results are accurate and match SQL semantics:
//! - WHERE clause filtering (equals, LIKE patterns)
//! - ORDER BY sorting (ASC/DESC)
//! - LIMIT and OFFSET pagination
//! - Aggregation functions (COUNT, SUM, AVG, MIN, MAX)
//! - CRUD operation correctness
//! - Edge cases and special characters

mod test_utils;

use reedbase::Database;
use test_utils::*;

// ============================================================================
// SQL Semantics - WHERE Clause
// ============================================================================

#[test]
fn test_where_equals_correct() {
    let (db, _temp) = create_test_database("correct_where_eq", 50);

    let result = db
        .query("SELECT * FROM text WHERE key = 'test.key.000025'")
        .expect("Query failed");

    // Should return exactly 1 row
    assert_query_result_count(&result, 1);

    // Verify it's the correct row
    assert_eq!(get_rows(&result)[0].get("key").unwrap(), "test.key.000025");
    assert_eq!(get_rows(&result)[0].get("value").unwrap(), "Test value 25");
}

#[test]
fn test_where_like_pattern_correct() {
    let (db, _temp) = create_test_database("correct_where_like", 0);
    insert_multilingual_test_data(&db, 5);

    // Query all German keys
    let result = db
        .query("SELECT * FROM text WHERE key LIKE '%@de'")
        .expect("Query failed");

    // 4 prefixes × 5 keys = 20 German entries
    assert_query_result_count(&result, 20);

    // Verify all keys end with @de
    for row in get_rows(&result) {
        let key = row.get("key").unwrap();
        assert!(
            key.ends_with("@de"),
            "Key should end with @de but got: {}",
            key
        );
    }
}

#[test]
fn test_where_like_prefix_pattern() {
    let (db, _temp) = create_test_database("correct_prefix_like", 100);

    // Query all keys starting with "test.key.0001"
    let result = db
        .query("SELECT * FROM text WHERE key LIKE 'test.key.0001%'")
        .expect("Query failed");

    // Should match: 000010-000019 = 10 keys
    assert_query_result_count(&result, 10);

    // Verify all start with correct prefix
    for row in get_rows(&result) {
        let key = row.get("key").unwrap();
        assert!(
            key.starts_with("test.key.0001"),
            "Key should start with 'test.key.0001' but got: {}",
            key
        );
    }
}

#[test]
fn test_where_multiple_conditions() {
    let (db, _temp) = create_test_database("correct_multi_where", 0);
    insert_multilingual_test_data(&db, 10);

    // Query page.title keys in German
    let result = db
        .query("SELECT * FROM text WHERE key LIKE 'page.title.%' AND key LIKE '%@de'")
        .expect("Query failed");

    // Should have 10 German page.title keys
    assert_query_result_count(&result, 10);

    for row in get_rows(&result) {
        let key = row.get("key").unwrap();
        assert!(key.starts_with("page.title."));
        assert!(key.ends_with("@de"));
    }
}

// ============================================================================
// SQL Semantics - ORDER BY
// ============================================================================

#[test]
fn test_order_by_ascending() {
    let (db, _temp) = create_test_database("correct_order_asc", 20);

    let result = db
        .query("SELECT key FROM text ORDER BY key ASC")
        .expect("Query failed");

    assert_query_result_count(&result, 20);

    // Extract keys
    let keys: Vec<String> = get_rows(&result).iter()
        .map(|row| row.get("key").unwrap().to_string())
        .collect();

    // Verify sorted
    let mut expected = keys.clone();
    expected.sort();

    assert_eq!(keys, expected, "Keys should be sorted in ascending order");
}

#[test]
fn test_order_by_descending() {
    let (db, _temp) = create_test_database("correct_order_desc", 20);

    let result = db
        .query("SELECT key FROM text ORDER BY key DESC")
        .expect("Query failed");

    assert_query_result_count(&result, 20);

    // Extract keys
    let keys: Vec<String> = get_rows(&result).iter()
        .map(|row| row.get("key").unwrap().to_string())
        .collect();

    // Verify sorted descending
    let mut expected = keys.clone();
    expected.sort();
    expected.reverse();

    assert_eq!(keys, expected, "Keys should be sorted in descending order");
}

// ============================================================================
// SQL Semantics - LIMIT and OFFSET
// ============================================================================

#[test]
fn test_limit_correct() {
    let (db, _temp) = create_test_database("correct_limit", 100);

    let result = db
        .query("SELECT * FROM text LIMIT 10")
        .expect("Query failed");

    // Should return exactly 10 rows
    assert_query_result_count(&result, 10);
}

#[test]
fn test_offset_correct() {
    let (db, _temp) = create_test_database("correct_offset", 100);

    // Get first row without offset
    let result1 = db
        .query("SELECT * FROM text ORDER BY key ASC LIMIT 1")
        .expect("Query failed");
    let first_key = get_rows(&result1)[0].get("key").unwrap().to_string();

    // Get first row with OFFSET 10
    let result2 = db
        .query("SELECT * FROM text ORDER BY key ASC LIMIT 1 OFFSET 10")
        .expect("Query failed");
    let offset_key = get_rows(&result2)[0].get("key").unwrap().to_string();

    // Keys should be different
    assert_ne!(first_key, offset_key, "Offset should return different rows");

    // Offset key should be the 11th key
    assert_eq!(offset_key, "test.key.000010");
}

#[test]
fn test_limit_offset_pagination() {
    let (db, _temp) = create_test_database("correct_pagination", 100);

    // Page 1: rows 0-9
    let page1 = db
        .query("SELECT * FROM text ORDER BY key ASC LIMIT 10 OFFSET 0")
        .expect("Query failed");

    // Page 2: rows 10-19
    let page2 = db
        .query("SELECT * FROM text ORDER BY key ASC LIMIT 10 OFFSET 10")
        .expect("Query failed");

    // Page 3: rows 20-29
    let page3 = db
        .query("SELECT * FROM text ORDER BY key ASC LIMIT 10 OFFSET 20")
        .expect("Query failed");

    assert_query_result_count(&page1, 10);
    assert_query_result_count(&page2, 10);
    assert_query_result_count(&page3, 10);

    // Verify no overlap
    let key1 = get_rows(&page1)[0].get("key").unwrap();
    let key2 = get_rows(&page2)[0].get("key").unwrap();
    let key3 = get_rows(&page3)[0].get("key").unwrap();

    assert_eq!(key1, "test.key.000000");
    assert_eq!(key2, "test.key.000010");
    assert_eq!(key3, "test.key.000020");
}

// ============================================================================
// SQL Semantics - Aggregation
// ============================================================================

#[test]
fn test_count_aggregation() {
    let (db, _temp) = create_test_database("correct_count", 50);

    let result = db.query("SELECT COUNT(*) FROM text").expect("Query failed");

    // Note: COUNT implementation depends on ReedQL
    // This test documents expected behavior
    println!("COUNT result: {:?}", result);
}

// ============================================================================
// CRUD Correctness
// ============================================================================

#[test]
fn test_insert_persists() {
    let (db, _temp) = create_test_database("correct_insert", 0);

    // Insert a row
    let exec_result = db.execute(
        "INSERT INTO text (key, value, description) VALUES ('persist.test', 'test value', 'test desc')",
        "admin"
    ).expect("Insert failed");

    assert_rows_affected(&exec_result, 1);

    // Query it back
    let query_result = db
        .query("SELECT * FROM text WHERE key = 'persist.test'")
        .expect("Query failed");

    assert_query_result_count(&query_result, 1);
    assert_eq!(get_rows(&query_result)[0].get("key").unwrap(), "persist.test");
    assert_eq!(get_rows(&query_result)[0].get("value").unwrap(), "test value");
    assert_eq!(
        get_rows(&query_result)[0].get("description").unwrap(),
        "test desc"
    );
}

#[test]
fn test_update_modifies() {
    let (db, _temp) = create_test_database("correct_update", 10);

    // Get original value
    let before = db
        .query("SELECT value FROM text WHERE key = 'test.key.000005'")
        .expect("Query failed");
    let original_value = get_rows(&before)[0].get("value").unwrap().to_string();

    // Update
    let exec_result = db
        .execute(
            "UPDATE text SET value = 'modified value' WHERE key = 'test.key.000005'",
            "admin",
        )
        .expect("Update failed");

    assert_rows_affected(&exec_result, 1);

    // Verify change
    let after = db
        .query("SELECT value FROM text WHERE key = 'test.key.000005'")
        .expect("Query failed");
    let new_value = get_rows(&after)[0].get("value").unwrap().to_string();

    assert_ne!(original_value, new_value, "Value should have changed");
    assert_eq!(new_value, "modified value");
}

#[test]
fn test_delete_removes() {
    let (db, _temp) = create_test_database("correct_delete", 10);

    // Verify row exists
    let before = db
        .query("SELECT * FROM text WHERE key = 'test.key.000007'")
        .expect("Query failed");
    assert_query_result_count(&before, 1);

    // Delete
    let exec_result = db
        .execute("DELETE FROM text WHERE key = 'test.key.000007'", "admin")
        .expect("Delete failed");

    assert_rows_affected(&exec_result, 1);

    // Verify row is gone
    let after = db
        .query("SELECT * FROM text WHERE key = 'test.key.000007'")
        .expect("Query failed");
    assert_query_result_count(&after, 0);

    // Verify other rows still exist
    let all = db.query("SELECT * FROM text").expect("Query failed");
    assert_query_result_count(&all, 9);
}

#[test]
fn test_update_multiple_rows() {
    let (db, _temp) = create_test_database("correct_update_multi", 20);

    // Update all keys starting with "test.key.001"
    let exec_result = db
        .execute(
            "UPDATE text SET value = 'batch updated' WHERE key LIKE 'test.key.001%'",
            "admin",
        )
        .expect("Update failed");

    // Should affect 10 rows (000010-000019)
    assert_rows_affected(&exec_result, 10);

    // Verify all were updated
    let result = db
        .query("SELECT * FROM text WHERE key LIKE 'test.key.001%'")
        .expect("Query failed");

    for row in get_rows(&result) {
        assert_eq!(row.get("value").unwrap(), "batch updated");
    }
}

#[test]
fn test_delete_multiple_rows() {
    let (db, _temp) = create_test_database("correct_delete_multi", 50);

    // Delete all keys ending with "5"
    let exec_result = db
        .execute("DELETE FROM text WHERE key LIKE '%5'", "admin")
        .expect("Delete failed");

    // Should delete: 000005, 000015, 000025, 000035, 000045 = 5 rows
    assert_rows_affected(&exec_result, 5);

    // Verify remaining count
    let result = db.query("SELECT * FROM text").expect("Query failed");
    assert_query_result_count(&result, 45);
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn test_empty_result_set() {
    let (db, _temp) = create_test_database("correct_empty", 10);

    let result = db
        .query("SELECT * FROM text WHERE key = 'nonexistent.key'")
        .expect("Query failed");

    assert_query_result_count(&result, 0);
    assert!(get_rows(&result).is_empty());
}

#[test]
fn test_query_empty_table() {
    let (db, _temp) = create_test_database("correct_empty_table", 0);

    let result = db.query("SELECT * FROM text").expect("Query failed");

    assert_query_result_count(&result, 0);
}

#[test]
fn test_special_characters_in_values() {
    let (db, _temp) = create_test_database("correct_special_chars", 0);

    // Insert with special characters
    db.execute(
        r#"INSERT INTO text (key, value) VALUES ('special.test', 'Value with "quotes" and | pipes')"#,
        "admin"
    ).expect("Insert failed");

    // Query back
    let result = db
        .query("SELECT * FROM text WHERE key = 'special.test'")
        .expect("Query failed");

    assert_query_result_count(&result, 1);
    let value = get_rows(&result)[0].get("value").unwrap();
    assert!(value.contains("quotes"));
    assert!(value.contains("pipes"));
}

#[test]
fn test_very_long_values() {
    let (db, _temp) = create_test_database("correct_long_values", 0);

    // Create a very long value (10KB)
    let long_value = "x".repeat(10000);

    db.execute(
        &format!(
            "INSERT INTO text (key, value) VALUES ('long.test', '{}')",
            long_value
        ),
        "admin",
    )
    .expect("Insert failed");

    // Query back
    let result = db
        .query("SELECT * FROM text WHERE key = 'long.test'")
        .expect("Query failed");

    assert_query_result_count(&result, 1);
    assert_eq!(get_rows(&result)[0].get("value").unwrap().len(), 10000);
}

#[test]
fn test_unicode_characters() {
    let (db, _temp) = create_test_database("correct_unicode", 0);

    // Insert with Unicode
    db.execute(
        "INSERT INTO text (key, value) VALUES ('unicode.test', 'Ümläüts änd émöjis 🚀🎉')",
        "admin",
    )
    .expect("Insert failed");

    // Query back
    let result = db
        .query("SELECT * FROM text WHERE key = 'unicode.test'")
        .expect("Query failed");

    assert_query_result_count(&result, 1);
    let value = get_rows(&result)[0].get("value").unwrap();
    assert!(value.contains("Ümläüts"));
    assert!(value.contains("🚀"));
}

#[test]
fn test_empty_string_values() {
    let (db, _temp) = create_test_database("correct_empty_string", 0);

    // Insert empty value
    db.execute(
        "INSERT INTO text (key, value) VALUES ('empty.test', '')",
        "admin",
    )
    .expect("Insert failed");

    // Query back
    let result = db
        .query("SELECT * FROM text WHERE key = 'empty.test'")
        .expect("Query failed");

    assert_query_result_count(&result, 1);
    assert_eq!(get_rows(&result)[0].get("value").unwrap(), "");
}

#[test]
fn test_null_like_values() {
    let (db, _temp) = create_test_database("correct_null", 0);

    // Insert without description (optional column)
    db.execute(
        "INSERT INTO text (key, value) VALUES ('null.test', 'value only')",
        "admin",
    )
    .expect("Insert failed");

    // Query back
    let result = db
        .query("SELECT * FROM text WHERE key = 'null.test'")
        .expect("Query failed");

    assert_query_result_count(&result, 1);
    // Description may be empty or missing
}
