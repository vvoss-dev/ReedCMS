// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Concurrent operation benchmarks.
//!
//! Measures performance of:
//! - Concurrent reads
//! - Lock acquisition/contention
//! - Merge operations
//! - Conflict detection
//!
//! ## Performance Targets
//! - Concurrent reads (10 threads): < 2x single-thread time
//! - Lock acquisition: < 1ms uncontended
//! - Auto-merge (non-conflicting): < 5ms
//! - Conflict detection: < 10ms

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use reedbase::concurrent::TableLock;
use reedbase::merge::auto_merge;
use reedbase::tables::Table;
use std::collections::HashMap;
use std::sync::Arc;
use std::thread;
use tempfile::TempDir;

/// Generate test content.
fn generate_content(size: usize) -> Vec<u8> {
    (0..size).map(|i| (i % 256) as u8).collect()
}

/// Benchmark concurrent reads.
///
/// Target: < 2x single-thread time for 10 threads
fn bench_concurrent_reads(c: &mut Criterion) {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join(".reed");
    reedbase::registry::init_registry(&db_path).unwrap();
    reedbase::registry::set_base_path(db_path.clone());

    let table = Arc::new(Table::new(&db_path, "concurrent_bench"));
    let content = generate_content(10_240);
    table.init(&content, "system").unwrap();

    let mut group = c.benchmark_group("concurrent_reads");

    for thread_count in [1, 2, 4, 8, 10].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(thread_count),
            thread_count,
            |b, &threads| {
                b.iter(|| {
                    let mut handles = vec![];

                    for _ in 0..threads {
                        let table_clone = Arc::clone(&table);
                        let handle = thread::spawn(move || {
                            for _ in 0..100 {
                                black_box(table_clone.read_current().unwrap());
                            }
                        });
                        handles.push(handle);
                    }

                    for handle in handles {
                        handle.join().unwrap();
                    }
                });
            },
        );
    }

    group.finish();
}

/// Benchmark lock acquisition (uncontended).
///
/// Target: < 1ms
fn bench_lock_uncontended(c: &mut Criterion) {
    let temp_dir = TempDir::new().unwrap();
    let lock_path = temp_dir.path().join("test.lock");

    c.bench_function("lock_acquire_uncontended", |b| {
        b.iter(|| {
            let _lock = black_box(TableLock::acquire(&lock_path).unwrap());
            // Lock released at end of scope
        });
    });
}

/// Benchmark lock contention.
///
/// Target: Measure degradation under contention
fn bench_lock_contention(c: &mut Criterion) {
    let mut group = c.benchmark_group("lock_contention");
    group.sample_size(10); // Expensive operation

    for thread_count in [2, 4, 8].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(thread_count),
            thread_count,
            |b, &threads| {
                b.iter(|| {
                    let temp_dir = TempDir::new().unwrap();
                    let lock_path = Arc::new(temp_dir.path().join("contention.lock"));
                    let mut handles = vec![];

                    for _ in 0..threads {
                        let lock_path_clone = Arc::clone(&lock_path);
                        let handle = thread::spawn(move || {
                            for _ in 0..10 {
                                let _lock = TableLock::acquire(&lock_path_clone).unwrap();
                                // Simulate some work
                                std::thread::sleep(std::time::Duration::from_micros(100));
                            }
                        });
                        handles.push(handle);
                    }

                    for handle in handles {
                        handle.join().unwrap();
                    }
                });
            },
        );
    }

    group.finish();
}

/// Benchmark auto-merge for non-conflicting changes.
///
/// Target: < 5ms
fn bench_auto_merge_success(c: &mut Criterion) {
    c.bench_function("auto_merge_non_conflicting", |b| {
        b.iter(|| {
            // Base version
            let mut base = HashMap::new();
            base.insert("key1".to_string(), "value1".to_string());
            base.insert("key2".to_string(), "value2".to_string());
            base.insert("key3".to_string(), "value3".to_string());

            // Change A: modify key1
            let mut change_a = base.clone();
            change_a.insert("key1".to_string(), "modified_a".to_string());

            // Change B: modify key2 (different key)
            let mut change_b = base.clone();
            change_b.insert("key2".to_string(), "modified_b".to_string());

            black_box(auto_merge(&base, &change_a, &change_b).unwrap());
        });
    });
}

/// Benchmark conflict detection.
///
/// Target: < 10ms
fn bench_conflict_detection(c: &mut Criterion) {
    c.bench_function("conflict_detection", |b| {
        b.iter(|| {
            // Base version
            let mut base = HashMap::new();
            base.insert("key1".to_string(), "value1".to_string());
            base.insert("key2".to_string(), "value2".to_string());

            // Change A: modify key1
            let mut change_a = base.clone();
            change_a.insert("key1".to_string(), "modified_a".to_string());

            // Change B: also modify key1 (conflict!)
            let mut change_b = base.clone();
            change_b.insert("key1".to_string(), "modified_b".to_string());

            // Should detect conflict on key1
            let result = auto_merge(&base, &change_a, &change_b);
            black_box(result.is_err());
        });
    });
}

