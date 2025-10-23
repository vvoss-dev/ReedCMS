// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Core types for Smart Indices system.
//!
//! Defines structures for index entries, modifiers, and query filters
//! used across all index implementations.

/// Index entry pointing to a key in current.csv.
///
/// Contains parsed components of RBKS v2 keys for efficient indexing.
#[derive(Debug, Clone)]
pub struct KeyIndex {
    /// Row number in CSV (0-based, includes header)
    pub row: usize,

    /// Full key string (for verification)
    pub key: String,

    /// Namespace (first segment before dot)
    pub namespace: String,

    /// Full hierarchy path (e.g., ["page", "header", "logo"])
    pub hierarchy: Vec<String>,

    /// Parsed modifiers from angle brackets
    pub modifiers: Modifiers,
}

/// Parsed modifiers from RBKS v2 key angle brackets.
///
/// Format: `namespace.hierarchy<language,environment,season,variant>`
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Modifiers {
    /// Language code (e.g., "de", "en", "fr")
    pub language: Option<String>,

    /// Environment (e.g., "dev", "prod")
    pub environment: Option<String>,

    /// Season (e.g., "christmas", "easter")
    pub season: Option<String>,

    /// Variant (e.g., "mouse", "touch", "reader")
    pub variant: Option<String>,
}

/// Query filter for index lookups.
///
/// Supports filtering by namespace, language, environment, and hierarchical patterns.
#[derive(Debug, Default, Clone)]
pub struct QueryFilter {
    /// Filter by namespace (e.g., "page")
    pub namespace: Option<String>,

    /// Filter by language (e.g., "de")
    pub language: Option<String>,

    /// Filter by environment (e.g., "prod")
    pub environment: Option<String>,

    /// Filter by season (e.g., "christmas")
    pub season: Option<String>,

    /// Filter by variant (e.g., "mouse")
    pub variant: Option<String>,

    /// Hierarchical pattern with wildcards (e.g., ["page", "header", "*"])
    pub hierarchy_pattern: Option<Vec<String>>,
}

impl QueryFilter {
    /// Create a new empty query filter.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add namespace filter.
    pub fn with_namespace(mut self, namespace: impl Into<String>) -> Self {
        self.namespace = Some(namespace.into());
        self
    }

    /// Add language filter.
    pub fn with_language(mut self, language: impl Into<String>) -> Self {
        self.language = Some(language.into());
        self
    }

    /// Add environment filter.
    pub fn with_environment(mut self, environment: impl Into<String>) -> Self {
        self.environment = Some(environment.into());
        self
    }

    /// Add season filter.
    pub fn with_season(mut self, season: impl Into<String>) -> Self {
        self.season = Some(season.into());
        self
    }

    /// Add variant filter.
    pub fn with_variant(mut self, variant: impl Into<String>) -> Self {
        self.variant = Some(variant.into());
        self
    }

    /// Add hierarchy pattern filter.
    pub fn with_hierarchy(mut self, pattern: Vec<String>) -> Self {
        self.hierarchy_pattern = Some(pattern);
        self
    }

    /// Check if filter has any conditions.
    pub fn is_empty(&self) -> bool {
        self.namespace.is_none()
            && self.language.is_none()
            && self.environment.is_none()
            && self.season.is_none()
            && self.variant.is_none()
            && self.hierarchy_pattern.is_none()
    }
}
