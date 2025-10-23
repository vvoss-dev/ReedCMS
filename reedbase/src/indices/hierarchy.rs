// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Hierarchy trie for O(d) hierarchical wildcard queries.
//!
//! Supports patterns like `page.header.*` with efficient trie traversal.

use crate::error::ReedResult;
use crate::indices::types::KeyIndex;
use std::collections::HashMap;

/// Trie node for hierarchical indexing.
#[derive(Debug, Default)]
pub struct TrieNode {
    /// Row numbers at this exact path
    pub rows: Vec<usize>,
    /// Children: segment → TrieNode
    pub children: HashMap<String, TrieNode>,
}

/// Hierarchy trie for O(d) hierarchical queries.
pub struct HierarchyTrie {
    root: TrieNode,
}

impl HierarchyTrie {
    /// Create new hierarchy trie.
    pub fn new() -> Self {
        Self {
            root: TrieNode::default(),
        }
    }

    /// Build trie from key indices.
    ///
    /// ## Performance
    /// - O(n * d) where n = keys, d = average depth
    pub fn build(&mut self, keys: &[KeyIndex]) -> ReedResult<()> {
        self.root = TrieNode::default();

        for key_index in keys {
            let mut node = &mut self.root;

            for segment in &key_index.hierarchy {
                node = node
                    .children
                    .entry(segment.clone())
                    .or_insert_with(TrieNode::default);
            }

            node.rows.push(key_index.row);
        }

        Ok(())
    }

    /// Query with wildcard support.
    ///
    /// ## Input
    /// - `pattern` - Path segments, "*" for wildcard (e.g., ["page", "header", "*"])
    ///
    /// ## Output
    /// - Vec of matching row numbers
    ///
    /// ## Performance
    /// - O(d) where d = pattern depth
    /// - Wildcard (*) collects all descendants: O(d * m) where m = descendants
    pub fn query(&self, pattern: &[String]) -> Vec<usize> {
        if pattern.is_empty() {
            return Vec::new();
        }

        self.query_recursive(&self.root, pattern, 0)
    }

    fn query_recursive(&self, node: &TrieNode, pattern: &[String], depth: usize) -> Vec<usize> {
        if depth >= pattern.len() {
            return Vec::new();
        }

        let segment = &pattern[depth];

        // Wildcard: collect all descendants
        if segment == "*" {
            return self.collect_all_descendants(node);
        }

        // Exact match
        if let Some(child) = node.children.get(segment) {
            if depth == pattern.len() - 1 {
                // Last segment
                if pattern.len() == 1 || segment != "*" {
                    // Return rows at this exact node
                    return child.rows.clone();
                }
            }
            // Continue traversal
            return self.query_recursive(child, pattern, depth + 1);
        }

        Vec::new()
    }

    fn collect_all_descendants(&self, node: &TrieNode) -> Vec<usize> {
        let mut result = node.rows.clone();

        for child in node.children.values() {
            result.extend(self.collect_all_descendants(child));
        }

        result
    }

    /// Insert new key into trie.
    pub fn insert(&mut self, key_index: &KeyIndex) -> ReedResult<()> {
        let mut node = &mut self.root;

        for segment in &key_index.hierarchy {
            node = node
                .children
                .entry(segment.clone())
                .or_insert_with(TrieNode::default);
        }

        node.rows.push(key_index.row);

        Ok(())
    }

    /// Remove key at specific row.
    pub fn remove(&mut self, row: usize) -> ReedResult<()> {
        Self::remove_recursive(&mut self.root, row);
        Ok(())
    }

    fn remove_recursive(node: &mut TrieNode, row: usize) {
        node.rows.retain(|&r| r != row);

        for child in node.children.values_mut() {
            Self::remove_recursive(child, row);
        }
    }

    /// Calculate memory usage.
    pub fn memory_usage(&self) -> usize {
        self.node_memory(&self.root)
    }

    fn node_memory(&self, node: &TrieNode) -> usize {
        let mut size = node.rows.len() * 8 + 24; // Vec overhead

        for (key, child) in &node.children {
            size += key.len() + 24; // String
            size += self.node_memory(child);
        }

        size
    }

    /// Clear the trie.
    pub fn clear(&mut self) {
        self.root = TrieNode::default();
    }

    /// Get total number of nodes in trie.
    pub fn node_count(&self) -> usize {
        self.count_nodes(&self.root)
    }

    fn count_nodes(&self, node: &TrieNode) -> usize {
        1 + node
            .children
            .values()
            .map(|c| self.count_nodes(c))
            .sum::<usize>()
    }
}

impl Default for HierarchyTrie {
    fn default() -> Self {
        Self::new()
    }
}