/// Benchmark merge with multiple conflicts.
///
/// Target: < 20ms for 10 conflicts
fn bench_multiple_conflicts(c: &mut Criterion) {
    c.bench_function("multiple_conflicts", |b| {
        b.iter(|| {
            let mut base = HashMap::new();
            for i in 0..10 {
                base.insert(format!("key{}", i), format!("value{}", i));
            }

            // Change A: modify all keys
            let mut change_a = base.clone();
            for i in 0..10 {
                change_a.insert(format!("key{}", i), format!("modified_a_{}", i));
            }

            // Change B: also modify all keys (10 conflicts)
            let mut change_b = base.clone();
            for i in 0..10 {
                change_b.insert(format!("key{}", i), format!("modified_b_{}", i));
            }

            let result = auto_merge(&base, &change_a, &change_b);
            black_box(result.is_err());
        });
    });
}

/// Benchmark mixed workload (80% read, 20% write).
///
/// Target: Realistic performance under mixed load
fn bench_mixed_workload(c: &mut Criterion) {
    let mut group = c.benchmark_group("mixed_workload");
    group.sample_size(10);

    group.bench_function("80_read_20_write", |b| {
        b.iter(|| {
            let temp_dir = TempDir::new().unwrap();
            let db_path = temp_dir.path().join(".reed");
            reedbase::registry::init_registry(&db_path).unwrap();
            reedbase::registry::set_base_path(db_path.clone());

            let table = Arc::new(Table::new(&db_path, "mixed_bench"));
            let content = generate_content(1024);
            table.init(&content, "system").unwrap();

            let mut handles = vec![];

            // 8 read threads
            for _ in 0..8 {
                let table_clone = Arc::clone(&table);
                let handle = thread::spawn(move || {
                    for _ in 0..100 {
                        black_box(table_clone.read_current().unwrap());
                    }
                });
                handles.push(handle);
            }

            // 2 write threads
            for t in 0..2 {
                let table_clone = Arc::clone(&table);
                let handle = thread::spawn(move || {
                    let mut content = generate_content(1024);
                    for i in 0..50 {
                        content[0] = ((t * 50 + i) % 256) as u8;
                        table_clone.write(&content, "system").unwrap();
                    }
                });
                handles.push(handle);
            }

            for handle in handles {
                handle.join().unwrap();
            }
        });
    });

    group.finish();
}

/// Benchmark sequential vs concurrent write throughput.
///
/// Target: Measure overhead of locking
fn bench_write_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("write_throughput");
    group.sample_size(10);

    // Sequential writes
    group.bench_function("sequential_writes", |b| {
        b.iter(|| {
            let temp_dir = TempDir::new().unwrap();
            let db_path = temp_dir.path().join(".reed");
            reedbase::registry::init_registry(&db_path).unwrap();
            reedbase::registry::set_base_path(db_path.clone());

            let table = Table::new(&db_path, "seq_writes");
            let mut content = generate_content(1024);
            table.init(&content, "system").unwrap();

            for i in 0..100 {
                content[0] = (i % 256) as u8;
                table.write(&content, "system").unwrap();
            }
        });
    });

    // Concurrent writes (4 threads, 25 writes each)
    group.bench_function("concurrent_writes_4_threads", |b| {
        b.iter(|| {
            let temp_dir = TempDir::new().unwrap();
            let db_path = temp_dir.path().join(".reed");
            reedbase::registry::init_registry(&db_path).unwrap();
            reedbase::registry::set_base_path(db_path.clone());

            let table = Arc::new(Table::new(&db_path, "conc_writes"));
            let content = generate_content(1024);
            table.init(&content, "system").unwrap();

            let mut handles = vec![];

            for t in 0..4 {
                let table_clone = Arc::clone(&table);
                let mut content_clone = content.clone();
                let handle = thread::spawn(move || {
                    for i in 0..25 {
                        content_clone[0] = ((t * 25 + i) % 256) as u8;
                        table_clone.write(&content_clone, "system").unwrap();
                    }
                });
                handles.push(handle);
            }

            for handle in handles {
                handle.join().unwrap();
            }
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_concurrent_reads,
    bench_lock_uncontended,
    bench_lock_contention,
    bench_auto_merge_success,
    bench_conflict_detection,
    bench_multiple_conflicts,
    bench_mixed_workload,
    bench_write_throughput
);
criterion_main!(benches);
