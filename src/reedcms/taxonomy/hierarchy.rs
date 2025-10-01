// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Taxonomy hierarchy utilities.
//!
//! Provides utilities for navigating and managing hierarchical taxonomy structures.

use crate::reedcms::matrix::{read_matrix_csv, MatrixRecord, MatrixValue};
use crate::reedcms::reedstream::{
    current_timestamp, ReedError, ReedResponse, ReedResult, ResponseMetrics,
};
use crate::reedcms::taxonomy::terms::TermInfo;
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::PathBuf;

/// Retrieves all children of a term.
///
/// ## Input
/// - term_id: Parent term ID
/// - recursive: If true, returns all descendants; if false, only direct children
///
/// ## Output
/// - ReedResponse<Vec<TermInfo>>: List of child terms
///
/// ## Performance
/// - O(n) for direct children, O(n²) for recursive
/// - Target: <10ms for <1000 terms, <50ms for recursive with 10k+ terms
///
/// ## Error Conditions
/// - Term not found
/// - File I/O errors
///
/// ## Example Usage
/// ```rust
/// let children = get_children("Programming", false)?;
/// let all_descendants = get_children("Programming", true)?;
/// ```
pub fn get_children(term_id: &str, recursive: bool) -> ReedResult<ReedResponse<Vec<TermInfo>>> {
    let start = std::time::Instant::now();

    let taxonomy_path = PathBuf::from(".reed/taxonomie.matrix.csv");
    if !taxonomy_path.exists() {
        return Err(not_found_error("term", term_id));
    }

    let records = read_matrix_csv(&taxonomy_path)?;

    // Verify parent exists
    let parent_exists = records.iter().any(|r| {
        if let Some(MatrixValue::Single(tid)) = r.fields.get("term_id") {
            tid == term_id
        } else {
            false
        }
    });

    if !parent_exists {
        return Err(not_found_error("term", term_id));
    }

    let mut children = Vec::new();

    if recursive {
        // Collect all descendants using BFS
        let mut queue = VecDeque::new();
        queue.push_back(term_id.to_string());
        let mut visited = HashSet::new();

        while let Some(current) = queue.pop_front() {
            if visited.contains(&current) {
                continue;
            }
            visited.insert(current.clone());

            // Find direct children
            for record in &records {
                if let Some(MatrixValue::Single(pid)) = record.fields.get("parent_id") {
                    if pid == &current {
                        children.push(parse_term_info(record)?);
                        if let Some(MatrixValue::Single(tid)) = record.fields.get("term_id") {
                            queue.push_back(tid.clone());
                        }
                    }
                }
            }
        }
    } else {
        // Only direct children
        for record in &records {
            if let Some(MatrixValue::Single(pid)) = record.fields.get("parent_id") {
                if pid == term_id {
                    children.push(parse_term_info(record)?);
                }
            }
        }
    }

    let duration = start.elapsed();

    Ok(ReedResponse {
        data: children,
        source: "taxonomie.matrix.csv".to_string(),
        cached: false,
        timestamp: current_timestamp(),
        metrics: Some(ResponseMetrics {
            processing_time_us: duration.as_micros() as u64,
            memory_allocated: None,
            csv_files_accessed: 1,
            cache_info: None,
        }),
    })
}

