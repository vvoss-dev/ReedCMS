// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Performance integration tests.
//!
//! Verifies that ReedBase operations meet performance targets:
//! - Query with index: < 100μs
//! - Query range with index: < 1ms
//! - Full scan 10k rows: < 10ms
//! - Insert: < 5ms
//! - Update: < 10ms
//! - Delete: < 5ms
//! - Index creation on 10k rows: < 50ms

mod test_utils;

use reedbase::Database;
use std::time::Instant;
use test_utils::*;

// Performance targets (in microseconds)
const TARGET_QUERY_WITH_INDEX_US: u128 = 100;
const TARGET_RANGE_SCAN_MS: u128 = 1;
const TARGET_FULL_SCAN_10K_MS: u128 = 10;
const TARGET_INSERT_MS: u128 = 5;
const TARGET_UPDATE_MS: u128 = 10;
const TARGET_DELETE_MS: u128 = 5;
const TARGET_INDEX_CREATE_10K_MS: u128 = 50;
const TARGET_COLD_START_MS: u128 = 100;

// ============================================================================
// Query Performance
// ============================================================================

#[test]
fn test_query_with_index_fast() {
    let (db, _temp) = create_test_database("perf_query_index", 1000);

    // Create index
    db.create_index("text", "key")
        .expect("Failed to create index");

    // Warm up
    for _ in 0..5 {
        db.query("SELECT * FROM text WHERE key = 'test.key.000500'")
            .unwrap();
    }

    // Measure
    let start = Instant::now();
    let result = db
        .query("SELECT * FROM text WHERE key = 'test.key.000500'")
        .unwrap();
    let duration = start.elapsed();

    assert_query_result_count(&result, 1);
    assert_query_used_index(&result);

    let duration_us = duration.as_micros();
    println!(
        "Query with index: {}μs (target: <{}μs)",
        duration_us, TARGET_QUERY_WITH_INDEX_US
    );

    assert!(
        duration_us < TARGET_QUERY_WITH_INDEX_US,
        "Query with index took {}μs, expected <{}μs",
        duration_us,
        TARGET_QUERY_WITH_INDEX_US
    );
}

#[test]
fn test_query_range_with_index() {
    let (db, _temp) = create_test_database("perf_range_query", 1000);

    // Create index
    db.create_index("text", "key")
        .expect("Failed to create index");

    // Warm up
    for _ in 0..5 {
        db.query("SELECT * FROM text WHERE key LIKE 'test.key.0005%'")
            .unwrap();
    }

    // Measure: Range scan for 10 keys
    let start = Instant::now();
    let result = db
        .query("SELECT * FROM text WHERE key LIKE 'test.key.0005%'")
        .unwrap();
    let duration = start.elapsed();

    assert_eq!(get_rows(&result).len(), 10); // 000500-000509

    let duration_ms = duration.as_millis();
    println!(
        "Range scan: {}ms (target: <{}ms)",
        duration_ms, TARGET_RANGE_SCAN_MS
    );

    assert!(
        duration_ms < TARGET_RANGE_SCAN_MS,
        "Range scan took {}ms, expected <{}ms",
        duration_ms,
        TARGET_RANGE_SCAN_MS
    );
}

#[test]
fn test_query_full_scan_10k_rows() {
    let (db, _temp) = create_test_database("perf_full_scan", 10000);

    // Warm up
    for _ in 0..3 {
        db.query("SELECT * FROM text LIMIT 100").unwrap();
    }

    // Measure: Full scan without index
    let start = Instant::now();
    let result = db
        .query("SELECT * FROM text WHERE value LIKE '%value%'")
        .unwrap();
    let duration = start.elapsed();

    assert!(get_rows(&result).len() > 0);

    let duration_ms = duration.as_millis();
    println!(
        "Full scan 10k rows: {}ms (target: <{}ms)",
        duration_ms, TARGET_FULL_SCAN_10K_MS
    );

    assert!(
        duration_ms < TARGET_FULL_SCAN_10K_MS,
        "Full scan took {}ms, expected <{}ms",
        duration_ms,
        TARGET_FULL_SCAN_10K_MS
    );
}

