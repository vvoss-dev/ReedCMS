// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! JavaScript Dependency Resolver
//!
//! Resolves JavaScript module dependencies for both ES6 and CommonJS formats.
//! Builds dependency graph and returns modules in topological order.

use crate::reedcms::reedstream::{ReedError, ReedResult};
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

/// Module structure representing a JavaScript file and its dependencies.
#[derive(Debug, Clone)]
pub struct Module {
    pub path: String,
    pub content: String,
    pub dependencies: Vec<String>,
}

/// Dependency resolver for JavaScript modules.
///
/// ## Supported Import Formats
/// - ES6: `import { func } from './module.js'`
/// - ES6 default: `import module from './module.js'`
/// - CommonJS: `const module = require('./module.js')`
///
/// ## Resolution Strategy
/// 1. Parse import statements from entry point
/// 2. Resolve relative paths to absolute paths
/// 3. Load module contents
/// 4. Recursively resolve dependencies
/// 5. Build dependency graph
/// 6. Return modules in topological order (dependencies first)
pub struct DependencyResolver {
    modules: HashMap<String, Module>,
    visited: HashSet<String>,
    base_path: PathBuf,
}

impl DependencyResolver {
    /// Creates new dependency resolver with base path.
    ///
    /// ## Input
    /// - `base_path`: Base directory for resolving relative imports
    ///
    /// ## Example
    /// ```rust
    /// let resolver = DependencyResolver::new("templates/");
    /// ```
    pub fn new<P: AsRef<Path>>(base_path: P) -> Self {
        Self {
            modules: HashMap::new(),
            visited: HashSet::new(),
            base_path: base_path.as_ref().to_path_buf(),
        }
    }

    /// Adds entry point module to resolver.
    ///
    /// ## Input
    /// - `path`: Path to entry point file
    /// - `content`: Content of entry point file
    ///
    /// ## Example
    /// ```rust
    /// resolver.add_entry("layouts/landing/landing.js", &content)?;
    /// ```
    pub fn add_entry(&mut self, path: &str, content: &str) -> ReedResult<()> {
        let module = Module {
            path: path.to_string(),
            content: content.to_string(),
            dependencies: Vec::new(),
        };

        self.modules.insert(path.to_string(), module);
        Ok(())
    }

    /// Resolves all dependencies and returns modules in topological order.
    ///
    /// ## Output
    /// - Vec of Module in dependency order (dependencies first, entry last)
    ///
    /// ## Performance
    /// - < 50ms for typical module graph
    ///
    /// ## Error Conditions
    /// - Circular dependencies
    /// - Module not found
    /// - Invalid import syntax
    pub fn resolve(&mut self) -> ReedResult<Vec<Module>> {
        let entry_keys: Vec<String> = self.modules.keys().cloned().collect();

        for entry in &entry_keys {
            self.resolve_module(entry)?;
        }

        // Sort modules in dependency order
        Ok(self.topological_sort()?)
    }

    /// Resolves single module and its dependencies recursively.
    ///
    /// ## Process
    /// 1. Mark module as visited (prevent infinite recursion)
    /// 2. Parse import statements from content
    /// 3. Resolve import paths to absolute paths
    /// 4. Load dependency content if not already loaded
    /// 5. Recursively resolve dependency's dependencies
    fn resolve_module(&mut self, path: &str) -> ReedResult<()> {
        if self.visited.contains(path) {
            return Ok(());
        }

        self.visited.insert(path.to_string());

        let module = self
            .modules
            .get(path)
            .cloned()
            .ok_or_else(|| ReedError::NotFound {
                resource: format!("module: {}", path),
                context: Some("dependency resolver".to_string()),
            })?;

        // Parse imports from module content
        let imports = parse_imports(&module.content)?;

        for import_path in imports {
            let resolved_path = resolve_import_path(path, &import_path, &self.base_path)?;

            // Load dependency if not already loaded
            if !self.modules.contains_key(&resolved_path) {
                let content =
                    fs::read_to_string(&resolved_path).map_err(|e| ReedError::IoError {
                        operation: "read".to_string(),
                        path: resolved_path.clone(),
                        reason: e.to_string(),
                    })?;

                let dep_module = Module {
                    path: resolved_path.clone(),
                    content,
                    dependencies: Vec::new(),
                };

                self.modules.insert(resolved_path.clone(), dep_module);
            }

            // Add to dependency list
            if let Some(module) = self.modules.get_mut(path) {
                if !module.dependencies.contains(&resolved_path) {
                    module.dependencies.push(resolved_path.clone());
                }
            }

            // Recursively resolve dependencies
            self.resolve_module(&resolved_path)?;
        }

        Ok(())
    }

