// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! CLI taxonomy commands.
//!
//! Implements taxonomy management commands for the ReedCMS CLI.
//! Commands follow the pattern: reed taxonomy:action [args] [--flags]

use crate::reedcms::reedstream::{ReedError, ReedResponse, ReedResult};
use crate::reedcms::taxonomy::{
    assign_terms, create_term, delete_term, get_term, list_entities_by_term, list_terms,
    search_terms, unassign_terms, update_term, EntityType, TermInfo, TermUpdate,
};
use std::collections::HashMap;

/// Creates a new taxonomy term.
///
/// ## Usage
/// reed taxonomy:create <term> --category <category> [flags]
///
/// ## Required Flags
/// - --category: Category classification
///
/// ## Optional Flags
/// - --parent: Parent term ID for hierarchy
/// - --description: Term description
/// - --color: Hex color (#RRGGBB)
/// - --icon: Icon name
/// - --created-by: User ID (default: system)
///
/// ## Example
/// ```bash
/// reed taxonomy:create Rust --category Programming --description "Systems programming language"
/// reed taxonomy:create Async --category Programming --parent "Programming:Rust"
/// ```
pub fn create(
    args: &[String],
    flags: &HashMap<String, String>,
) -> ReedResult<ReedResponse<String>> {
    if args.is_empty() {
        return Err(ReedError::ValidationError {
            field: "term".to_string(),
            value: String::new(),
            constraint: "term name required".to_string(),
        });
    }

    let term = &args[0];

    let category = flags
        .get("category")
        .ok_or_else(|| ReedError::ValidationError {
            field: "category".to_string(),
            value: String::new(),
            constraint: "--category flag required".to_string(),
        })?;

    let parent_id = flags.get("parent").cloned();
    let description = flags.get("description").cloned();
    let color = flags.get("color").cloned();
    let icon = flags.get("icon").cloned();
    let created_by = flags
        .get("created-by")
        .map(|s| s.as_str())
        .unwrap_or("system");

    let response = create_term(
        term,
        parent_id,
        category,
        description,
        color,
        icon,
        created_by,
    )?;

    let term_info = response.data;
    let message = format!(
        "Term created: {} ({})\nID: {}\nCategory: {}",
        term_info.term, term_info.status, term_info.term_id, term_info.category
    );

    Ok(ReedResponse {
        data: message,
        source: response.source,
        cached: false,
        timestamp: response.timestamp,
        metrics: response.metrics,
    })
}

/// Lists taxonomy terms.
///
/// ## Usage
/// reed taxonomy:list [--category <category>] [--parent <parent_id>] [--status <status>] [--format <format>]
///
/// ## Optional Flags
/// - --category: Filter by category
/// - --parent: Filter by parent (use "root" for top-level)
/// - --status: Filter by status (active/inactive)
/// - --format: Output format (table/json/csv, default: table)
///
/// ## Example
/// ```bash
/// reed taxonomy:list
/// reed taxonomy:list --category Programming
/// reed taxonomy:list --parent root
/// reed taxonomy:list --format json
/// ```
pub fn list(flags: &HashMap<String, String>) -> ReedResult<ReedResponse<String>> {
    let category = flags.get("category").map(|s| s.as_str());
    let parent_id = flags.get("parent").map(|s| s.as_str());
    let status = flags.get("status").map(|s| s.as_str());
    let format = flags.get("format").map(|s| s.as_str()).unwrap_or("table");

    let response = list_terms(category, parent_id, status)?;
    let terms = response.data;

    let output = match format {
        "json" => serde_json::to_string_pretty(&terms).map_err(|e| ReedError::ValidationError {
            field: "json".to_string(),
            value: "serialization".to_string(),
            constraint: e.to_string(),
        })?,
        "csv" => {
            let mut output = "term_id,term,category,parent_id,status,usage_count\n".to_string();
            for term in &terms {
                output.push_str(&format!(
                    "{},{},{},{},{},{}\n",
                    term.term_id,
                    term.term,
                    term.category,
                    term.parent_id.as_deref().unwrap_or(""),
                    term.status,
                    term.usage_count
                ));
            }
            output
        }
        _ => format_terms_table(&terms),
    };

    Ok(ReedResponse {
        data: output,
        source: response.source,
        cached: false,
        timestamp: response.timestamp,
        metrics: response.metrics,
    })
}

