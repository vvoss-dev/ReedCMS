// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! ReedQL and Smart Indices benchmarks.
//!
//! Measures performance of:
//! - ReedQL query parsing
//! - Query execution (with and without indices)
//! - Smart Index lookups
//! - Range queries
//! - Aggregate functions
//!
//! ## Performance Targets
//! - Query parsing: < 1ms
//! - Indexed lookup: < 1ms
//! - Range query (1000 rows): < 10ms
//! - Aggregate query: < 50ms

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use reedbase::indices::{HashMapIndex, Index, IndexBuilder};
use reedbase::reedql::{executor::execute, parser::parse};
use std::collections::HashMap;

/// Create test data for queries.
fn create_test_data(rows: usize) -> Vec<HashMap<String, String>> {
    (0..rows)
        .map(|i| {
            let mut row = HashMap::new();
            row.insert("id".to_string(), i.to_string());
            row.insert("name".to_string(), format!("user_{}", i));
            row.insert("age".to_string(), ((i % 80) + 18).to_string());
            row.insert("city".to_string(), format!("City{}", i % 10));
            row.insert("score".to_string(), (i % 100).to_string());
            row
        })
        .collect()
}

/// Benchmark query parsing.
///
/// Target: < 1ms for complex queries
fn bench_query_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("reedql_parsing");

    group.bench_function("simple_select", |b| {
        b.iter(|| {
            let query = "SELECT * FROM users WHERE age > '25'";
            black_box(parse(query).unwrap());
        });
    });

    group.bench_function("complex_select", |b| {
        b.iter(|| {
            let query = "SELECT name, age FROM users WHERE age > '25' AND city = 'City1' ORDER BY age DESC LIMIT 10";
            black_box(parse(query).unwrap());
        });
    });

    group.bench_function("with_subquery", |b| {
        b.iter(|| {
            let query =
                "SELECT * FROM users WHERE id IN (SELECT user_id FROM orders WHERE total > '100')";
            black_box(parse(query).unwrap());
        });
    });

    group.finish();
}

/// Benchmark query execution without indices (table scan).
///
/// Target: < 100ms for 10k rows
fn bench_table_scan(c: &mut Criterion) {
    let mut group = c.benchmark_group("reedql_table_scan");

    for size in [1000, 5000, 10_000].iter() {
        let data = create_test_data(*size);

        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| {
                let query = parse("SELECT * FROM users WHERE age > '30'").unwrap();
                black_box(execute(&query, &data).unwrap());
            });
        });
    }

    group.finish();
}

/// Benchmark aggregate functions.
///
/// Target: < 50ms for COUNT/SUM/AVG over 10k rows
fn bench_aggregates(c: &mut Criterion) {
    let data = create_test_data(10_000);
    let mut group = c.benchmark_group("reedql_aggregates");

    group.bench_function("count", |b| {
        b.iter(|| {
            let query = parse("SELECT COUNT(*) FROM users WHERE age > '30'").unwrap();
            black_box(execute(&query, &data).unwrap());
        });
    });

    group.bench_function("sum", |b| {
        b.iter(|| {
            let query = parse("SELECT SUM(score) FROM users WHERE age > '30'").unwrap();
            black_box(execute(&query, &data).unwrap());
        });
    });

    group.bench_function("avg", |b| {
        b.iter(|| {
            let query = parse("SELECT AVG(score) FROM users WHERE age > '30'").unwrap();
            black_box(execute(&query, &data).unwrap());
        });
    });

    group.bench_function("multiple_aggs", |b| {
        b.iter(|| {
            // Note: Parser currently only supports single aggregate per query
            let query = parse("SELECT COUNT(*) FROM users").unwrap();
            black_box(execute(&query, &data).unwrap());
        });
    });

    group.finish();
}

/// Benchmark Smart Index operations.
///
/// Target: < 1ms for indexed lookup
fn bench_smart_indices(c: &mut Criterion) {
    let mut group = c.benchmark_group("smart_indices");

    // Build index
    let mut index: HashMapIndex<String, Vec<usize>> = HashMapIndex::new();
    for i in 0..10_000 {
        let age = ((i % 80) + 18).to_string();
        if let Ok(Some(mut ids)) = index.get(&age) {
            ids.push(i);
            index.insert(age, ids).unwrap();
        } else {
            index.insert(age, vec![i]).unwrap();
        }
    }

    group.bench_function("exact_lookup", |b| {
        b.iter(|| {
            let age = "25".to_string();
            black_box(index.get(&age).unwrap());
        });
    });

    // Note: HashMapIndex doesn't support range scans (not ordered)
    // Range scans would require BTreeIndex implementation

    group.finish();
}

/// Benchmark index build time.
///
/// Target: < 500ms for 10k rows
fn bench_index_build(c: &mut Criterion) {
    let mut group = c.benchmark_group("index_build");

    for size in [1000, 5000, 10_000].iter() {
        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &rows| {
            b.iter(|| {
                let mut index: HashMapIndex<String, Vec<usize>> = HashMapIndex::new();

                for i in 0..rows {
                    let age = ((i % 80) + 18).to_string();
                    if let Ok(Some(mut ids)) = index.get(&age) {
                        ids.push(i);
                        index.insert(age, ids).unwrap();
                    } else {
                        index.insert(age, vec![i]).unwrap();
                    }
                }

                black_box(index);
            });
        });
    }

    group.finish();
}

/// Benchmark GROUP BY operations.
///
/// Target: < 100ms for grouping 10k rows
fn bench_group_by(c: &mut Criterion) {
    let data = create_test_data(10_000);

    // Note: Parser currently doesn't support GROUP BY with aggregates in SELECT
    // These benchmarks test basic query performance instead

    c.bench_function("group_by_single_column", |b| {
        b.iter(|| {
            let query = parse("SELECT city FROM users").unwrap();
            black_box(execute(&query, &data).unwrap());
        });
    });

    c.bench_function("group_by_with_aggregates", |b| {
        b.iter(|| {
            let query = parse("SELECT COUNT(*) FROM users").unwrap();
            black_box(execute(&query, &data).unwrap());
        });
    });
}

/// Benchmark ORDER BY operations.
///
/// Target: < 200ms for sorting 10k rows
fn bench_order_by(c: &mut Criterion) {
    let data = create_test_data(10_000);

    c.bench_function("order_by_single", |b| {
        b.iter(|| {
            let query = parse("SELECT * FROM users ORDER BY age").unwrap();
            black_box(execute(&query, &data).unwrap());
        });
    });

    c.bench_function("order_by_desc", |b| {
        b.iter(|| {
            let query = parse("SELECT * FROM users ORDER BY age DESC").unwrap();
            black_box(execute(&query, &data).unwrap());
        });
    });
}

/// Benchmark LIMIT operations.
///
/// Target: < 10ms with early termination
fn bench_limit(c: &mut Criterion) {
    let data = create_test_data(10_000);

    let mut group = c.benchmark_group("limit");

    for limit in [10, 100, 1000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(limit), limit, |b, &lim| {
            b.iter(|| {
                let query = parse(&format!("SELECT * FROM users LIMIT {}", lim)).unwrap();
                black_box(execute(&query, &data).unwrap());
            });
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_query_parsing,
    bench_table_scan,
    bench_aggregates,
    bench_smart_indices,
    bench_index_build,
    bench_group_by,
    bench_order_by,
    bench_limit
);
criterion_main!(benches);
