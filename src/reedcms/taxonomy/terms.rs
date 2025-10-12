// Copyright 2025 Vivian Voss. Licensed under the Apache Licence, Version 2.0.
// SPDX-Licence-Identifier: Apache-2.0

//! Taxonomy term management.
//!
//! Provides CRUD operations for taxonomy terms with hierarchical support.

use crate::reedcms::matrix::{read_matrix_csv, write_matrix_csv, MatrixRecord, MatrixValue};
use crate::reedcms::reedstream::{
    current_timestamp, ReedError, ReedResponse, ReedResult, ResponseMetrics,
};
use std::fs;
use std::path::PathBuf;

/// Taxonomy term information.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TermInfo {
    pub term_id: String,
    pub term: String,
    pub parent_id: Option<String>,
    pub category: String,
    pub description: Option<String>,
    pub color: Option<String>,
    pub icon: Option<String>,
    pub status: String,
    pub created_by: String,
    pub usage_count: u32,
    pub created_at: String,
    pub updated_at: String,
}

/// Creates a new taxonomy term.
///
/// ## Input
/// - term: Term name (3-64 chars, alphanumeric + spaces/hyphens/underscores)
/// - parent_id: Optional parent term ID for hierarchy
/// - category: Category classification
/// - description: Optional description
/// - color: Optional hex color (#RRGGBB)
/// - icon: Optional icon name
/// - created_by: User ID creating the term
///
/// ## Output
/// - ReedResponse<TermInfo>: Created term information
///
/// ## Performance
/// - O(n) where n = number of existing terms (uniqueness check)
/// - Target: <10ms for <1000 terms
///
/// ## Error Conditions
/// - Invalid term name (length, characters)
/// - Duplicate term in same category
/// - Parent term does not exist
/// - Invalid colour format
/// - File I/O errors
///
/// ## Example Usage
/// ```rust
/// let term = create_term("Rust", None, "Programming", Some("Systems programming language"), None, None, "admin")?;
/// ```
pub fn create_term(
    term: &str,
    parent_id: Option<String>,
    category: &str,
    description: Option<String>,
    color: Option<String>,
    icon: Option<String>,
    created_by: &str,
) -> ReedResult<ReedResponse<TermInfo>> {
    let start = std::time::Instant::now();

    // Validate term name
    validate_term_name(term)?;

    // Validate category
    if category.is_empty() || category.len() > 32 {
        return Err(validation_error("category", category, "1-32 characters"));
    }

    // Validate colour if provided
    if let Some(ref c) = color {
        validate_color(c)?;
    }

    // Ensure .reed directory exists
    fs::create_dir_all(".reed")?;
    let taxonomy_path = PathBuf::from(".reed/taxonomie.matrix.csv");

    // Read existing terms
    let mut records = if taxonomy_path.exists() {
        read_matrix_csv(&taxonomy_path)?
    } else {
        Vec::new()
    };

    // Check for duplicate term in same category
    for record in &records {
        if let Some(MatrixValue::Single(rec_term)) = record.fields.get("term") {
            if let Some(MatrixValue::Single(rec_category)) = record.fields.get("category") {
                if rec_term == term && rec_category == category {
                    return Err(validation_error(
                        "term",
                        term,
                        &format!("already exists in category '{}'", category),
                    ));
                }
            }
        }
    }

    // Verify parent exists if specified
    if let Some(ref pid) = parent_id {
        let parent_exists = records.iter().any(|r| {
            if let Some(MatrixValue::Single(term_id)) = r.fields.get("term_id") {
                term_id == pid
            } else {
                false
            }
        });

        if !parent_exists {
            return Err(validation_error(
                "parent_id",
                pid,
                "parent term does not exist",
            ));
        }
    }

    // Generate term_id (use term as key for simplicity, could use UUID)
    let term_id = format!("{}:{}", category, term);

    // Create timestamps
    let now = chrono::Utc::now().to_rfc3339();

    // Create record
    let mut record = MatrixRecord::new();
    record.add_field("term_id".to_string(), MatrixValue::Single(term_id.clone()));
    record.add_field("term".to_string(), MatrixValue::Single(term.to_string()));
    record.add_field(
        "category".to_string(),
        MatrixValue::Single(category.to_string()),
    );
    record.add_field(
        "status".to_string(),
        MatrixValue::Single("active".to_string()),
    );
    record.add_field(
        "created_by".to_string(),
        MatrixValue::Single(created_by.to_string()),
    );
    record.add_field(
        "usage_count".to_string(),
        MatrixValue::Single("0".to_string()),
    );
    record.add_field("created_at".to_string(), MatrixValue::Single(now.clone()));
    record.add_field("updated_at".to_string(), MatrixValue::Single(now.clone()));

    if let Some(pid) = parent_id.clone() {
        record.add_field("parent_id".to_string(), MatrixValue::Single(pid));
    }
    if let Some(desc) = description.clone() {
        record.add_field("description".to_string(), MatrixValue::Single(desc));
    }
    if let Some(c) = color.clone() {
        record.add_field("color".to_string(), MatrixValue::Single(c));
    }
    if let Some(i) = icon.clone() {
        record.add_field("icon".to_string(), MatrixValue::Single(i));
    }

    record.set_description("Taxonomy term".to_string());

    records.push(record);

    // Write to file
    let field_names = vec![
        "term_id",
        "term",
        "category",
        "parent_id",
        "description",
        "color",
        "icon",
        "status",
        "created_by",
        "usage_count",
        "created_at",
        "updated_at",
    ];
    write_matrix_csv(&taxonomy_path, &records, &field_names)?;

    let duration = start.elapsed();

    // Build response
    let term_info = TermInfo {
        term_id,
        term: term.to_string(),
        parent_id,
        category: category.to_string(),
        description,
        color,
        icon,
        status: "active".to_string(),
        created_by: created_by.to_string(),
        usage_count: 0,
        created_at: now.clone(),
        updated_at: now,
    };

    Ok(ReedResponse {
        data: term_info,
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

/// Retrieves a taxonomy term by ID.
///
/// ## Input
/// - term_id: Term ID to retrieve
///
/// ## Output
/// - ReedResponse<TermInfo>: Term information
///
/// ## Performance
/// - O(n) where n = number of terms
/// - Target: <5ms for <1000 terms
///
/// ## Error Conditions
/// - Term not found
/// - File I/O errors
///
/// ## Example Usage
/// ```rust
/// let term = get_term("Programming:Rust")?;
/// ```
pub fn get_term(term_id: &str) -> ReedResult<ReedResponse<TermInfo>> {
    let start = std::time::Instant::now();

    let taxonomy_path = PathBuf::from(".reed/taxonomie.matrix.csv");
    if !taxonomy_path.exists() {
        return Err(not_found_error("term", term_id));
    }

    let records = read_matrix_csv(&taxonomy_path)?;

    // Find term
    let record = records
        .iter()
        .find(|r| {
            if let Some(MatrixValue::Single(tid)) = r.fields.get("term_id") {
                tid == term_id
            } else {
                false
            }
        })
        .ok_or_else(|| not_found_error("term", term_id))?;

    let duration = start.elapsed();

    // Parse term info
    let term_info = parse_term_info(record)?;

    Ok(ReedResponse {
        data: term_info,
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

/// Lists taxonomy terms with optional filtering.
///
/// ## Input
/// - category: Optional category filter
/// - parent_id: Optional parent filter (use "root" for top-level)
/// - status: Optional status filter
///
/// ## Output
/// - ReedResponse<Vec<TermInfo>>: List of terms
///
/// ## Performance
/// - O(n) where n = number of terms
/// - Target: <10ms for <1000 terms
///
/// ## Error Conditions
/// - File I/O errors
///
/// ## Example Usage
/// ```rust
/// let terms = list_terms(Some("Programming"), None, Some("active"))?;
/// ```
pub fn list_terms(
    category: Option<&str>,
    parent_id: Option<&str>,
    status: Option<&str>,
) -> ReedResult<ReedResponse<Vec<TermInfo>>> {
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
                csv_files_accessed: 1,
                cache_info: None,
            }),
        });
    }

    let records = read_matrix_csv(&taxonomy_path)?;

    // Filter records
    let mut terms = Vec::new();
    for record in &records {
        // Category filter
        if let Some(cat) = category {
            if let Some(MatrixValue::Single(rec_cat)) = record.fields.get("category") {
                if rec_cat != cat {
                    continue;
                }
            } else {
                continue;
            }
        }

        // Parent filter
        if let Some(pid) = parent_id {
            if pid == "root" {
                // Root level: no parent_id or empty parent_id
                if let Some(MatrixValue::Single(rec_pid)) = record.fields.get("parent_id") {
                    if !rec_pid.is_empty() {
                        continue;
                    }
                }
            } else {
                if let Some(MatrixValue::Single(rec_pid)) = record.fields.get("parent_id") {
                    if rec_pid.is_empty() || rec_pid != pid {
                        continue;
                    }
                } else {
                    continue;
                }
            }
        }

        // Status filter
        if let Some(st) = status {
            if let Some(MatrixValue::Single(rec_status)) = record.fields.get("status") {
                if rec_status != st {
                    continue;
                }
            } else {
                continue;
            }
        }

        terms.push(parse_term_info(record)?);
    }

    let duration = start.elapsed();

    Ok(ReedResponse {
        data: terms,
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

/// Searches taxonomy terms by text.
///
/// ## Input
/// - query: Search query (searches term name, category, and description)
/// - category: Optional category filter
///
/// ## Output
/// - ReedResponse<Vec<TermInfo>>: Matching terms
///
/// ## Performance
/// - O(n) where n = number of terms
/// - Target: <50ms for 10,000+ terms
///
/// ## Error Conditions
/// - File I/O errors
///
/// ## Example Usage
/// ```rust
/// let results = search_terms("rust", Some("Programming"))?;
/// ```
pub fn search_terms(
    query: &str,
    category: Option<&str>,
) -> ReedResult<ReedResponse<Vec<TermInfo>>> {
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
                csv_files_accessed: 1,
                cache_info: None,
            }),
        });
    }

    let records = read_matrix_csv(&taxonomy_path)?;
    let query_lower = query.to_lowercase();

    // Search records
    let mut terms = Vec::new();
    for record in &records {
        // Category filter
        if let Some(cat) = category {
            if let Some(MatrixValue::Single(rec_cat)) = record.fields.get("category") {
                if rec_cat != cat {
                    continue;
                }
            } else {
                continue;
            }
        }

        // Text search in term name, category, and description
        let term_name = if let Some(MatrixValue::Single(t)) = record.fields.get("term") {
            t.to_lowercase()
        } else {
            String::new()
        };

        let category_name = if let Some(MatrixValue::Single(c)) = record.fields.get("category") {
            c.to_lowercase()
        } else {
            String::new()
        };

        let description = if let Some(MatrixValue::Single(d)) = record.fields.get("description") {
            d.to_lowercase()
        } else {
            String::new()
        };

        if term_name.contains(&query_lower)
            || category_name.contains(&query_lower)
            || description.contains(&query_lower)
        {
            terms.push(parse_term_info(record)?);
        }
    }

    let duration = start.elapsed();

    Ok(ReedResponse {
        data: terms,
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

/// Updates a taxonomy term.
///
/// ## Input
/// - term_id: Term ID to update
/// - update: TermUpdate with fields to change
///
/// ## Output
/// - ReedResponse<TermInfo>: Updated term information
///
/// ## Performance
/// - O(n) where n = number of terms
/// - Target: <10ms for <1000 terms
///
/// ## Error Conditions
/// - Term not found
/// - Invalid field values
/// - Circular parent reference
/// - File I/O errors
///
/// ## Example Usage
/// ```rust
/// let update = TermUpdate {
///     description: Some(Some("Updated description".to_string())),
///     ..Default::default()
/// };
/// let term = update_term("Programming:Rust", update)?;
/// ```
pub fn update_term(term_id: &str, update: TermUpdate) -> ReedResult<ReedResponse<TermInfo>> {
    let start = std::time::Instant::now();

    let taxonomy_path = PathBuf::from(".reed/taxonomie.matrix.csv");
    if !taxonomy_path.exists() {
        return Err(not_found_error("term", term_id));
    }

    let mut records = read_matrix_csv(&taxonomy_path)?;

    // Find term index
    let idx = records
        .iter()
        .position(|r| {
            if let Some(MatrixValue::Single(tid)) = r.fields.get("term_id") {
                tid == term_id
            } else {
                false
            }
        })
        .ok_or_else(|| not_found_error("term", term_id))?;

    // Validate updates before applying them
    if let Some(ref term) = update.term {
        validate_term_name(term)?;
    }

    if let Some(Some(ref pid)) = update.parent_id {
        // Verify parent exists
        let parent_exists = records.iter().any(|r| {
            if let Some(MatrixValue::Single(tid)) = r.fields.get("term_id") {
                tid == pid
            } else {
                false
            }
        });

        if !parent_exists {
            return Err(validation_error(
                "parent_id",
                pid,
                "parent term does not exist",
            ));
        }

        // Check for circular reference
        if pid == term_id {
            return Err(validation_error(
                "parent_id",
                pid,
                "cannot set term as its own parent",
            ));
        }
    }

    if let Some(Some(ref c)) = update.color {
        validate_color(c)?;
    }

    // Apply updates
    let record = &mut records[idx];
    let now = chrono::Utc::now().to_rfc3339();

    if let Some(term) = update.term {
        record
            .fields
            .insert("term".to_string(), MatrixValue::Single(term));
    }

    if let Some(parent_id) = update.parent_id {
        if let Some(pid) = parent_id {
            record
                .fields
                .insert("parent_id".to_string(), MatrixValue::Single(pid));
        } else {
            record.fields.remove("parent_id");
        }
    }

    if let Some(description) = update.description {
        if let Some(desc) = description {
            record
                .fields
                .insert("description".to_string(), MatrixValue::Single(desc));
        } else {
            record.fields.remove("description");
        }
    }

    if let Some(color) = update.color {
        if let Some(c) = color {
            record
                .fields
                .insert("color".to_string(), MatrixValue::Single(c));
        } else {
            record.fields.remove("color");
        }
    }

    if let Some(icon) = update.icon {
        if let Some(i) = icon {
            record
                .fields
                .insert("icon".to_string(), MatrixValue::Single(i));
        } else {
            record.fields.remove("icon");
        }
    }

    if let Some(status) = update.status {
        if status != "active" && status != "inactive" {
            return Err(validation_error(
                "status",
                &status,
                "must be 'active' or 'inactive'",
            ));
        }
        record
            .fields
            .insert("status".to_string(), MatrixValue::Single(status));
    }

    record
        .fields
        .insert("updated_at".to_string(), MatrixValue::Single(now));

    // Write changes
    let field_names = vec![
        "term_id",
        "term",
        "category",
        "parent_id",
        "description",
        "color",
        "icon",
        "status",
        "created_by",
        "usage_count",
        "created_at",
        "updated_at",
    ];
    write_matrix_csv(&taxonomy_path, &records, &field_names)?;

    // Re-read for response
    let records = read_matrix_csv(&taxonomy_path)?;
    let updated_record = records
        .iter()
        .find(|r| {
            if let Some(MatrixValue::Single(tid)) = r.fields.get("term_id") {
                tid == term_id
            } else {
                false
            }
        })
        .unwrap();
    let term_info = parse_term_info(updated_record)?;

    let duration = start.elapsed();

    Ok(ReedResponse {
        data: term_info,
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

/// Deletes a taxonomy term.
///
/// ## Input
/// - term_id: Term ID to delete
/// - force: If true, deletes children; if false, fails if children exist
///
/// ## Output
/// - ReedResponse<()>: Success confirmation
///
/// ## Performance
/// - O(n) where n = number of terms
/// - Target: <10ms for <1000 terms
///
/// ## Error Conditions
/// - Term not found
/// - Term has children and force=false
/// - Term is assigned to entities
/// - File I/O errors
///
/// ## Example Usage
/// ```rust
/// delete_term("Programming:Rust", false)?;
/// ```
pub fn delete_term(term_id: &str, force: bool) -> ReedResult<ReedResponse<()>> {
    let start = std::time::Instant::now();

    let taxonomy_path = PathBuf::from(".reed/taxonomie.matrix.csv");
    if !taxonomy_path.exists() {
        return Err(not_found_error("term", term_id));
    }

    let mut records = read_matrix_csv(&taxonomy_path)?;

    // Find term
    let term_exists = records.iter().any(|r| {
        if let Some(MatrixValue::Single(tid)) = r.fields.get("term_id") {
            tid == term_id
        } else {
            false
        }
    });

    if !term_exists {
        return Err(not_found_error("term", term_id));
    }

    // Check for children
    let has_children = records.iter().any(|r| {
        if let Some(MatrixValue::Single(pid)) = r.fields.get("parent_id") {
            pid == term_id
        } else {
            false
        }
    });

    if has_children && !force {
        return Err(validation_error(
            "term_id",
            term_id,
            "term has children (use force=true to delete)",
        ));
    }

    // Check for entity assignments
    let entity_taxonomy_path = PathBuf::from(".reed/entity_taxonomy.matrix.csv");
    if entity_taxonomy_path.exists() {
        let entity_records = read_matrix_csv(&entity_taxonomy_path)?;
        for record in &entity_records {
            if let Some(MatrixValue::List(terms)) = record.fields.get("term_ids") {
                if terms.iter().any(|t| t.trim() == term_id) {
                    return Err(validation_error(
                        "term_id",
                        term_id,
                        "term is assigned to entities",
                    ));
                }
            }
        }
    }

    // Delete term and children if force=true
    if force {
        records.retain(|r| {
            let is_term = if let Some(MatrixValue::Single(tid)) = r.fields.get("term_id") {
                tid == term_id
            } else {
                false
            };

            let is_child = if let Some(MatrixValue::Single(pid)) = r.fields.get("parent_id") {
                pid == term_id
            } else {
                false
            };

            !is_term && !is_child
        });
    } else {
        records.retain(|r| {
            if let Some(MatrixValue::Single(tid)) = r.fields.get("term_id") {
                tid != term_id
            } else {
                true
            }
        });
    }

    // Write changes
    let field_names = vec![
        "term_id",
        "term",
        "category",
        "parent_id",
        "description",
        "color",
        "icon",
        "status",
        "created_by",
        "usage_count",
        "created_at",
        "updated_at",
    ];
    write_matrix_csv(&taxonomy_path, &records, &field_names)?;

    let duration = start.elapsed();

    Ok(ReedResponse {
        data: (),
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

/// Term update structure.
#[derive(Debug, Default, Clone)]
pub struct TermUpdate {
    pub term: Option<String>,
    pub parent_id: Option<Option<String>>,
    pub description: Option<Option<String>>,
    pub color: Option<Option<String>>,
    pub icon: Option<Option<String>>,
    pub status: Option<String>,
}

// Helper functions

fn validate_term_name(term: &str) -> ReedResult<()> {
    if term.len() < 2 || term.len() > 64 {
        return Err(validation_error("term", term, "2-64 characters"));
    }

    if !term
        .chars()
        .all(|c| c.is_alphanumeric() || c == ' ' || c == '-' || c == '_')
    {
        return Err(validation_error(
            "term",
            term,
            "alphanumeric + spaces/hyphens/underscores only",
        ));
    }

    Ok(())
}

fn validate_color(color: &str) -> ReedResult<()> {
    if !color.starts_with('#') || color.len() != 7 {
        return Err(validation_error("color", color, "must be #RRGGBB format"));
    }

    if !color[1..].chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(validation_error(
            "color",
            color,
            "must contain valid hex digits",
        ));
    }

    Ok(())
}

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