/// Shows detailed term information.
///
/// ## Usage
/// reed taxonomy:show <term_id>
///
/// ## Example
/// ```bash
/// reed taxonomy:show "Programming:Rust"
/// ```
pub fn show(args: &[String]) -> ReedResult<ReedResponse<String>> {
    if args.is_empty() {
        return Err(ReedError::ValidationError {
            field: "term_id".to_string(),
            value: String::new(),
            constraint: "term_id required".to_string(),
        });
    }

    let term_id = &args[0];
    let response = get_term(term_id)?;
    let term = response.data;

    let output = format!(
        "Term ID: {}\nTerm: {}\nCategory: {}\nParent: {}\nDescription: {}\nColor: {}\nIcon: {}\nStatus: {}\nCreated By: {}\nUsage Count: {}\nCreated: {}\nUpdated: {}",
        term.term_id,
        term.term,
        term.category,
        term.parent_id.as_deref().unwrap_or("(none)"),
        term.description.as_deref().unwrap_or("(none)"),
        term.color.as_deref().unwrap_or("(none)"),
        term.icon.as_deref().unwrap_or("(none)"),
        term.status,
        term.created_by,
        term.usage_count,
        term.created_at,
        term.updated_at
    );

    Ok(ReedResponse {
        data: output,
        source: response.source,
        cached: false,
        timestamp: response.timestamp,
        metrics: response.metrics,
    })
}

/// Searches taxonomy terms.
///
/// ## Usage
/// reed taxonomy:search <query> [--category <category>] [--format <format>]
///
/// ## Optional Flags
/// - --category: Filter by category
/// - --format: Output format (table/json/csv, default: table)
///
/// ## Example
/// ```bash
/// reed taxonomy:search rust
/// reed taxonomy:search "systems programming" --category Programming
/// ```
pub fn search(
    args: &[String],
    flags: &HashMap<String, String>,
) -> ReedResult<ReedResponse<String>> {
    if args.is_empty() {
        return Err(ReedError::ValidationError {
            field: "query".to_string(),
            value: String::new(),
            constraint: "search query required".to_string(),
        });
    }

    let query = &args[0];
    let category = flags.get("category").map(|s| s.as_str());
    let format = flags.get("format").map(|s| s.as_str()).unwrap_or("table");

    let response = search_terms(query, category)?;
    let terms = response.data;

    let output = match format {
        "json" => serde_json::to_string_pretty(&terms).map_err(|e| ReedError::ValidationError {
            field: "json".to_string(),
            value: "serialization".to_string(),
            constraint: e.to_string(),
        })?,
        "csv" => {
            let mut output = "term_id,term,category,description,status,usage_count\n".to_string();
            for term in &terms {
                output.push_str(&format!(
                    "{},{},{},{},{},{}\n",
                    term.term_id,
                    term.term,
                    term.category,
                    term.description.as_deref().unwrap_or(""),
                    term.status,
                    term.usage_count
                ));
            }
            output
        }
        _ => {
            if terms.is_empty() {
                format!("No terms found matching '{}'", query)
            } else {
                format!(
                    "Found {} term(s) matching '{}':\n\n{}",
                    terms.len(),
                    query,
                    format_terms_table(&terms)
                )
            }
        }
    };

    Ok(ReedResponse {
        data: output,
        source: response.source,
        cached: false,
        timestamp: response.timestamp,
        metrics: response.metrics,
    })
}

