// Copyright 2025 Vivian Voss. Licensed under the Apache Licence, Version 2.0.
// SPDX-Licence-Identifier: Apache-2.0

//! Entity-term assignment management.
//!
//! Provides operations for assigning taxonomy terms to entities.

use crate::reedcms::matrix::{read_matrix_csv, write_matrix_csv, MatrixRecord, MatrixValue};
use crate::reedcms::reedstream::{
    current_timestamp, ReedError, ReedResponse, ReedResult, ResponseMetrics,
};
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;

/// Entity types supported by taxonomy system.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum EntityType {
    User,
    Content,
    Template,
    Route,
    Site,
    Project,
    Asset,
    Role,
}

impl FromStr for EntityType {
    type Err = ReedError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "user" => Ok(EntityType::User),
            "content" => Ok(EntityType::Content),
            "template" => Ok(EntityType::Template),
            "route" => Ok(EntityType::Route),
            "site" => Ok(EntityType::Site),
            "project" => Ok(EntityType::Project),
            "asset" => Ok(EntityType::Asset),
            "role" => Ok(EntityType::Role),
            _ => Err(validation_error(
                "entity_type",
                s,
                "must be user/content/template/route/site/project/asset/role",
            )),
        }
    }
}

impl EntityType {
    pub fn as_str(&self) -> &str {
        match self {
            EntityType::User => "user",
            EntityType::Content => "content",
            EntityType::Template => "template",
            EntityType::Route => "route",
            EntityType::Site => "site",
            EntityType::Project => "project",
            EntityType::Asset => "asset",
            EntityType::Role => "role",
        }
    }
}

/// Entity-term assignment information.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EntityTerms {
    pub entity_type: EntityType,
    pub entity_id: String,
    pub term_ids: Vec<String>,
    pub assigned_by: String,
    pub assigned_at: String,
    pub updated_at: String,
}

/// Assigns taxonomy terms to an entity.
///
/// ## Input
/// - entity_type: Type of entity (user, content, template, etc.)
/// - entity_id: Entity ID
/// - term_ids: Vector of term IDs to assign
/// - assigned_by: User ID performing the assignment
///
/// ## Output
/// - ReedResponse<EntityTerms>: Updated entity-term assignments
///
/// ## Performance
/// - O(n + m) where n = existing assignments, m = terms to verify
/// - Target: <10ms for <1000 assignments
///
/// ## Error Conditions
/// - Invalid entity type
/// - Term does not exist
/// - File I/O errors
///
/// ## Example Usage
/// ```rust
/// let terms = assign_terms(EntityType::Content, "post-123", vec!["Programming:Rust".to_string()], "admin")?;
/// ```
pub fn assign_terms(
    entity_type: EntityType,
    entity_id: &str,
    term_ids: Vec<String>,
    assigned_by: &str,
) -> ReedResult<ReedResponse<EntityTerms>> {
    let start = std::time::Instant::now();

    // Validate entity_id
    if entity_id.is_empty() {
        return Err(validation_error("entity_id", entity_id, "cannot be empty"));
    }

    // Verify all terms exist
    verify_terms_exist(&term_ids)?;

    // Ensure .reed directory exists
    fs::create_dir_all(".reed")?;
    let entity_taxonomy_path = PathBuf::from(".reed/entity_taxonomy.matrix.csv");

    // Read existing assignments
    let mut records = if entity_taxonomy_path.exists() {
        read_matrix_csv(&entity_taxonomy_path)?
    } else {
        Vec::new()
    };

    // Build entity key
    let entity_key = format!("{}:{}", entity_type.as_str(), entity_id);

    // Find or create record
    let now = chrono::Utc::now().to_rfc3339();
    let idx = records.iter().position(|r| {
        if let Some(MatrixValue::Single(ek)) = r.fields.get("entity_key") {
            ek == &entity_key
        } else {
            false
        }
    });

    if let Some(i) = idx {
        // Update existing
        let record = &mut records[i];
        record
            .fields
            .insert("term_ids".to_string(), MatrixValue::List(term_ids.clone()));
        record
            .fields
            .insert("updated_at".to_string(), MatrixValue::Single(now.clone()));
    } else {
        // Create new
        let mut record = MatrixRecord::new();
        record.add_field(
            "entity_key".to_string(),
            MatrixValue::Single(entity_key.clone()),
        );
        record.add_field(
            "entity_type".to_string(),
            MatrixValue::Single(entity_type.as_str().to_string()),
        );
        record.add_field(
            "entity_id".to_string(),
            MatrixValue::Single(entity_id.to_string()),
        );
        record.add_field("term_ids".to_string(), MatrixValue::List(term_ids.clone()));
        record.add_field(
            "assigned_by".to_string(),
            MatrixValue::Single(assigned_by.to_string()),
        );
        record.add_field("assigned_at".to_string(), MatrixValue::Single(now.clone()));
        record.add_field("updated_at".to_string(), MatrixValue::Single(now.clone()));
        record.set_description("Entity-term assignment".to_string());

        records.push(record);
    }

    // Write to file
    let field_names = vec![
        "entity_key",
        "entity_type",
        "entity_id",
        "term_ids",
        "assigned_by",
        "assigned_at",
        "updated_at",
    ];
    write_matrix_csv(&entity_taxonomy_path, &records, &field_names)?;

    // Increment usage_count for assigned terms
    increment_term_usage(&term_ids)?;

    let duration = start.elapsed();

    // Build response
    let entity_terms = EntityTerms {
        entity_type,
        entity_id: entity_id.to_string(),
        term_ids: term_ids.clone(),
        assigned_by: assigned_by.to_string(),
        assigned_at: now.clone(),
        updated_at: now,
    };

    Ok(ReedResponse {
        data: entity_terms,
        source: "entity_taxonomy.matrix.csv".to_string(),
        cached: false,
        timestamp: current_timestamp(),
        metrics: Some(ResponseMetrics {
            processing_time_us: duration.as_micros() as u64,
            memory_allocated: None,
            csv_files_accessed: 2,
            cache_info: None,
        }),
    })
}