/// Retrieves the complete ancestry path of a term.
///
/// ## Input
/// - term_id: Term ID to get ancestry for
///
/// ## Output
/// - ReedResponse<Vec<TermInfo>>: List of ancestors from root to term (excluding the term itself)
///
/// ## Performance
/// - O(d) where d = depth of term in hierarchy
/// - Target: <5ms for depth <10
///
/// ## Error Conditions
/// - Term not found
/// - Circular reference detected
/// - File I/O errors
///
/// ## Example Usage
/// ```rust
/// let ancestors = get_ancestors("Programming:Rust:Async")?;
/// // Returns: [Programming, Programming:Rust]
/// ```
pub fn get_ancestors(term_id: &str) -> ReedResult<ReedResponse<Vec<TermInfo>>> {
    let start = std::time::Instant::now();

    let taxonomy_path = PathBuf::from(".reed/taxonomie.matrix.csv");
    if !taxonomy_path.exists() {
        return Err(not_found_error("term", term_id));
    }

    let records = read_matrix_csv(&taxonomy_path)?;

    // Find the term
    let term_record = records
        .iter()
        .find(|r| {
            if let Some(MatrixValue::Single(tid)) = r.fields.get("term_id") {
                tid == term_id
            } else {
                false
            }
        })
        .ok_or_else(|| not_found_error("term", term_id))?;

    let mut ancestors = Vec::new();
    let mut visited = HashSet::new();
    let mut current_parent =
        if let Some(MatrixValue::Single(pid)) = term_record.fields.get("parent_id") {
            Some(pid.clone())
        } else {
            None
        };

    // Traverse up the hierarchy
    while let Some(parent_id) = current_parent {
        if visited.contains(&parent_id) {
            return Err(validation_error(
                "term_id",
                term_id,
                "circular reference detected in ancestry",
            ));
        }
        visited.insert(parent_id.clone());

        // Find parent record
        let parent_record = records
            .iter()
            .find(|r| {
                if let Some(MatrixValue::Single(tid)) = r.fields.get("term_id") {
                    tid == &parent_id
                } else {
                    false
                }
            })
            .ok_or_else(|| not_found_error("term", &parent_id))?;

        ancestors.push(parse_term_info(parent_record)?);
        current_parent =
            if let Some(MatrixValue::Single(pid)) = parent_record.fields.get("parent_id") {
                Some(pid.clone())
            } else {
                None
            };
    }

    // Reverse to get root-to-term order
    ancestors.reverse();

    let duration = start.elapsed();

    Ok(ReedResponse {
        data: ancestors,
        source: "taxonomie.matrix.csv".to_string(),
        cached: false,
        timestamp: current_timestamp(),
        metrics: Some(ResponseMetrics {
            processing_time_us: duration.as_micros() as u64,
            memory_allocated: None,
            csv_files_accessed: 1,
            cache_info: None,
        }),
    })
}

/// Gets the complete path string for a term.
///
/// ## Input
/// - term_id: Term ID
/// - separator: Path separator (default: " > ")
///
/// ## Output
/// - ReedResponse<String>: Full path string (e.g., "Programming > Rust > Async")
///
/// ## Performance
/// - O(d) where d = depth of term
/// - Target: <5ms for depth <10
///
/// ## Error Conditions
/// - Term not found
/// - File I/O errors
///
/// ## Example Usage
/// ```rust
/// let path = get_path("Programming:Rust:Async", " / ")?;
/// // Returns: "Programming / Rust / Async"
/// ```
pub fn get_path(term_id: &str, separator: &str) -> ReedResult<ReedResponse<String>> {
    let start = std::time::Instant::now();

    let taxonomy_path = PathBuf::from(".reed/taxonomie.matrix.csv");
    if !taxonomy_path.exists() {
        return Err(not_found_error("term", term_id));
    }

    let records = read_matrix_csv(&taxonomy_path)?;

    // Find the term
    let term_record = records
        .iter()
        .find(|r| {
            if let Some(MatrixValue::Single(tid)) = r.fields.get("term_id") {
                tid == term_id
            } else {
                false
            }
        })
        .ok_or_else(|| not_found_error("term", term_id))?;

    let term_name = if let Some(MatrixValue::Single(name)) = term_record.fields.get("term") {
        name.clone()
    } else {
        return Err(validation_error("term", term_id, "missing term name"));
    };

    // Get ancestors
    let ancestors_response = get_ancestors(term_id)?;
    let ancestors = ancestors_response.data;

    // Build path
    let mut path_parts: Vec<String> = ancestors.iter().map(|t| t.term.clone()).collect();
    path_parts.push(term_name);

    let path = path_parts.join(separator);
    let duration = start.elapsed();

    Ok(ReedResponse {
        data: path,
        source: "taxonomie.matrix.csv".to_string(),
        cached: false,
        timestamp: current_timestamp(),
        metrics: Some(ResponseMetrics {
            processing_time_us: duration.as_micros() as u64,
            memory_allocated: None,
            csv_files_accessed: 1,
            cache_info: None,
        }),
    })
}