/// Updates a taxonomy term.
///
/// ## Usage
/// reed taxonomy:update <term_id> [--term <new_name>] [--parent <parent_id>] [flags]
///
/// ## Optional Flags
/// - --term: New term name
/// - --parent: New parent ID (use "none" to remove parent)
/// - --description: New description (use "none" to remove)
/// - --color: New color (use "none" to remove)
/// - --icon: New icon (use "none" to remove)
/// - --status: New status (active/inactive)
///
/// ## Example
/// ```bash
/// reed taxonomy:update "Programming:Rust" --description "Updated description"
/// reed taxonomy:update "Programming:Rust" --parent none
/// ```
pub fn update(
    args: &[String],
    flags: &HashMap<String, String>,
) -> ReedResult<ReedResponse<String>> {
    if args.is_empty() {
        return Err(ReedError::ValidationError {
            field: "term_id".to_string(),
            value: String::new(),
            constraint: "term_id required".to_string(),
        });
    }

    let term_id = &args[0];

    let mut term_update = TermUpdate::default();

    if let Some(term) = flags.get("term") {
        term_update.term = Some(term.clone());
    }

    if let Some(parent) = flags.get("parent") {
        term_update.parent_id = Some(if parent == "none" {
            None
        } else {
            Some(parent.clone())
        });
    }

    if let Some(description) = flags.get("description") {
        term_update.description = Some(if description == "none" {
            None
        } else {
            Some(description.clone())
        });
    }

    if let Some(color) = flags.get("color") {
        term_update.color = Some(if color == "none" {
            None
        } else {
            Some(color.clone())
        });
    }

    if let Some(icon) = flags.get("icon") {
        term_update.icon = Some(if icon == "none" {
            None
        } else {
            Some(icon.clone())
        });
    }

    if let Some(status) = flags.get("status") {
        term_update.status = Some(status.clone());
    }

    let response = update_term(term_id, term_update)?;
    let term = response.data;

    let output = format!(
        "Term updated: {}\nID: {}\nCategory: {}\nStatus: {}",
        term.term, term.term_id, term.category, term.status
    );

    Ok(ReedResponse {
        data: output,
        source: response.source,
        cached: false,
        timestamp: response.timestamp,
        metrics: response.metrics,
    })
}

/// Deletes a taxonomy term.
///
/// ## Usage
/// reed taxonomy:delete <term_id> [--force]
///
/// ## Optional Flags
/// - --force: Delete term and all children
///
/// ## Example
/// ```bash
/// reed taxonomy:delete "Programming:Rust"
/// reed taxonomy:delete "Programming" --force
/// ```
pub fn delete(
    args: &[String],
    flags: &HashMap<String, String>,
) -> ReedResult<ReedResponse<String>> {
    if args.is_empty() {
        return Err(ReedError::ValidationError {
            field: "term_id".to_string(),
            value: String::new(),
            constraint: "term_id required".to_string(),
        });
    }

    let term_id = &args[0];
    let force = flags.contains_key("force");

    let response = delete_term(term_id, force)?;

    let output = if force {
        format!("Term and children deleted: {}", term_id)
    } else {
        format!("Term deleted: {}", term_id)
    };

    Ok(ReedResponse {
        data: output,
        source: response.source,
        cached: false,
        timestamp: response.timestamp,
        metrics: response.metrics,
    })
}

/// Assigns taxonomy terms to an entity.
///
/// ## Usage
/// reed taxonomy:assign <entity_type>:<entity_id> <term_id> [<term_id> ...] [--assigned-by <user>]
///
/// ## Entity Types
/// user, content, template, route, site, project, asset, role
///
/// ## Optional Flags
/// - --assigned-by: User ID performing assignment (default: system)
///
/// ## Example
/// ```bash
/// reed taxonomy:assign content:post-123 "Programming:Rust" "Programming:Async"
/// reed taxonomy:assign user:admin "Role:Administrator"
/// ```
pub fn assign(
    args: &[String],
    flags: &HashMap<String, String>,
) -> ReedResult<ReedResponse<String>> {
    if args.len() < 2 {
        return Err(ReedError::ValidationError {
            field: "args".to_string(),
            value: String::new(),
            constraint:
                "usage: taxonomy:assign <entity_type>:<entity_id> <term_id> [<term_id> ...]"
                    .to_string(),
        });
    }

    // Parse entity_type:entity_id
    let entity_parts: Vec<&str> = args[0].split(':').collect();
    if entity_parts.len() != 2 {
        return Err(ReedError::ValidationError {
            field: "entity".to_string(),
            value: args[0].clone(),
            constraint: "must be <entity_type>:<entity_id>".to_string(),
        });
    }

    let entity_type = EntityType::from_str(entity_parts[0])?;
    let entity_id = entity_parts[1];

    let term_ids: Vec<String> = args[1..].iter().map(|s| s.clone()).collect();
    let assigned_by = flags
        .get("assigned-by")
        .map(|s| s.as_str())
        .unwrap_or("system");

    let response = assign_terms(entity_type, entity_id, term_ids.clone(), assigned_by)?;

    let output = format!(
        "Assigned {} term(s) to {}:{}\nTerms: {}",
        term_ids.len(),
        entity_type.as_str(),
        entity_id,
        term_ids.join(", ")
    );

    Ok(ReedResponse {
        data: output,
        source: response.source,
        cached: false,
        timestamp: response.timestamp,
        metrics: response.metrics,
    })
}

