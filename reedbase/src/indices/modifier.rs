// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Modifier indices for language, environment, season, and variant lookups.
//!
//! Generic implementation for all modifier-based indices with O(1) HashMap lookups.

use crate::error::ReedResult;
use crate::indices::types::KeyIndex;
use std::collections::HashMap;

/// Generic modifier index for O(1) lookups.
///
/// Used for language, environment, season, and variant indices.
pub struct ModifierIndex {
    /// modifier_value → Vec<row_numbers>
    map: HashMap<String, Vec<usize>>,
    /// Function to extract modifier from KeyIndex
    extractor: fn(&KeyIndex) -> Option<String>,
}

impl ModifierIndex {
    /// Create new modifier index with custom extractor.
    pub fn new(extractor: fn(&KeyIndex) -> Option<String>) -> Self {
        Self {
            map: HashMap::new(),
            extractor,
        }
    }

    /// Create language index.
    pub fn language() -> Self {
        Self::new(|k| k.modifiers.language.clone())
    }

    /// Create environment index.
    pub fn environment() -> Self {
        Self::new(|k| k.modifiers.environment.clone())
    }

    /// Create season index.
    pub fn season() -> Self {
        Self::new(|k| k.modifiers.season.clone())
    }

    /// Create variant index.
    pub fn variant() -> Self {
        Self::new(|k| k.modifiers.variant.clone())
    }

    /// Build index from key indices.
    pub fn build(&mut self, keys: &[KeyIndex]) -> ReedResult<()> {
        self.map.clear();

        for key_index in keys {
            if let Some(value) = (self.extractor)(key_index) {
                self.map
                    .entry(value)
                    .or_insert_with(Vec::new)
                    .push(key_index.row);
            }
        }

        Ok(())
    }

    /// Query index for a modifier value.
    pub fn query(&self, value: &str) -> Option<&[usize]> {
        self.map.get(value).map(|v| v.as_slice())
    }

    /// Insert a new key into the index.
    pub fn insert(&mut self, key_index: &KeyIndex) -> ReedResult<()> {
        if let Some(value) = (self.extractor)(key_index) {
            self.map
                .entry(value)
                .or_insert_with(Vec::new)
                .push(key_index.row);
        }

        Ok(())
    }

    /// Remove a key at specific row.
    pub fn remove(&mut self, row: usize) -> ReedResult<()> {
        for rows in self.map.values_mut() {
            rows.retain(|&r| r != row);
        }

        Ok(())
    }

    /// Get number of unique values.
    pub fn value_count(&self) -> usize {
        self.map.len()
    }

    /// Get total number of indexed keys.
    pub fn key_count(&self) -> usize {
        self.map.values().map(|v| v.len()).sum()
    }

    /// Calculate memory usage.
    pub fn memory_usage(&self) -> usize {
        self.map
            .iter()
            .map(|(key, values)| key.len() + 24 + (values.len() * 8) + 24)
            .sum()
    }

    /// Clear the index.
    pub fn clear(&mut self) {
        self.map.clear();
    }
}
