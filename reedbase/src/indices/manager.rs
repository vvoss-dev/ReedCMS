// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Index manager coordinating all indices and handling combined queries.

use crate::error::{ReedError, ReedResult};
use crate::indices::hierarchy::HierarchyTrie;
use crate::indices::modifier::ModifierIndex;
use crate::indices::namespace::NamespaceIndex;
use crate::indices::types::{KeyIndex, Modifiers, QueryFilter};
use crate::schema::rbks;
use crate::tables::Table;
use std::collections::HashSet;
use std::path::Path;

/// Index manager coordinating all indices.
pub struct IndexManager {
    namespace: NamespaceIndex,
    language: ModifierIndex,
    environment: ModifierIndex,
    season: ModifierIndex,
    variant: ModifierIndex,
    hierarchy: HierarchyTrie,
}

impl IndexManager {
    /// Create new index manager.
    pub fn new() -> Self {
        Self {
            namespace: NamespaceIndex::new(),
            language: ModifierIndex::language(),
            environment: ModifierIndex::environment(),
            season: ModifierIndex::season(),
            variant: ModifierIndex::variant(),
            hierarchy: HierarchyTrie::new(),
        }
    }

    /// Build all indices from table.
    ///
    /// ## Input
    /// - `base_path` - ReedBase directory path
    /// - `table_name` - Table to index (e.g., "text")
    ///
    /// ## Performance
    /// - O(n * d) where n = keys, d = average depth
    /// - < 50ms for 10,000 keys
    pub fn build(&mut self, base_path: &Path, table_name: &str) -> ReedResult<()> {
        let keys = self.parse_keys(base_path, table_name)?;

        self.namespace.build(&keys)?;
        self.language.build(&keys)?;
        self.environment.build(&keys)?;
        self.season.build(&keys)?;
        self.variant.build(&keys)?;
        self.hierarchy.build(&keys)?;

        Ok(())
    }

    /// Combined query with multiple filters.
    ///
    /// ## Input
    /// - `filter` - Query filter with optional conditions
    ///
    /// ## Output
    /// - Vec of row numbers matching ALL filters (set intersection)
    ///
    /// ## Performance
    /// - O(1) per index lookup + O(k) intersection where k = result size
    /// - < 50μs for typical queries
    pub fn query(&self, filter: &QueryFilter) -> ReedResult<Vec<usize>> {
        if filter.is_empty() {
            return Ok(Vec::new());
        }

        let mut result_sets: Vec<HashSet<usize>> = Vec::new();

        // Namespace filter
        if let Some(namespace) = &filter.namespace {
            if let Some(rows) = self.namespace.query(namespace) {
                result_sets.push(rows.iter().copied().collect());
            } else {
                return Ok(Vec::new()); // No matches
            }
        }

        // Language filter
        if let Some(language) = &filter.language {
            if let Some(rows) = self.language.query(language) {
                result_sets.push(rows.iter().copied().collect());
            } else {
                return Ok(Vec::new());
            }
        }

        // Environment filter
        if let Some(environment) = &filter.environment {
            if let Some(rows) = self.environment.query(environment) {
                result_sets.push(rows.iter().copied().collect());
            } else {
                return Ok(Vec::new());
            }
        }

        // Season filter
        if let Some(season) = &filter.season {
            if let Some(rows) = self.season.query(season) {
                result_sets.push(rows.iter().copied().collect());
            } else {
                return Ok(Vec::new());
            }
        }

        // Variant filter
        if let Some(variant) = &filter.variant {
            if let Some(rows) = self.variant.query(variant) {
                result_sets.push(rows.iter().copied().collect());
            } else {
                return Ok(Vec::new());
            }
        }

        // Hierarchy filter
        if let Some(pattern) = &filter.hierarchy_pattern {
            let rows = self.hierarchy.query(pattern);
            if rows.is_empty() {
                return Ok(Vec::new());
            }
            result_sets.push(rows.into_iter().collect());
        }

        // Intersection of all filters
        if result_sets.is_empty() {
            return Ok(Vec::new());
        }

        let mut intersection = result_sets[0].clone();
        for set in &result_sets[1..] {
            intersection = intersection.intersection(set).copied().collect();
        }

        let mut result: Vec<usize> = intersection.into_iter().collect();
        result.sort_unstable();

        Ok(result)
    }