/// Removes taxonomy term assignments from an entity.
///
/// ## Usage
/// reed taxonomy:unassign <entity_type>:<entity_id> [<term_id> ...]
///
/// ## Example
/// ```bash
/// reed taxonomy:unassign content:post-123 "Programming:Rust"
/// reed taxonomy:unassign content:post-123
/// ```
pub fn unassign(args: &[String]) -> ReedResult<ReedResponse<String>> {
    if args.is_empty() {
        return Err(ReedError::ValidationError {
            field: "args".to_string(),
            value: String::new(),
            constraint: "usage: taxonomy:unassign <entity_type>:<entity_id> [<term_id> ...]"
                .to_string(),
        });
    }

    // Parse entity_type:entity_id
    let entity_parts: Vec<&str> = args[0].split(':').collect();
    if entity_parts.len() != 2 {
        return Err(ReedError::ValidationError {
            field: "entity".to_string(),
            value: args[0].clone(),
            constraint: "must be <entity_type>:<entity_id>".to_string(),
        });
    }

    let entity_type = EntityType::from_str(entity_parts[0])?;
    let entity_id = entity_parts[1];

    let term_ids = if args.len() > 1 {
        Some(args[1..].iter().map(|s| s.clone()).collect())
    } else {
        None
    };

    let response = unassign_terms(entity_type, entity_id, term_ids.clone())?;

    let output = if let Some(terms) = term_ids {
        format!(
            "Removed {} term(s) from {}:{}\nTerms: {}",
            terms.len(),
            entity_type.as_str(),
            entity_id,
            terms.join(", ")
        )
    } else {
        format!(
            "Removed all terms from {}:{}",
            entity_type.as_str(),
            entity_id
        )
    };

    Ok(ReedResponse {
        data: output,
        source: response.source,
        cached: false,
        timestamp: response.timestamp,
        metrics: response.metrics,
    })
}

/// Lists entities assigned to a taxonomy term.
///
/// ## Usage
/// reed taxonomy:entities <term_id> [--type <entity_type>] [--format <format>]
///
/// ## Optional Flags
/// - --type: Filter by entity type
/// - --format: Output format (table/json/csv, default: table)
///
/// ## Example
/// ```bash
/// reed taxonomy:entities "Programming:Rust"
/// reed taxonomy:entities "Programming:Rust" --type content
/// ```
pub fn entities(
    args: &[String],
    flags: &HashMap<String, String>,
) -> ReedResult<ReedResponse<String>> {
    if args.is_empty() {
        return Err(ReedError::ValidationError {
            field: "term_id".to_string(),
            value: String::new(),
            constraint: "term_id required".to_string(),
        });
    }

    let term_id = &args[0];
    let entity_type = if let Some(type_str) = flags.get("type") {
        Some(EntityType::from_str(type_str)?)
    } else {
        None
    };
    let format = flags.get("format").map(|s| s.as_str()).unwrap_or("table");

    let response = list_entities_by_term(term_id, entity_type)?;
    let entities = response.data;

    let output = match format {
        "json" => {
            serde_json::to_string_pretty(&entities).map_err(|e| ReedError::ValidationError {
                field: "json".to_string(),
                value: "serialization".to_string(),
                constraint: e.to_string(),
            })?
        }
        "csv" => {
            let mut output =
                "entity_type,entity_id,term_count,assigned_by,assigned_at\n".to_string();
            for entity in &entities {
                output.push_str(&format!(
                    "{},{},{},{},{}\n",
                    entity.entity_type.as_str(),
                    entity.entity_id,
                    entity.term_ids.len(),
                    entity.assigned_by,
                    entity.assigned_at
                ));
            }
            output
        }
        _ => {
            if entities.is_empty() {
                format!("No entities found with term '{}'", term_id)
            } else {
                format!(
                    "Found {} entit(ies) with term '{}':\n\n{}",
                    entities.len(),
                    term_id,
                    format_entities_table(&entities)
                )
            }
        }
    };

    Ok(ReedResponse {
        data: output,
        source: response.source,
        cached: false,
        timestamp: response.timestamp,
        metrics: response.metrics,
    })
}