/// Gets the depth of a term in the hierarchy.
///
/// ## Input
/// - term_id: Term ID
///
/// ## Output
/// - ReedResponse<usize>: Depth (0 = root level)
///
/// ## Performance
/// - O(d) where d = depth of term
/// - Target: <5ms for depth <10
///
/// ## Error Conditions
/// - Term not found
/// - Circular reference
/// - File I/O errors
///
/// ## Example Usage
/// ```rust
/// let depth = get_depth("Programming:Rust:Async")?;
/// // Returns: 2 (Programming = 0, Rust = 1, Async = 2)
/// ```
pub fn get_depth(term_id: &str) -> ReedResult<ReedResponse<usize>> {
    let start = std::time::Instant::now();

    let ancestors_response = get_ancestors(term_id)?;
    let depth = ancestors_response.data.len();
    let duration = start.elapsed();

    Ok(ReedResponse {
        data: depth,
        source: "taxonomie.matrix.csv".to_string(),
        cached: false,
        timestamp: current_timestamp(),
        metrics: Some(ResponseMetrics {
            processing_time_us: duration.as_micros() as u64,
            memory_allocated: None,
            csv_files_accessed: 1,
            cache_info: None,
        }),
    })
}

/// Checks for circular references in hierarchy.
///
/// ## Input
/// - term_id: Term ID to check
/// - new_parent_id: Proposed new parent ID
///
/// ## Output
/// - ReedResponse<bool>: true if circular reference would be created
///
/// ## Performance
/// - O(d) where d = depth of hierarchy
/// - Target: <5ms for depth <10
///
/// ## Error Conditions
/// - Term not found
/// - File I/O errors
///
/// ## Example Usage
/// ```rust
/// let would_be_circular = has_circular_reference("Programming", "Programming:Rust")?;
/// // Returns: true (Rust is descendant of Programming)
/// ```
pub fn has_circular_reference(
    term_id: &str,
    new_parent_id: &str,
) -> ReedResult<ReedResponse<bool>> {
    let start = std::time::Instant::now();

    // Cannot be own parent
    if term_id == new_parent_id {
        return Ok(ReedResponse {
            data: true,
            source: "taxonomie.matrix.csv".to_string(),
            cached: false,
            timestamp: current_timestamp(),
            metrics: Some(ResponseMetrics {
                processing_time_us: start.elapsed().as_micros() as u64,
                memory_allocated: None,
                csv_files_accessed: 0,
                cache_info: None,
            }),
        });
    }

    // Get all descendants of term_id
    let descendants_response = get_children(term_id, true)?;
    let descendants = descendants_response.data;

    // Check if new_parent_id is among descendants
    let is_circular = descendants.iter().any(|d| d.term_id == new_parent_id);

    let duration = start.elapsed();

    Ok(ReedResponse {
        data: is_circular,
        source: "taxonomie.matrix.csv".to_string(),
        cached: false,
        timestamp: current_timestamp(),
        metrics: Some(ResponseMetrics {
            processing_time_us: duration.as_micros() as u64,
            memory_allocated: None,
            csv_files_accessed: 1,
            cache_info: None,
        }),
    })
}

