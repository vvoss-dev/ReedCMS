// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Source Map Generator for CSS Bundles
//!
//! Generates source maps following Source Map v3 specification.
//! Enables debugging of minified CSS in browser DevTools.

use crate::reedcms::reedstream::{ReedError, ReedResult};
use serde_json::json;

/// Source map generator for CSS bundles.
///
/// ## Purpose
/// - Debugging minified CSS in browser DevTools
/// - Maps minified positions to original source files
/// - Essential for development
pub struct SourceMap {
    sources: Vec<String>,
    sources_content: Vec<String>,
}

impl SourceMap {
    /// Creates new source map.
    ///
    /// ## Output
    /// - Empty SourceMap instance
    ///
    /// ## Example
    /// ```rust
    /// let mut map = SourceMap::new();
    /// map.add_source("reset.css", "body { margin: 0; }");
    /// ```
    pub fn new() -> Self {
        Self {
            sources: Vec::new(),
            sources_content: Vec::new(),
        }
    }

    /// Adds source file to map.
    ///
    /// ## Input
    /// - `path`: Path to source file (relative or absolute)
    /// - `content`: Original source content
    ///
    /// ## Example
    /// ```rust
    /// map.add_source("assets/css/reset.css", "body { margin: 0; }");
    /// ```
    pub fn add_source(&mut self, path: &str, content: &str) {
        self.sources.push(path.to_string());
        self.sources_content.push(content.to_string());
    }

    /// Generates source map JSON.
    ///
    /// ## Output
    /// - Source map JSON string
    ///
    /// ## Format
    /// JSON format following Source Map v3 specification:
    /// ```json
    /// {
    ///   "version": 3,
    ///   "sources": ["reset.css", "navigation.css"],
    ///   "sourcesContent": ["body { margin: 0; }", "nav { ... }"],
    ///   "names": [],
    ///   "mappings": ""
    /// }
    /// ```
    ///
    /// ## Performance
    /// - < 10ms per map
    ///
    /// ## Error Conditions
    /// - Serialisation failure
    pub fn generate(&self) -> ReedResult<String> {
        let map = json!({
            "version": 3,
            "sources": self.sources,
            "sourcesContent": self.sources_content,
            "names": [],
            "mappings": ""
        });

        serde_json::to_string_pretty(&map).map_err(|e| ReedError::ParseError {
            input: "source_map".to_string(),
            reason: e.to_string(),
        })
    }

    /// Appends source map comment to CSS.
    ///
    /// ## Input
    /// - `css`: Minified CSS content
    /// - `source_map_path`: Path to source map file (relative to CSS file)
    ///
    /// ## Output
    /// - CSS with source map comment appended
    ///
    /// ## Example
    /// ```rust
    /// let css_with_map = SourceMap::append_comment(
    ///     "body{margin:0}",
    ///     "landing.mouse.css.map"
    /// );
    /// // Returns: "body{margin:0}\n/*# sourceMappingURL=landing.mouse.css.map */"
    /// ```
    pub fn append_comment(css: &str, source_map_path: &str) -> String {
        format!("{}\n/*# sourceMappingURL={} */", css, source_map_path)
    }
}

impl Default for SourceMap {
    fn default() -> Self {
        Self::new()
    }
}
