// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Versioning and delta operation benchmarks.
//!
//! Measures performance of:
//! - Binary delta generation
//! - Delta application
//! - Version index operations
//! - Backup/restore operations
//!
//! ## Performance Targets
//! - Delta generation: < 50ms for 1KB file
//! - Delta application: < 20ms
//! - Version index lookup: < 1ms
//! - Backup creation: < 500ms for 10MB

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use reedbase::backup::{create_backup, restore_backup};
use reedbase::tables::Table;
use reedbase::version::delta::{apply_delta, generate_delta};
use reedbase::version::index::VersionIndices;
use tempfile::TempDir;

/// Generate test data.
fn generate_test_data(size: usize) -> Vec<u8> {
    (0..size).map(|i| (i % 256) as u8).collect()
}

/// Benchmark binary delta generation.
///
/// Target: < 50ms for 1KB
fn bench_delta_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("delta_generation");

    for size in [256, 512, 1024, 4096, 10_240].iter() {
        let old_data = generate_test_data(*size);
        let mut new_data = old_data.clone();

        // Modify 10% of data
        for i in (0..*size).step_by(10) {
            new_data[i] = new_data[i].wrapping_add(1);
        }

        group.throughput(Throughput::Bytes(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| {
                black_box(generate_delta(&old_data, &new_data).unwrap());
            });
        });
    }

    group.finish();
}

/// Benchmark delta application.
///
/// Target: < 20ms
fn bench_delta_application(c: &mut Criterion) {
    let mut group = c.benchmark_group("delta_application");

    for size in [256, 512, 1024, 4096, 10_240].iter() {
        let old_data = generate_test_data(*size);
        let mut new_data = old_data.clone();
        for i in (0..*size).step_by(10) {
            new_data[i] = new_data[i].wrapping_add(1);
        }

        let delta_info = generate_delta(&old_data, &new_data).unwrap();

        group.throughput(Throughput::Bytes(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| {
                black_box(apply_delta(&old_data, &delta_info.data).unwrap());
            });
        });
    }

    group.finish();
}

/// Benchmark version index insert.
///
/// Target: < 1ms per insert
fn bench_version_index_insert(c: &mut Criterion) {
    c.bench_function("version_index_insert", |b| {
        let temp_dir = TempDir::new().unwrap();
        let ts_path = temp_dir.path().join("ts.idx");
        let frame_path = temp_dir.path().join("frame.idx");
        let mut indices = VersionIndices::open_or_create(&ts_path, &frame_path).unwrap();
        let mut counter = 0;

        b.iter(|| {
            let timestamp = format!("2025-01-01T00:00:{:02}.000Z", counter % 60);
            let frame = format!("F{:04}", counter / 100);
            black_box(indices.insert(counter, timestamp, frame).unwrap());
            counter += 1;
        });
    });
}

/// Benchmark version index timestamp lookup.
///
/// Target: < 1ms
fn bench_version_index_lookup(c: &mut Criterion) {
    let temp_dir = TempDir::new().unwrap();
    let ts_path = temp_dir.path().join("ts.idx");
    let frame_path = temp_dir.path().join("frame.idx");
    let mut indices = VersionIndices::open_or_create(&ts_path, &frame_path).unwrap();

    // Pre-populate
    for i in 0..10_000 {
        let timestamp = format!(
            "2025-01-01T{:02}:{:02}:{:02}.000Z",
            i / 3600,
            (i % 3600) / 60,
            i % 60
        );
        let frame = format!("F{:04}", i / 100);
        indices.insert(i, timestamp, frame).unwrap();
    }

    let mut group = c.benchmark_group("version_index_lookup");

    group.bench_function("exact_timestamp", |b| {
        b.iter(|| {
            let timestamp = "2025-01-01T00:30:00.000Z".to_string();
            black_box(
                indices
                    .query_timestamp_range(&timestamp, &timestamp)
                    .unwrap(),
            );
        });
    });

    group.bench_function("frame_lookup", |b| {
        b.iter(|| {
            let frame = "F0050".to_string();
            black_box(indices.query_frame(&frame).unwrap());
        });
    });

    group.bench_function("range_query_small", |b| {
        b.iter(|| {
            let start = "2025-01-01T00:20:00.000Z".to_string();
            let end = "2025-01-01T00:25:00.000Z".to_string();
            black_box(indices.query_timestamp_range(&start, &end).unwrap());
        });
    });

    group.bench_function("range_query_large", |b| {
        b.iter(|| {
            let start = "2025-01-01T00:00:00.000Z".to_string();
            let end = "2025-01-01T01:00:00.000Z".to_string();
            black_box(indices.query_timestamp_range(&start, &end).unwrap());
        });
    });

    group.finish();
}