/// Shows taxonomy usage statistics.
///
/// ## Usage
/// reed taxonomy:usage <term_id>
///
/// ## Example
/// ```bash
/// reed taxonomy:usage "Programming:Rust"
/// ```
pub fn usage(args: &[String]) -> ReedResult<ReedResponse<String>> {
    if args.is_empty() {
        return Err(ReedError::ValidationError {
            field: "term_id".to_string(),
            value: String::new(),
            constraint: "term_id required".to_string(),
        });
    }

    let term_id = &args[0];

    // Get term info
    let term_response = get_term(term_id)?;
    let term = term_response.data;

    // Get entities using this term
    let entities_response = list_entities_by_term(term_id, None)?;
    let entities = entities_response.data;

    // Group by entity type
    let mut type_counts: HashMap<String, usize> = HashMap::new();
    for entity in &entities {
        *type_counts
            .entry(entity.entity_type.as_str().to_string())
            .or_insert(0) += 1;
    }

    let mut output = format!(
        "Usage statistics for term '{}' ({})\n\nTotal usage count: {}\nTotal entities: {}\n\nBreakdown by entity type:\n",
        term.term, term.term_id, term.usage_count, entities.len()
    );

    for (entity_type, count) in type_counts {
        output.push_str(&format!("  {}: {}\n", entity_type, count));
    }

    Ok(ReedResponse {
        data: output,
        source: term_response.source,
        cached: false,
        timestamp: term_response.timestamp,
        metrics: term_response.metrics,
    })
}

// Helper functions for table formatting

fn format_terms_table(terms: &[TermInfo]) -> String {
    if terms.is_empty() {
        return "No terms found.".to_string();
    }

    let mut output = String::new();
    output.push_str("Term ID                  | Term              | Category      | Parent         | Status   | Usage\n");
    output.push_str("-------------------------|-------------------|---------------|----------------|----------|------\n");

    for term in terms {
        output.push_str(&format!(
            "{:<24} | {:<17} | {:<13} | {:<14} | {:<8} | {}\n",
            truncate(&term.term_id, 24),
            truncate(&term.term, 17),
            truncate(&term.category, 13),
            truncate(term.parent_id.as_deref().unwrap_or("-"), 14),
            term.status,
            term.usage_count
        ));
    }

    output
}

fn format_entities_table(entities: &[crate::reedcms::taxonomy::EntityTerms]) -> String {
    if entities.is_empty() {
        return "No entities found.".to_string();
    }

    let mut output = String::new();
    output.push_str(
        "Entity Type | Entity ID           | Term Count | Assigned By    | Assigned At\n",
    );
    output.push_str(
        "------------|---------------------|------------|----------------|--------------------\n",
    );

    for entity in entities {
        output.push_str(&format!(
            "{:<11} | {:<19} | {:<10} | {:<14} | {}\n",
            entity.entity_type.as_str(),
            truncate(&entity.entity_id, 19),
            entity.term_ids.len(),
            truncate(&entity.assigned_by, 14),
            truncate(&entity.assigned_at, 19)
        ));
    }

    output
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[0..max_len.saturating_sub(3)])
    }
}