    /// Insert new key into all indices.
    pub fn insert(&mut self, key_index: &KeyIndex) -> ReedResult<()> {
        self.namespace.insert(key_index)?;
        self.language.insert(key_index)?;
        self.environment.insert(key_index)?;
        self.season.insert(key_index)?;
        self.variant.insert(key_index)?;
        self.hierarchy.insert(key_index)?;

        Ok(())
    }

    /// Remove key at specific row from all indices.
    pub fn remove(&mut self, row: usize) -> ReedResult<()> {
        self.namespace.remove(row)?;
        self.language.remove(row)?;
        self.environment.remove(row)?;
        self.season.remove(row)?;
        self.variant.remove(row)?;
        self.hierarchy.remove(row)?;

        Ok(())
    }

    /// Get total memory usage of all indices.
    pub fn memory_usage(&self) -> usize {
        self.namespace.memory_usage()
            + self.language.memory_usage()
            + self.environment.memory_usage()
            + self.season.memory_usage()
            + self.variant.memory_usage()
            + self.hierarchy.memory_usage()
    }

    /// Clear all indices.
    pub fn clear(&mut self) {
        self.namespace.clear();
        self.language.clear();
        self.environment.clear();
        self.season.clear();
        self.variant.clear();
        self.hierarchy.clear();
    }

    /// Parse all keys from table into KeyIndex structures.
    fn parse_keys(&self, base_path: &Path, table_name: &str) -> ReedResult<Vec<KeyIndex>> {
        let table = Table::new(base_path, table_name);
        let content = table.read_current().map_err(|_| ReedError::TableNotFound {
            name: table_name.to_string(),
        })?;

        let rows = crate::tables::parse_csv(&content)?;
        let mut keys = Vec::new();

        for (row_num, row) in rows.iter().enumerate() {
            // Skip empty rows
            if row.key.is_empty() {
                continue;
            }

            // Parse key
            match self.parse_key(&row.key, row_num) {
                Ok(key_index) => keys.push(key_index),
                Err(_) => {
                    // Skip malformed keys (don't fail entire index build)
                    continue;
                }
            }
        }

        Ok(keys)
    }

    /// Parse single key into KeyIndex.
    fn parse_key(&self, key: &str, row: usize) -> ReedResult<KeyIndex> {
        // Use RBKS v2 parser
        let parsed = rbks::parse_key(key)?;

        // Split base into hierarchy segments (e.g., "page.header.title" → ["page", "header", "title"])
        let hierarchy: Vec<String> = parsed.base.split('.').map(|s| s.to_string()).collect();

        // Extract namespace (first hierarchy segment)
        let namespace = hierarchy
            .first()
            .cloned()
            .unwrap_or_else(|| "unknown".to_string());

        // Convert parsed modifiers
        let modifiers = Modifiers {
            language: parsed.modifiers.language,
            environment: parsed.modifiers.environment,
            season: parsed.modifiers.season,
            variant: parsed.modifiers.variant,
        };

        Ok(KeyIndex {
            row,
            key: key.to_string(),
            namespace,
            hierarchy,
            modifiers,
        })
    }

    /// Get index statistics.
    pub fn stats(&self) -> IndexStats {
        IndexStats {
            total_keys: self.namespace.key_count(),
            namespaces: self.namespace.namespace_count(),
            languages: self.language.value_count(),
            environments: self.environment.value_count(),
            seasons: self.season.value_count(),
            variants: self.variant.value_count(),
            trie_nodes: self.hierarchy.node_count(),
            memory_bytes: self.memory_usage(),
        }
    }
}

impl Default for IndexManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Index statistics.
#[derive(Debug, Clone)]
pub struct IndexStats {
    pub total_keys: usize,
    pub namespaces: usize,
    pub languages: usize,
    pub environments: usize,
    pub seasons: usize,
    pub variants: usize,
    pub trie_nodes: usize,
    pub memory_bytes: usize,
}

impl IndexStats {
    /// Format as human-readable string.
    pub fn to_string(&self) -> String {
        format!(
            "Index Statistics:\n\
             Total keys: {}\n\
             Namespaces: {}\n\
             Languages: {}\n\
             Environments: {}\n\
             Seasons: {}\n\
             Variants: {}\n\
             Trie nodes: {}\n\
             Memory: {:.2} MB",
            self.total_keys,
            self.namespaces,
            self.languages,
            self.environments,
            self.seasons,
            self.variants,
            self.trie_nodes,
            self.memory_bytes as f64 / 1_048_576.0
        )
    }
}