/// Benchmark version index statistics.
///
/// Target: < 10ms
fn bench_version_index_stats(c: &mut Criterion) {
    let temp_dir = TempDir::new().unwrap();
    let ts_path = temp_dir.path().join("ts.idx");
    let frame_path = temp_dir.path().join("frame.idx");
    let mut indices = VersionIndices::open_or_create(&ts_path, &frame_path).unwrap();

    for i in 0..1000 {
        let timestamp = format!("2025-01-01T00:{:02}:{:02}.000Z", i / 60, i % 60);
        let frame = format!("F{:03}", i / 10);
        indices.insert(i, timestamp, frame).unwrap();
    }

    c.bench_function("version_index_stats", |b| {
        b.iter(|| {
            black_box(indices.stats());
        });
    });
}

/// Benchmark backup creation.
///
/// Target: < 500ms for 10MB table
fn bench_backup_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("backup_creation");
    group.sample_size(10); // Expensive operation

    for size in [10_240, 102_400, 1_024_000].iter() {
        group.throughput(Throughput::Bytes(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &bytes| {
            b.iter_batched(
                || {
                    let temp_dir = TempDir::new().unwrap();
                    let db_path = temp_dir.path().join(".reed");
                    reedbase::registry::init_registry(&db_path).unwrap();
                    reedbase::registry::set_base_path(db_path.clone());

                    let table = Table::new(&db_path, "backup_bench");
                    let content = generate_test_data(bytes);
                    table.init(&content, "system").unwrap();
                    (table, temp_dir)
                },
                |(table, _temp)| {
                    black_box(create_backup(&table).unwrap());
                },
                criterion::BatchSize::SmallInput,
            );
        });
    }

    group.finish();
}

/// Benchmark backup restoration.
///
/// Target: < 1s for 10MB backup
fn bench_backup_restoration(c: &mut Criterion) {
    let mut group = c.benchmark_group("backup_restoration");
    group.sample_size(10);

    for size in [10_240, 102_400].iter() {
        group.throughput(Throughput::Bytes(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &bytes| {
            b.iter_batched(
                || {
                    let temp_dir = TempDir::new().unwrap();
                    let db_path = temp_dir.path().join(".reed");
                    reedbase::registry::init_registry(&db_path).unwrap();
                    reedbase::registry::set_base_path(db_path.clone());

                    let table = Table::new(&db_path, "restore_bench");
                    let content = generate_test_data(bytes);
                    table.init(&content, "system").unwrap();
                    let backup_path = create_backup(&table).unwrap();
                    (backup_path, temp_dir)
                },
                |(backup_path, temp_dir)| {
                    let restore_path = temp_dir.path().join("restored");
                    black_box(restore_backup(&backup_path, &restore_path).unwrap());
                },
                criterion::BatchSize::SmallInput,
            );
        });
    }

    group.finish();
}

/// Benchmark delta compression ratio.
///
/// Measure how well deltas compress changes
fn bench_delta_compression_ratio(c: &mut Criterion) {
    let mut group = c.benchmark_group("delta_compression_ratio");

    let size = 4096;
    for change_percent in [1, 5, 10, 25, 50, 100].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(change_percent),
            change_percent,
            |b, &percent| {
                b.iter(|| {
                    let old_data = generate_test_data(size);
                    let mut new_data = old_data.clone();

                    let changes = (size * percent) / 100;
                    for i in 0..changes {
                        new_data[i] = new_data[i].wrapping_add(1);
                    }

                    let delta = generate_delta(&old_data, &new_data).unwrap();

                    // Report compression ratio
                    let ratio = (delta.data.len() as f64 / size as f64) * 100.0;
                    black_box((delta, ratio));
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_delta_generation,
    bench_delta_application,
    bench_version_index_insert,
    bench_version_index_lookup,
    bench_version_index_stats,
    bench_backup_creation,
    bench_backup_restoration,
    bench_delta_compression_ratio
);
criterion_main!(benches);
