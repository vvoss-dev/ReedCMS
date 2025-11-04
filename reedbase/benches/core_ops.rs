// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Core Table operation benchmarks (byte-level API).
//!
//! Measures performance of:
//! - read_current() - Read current content
//! - write() - Write new content with delta
//! - rollback() - Rollback to previous version
//! - list_versions() - List version history
//!
//! ## Performance Targets
//! - read_current: < 10ms for 1MB file
//! - write: < 50ms for 1MB file (with delta)
//! - rollback: < 100ms
//! - list_versions: < 10ms for 100 versions

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use reedbase::tables::Table;
use tempfile::TempDir;

/// Generate test content of specified size.
fn generate_content(size: usize) -> Vec<u8> {
    (0..size).map(|i| (i % 256) as u8).collect()
}

/// Benchmark read_current().
///
/// Target: < 10ms for 1MB
fn bench_read_current(c: &mut Criterion) {
    let mut group = c.benchmark_group("read_current");

    for size in [1024, 10_240, 102_400, 1_024_000].iter() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join(".reed");
        reedbase::registry::init_registry(&db_path).unwrap();
        reedbase::registry::set_base_path(db_path.clone());

        let table = Table::new(&db_path, "bench_read");
        let content = generate_content(*size);
        table.init(&content, "system").unwrap();

        group.throughput(Throughput::Bytes(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| {
                black_box(table.read_current().unwrap());
            });
        });
    }

    group.finish();
}

/// Benchmark write() with delta generation.
///
/// Target: < 50ms for 1MB file
fn bench_write_with_delta(c: &mut Criterion) {
    let mut group = c.benchmark_group("write_with_delta");
    group.sample_size(20); // Expensive operation

    for size in [1024, 10_240, 102_400].iter() {
        group.throughput(Throughput::Bytes(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &bytes| {
            b.iter_batched(
                || {
                    let temp_dir = TempDir::new().unwrap();
                    let db_path = temp_dir.path().join(".reed");
                    reedbase::registry::init_registry(&db_path).unwrap();
                    reedbase::registry::set_base_path(db_path.clone());

                    let table = Table::new(&db_path, "bench_write");
                    let content = generate_content(bytes);
                    table.init(&content, "system").unwrap();

                    // Modified content (10% changes)
                    let mut new_content = content.clone();
                    for i in (0..bytes).step_by(10) {
                        new_content[i] = new_content[i].wrapping_add(1);
                    }

                    (table, new_content, temp_dir)
                },
                |(table, new_content, _temp)| {
                    black_box(table.write(&new_content, "system").unwrap());
                },
                criterion::BatchSize::SmallInput,
            );
        });
    }

    group.finish();
}

/// Benchmark list_versions().
///
/// Target: < 10ms for 100 versions
fn bench_list_versions(c: &mut Criterion) {
    let mut group = c.benchmark_group("list_versions");

    for version_count in [10, 50, 100].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(version_count),
            version_count,
            |b, &count| {
                b.iter_batched(
                    || {
                        let temp_dir = TempDir::new().unwrap();
                        let db_path = temp_dir.path().join(".reed");
                        reedbase::registry::init_registry(&db_path).unwrap();
                        reedbase::registry::set_base_path(db_path.clone());

                        let table = Table::new(&db_path, "bench_versions");
                        let mut content = generate_content(1024);
                        table.init(&content, "system").unwrap();

                        // Create version history
                        for i in 0..count {
                            content[0] = content[0].wrapping_add(1);
                            table.write(&content, "system").unwrap();
                        }

                        (table, temp_dir)
                    },
                    |(table, _temp)| {
                        black_box(table.list_versions().unwrap());
                    },
                    criterion::BatchSize::SmallInput,
                );
            },
        );
    }

    group.finish();
}

/// Benchmark rollback().
///
/// Target: < 100ms
fn bench_rollback(c: &mut Criterion) {
    let mut group = c.benchmark_group("rollback");
    group.sample_size(10); // Very expensive

    for version_count in [5, 10, 20].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(version_count),
            version_count,
            |b, &count| {
                b.iter_batched(
                    || {
                        let temp_dir = TempDir::new().unwrap();
                        let db_path = temp_dir.path().join(".reed");
                        reedbase::registry::init_registry(&db_path).unwrap();
                        reedbase::registry::set_base_path(db_path.clone());

                        let table = Table::new(&db_path, "bench_rollback");
                        let mut content = generate_content(10_240);
                        table.init(&content, "system").unwrap();

                        let mut timestamps = vec![];

                        // Create version history
                        for _ in 0..count {
                            content[0] = content[0].wrapping_add(1);
                            let result = table.write(&content, "system").unwrap();
                            timestamps.push(result.timestamp);
                        }

                        // Rollback to middle version
                        let target_ts = timestamps[count / 2];
                        (table, target_ts, temp_dir)
                    },
                    |(table, target_ts, _temp)| {
                        black_box(table.rollback(target_ts, "system").unwrap());
                    },
                    criterion::BatchSize::SmallInput,
                );
            },
        );
    }

    group.finish();
}

/// Benchmark read_current_as_rows() CSV parsing.
///
/// Target: < 20ms for 1000 rows
fn bench_read_as_rows(c: &mut Criterion) {
    let mut group = c.benchmark_group("read_as_rows");

    for row_count in [100, 500, 1000].iter() {
        group.throughput(Throughput::Elements(*row_count as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(row_count),
            row_count,
            |b, &rows| {
                b.iter_batched(
                    || {
                        let temp_dir = TempDir::new().unwrap();
                        let db_path = temp_dir.path().join(".reed");
                        reedbase::registry::init_registry(&db_path).unwrap();
                        reedbase::registry::set_base_path(db_path.clone());

                        let table = Table::new(&db_path, "bench_rows");

                        // Generate CSV content
                        let mut csv = String::from("key|value1|value2\n");
                        for i in 0..rows {
                            csv.push_str(&format!("row.{}|value{}|data{}\n", i, i, i));
                        }

                        table.init(csv.as_bytes(), "system").unwrap();
                        (table, temp_dir)
                    },
                    |(table, _temp)| {
                        black_box(table.read_current_as_rows().unwrap());
                    },
                    criterion::BatchSize::SmallInput,
                );
            },
        );
    }

    group.finish();
}

/// Benchmark table.exists() check.
///
/// Target: < 1ms
fn bench_exists_check(c: &mut Criterion) {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join(".reed");
    reedbase::registry::init_registry(&db_path).unwrap();
    reedbase::registry::set_base_path(db_path.clone());

    let table = Table::new(&db_path, "bench_exists");
    table.init(b"test content", "system").unwrap();

    c.bench_function("table_exists", |b| {
        b.iter(|| {
            black_box(table.exists());
        });
    });
}

/// Benchmark concurrent reads (no locking needed).
///
/// Target: Linear scaling with threads
fn bench_concurrent_reads(c: &mut Criterion) {
    use std::sync::Arc;
    use std::thread;

    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join(".reed");
    reedbase::registry::init_registry(&db_path).unwrap();
    reedbase::registry::set_base_path(db_path.clone());

    let table = Arc::new(Table::new(&db_path, "bench_concurrent"));
    let content = generate_content(10_240);
    table.init(&content, "system").unwrap();

    let mut group = c.benchmark_group("concurrent_reads");

    for thread_count in [1, 2, 4, 8].iter() {
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

criterion_group!(
    benches,
    bench_read_current,
    bench_write_with_delta,
    bench_list_versions,
    bench_rollback,
    bench_read_as_rows,
    bench_exists_check,
    bench_concurrent_reads
);
criterion_main!(benches);