    /// Sorts modules in topological order using depth-first search.
    ///
    /// ## Output
    /// - Modules sorted such that dependencies appear before dependents
    ///
    /// ## Error Conditions
    /// - Circular dependencies detected
    fn topological_sort(&self) -> ReedResult<Vec<Module>> {
        let mut sorted = Vec::new();
        let mut visited = HashSet::new();
        let mut temp_mark = HashSet::new();

        for path in self.modules.keys() {
            if !visited.contains(path) {
                self.visit_module(path, &mut visited, &mut temp_mark, &mut sorted)?;
            }
        }

        Ok(sorted)
    }

    /// Visits module in depth-first search for topological sort.
    fn visit_module(
        &self,
        path: &str,
        visited: &mut HashSet<String>,
        temp_mark: &mut HashSet<String>,
        sorted: &mut Vec<Module>,
    ) -> ReedResult<()> {
        if temp_mark.contains(path) {
            return Err(ReedError::ConfigError {
                component: "dependency_resolver".to_string(),
                reason: format!("Circular dependency detected involving: {}", path),
            });
        }

        if visited.contains(path) {
            return Ok(());
        }

        temp_mark.insert(path.to_string());

        if let Some(module) = self.modules.get(path) {
            for dep in &module.dependencies {
                self.visit_module(dep, visited, temp_mark, sorted)?;
            }

            sorted.push(module.clone());
        }

        temp_mark.remove(path);
        visited.insert(path.to_string());

        Ok(())
    }
}

/// Parses import statements from JavaScript content.
///
/// ## Input
/// - `content`: JavaScript source code
///
/// ## Output
/// - Vec of import paths
///
/// ## Patterns Matched
/// - ES6: `import { x } from './module.js'`
/// - ES6 default: `import x from './module.js'`
/// - CommonJS: `const x = require('./module.js')`
///
/// ## Example
/// ```rust
/// let imports = parse_imports("import { foo } from './utils.js';")?;
/// assert_eq!(imports, vec!["./utils.js"]);
/// ```
pub fn parse_imports(content: &str) -> ReedResult<Vec<String>> {
    let mut imports = Vec::new();

    // ES6 imports: import ... from '...'
    let es6_re = Regex::new(r#"import\s+.*\s+from\s+['"]([^'"]+)['"]"#).map_err(|e| {
        ReedError::ParseError {
            input: "ES6 import regex".to_string(),
            reason: e.to_string(),
        }
    })?;

    for cap in es6_re.captures_iter(content) {
        imports.push(cap[1].to_string());
    }

    // CommonJS requires: require('...')
    let cjs_re = Regex::new(r#"require\s*\(\s*['"]([^'"]+)['"]\s*\)"#).map_err(|e| {
        ReedError::ParseError {
            input: "CommonJS require regex".to_string(),
            reason: e.to_string(),
        }
    })?;

    for cap in cjs_re.captures_iter(content) {
        imports.push(cap[1].to_string());
    }

    Ok(imports)
}

/// Resolves import path relative to importing file.
///
/// ## Input
/// - `current_path`: Path to file containing the import
/// - `import_path`: Relative import path from import statement
/// - `base_path`: Base directory for resolution
///
/// ## Output
/// - Absolute path to imported module
///
/// ## Examples
/// - Current: `templates/layouts/blog/blog.js`
/// - Import: `../core/utils.js`
/// - Result: `templates/core/utils.js`
///
/// ## Error Conditions
/// - Cannot resolve parent directory
/// - Path not found
pub fn resolve_import_path(
    current_path: &str,
    import_path: &str,
    base_path: &Path,
) -> ReedResult<String> {
    let current = Path::new(current_path);

    let parent = current.parent().ok_or_else(|| ReedError::ConfigError {
        component: "path_resolution".to_string(),
        reason: format!("Cannot resolve parent of {}", current_path),
    })?;

    let resolved = parent.join(import_path);

    // Try to canonicalize, but if it fails, use the joined path
    let final_path = match resolved.canonicalize() {
        Ok(canonical) => canonical,
        Err(_) => {
            // If canonicalize fails, try relative to base_path
            let relative_to_base = base_path.join(import_path);
            match relative_to_base.canonicalize() {
                Ok(canonical) => canonical,
                Err(_) => resolved, // Use joined path as fallback
            }
        }
    };

    Ok(final_path.display().to_string())
}