/// Retrieves taxonomy terms assigned to an entity.
///
/// ## Input
/// - entity_type: Type of entity
/// - entity_id: Entity ID
///
/// ## Output
/// - ReedResponse<EntityTerms>: Entity-term assignments
///
/// ## Performance
/// - O(n) where n = number of entity-term assignments
/// - Target: <5ms for <1000 assignments
///
/// ## Error Conditions
/// - Entity not found
/// - Invalid entity type
/// - File I/O errors
///
/// ## Example Usage
/// ```rust
/// let terms = get_entity_terms(EntityType::Content, "post-123")?;
/// ```
pub fn get_entity_terms(
    entity_type: EntityType,
    entity_id: &str,
) -> ReedResult<ReedResponse<EntityTerms>> {
    let start = std::time::Instant::now();

    let entity_taxonomy_path = PathBuf::from(".reed/entity_taxonomy.matrix.csv");
    if !entity_taxonomy_path.exists() {
        return Err(not_found_error("entity", entity_id));
    }

    let records = read_matrix_csv(&entity_taxonomy_path)?;
    let entity_key = format!("{}:{}", entity_type.as_str(), entity_id);

    // Find record
    let record = records
        .iter()
        .find(|r| {
            if let Some(MatrixValue::Single(ek)) = r.fields.get("entity_key") {
                ek == &entity_key
            } else {
                false
            }
        })
        .ok_or_else(|| not_found_error("entity", entity_id))?;

    let duration = start.elapsed();

    // Parse entity terms
    let entity_terms = parse_entity_terms(record)?;

    Ok(ReedResponse {
        data: entity_terms,
        source: "entity_taxonomy.matrix.csv".to_string(),
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

/// Lists all entities assigned to a specific term.
///
/// ## Input
/// - term_id: Term ID to search for
/// - entity_type: Optional entity type filter
///
/// ## Output
/// - ReedResponse<Vec<EntityTerms>>: List of entities with this term
///
/// ## Performance
/// - O(n) where n = number of entity-term assignments
/// - Target: <20ms for <1000 assignments
///
/// ## Error Conditions
/// - File I/O errors
///
/// ## Example Usage
/// ```rust
/// let entities = list_entities_by_term("Programming:Rust", Some(EntityType::Content))?;
/// ```
pub fn list_entities_by_term(
    term_id: &str,
    entity_type: Option<EntityType>,
) -> ReedResult<ReedResponse<Vec<EntityTerms>>> {
    let start = std::time::Instant::now();

    let entity_taxonomy_path = PathBuf::from(".reed/entity_taxonomy.matrix.csv");
    if !entity_taxonomy_path.exists() {
        return Ok(ReedResponse {
            data: Vec::new(),
            source: "entity_taxonomy.matrix.csv".to_string(),
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

    let records = read_matrix_csv(&entity_taxonomy_path)?;

    let mut entities = Vec::new();
    for record in &records {
        // Entity type filter
        if let Some(et) = entity_type {
            if let Some(MatrixValue::Single(rec_type)) = record.fields.get("entity_type") {
                if rec_type != et.as_str() {
                    continue;
                }
            } else {
                continue;
            }
        }

        // Check if term_id is in term_ids
        let has_term = match record.fields.get("term_ids") {
            Some(MatrixValue::List(term_ids)) => term_ids.iter().any(|t| t.trim() == term_id),
            Some(MatrixValue::Single(single_term)) => single_term.trim() == term_id,
            _ => false,
        };

        if has_term {
            entities.push(parse_entity_terms(record)?);
        }
    }

    let duration = start.elapsed();

    Ok(ReedResponse {
        data: entities,
        source: "entity_taxonomy.matrix.csv".to_string(),
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

/// Removes taxonomy term assignments from an entity.
///
/// ## Input
/// - entity_type: Type of entity
/// - entity_id: Entity ID
/// - term_ids: Optional vector of specific term IDs to remove (if None, removes all)
///
/// ## Output
/// - ReedResponse<()>: Success confirmation
///
/// ## Performance
/// - O(n) where n = number of entity-term assignments
/// - Target: <10ms for <1000 assignments
///
/// ## Error Conditions
/// - Entity not found
/// - File I/O errors
///
/// ## Example Usage
/// ```rust
/// unassign_terms(EntityType::Content, "post-123", Some(vec!["Programming:Rust".to_string()]))?;
/// ```
pub fn unassign_terms(
    entity_type: EntityType,
    entity_id: &str,
    term_ids: Option<Vec<String>>,
) -> ReedResult<ReedResponse<()>> {
    let start = std::time::Instant::now();

    let entity_taxonomy_path = PathBuf::from(".reed/entity_taxonomy.matrix.csv");
    if !entity_taxonomy_path.exists() {
        return Err(not_found_error("entity", entity_id));
    }

    let mut records = read_matrix_csv(&entity_taxonomy_path)?;
    let entity_key = format!("{}:{}", entity_type.as_str(), entity_id);

    // Find record
    let idx = records
        .iter()
        .position(|r| {
            if let Some(MatrixValue::Single(ek)) = r.fields.get("entity_key") {
                ek == &entity_key
            } else {
                false
            }
        })
        .ok_or_else(|| not_found_error("entity", entity_id))?;

    if let Some(specific_terms) = term_ids {
        // Remove specific terms
        let record = &mut records[idx];
        let current_terms: HashSet<String> = match record.fields.get("term_ids") {
            Some(MatrixValue::List(terms)) => terms.iter().cloned().collect(),
            Some(MatrixValue::Single(term)) if !term.is_empty() => {
                let mut set = HashSet::new();
                set.insert(term.clone());
                set
            }
            _ => HashSet::new(),
        };

        let terms_to_remove: HashSet<String> = specific_terms.iter().cloned().collect();
        let remaining: Vec<String> = current_terms
            .difference(&terms_to_remove)
            .cloned()
            .collect();

        if remaining.is_empty() {
            // Remove entire record if no terms remain
            records.remove(idx);
        } else {
            // Update with remaining terms
            record
                .fields
                .insert("term_ids".to_string(), MatrixValue::List(remaining));
            record.fields.insert(
                "updated_at".to_string(),
                MatrixValue::Single(chrono::Utc::now().to_rfc3339()),
            );
        }

        // Decrement usage_count
        decrement_term_usage(&specific_terms)?;
    } else {
        // Remove all terms for this entity
        let terms = match records[idx].fields.get("term_ids") {
            Some(MatrixValue::List(term_list)) => term_list.clone(),
            Some(MatrixValue::Single(term)) if !term.is_empty() => vec![term.clone()],
            _ => Vec::new(),
        };

        records.remove(idx);

        // Decrement usage_count
        decrement_term_usage(&terms)?;
    }

    // Write changes
    let field_names = vec![
        "entity_key",
        "entity_type",
        "entity_id",
        "term_ids",
        "assigned_by",
        "assigned_at",
        "updated_at",
    ];
    write_matrix_csv(&entity_taxonomy_path, &records, &field_names)?;

    let duration = start.elapsed();

    Ok(ReedResponse {
        data: (),
        source: "entity_taxonomy.matrix.csv".to_string(),
        cached: false,
        timestamp: current_timestamp(),
        metrics: Some(ResponseMetrics {
            processing_time_us: duration.as_micros() as u64,
            memory_allocated: None,
            csv_files_accessed: 2,
            cache_info: None,
        }),
    })
}

// Helper functions

fn verify_terms_exist(term_ids: &[String]) -> ReedResult<()> {
    let taxonomy_path = PathBuf::from(".reed/taxonomie.matrix.csv");
    if !taxonomy_path.exists() {
        return Err(validation_error(
            "term_ids",
            &term_ids.join(","),
            "no terms exist",
        ));
    }

    let records = read_matrix_csv(&taxonomy_path)?;
    let existing_terms: HashSet<String> = records
        .iter()
        .filter_map(|r| {
            if let Some(MatrixValue::Single(tid)) = r.fields.get("term_id") {
                Some(tid.clone())
            } else {
                None
            }
        })
        .collect();

    for term_id in term_ids {
        if !existing_terms.contains(term_id) {
            return Err(not_found_error("term", term_id));
        }
    }

    Ok(())
}

fn increment_term_usage(term_ids: &[String]) -> ReedResult<()> {
    update_term_usage(term_ids, 1)
}

fn decrement_term_usage(term_ids: &[String]) -> ReedResult<()> {
    update_term_usage(term_ids, -1)
}

fn update_term_usage(term_ids: &[String], delta: i32) -> ReedResult<()> {
    let taxonomy_path = PathBuf::from(".reed/taxonomie.matrix.csv");
    if !taxonomy_path.exists() {
        return Ok(());
    }

    let mut records = read_matrix_csv(&taxonomy_path)?;

    for term_id in term_ids {
        if let Some(record) = records.iter_mut().find(|r| {
            if let Some(MatrixValue::Single(tid)) = r.fields.get("term_id") {
                tid == term_id
            } else {
                false
            }
        }) {
            let current: i32 =
                if let Some(MatrixValue::Single(count)) = record.fields.get("usage_count") {
                    count.parse().ok().unwrap_or(0)
                } else {
                    0
                };

            let new_count = (current + delta).max(0);
            record.fields.insert(
                "usage_count".to_string(),
                MatrixValue::Single(new_count.to_string()),
            );
        }
    }

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
    Ok(())
}

fn parse_entity_terms(record: &MatrixRecord) -> ReedResult<EntityTerms> {
    let entity_type_str = if let Some(MatrixValue::Single(et)) = record.fields.get("entity_type") {
        et.clone()
    } else {
        return Err(validation_error("entity_type", "", "missing"));
    };

    let entity_type = entity_type_str.parse::<EntityType>()?;

    let term_ids = match record.fields.get("term_ids") {
        Some(MatrixValue::List(ids)) => ids.clone(),
        Some(MatrixValue::Single(id)) if !id.is_empty() => vec![id.clone()],
        _ => Vec::new(),
    };

    let get_single = |field: &str| -> String {
        if let Some(MatrixValue::Single(val)) = record.fields.get(field) {
            val.clone()
        } else {
            String::new()
        }
    };

    Ok(EntityTerms {
        entity_type,
        entity_id: get_single("entity_id"),
        term_ids,
        assigned_by: get_single("assigned_by"),
        assigned_at: get_single("assigned_at"),
        updated_at: get_single("updated_at"),
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