/// Builds a complete taxonomy tree structure.
///
/// ## Input
/// - category: Optional category filter
///
/// ## Output
/// - ReedResponse<Vec<TermTree>>: Tree structure with nested children
///
/// ## Performance
/// - O(n²) where n = number of terms
/// - Target: <100ms for <1000 terms
///
/// ## Error Conditions
/// - File I/O errors
///
/// ## Example Usage
/// ```rust
/// let tree = get_tree(Some("Programming"))?;
/// ```
pub fn get_tree(category: Option<&str>) -> ReedResult<ReedResponse<Vec<TermTree>>> {
    let start = std::time::Instant::now();

    let taxonomy_path = PathBuf::from(".reed/taxonomie.matrix.csv");
    if !taxonomy_path.exists() {
        return Ok(ReedResponse {
            data: Vec::new(),
            source: "taxonomie.matrix.csv".to_string(),
            cached: false,
            timestamp: current_timestamp(),
            metrics: Some(ResponseMetrics {
                processing_time_us: start.elapsed().as_micros() as u64,
                memory_allocated: None,
                csv_files_accessed: 0,
                cache_info: None,
            }),
        });
    }

    let records = read_matrix_csv(&taxonomy_path)?;

    // Filter by category if specified
    let filtered_records: Vec<&MatrixRecord> = if let Some(cat) = category {
        records
            .iter()
            .filter(|r| {
                if let Some(MatrixValue::Single(rec_cat)) = r.fields.get("category") {
                    rec_cat == cat
                } else {
                    false
                }
            })
            .collect()
    } else {
        records.iter().collect()
    };

    // Build term info map
    let mut term_map: HashMap<String, TermInfo> = HashMap::new();
    for record in &filtered_records {
        if let Some(MatrixValue::Single(tid)) = record.fields.get("term_id") {
            term_map.insert(tid.clone(), parse_term_info(record)?);
        }
    }

    // Find root terms (no parent or empty parent)
    let mut roots = Vec::new();
    for record in &filtered_records {
        let is_root = if let Some(MatrixValue::Single(pid)) = record.fields.get("parent_id") {
            pid.is_empty()
        } else {
            true
        };

        if is_root {
            if let Some(MatrixValue::Single(term_id)) = record.fields.get("term_id") {
                let tree = build_tree_node(term_id, &term_map, &filtered_records)?;
                roots.push(tree);
            }
        }
    }

    let duration = start.elapsed();

    Ok(ReedResponse {
        data: roots,
        source: "taxonomie.matrix.csv".to_string(),
        cached: false,
        timestamp: current_timestamp(),
        metrics: Some(ResponseMetrics {
            processing_time_us: duration.as_micros() as u64,
            memory_allocated: None,
            csv_files_accessed: 1,
            cache_info: None,
        }),
    })
}

/// Tree node structure for hierarchical display.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TermTree {
    pub term: TermInfo,
    pub children: Vec<TermTree>,
}

fn build_tree_node(
    term_id: &str,
    term_map: &HashMap<String, TermInfo>,
    records: &[&MatrixRecord],
) -> ReedResult<TermTree> {
    let term = term_map
        .get(term_id)
        .ok_or_else(|| not_found_error("term", term_id))?
        .clone();

    let mut children = Vec::new();

    // Find direct children
    for record in records {
        if let Some(MatrixValue::Single(pid)) = record.fields.get("parent_id") {
            if pid == term_id {
                if let Some(MatrixValue::Single(child_id)) = record.fields.get("term_id") {
                    let child_tree = build_tree_node(child_id, term_map, records)?;
                    children.push(child_tree);
                }
            }
        }
    }

    Ok(TermTree { term, children })
}

// Helper functions

fn parse_term_info(record: &MatrixRecord) -> ReedResult<TermInfo> {
    let get_single = |field: &str| -> Option<String> {
        if let Some(MatrixValue::Single(val)) = record.fields.get(field) {
            if val.is_empty() {
                None
            } else {
                Some(val.clone())
            }
        } else {
            None
        }
    };

    Ok(TermInfo {
        term_id: get_single("term_id").unwrap_or_default(),
        term: get_single("term").unwrap_or_default(),
        parent_id: get_single("parent_id"),
        category: get_single("category").unwrap_or_default(),
        description: get_single("description"),
        color: get_single("color"),
        icon: get_single("icon"),
        status: get_single("status").unwrap_or_else(|| "active".to_string()),
        created_by: get_single("created_by").unwrap_or_default(),
        usage_count: get_single("usage_count")
            .and_then(|v| v.parse().ok())
            .unwrap_or(0),
        created_at: get_single("created_at").unwrap_or_default(),
        updated_at: get_single("updated_at").unwrap_or_default(),
    })
}

fn validation_error(field: &str, value: &str, constraint: &str) -> ReedError {
    ReedError::ValidationError {
        field: field.to_string(),
        value: value.to_string(),
        constraint: constraint.to_string(),
    }
}

fn not_found_error(resource: &str, id: &str) -> ReedError {
    ReedError::NotFound {
        resource: format!("{}: {}", resource, id),
        context: None,
    }
}
