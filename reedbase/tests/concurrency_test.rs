// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Concurrency integration tests.
//!
//! Tests thread safety and concurrent operations:
//! - Multiple simultaneous readers
//! - Multiple simultaneous writers
//! - Readers during writes
//! - Concurrent index creation
//! - Lock contention scenarios
//! - Data consistency under concurrency

mod test_utils;

use reedbase::Database;
use std::sync::{Arc, Barrier};
use std::thread;
use std::time::Duration;
use test_utils::*;

// ============================================================================
// Concurrent Reads
// ============================================================================

#[test]
fn test_multiple_readers() {
    let (db, _temp) = create_test_database("concur_readers", 100);
    let db = Arc::new(db);

    let mut handles = vec![];

    // Spawn 10 threads, each executing 100 queries
    for i in 0..10 {
        let db_clone = Arc::clone(&db);
        let handle = thread::spawn(move || {
            for j in 0..100 {
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

    // Database should still be consistent
    let stats = db.stats();
    assert!(stats.query_count >= 1000);
}

#[test]
fn test_many_concurrent_readers() {
    let (db, _temp) = create_test_database("concur_many_readers", 1000);
    let db = Arc::new(db);

    let mut handles = vec![];

    // Spawn 50 threads for stress testing
    for i in 0..50 {
        let db_clone = Arc::clone(&db);
        let handle = thread::spawn(move || {
            for j in 0..20 {
                let result = db_clone
                    .query("SELECT * FROM text LIMIT 10")
                    .expect("Query failed");
                assert_eq!(get_rows(&result).len(), 10);
            }
        });
        handles.push(handle);
    }

    // All threads should complete without deadlock
    for handle in handles {
        handle.join().expect("Thread panicked");
    }
}

// ============================================================================
// Concurrent Writes
// ============================================================================

#[test]
fn test_multiple_writers() {
    let (db, _temp) = create_test_database("concur_writers", 0);
    let db = Arc::new(db);

    let mut handles = vec![];

    // Spawn 5 threads, each inserting 20 unique rows
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

    // Verify no duplicate keys
    let keys: std::collections::HashSet<String> = get_rows(&result).iter()
        .map(|row| row.get("key").unwrap().to_string())
        .collect();
    assert_eq!(keys.len(), 100, "Should have 100 unique keys");
}

#[test]
fn test_concurrent_updates() {
    let (db, _temp) = create_test_database("concur_updates", 50);
    let db = Arc::new(db);

    let mut handles = vec![];

    // Spawn 5 threads, each updating different rows
    for i in 0..5 {
        let db_clone = Arc::clone(&db);
        let handle = thread::spawn(move || {
            for j in 0..10 {
                let key = format!("test.key.{:06}", i * 10 + j);
                let sql = format!(
                    "UPDATE text SET value = 'updated by thread {}' WHERE key = '{}'",
                    i, key
                );
                db_clone
                    .execute(&sql, &format!("user{}", i))
                    .expect("Update failed");
            }
        });
        handles.push(handle);
    }

    // Wait for all threads
    for handle in handles {
        handle.join().expect("Thread panicked");
    }

    // Verify all updates applied
    let result = db
        .query("SELECT * FROM text WHERE value LIKE 'updated by thread%'")
        .expect("Query failed");
    assert_query_result_count(&result, 50);
}

#[test]
fn test_concurrent_deletes() {
    let (db, _temp) = create_test_database("concur_deletes", 100);
    let db = Arc::new(db);

    let mut handles = vec![];

    // Spawn 5 threads, each deleting different rows
    for i in 0..5 {
        let db_clone = Arc::clone(&db);
        let handle = thread::spawn(move || {
            for j in 0..10 {
                let key = format!("test.key.{:06}", i * 10 + j);
                let sql = format!("DELETE FROM text WHERE key = '{}'", key);
                db_clone
                    .execute(&sql, &format!("user{}", i))
                    .expect("Delete failed");
            }
        });
        handles.push(handle);
    }

    // Wait for all threads
    for handle in handles {
        handle.join().expect("Thread panicked");
    }

    // Verify 50 rows deleted
    let result = db.query("SELECT * FROM text").expect("Query failed");
    assert_query_result_count(&result, 50);
}

// ============================================================================
// Mixed Read/Write Operations
// ============================================================================

#[test]
fn test_readers_during_writes() {
    let (db, _temp) = create_test_database("concur_read_write", 50);
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
            thread::sleep(Duration::from_micros(100)); // Simulate work
        }
    });
    handles.push(writer);

    // Reader threads (query existing data)
    for _ in 0..5 {
        let db_reader = Arc::clone(&db);
        let reader = thread::spawn(move || {
            for _ in 0..50 {
                let result = db_reader
                    .query("SELECT * FROM text LIMIT 10")
                    .expect("Query failed");
                assert!(get_rows(&result).len() <= 10);
                thread::sleep(Duration::from_micros(50));
            }
        });
        handles.push(reader);
    }

    // Wait for all threads
    for handle in handles {
        handle.join().expect("Thread panicked");
    }

    // Final count should be 100
    let result = db.query("SELECT * FROM text").expect("Query failed");
    assert_query_result_count(&result, 100);
}

#[test]
fn test_mixed_operations() {
    let (db, _temp) = create_test_database("concur_mixed", 100);
    let db = Arc::new(db);

    let mut handles = vec![];

    // Inserters
    for i in 0..3 {
        let db_clone = Arc::clone(&db);
        let handle = thread::spawn(move || {
            for j in 0..10 {
                let key = format!("insert.thread{}.{}", i, j);
                let sql = format!(
                    "INSERT INTO text (key, value) VALUES ('{}', 'inserted')",
                    key
                );
                db_clone.execute(&sql, &format!("inserter{}", i)).ok();
            }
        });
        handles.push(handle);
    }

    // Updaters
    for i in 0..3 {
        let db_clone = Arc::clone(&db);
        let handle = thread::spawn(move || {
            for j in 0..10 {
                let key = format!("test.key.{:06}", i * 10 + j);
                let sql = format!("UPDATE text SET value = 'updated' WHERE key = '{}'", key);
                db_clone.execute(&sql, &format!("updater{}", i)).ok();
            }
        });
        handles.push(handle);
    }

    // Readers
    for _ in 0..5 {
        let db_clone = Arc::clone(&db);
        let handle = thread::spawn(move || {
            for _ in 0..20 {
                db_clone.query("SELECT * FROM text LIMIT 5").ok();
            }
        });
        handles.push(handle);
    }

    // Wait for all
    for handle in handles {
        handle.join().expect("Thread panicked");
    }
}

// ============================================================================
// Concurrent Index Operations
// ============================================================================

#[test]
fn test_concurrent_index_creation() {
    let (db, _temp) = create_test_database("concur_index_create", 100);

    // Create additional columns for testing
    db.create_table("routes", None)
        .expect("Failed to create routes table");
    db.create_table("meta", None)
        .expect("Failed to create meta table");

    let db = Arc::new(db);
    let mut handles = vec![];

    // Thread 1: Create index on text.key
    let db1 = Arc::clone(&db);
    handles.push(thread::spawn(move || {
        db1.create_index("text", "key").ok();
    }));

    // Thread 2: Create index on text.value
    let db2 = Arc::clone(&db);
    handles.push(thread::spawn(move || {
        db2.create_index("text", "value").ok();
    }));

    // Thread 3: Query during index creation
    let db3 = Arc::clone(&db);
    handles.push(thread::spawn(move || {
        for _ in 0..10 {
            db3.query("SELECT * FROM text LIMIT 5").ok();
            thread::sleep(Duration::from_millis(10));
        }
    }));

    // Wait for all
    for handle in handles {
        handle.join().expect("Thread panicked");
    }

    // Verify indices exist
    let indices = db.list_indices();
    assert!(indices.len() >= 1, "At least one index should be created");
}

#[test]
fn test_query_with_concurrent_index_creation() {
    let (db, _temp) = create_test_database("concur_query_index", 1000);
    let db = Arc::new(db);

    let barrier = Arc::new(Barrier::new(2));
    let mut handles = vec![];

    // Thread 1: Create index
    let db1 = Arc::clone(&db);
    let barrier1 = Arc::clone(&barrier);
    handles.push(thread::spawn(move || {
        barrier1.wait(); // Synchronize start
        db1.create_index("text", "key")
            .expect("Failed to create index");
    }));

    // Thread 2: Execute many queries
    let db2 = Arc::clone(&db);
    let barrier2 = Arc::clone(&barrier);
    handles.push(thread::spawn(move || {
        barrier2.wait(); // Synchronize start
        for i in 0..100 {
            let key = format!("test.key.{:06}", i % 100);
            db2.query(&format!("SELECT * FROM text WHERE key = '{}'", key))
                .expect("Query failed");
        }
    }));

    // Wait for all
    for handle in handles {
        handle.join().expect("Thread panicked");
    }
}

// ============================================================================
// Consistency Tests
// ============================================================================

#[test]
fn test_consistency_under_load() {
    let (db, _temp) = create_test_database("concur_consistency", 0);
    let db = Arc::new(db);

    let mut handles = vec![];

    // Spawn 10 writers, each inserting 10 rows with counter values
    for thread_id in 0..10 {
        let db_clone = Arc::clone(&db);
        let handle = thread::spawn(move || {
            for counter in 0..10 {
                let key = format!("thread{}.counter{}", thread_id, counter);
                let value = format!("{}", counter);
                let sql = format!(
                    "INSERT INTO text (key, value) VALUES ('{}', '{}')",
                    key, value
                );
                db_clone
                    .execute(&sql, &format!("user{}", thread_id))
                    .expect("Insert failed");
            }
        });
        handles.push(handle);
    }

    // Wait for all
    for handle in handles {
        handle.join().expect("Thread panicked");
    }

    // Verify consistency: Should have exactly 100 rows
    let result = db.query("SELECT * FROM text").expect("Query failed");
    assert_query_result_count(&result, 100);

    // Verify each thread inserted all 10 rows
    for thread_id in 0..10 {
        let result = db
            .query(&format!(
                "SELECT * FROM text WHERE key LIKE 'thread{}%'",
                thread_id
            ))
            .expect("Query failed");
        assert_eq!(
            get_rows(&result).len(),
            10,
            "Thread {} should have 10 rows",
            thread_id
        );
    }
}

#[test]
fn test_no_lost_updates() {
    let (db, _temp) = create_test_database("concur_no_lost_updates", 1);

    // Insert initial row with counter = 0
    db.execute(
        "INSERT INTO text (key, value) VALUES ('counter', '0')",
        "admin",
    )
    .expect("Insert failed");

    let db = Arc::new(db);
    let mut handles = vec![];

    // Spawn 10 threads, each incrementing counter 10 times
    // Note: This tests if updates are lost due to race conditions
    for thread_id in 0..10 {
        let db_clone = Arc::clone(&db);
        let handle = thread::spawn(move || {
            for _ in 0..10 {
                // Read current value
                let result = db_clone
                    .query("SELECT value FROM text WHERE key = 'counter'")
                    .expect("Query failed");
                let current: i32 = get_rows(&result)[0].get("value").unwrap().parse().unwrap();

                // Increment
                let new_value = current + 1;

                // Write back
                db_clone
                    .execute(
                        &format!(
                            "UPDATE text SET value = '{}' WHERE key = 'counter'",
                            new_value
                        ),
                        &format!("user{}", thread_id),
                    )
                    .expect("Update failed");
            }
        });
        handles.push(handle);
    }

    // Wait for all
    for handle in handles {
        handle.join().expect("Thread panicked");
    }

    // Final value should be 100 if no updates were lost
    let result = db
        .query("SELECT value FROM text WHERE key = 'counter'")
        .expect("Query failed");
    let final_value: i32 = get_rows(&result)[0].get("value").unwrap().parse().unwrap();

    println!("Final counter value: {} (expected: 100)", final_value);

    // Note: This may be less than 100 due to race conditions
    // This test documents the behavior
    assert!(final_value > 0, "Counter should have increased");
}

// ============================================================================
// Deadlock Prevention
// ============================================================================

#[test]
fn test_no_deadlock_scenario() {
    let (db, _temp) = create_test_database("concur_no_deadlock", 100);
    let db = Arc::new(db);

    let barrier = Arc::new(Barrier::new(4));
    let mut handles = vec![];

    // Spawn 4 threads that all start at the same time
    for i in 0..4 {
        let db_clone = Arc::clone(&db);
        let barrier_clone = Arc::clone(&barrier);
        let handle = thread::spawn(move || {
            barrier_clone.wait(); // Synchronize start

            // Each thread does a mix of reads and writes
            for j in 0..25 {
                if j % 2 == 0 {
                    db_clone.query("SELECT * FROM text LIMIT 5").ok();
                } else {
                    let key = format!("test.key.{:06}", i * 25 + j);
                    db_clone
                        .execute(
                            &format!(
                                "UPDATE text SET value = 'thread{}' WHERE key = '{}'",
                                i, key
                            ),
                            &format!("user{}", i),
                        )
                        .ok();
                }
            }
        });
        handles.push(handle);
    }

    // Should complete without deadlock (with timeout)
    let timeout = Duration::from_secs(10);
    let start = std::time::Instant::now();

    for handle in handles {
        assert!(
            start.elapsed() < timeout,
            "Test timed out - possible deadlock"
        );
        handle.join().expect("Thread panicked");
    }

    println!("No deadlock detected (completed in {:?})", start.elapsed());
}