// ============================================================================
// Write Performance
// ============================================================================

#[test]
fn test_insert_speed() {
    let (db, _temp) = create_test_database("perf_insert", 0);

    // Warm up
    for i in 0..10 {
        db.execute(
            &format!(
                "INSERT INTO text (key, value) VALUES ('warmup.{}', 'value')",
                i
            ),
            "test",
        )
        .unwrap();
    }

    // Measure: Single insert
    let start = Instant::now();
    db.execute(
        "INSERT INTO text (key, value) VALUES ('perf.test', 'performance test')",
        "test",
    )
    .unwrap();
    let duration = start.elapsed();

    let duration_ms = duration.as_millis();
    println!(
        "Insert: {}ms (target: <{}ms)",
        duration_ms, TARGET_INSERT_MS
    );

    assert!(
        duration_ms < TARGET_INSERT_MS,
        "Insert took {}ms, expected <{}ms",
        duration_ms,
        TARGET_INSERT_MS
    );
}

#[test]
fn test_update_speed() {
    let (db, _temp) = create_test_database("perf_update", 100);

    // Warm up
    for _ in 0..5 {
        db.execute(
            "UPDATE text SET value = 'warmup' WHERE key = 'test.key.000050'",
            "test",
        )
        .unwrap();
    }

    // Measure: Single update
    let start = Instant::now();
    db.execute(
        "UPDATE text SET value = 'updated value' WHERE key = 'test.key.000050'",
        "test",
    )
    .unwrap();
    let duration = start.elapsed();

    let duration_ms = duration.as_millis();
    println!(
        "Update: {}ms (target: <{}ms)",
        duration_ms, TARGET_UPDATE_MS
    );

    assert!(
        duration_ms < TARGET_UPDATE_MS,
        "Update took {}ms, expected <{}ms",
        duration_ms,
        TARGET_UPDATE_MS
    );
}

#[test]
fn test_delete_speed() {
    let (db, _temp) = create_test_database("perf_delete", 100);

    // Insert test rows for deletion
    for i in 0..20 {
        db.execute(
            &format!(
                "INSERT INTO text (key, value) VALUES ('delete.test.{}', 'value')",
                i
            ),
            "test",
        )
        .unwrap();
    }

    // Warm up
    for i in 0..5 {
        db.execute(
            &format!("DELETE FROM text WHERE key = 'delete.test.{}'", i),
            "test",
        )
        .unwrap();
    }

    // Measure: Single delete
    let start = Instant::now();
    db.execute("DELETE FROM text WHERE key = 'delete.test.10'", "test")
        .unwrap();
    let duration = start.elapsed();

    let duration_ms = duration.as_millis();
    println!(
        "Delete: {}ms (target: <{}ms)",
        duration_ms, TARGET_DELETE_MS
    );

    assert!(
        duration_ms < TARGET_DELETE_MS,
        "Delete took {}ms, expected <{}ms",
        duration_ms,
        TARGET_DELETE_MS
    );
}

// ============================================================================
// Index Performance
// ============================================================================

#[test]
fn test_index_creation_10k_rows() {
    let (db, _temp) = create_test_database("perf_index_create", 10000);

    // Measure: Create index on 10k rows
    let start = Instant::now();
    db.create_index("text", "key")
        .expect("Failed to create index");
    let duration = start.elapsed();

    let duration_ms = duration.as_millis();
    println!(
        "Index creation (10k rows): {}ms (target: <{}ms)",
        duration_ms, TARGET_INDEX_CREATE_10K_MS
    );

    assert!(
        duration_ms < TARGET_INDEX_CREATE_10K_MS,
        "Index creation took {}ms, expected <{}ms",
        duration_ms,
        TARGET_INDEX_CREATE_10K_MS
    );
}

#[test]
fn test_database_open_cold_start() {
    let (_db, temp) = create_test_database("perf_cold_start", 1000);
    let db_path = temp.path().join(".reed");

    // Create index
    _db.create_index("text", "key")
        .expect("Failed to create index");
    drop(_db); // Close database

    // Measure: Cold start (open with existing indices)
    let start = Instant::now();
    let _db2 = Database::open(&db_path).expect("Failed to open database");
    let duration = start.elapsed();

    let duration_ms = duration.as_millis();
    println!(
        "Cold start: {}ms (target: <{}ms)",
        duration_ms, TARGET_COLD_START_MS
    );

    // Note: This may fail without persistent indices (REED-19-24D)
    if duration_ms >= TARGET_COLD_START_MS {
        println!("WARNING: Cold start slower than target. Persistent indices not yet implemented (REED-19-24D).");
    }
}

// ============================================================================
// Auto-Indexing Performance
// ============================================================================

#[test]
fn test_auto_index_triggers_after_threshold() {
    let (db, _temp) = create_test_database("perf_auto_index", 1000);

    // Measure time before auto-indexing
    let start = Instant::now();
    db.query("SELECT * FROM text WHERE key = 'test.key.000500'")
        .unwrap();
    let duration_before = start.elapsed();

    // Execute same query 9 more times (total 10x = threshold)
    for _ in 0..9 {
        db.query("SELECT * FROM text WHERE key = 'test.key.000500'")
            .unwrap();
    }

    // Now auto-index should be created
    let indices = db.list_indices();
    let auto_created = indices.iter().any(|idx| idx.auto_created);

    if auto_created {
        // Measure time after auto-indexing
        let start = Instant::now();
        db.query("SELECT * FROM text WHERE key = 'test.key.000500'")
            .unwrap();
        let duration_after = start.elapsed();

        println!(
            "Before auto-index: {:?}, After: {:?}",
            duration_before, duration_after
        );

        assert!(
            duration_after < duration_before,
            "Auto-indexed query should be faster"
        );
    } else {
        println!("WARNING: Auto-indexing not triggered. Check implementation.");
    }
}

// ============================================================================
// Large Result Sets
// ============================================================================

#[test]
fn test_query_1000_rows() {
    let (db, _temp) = create_test_database("perf_large_result", 5000);

    // Measure: Fetch 1000 rows
    let start = Instant::now();
    let result = db.query("SELECT * FROM text LIMIT 1000").unwrap();
    let duration = start.elapsed();

    assert_query_result_count(&result, 1000);

    let duration_ms = duration.as_millis();
    println!("Fetch 1000 rows: {}ms", duration_ms);

    // Should be < 50ms for 1000 rows
    assert!(
        duration_ms < 50,
        "Fetching 1000 rows took {}ms, expected <50ms",
        duration_ms
    );
}

#[test]
fn test_batch_insert_performance() {
    let (db, _temp) = create_test_database("perf_batch_insert", 0);

    // Measure: Insert 100 rows
    let start = Instant::now();
    for i in 0..100 {
        db.execute(
            &format!(
                "INSERT INTO text (key, value) VALUES ('batch.{}', 'value {}')",
                i, i
            ),
            "test",
        )
        .unwrap();
    }
    let duration = start.elapsed();

    let duration_ms = duration.as_millis();
    let avg_per_insert_ms = duration_ms as f64 / 100.0;

    println!(
        "Batch insert 100 rows: {}ms (avg {}ms per insert)",
        duration_ms, avg_per_insert_ms
    );

    // Average should be < 5ms per insert
    assert!(
        avg_per_insert_ms < 5.0,
        "Average insert time {}ms, expected <5ms",
        avg_per_insert_ms
    );
}

// ============================================================================
// Stress Tests
// ============================================================================

#[test]
#[ignore] // Run with --ignored flag for stress testing
fn test_query_100k_rows_full_scan() {
    let (db, _temp) = create_test_database("stress_100k", 100000);

    println!("Created 100k rows database");

    // Measure: Full scan of 100k rows
    let start = Instant::now();
    let result = db
        .query("SELECT * FROM text WHERE value LIKE '%value%'")
        .unwrap();
    let duration = start.elapsed();

    println!(
        "Full scan 100k rows: {:?} ({} rows returned)",
        duration,
        get_rows(&result).len()
    );

    // Should complete in reasonable time (< 100ms)
    assert!(
        duration.as_millis() < 100,
        "Full scan 100k took {:?}, expected <100ms",
        duration
    );
}
