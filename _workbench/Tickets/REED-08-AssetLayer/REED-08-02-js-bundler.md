# REED-08-02: JavaScript Bundler and Minifier

## MANDATORY Development Standards

**CRITICAL**: Every implementation MUST follow these standards:

- **Language**: All documentation and code comments in BBC English
- **Principle**: KISS (Keep It Simple, Stupid) - minimal code and professional inline documentation
- **File Naming**: File name = Unique theme - crystal clear what single topic this file handles
- **Files**: One file = One responsibility
- **Functions**: One function = One distinctive job
- **Shared Functions**: Same patterns = One shared function (CONFIG and SYSTEM use identical logic)
- **Testing**: Separate test files as `{name}.test.rs` - never inline `#[cfg(test)]` modules
- **Avoid**: Avoid Swiss Army knife functions
- **Avoid**: Generic file names like `handler.rs`, `middleware.rs`, `utils.rs` - every topic (auth, login,...) has its own scoped rs service file
- **Templates**: See `_workbench/Tickets/templates/service-template.md` for complete implementation guide
- **Testing**: See `_workbench/Tickets/templates/service-template.test.md` for test structure

## Ticket Information
- **ID**: REED-08-02
- **Title**: JavaScript Bundler and Minifier
- **Layer**: Asset Layer (REED-08)
- **Priority**: Medium
- **Status**: Open
- **Complexity**: Medium
- **Dependencies**: None

## Summary Reference
- **Section**: JS Bundler
- **Lines**: 1028-1030 in project_summary.md
- **Key Concepts**: JavaScript bundling, minification, tree shaking, module resolution

## Objective
Implement JavaScript bundler that combines multiple JS modules per layout variant, minifies output, performs tree shaking to remove unused code, and generates optimised bundles for production deployment.

## Requirements

### JavaScript Directory Structure
```
assets/js/
├── core/
│   ├── reedapi.js          # ReedAPI client
│   ├── utils.js            # Utility functions
│   └── events.js           # Event system
├── layouts/
│   ├── knowledge/
│   │   ├── knowledge.mouse.js
│   │   ├── knowledge.touch.js
│   │   └── knowledge.reader.js
│   └── blog/
│       ├── blog.mouse.js
│       ├── blog.touch.js
│       └── blog.reader.js
└── components/
    ├── navigation.js
    ├── search.js
    └── forms.js
```

### Output Structure
```
public/js/
├── knowledge.mouse.js       # Bundled and minified
├── knowledge.mouse.js.map   # Source map
├── knowledge.touch.js
├── knowledge.touch.js.map
├── blog.mouse.js
└── blog.mouse.js.map
```

### Implementation (`src/reedcms/assets/js/bundler.rs`)

```rust
/// Bundles and minifies JavaScript files for layouts.
///
/// ## Process
/// 1. Discover layout JS files
/// 2. Resolve import/require statements
/// 3. Combine modules in dependency order
/// 4. Perform tree shaking (remove unused exports)
/// 5. Minify JavaScript
/// 6. Generate source maps
/// 7. Write to public/js/
///
/// ## Module Resolution
/// - ES6 imports: import { func } from './module.js'
/// - CommonJS: const module = require('./module.js')
/// - Relative paths resolved to assets/js/
///
/// ## Performance
/// - Bundle time: < 1s for 10 layouts
/// - Minification: ~60% size reduction
/// - Tree shaking: ~20% additional reduction
///
/// ## Output
/// ```
/// Bundling JS for knowledge.mouse...
/// - Included: reedapi.js (12.3 KB)
/// - Included: utils.js (4.2 KB)
/// - Included: navigation.js (6.1 KB)
/// - Included: knowledge.mouse.js (8.4 KB)
/// → Output: public/js/knowledge.mouse.js (9.2 KB, -70%)
/// ✓ Bundle complete
/// ```
pub fn bundle_js(layout: &str, variant: &str) -> ReedResult<BundleResult> {
    println!("📦 Bundling JS for {}.{}...", layout, variant);

    // 1. Load entry point
    let entry_point = format!("assets/js/layouts/{}/{}.{}.js", layout, layout, variant);
    let entry_content = std::fs::read_to_string(&entry_point).map_err(|e| ReedError::IoError {
        operation: "read".to_string(),
        path: entry_point.clone(),
        reason: e.to_string(),
    })?;

    // 2. Resolve dependencies
    let mut resolver = DependencyResolver::new();
    resolver.add_entry(&entry_point, &entry_content)?;

    let modules = resolver.resolve()?;

    // 3. Combine modules
    let mut combined_js = String::new();
    let mut source_map = SourceMap::new();

    for module in &modules {
        println!("  - Included: {} ({} KB)", module.path, module.content.len() / 1024);
        source_map.add_source(&module.path);
        combined_js.push_str(&wrap_module(&module.path, &module.content));
        combined_js.push('\n');
    }

    let original_size = combined_js.len();

    // 4. Tree shaking
    let shaken = tree_shake(&combined_js, &modules)?;
    let shaken_size = shaken.len();

    // 5. Minify JavaScript
    let minified = minify_js(&shaken)?;
    let minified_size = minified.len();

    // 6. Generate source map
    let source_map_content = source_map.generate()?;

    // 7. Write output files
    let output_path = format!("public/js/{}.{}.js", layout, variant);
    let source_map_path = format!("{}.map", output_path);

    write_js_file(&output_path, &minified)?;
    write_source_map(&source_map_path, &source_map_content)?;

    let reduction = ((original_size - minified_size) as f64 / original_size as f64 * 100.0) as u32;

    println!(
        "  → Output: {} ({} KB, -{}%)",
        output_path,
        minified_size / 1024,
        reduction
    );
    println!("✓ Bundle complete");

    Ok(BundleResult {
        output_path,
        original_size,
        shaken_size,
        minified_size,
        reduction_percent: reduction,
    })
}

/// Bundles JS for all layouts and variants.
pub fn bundle_all_js() -> ReedResult<Vec<BundleResult>> {
    let layouts = discover_layouts()?;
    let variants = vec!["mouse", "touch", "reader"];

    let mut results = Vec::new();
    let mut total_original = 0;
    let mut total_minified = 0;

    for layout in layouts {
        for variant in &variants {
            match bundle_js(&layout, variant) {
                Ok(result) => {
                    total_original += result.original_size;
                    total_minified += result.minified_size;
                    results.push(result);
                }
                Err(e) => {
                    eprintln!("⚠ Failed to bundle {}.{}: {:?}", layout, variant, e);
                }
            }
        }
    }

    let total_reduction =
        ((total_original - total_minified) as f64 / total_original as f64 * 100.0) as u32;

    println!("\n📊 Total JS Bundle Statistics:");
    println!("  Original size: {} KB", total_original / 1024);
    println!("  Minified size: {} KB", total_minified / 1024);
    println!("  Size reduction: {}%", total_reduction);

    Ok(results)
}

/// Bundle result structure.
#[derive(Debug, Clone)]
pub struct BundleResult {
    pub output_path: String,
    pub original_size: usize,
    pub shaken_size: usize,
    pub minified_size: usize,
    pub reduction_percent: u32,
}

/// Wraps module in IIFE to prevent global scope pollution.
///
/// ## Example
/// ```js
/// // Input
/// export function foo() { return 42; }
///
/// // Output
/// (function(exports) {
///   function foo() { return 42; }
///   exports.foo = foo;
/// })(window.modules = window.modules || {});
/// ```
fn wrap_module(path: &str, content: &str) -> String {
    format!(
        "(function(module, exports) {{\n{}\n}})({{exports: {{}}}}, {{}});\n",
        content
    )
}

/// Discovers all layouts from templates directory.
fn discover_layouts() -> ReedResult<Vec<String>> {
    // Same implementation as CSS bundler
    Ok(vec!["knowledge".to_string(), "blog".to_string()])
}
```

### Dependency Resolver (`src/reedcms/assets/js/resolver.rs`)

```rust
/// Resolves JavaScript module dependencies.
///
/// ## Supported Import Formats
/// - ES6: import { func } from './module.js'
/// - ES6 default: import module from './module.js'
/// - CommonJS: const module = require('./module.js')
///
/// ## Resolution Strategy
/// 1. Parse import statements
/// 2. Resolve relative paths
/// 3. Load module contents
/// 4. Recursively resolve dependencies
/// 5. Build dependency graph
/// 6. Return modules in topological order
pub struct DependencyResolver {
    modules: HashMap<String, Module>,
    visited: HashSet<String>,
}

impl DependencyResolver {
    pub fn new() -> Self {
        Self {
            modules: HashMap::new(),
            visited: HashSet::new(),
        }
    }

    /// Adds entry point module.
    pub fn add_entry(&mut self, path: &str, content: &str) -> ReedResult<()> {
        let module = Module {
            path: path.to_string(),
            content: content.to_string(),
            dependencies: Vec::new(),
        };

        self.modules.insert(path.to_string(), module);
        Ok(())
    }

    /// Resolves all dependencies.
    pub fn resolve(&mut self) -> ReedResult<Vec<Module>> {
        let entry_keys: Vec<String> = self.modules.keys().cloned().collect();

        for entry in entry_keys {
            self.resolve_module(&entry)?;
        }

        // Sort modules in dependency order
        Ok(self.topological_sort())
    }

    /// Resolves single module and its dependencies.
    fn resolve_module(&mut self, path: &str) -> ReedResult<()> {
        if self.visited.contains(path) {
            return Ok(());
        }

        self.visited.insert(path.to_string());

        let module = self.modules.get(path).cloned().ok_or_else(|| ReedError::NotFound {
            resource: path.to_string(),
            context: Some("Module not found".to_string()),
        })?;

        // Parse imports from module content
        let imports = parse_imports(&module.content)?;

        for import_path in imports {
            let resolved_path = resolve_import_path(path, &import_path)?;

            // Load dependency if not already loaded
            if !self.modules.contains_key(&resolved_path) {
                let content = std::fs::read_to_string(&resolved_path).map_err(|e| {
                    ReedError::IoError {
                        operation: "read".to_string(),
                        path: resolved_path.clone(),
                        reason: e.to_string(),
                    }
                })?;

                let dep_module = Module {
                    path: resolved_path.clone(),
                    content,
                    dependencies: Vec::new(),
                };

                self.modules.insert(resolved_path.clone(), dep_module);
            }

            // Recursively resolve dependencies
            self.resolve_module(&resolved_path)?;
        }

        Ok(())
    }

    /// Sorts modules in topological order (dependencies first).
    fn topological_sort(&self) -> Vec<Module> {
        // Simplified - full implementation would use proper topological sort
        self.modules.values().cloned().collect()
    }
}

/// Module structure.
#[derive(Debug, Clone)]
pub struct Module {
    pub path: String,
    pub content: String,
    pub dependencies: Vec<String>,
}

/// Parses import statements from JavaScript.
///
/// ## Patterns Matched
/// - import { x } from './module.js'
/// - import x from './module.js'
/// - const x = require('./module.js')
fn parse_imports(content: &str) -> ReedResult<Vec<String>> {
    use regex::Regex;

    let mut imports = Vec::new();

    // ES6 imports
    let es6_re = Regex::new(r#"import\s+.*\s+from\s+['"]([^'"]+)['"]"#).unwrap();
    for cap in es6_re.captures_iter(content) {
        imports.push(cap[1].to_string());
    }

    // CommonJS requires
    let cjs_re = Regex::new(r#"require\s*\(\s*['"]([^'"]+)['"]\s*\)"#).unwrap();
    for cap in cjs_re.captures_iter(content) {
        imports.push(cap[1].to_string());
    }

    Ok(imports)
}

/// Resolves import path relative to importing file.
///
/// ## Examples
/// - Current: assets/js/layouts/blog/blog.mouse.js
/// - Import: ../core/utils.js
/// - Result: assets/js/core/utils.js
fn resolve_import_path(current_path: &str, import_path: &str) -> ReedResult<String> {
    use std::path::{Path, PathBuf};

    let current = Path::new(current_path);
    let parent = current.parent().ok_or_else(|| ReedError::ConfigError {
        component: "path_resolution".to_string(),
        reason: format!("Cannot resolve parent of {}", current_path),
    })?;

    let resolved = parent.join(import_path);
    let canonical = resolved.canonicalize().unwrap_or(resolved);

    Ok(canonical.display().to_string())
}
```

### JavaScript Minifier (`src/reedcms/assets/js/minifier.rs`)

```rust
/// Minifies JavaScript content.
///
/// ## Minification Steps
/// 1. Remove comments (// and /* */)
/// 2. Remove unnecessary whitespace
/// 3. Shorten variable names in local scope
/// 4. Remove console.log statements (PROD only)
/// 5. Optimize expressions
///
/// ## Performance
/// - Minification: < 20ms per KB
/// - Size reduction: ~50-60%
///
/// ## Examples
/// ```js
/// // Input
/// function calculateSum(numbers) {
///     // Calculate sum of array
///     let result = 0;
///     for (let i = 0; i < numbers.length; i++) {
///         result += numbers[i];
///     }
///     return result;
/// }
///
/// // Output
/// function calculateSum(n){let r=0;for(let i=0;i<n.length;i++){r+=n[i]}return r}
/// ```
pub fn minify_js(js: &str) -> ReedResult<String> {
    let mut result = js.to_string();

    // 1. Remove comments
    result = remove_js_comments(&result);

    // 2. Remove whitespace
    result = remove_js_whitespace(&result);

    // 3. Remove console.log (PROD only)
    if is_prod_environment() {
        result = remove_console_logs(&result);
    }

    Ok(result)
}

/// Removes JavaScript comments.
fn remove_js_comments(js: &str) -> String {
    let mut result = String::new();
    let mut chars = js.chars().peekable();
    let mut in_string = false;
    let mut string_char = ' ';

    while let Some(c) = chars.next() {
        // Track string literals to preserve them
        if (c == '"' || c == '\'') && !in_string {
            in_string = true;
            string_char = c;
            result.push(c);
        } else if c == string_char && in_string {
            in_string = false;
            result.push(c);
        } else if !in_string {
            if c == '/' && chars.peek() == Some(&'/') {
                // Single-line comment
                while let Some(ch) = chars.next() {
                    if ch == '\n' {
                        break;
                    }
                }
            } else if c == '/' && chars.peek() == Some(&'*') {
                // Multi-line comment
                chars.next(); // Skip '*'
                let mut prev = ' ';
                while let Some(ch) = chars.next() {
                    if prev == '*' && ch == '/' {
                        break;
                    }
                    prev = ch;
                }
            } else {
                result.push(c);
            }
        } else {
            result.push(c);
        }
    }

    result
}

/// Removes unnecessary whitespace from JavaScript.
fn remove_js_whitespace(js: &str) -> String {
    // Simplified - full implementation would preserve necessary spaces
    js.lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("")
}

/// Removes console.log statements.
fn remove_console_logs(js: &str) -> String {
    use regex::Regex;
    let re = Regex::new(r"console\.log\([^)]*\);?").unwrap();
    re.replace_all(js, "").to_string()
}

/// Checks if running in PROD environment.
fn is_prod_environment() -> bool {
    std::env::var("REED_ENV")
        .unwrap_or_else(|_| "PROD".to_string())
        .to_uppercase()
        == "PROD"
}
```

### Tree Shaking (`src/reedcms/assets/js/tree_shake.rs`)

```rust
/// Performs tree shaking to remove unused exports.
///
/// ## Process
/// 1. Parse all export statements
/// 2. Track which exports are imported
/// 3. Remove unused exports and their dependencies
/// 4. Remove unreachable code
///
/// ## Benefits
/// - Reduces bundle size by ~20%
/// - Eliminates dead code
/// - Improves load times
///
/// ## Limitations
/// - Only works with static imports/exports
/// - Cannot analyse dynamic require()
pub fn tree_shake(js: &str, modules: &[Module]) -> ReedResult<String> {
    // Build export/import graph
    let export_graph = build_export_graph(modules)?;
    let import_graph = build_import_graph(modules)?;

    // Find used exports
    let used_exports = find_used_exports(&export_graph, &import_graph);

    // Remove unused code
    let shaken = remove_unused_code(js, &used_exports)?;

    Ok(shaken)
}

/// Builds export graph from modules.
fn build_export_graph(modules: &[Module]) -> ReedResult<HashMap<String, Vec<String>>> {
    let mut graph = HashMap::new();

    for module in modules {
        let exports = parse_exports(&module.content)?;
        graph.insert(module.path.clone(), exports);
    }

    Ok(graph)
}

/// Builds import graph from modules.
fn build_import_graph(modules: &[Module]) -> ReedResult<HashMap<String, Vec<String>>> {
    let mut graph = HashMap::new();

    for module in modules {
        let imports = parse_import_names(&module.content)?;
        graph.insert(module.path.clone(), imports);
    }

    Ok(graph)
}

/// Finds used exports across all modules.
fn find_used_exports(
    exports: &HashMap<String, Vec<String>>,
    imports: &HashMap<String, Vec<String>>,
) -> HashSet<String> {
    let mut used = HashSet::new();

    for import_list in imports.values() {
        for import in import_list {
            used.insert(import.clone());
        }
    }

    used
}

/// Removes unused code from JavaScript.
fn remove_unused_code(js: &str, used_exports: &HashSet<String>) -> ReedResult<String> {
    // Simplified implementation
    // Full implementation would parse AST and remove unused functions
    Ok(js.to_string())
}

/// Parses export statements from JavaScript.
fn parse_exports(content: &str) -> ReedResult<Vec<String>> {
    use regex::Regex;

    let mut exports = Vec::new();

    // export function name()
    let fn_re = Regex::new(r"export\s+function\s+(\w+)").unwrap();
    for cap in fn_re.captures_iter(content) {
        exports.push(cap[1].to_string());
    }

    // export const name
    let const_re = Regex::new(r"export\s+const\s+(\w+)").unwrap();
    for cap in const_re.captures_iter(content) {
        exports.push(cap[1].to_string());
    }

    Ok(exports)
}

/// Parses imported names from JavaScript.
fn parse_import_names(content: &str) -> ReedResult<Vec<String>> {
    use regex::Regex;

    let mut imports = Vec::new();

    // import { name1, name2 }
    let re = Regex::new(r"import\s+\{([^}]+)\}").unwrap();
    for cap in re.captures_iter(content) {
        let names = cap[1].split(',').map(|s| s.trim().to_string());
        imports.extend(names);
    }

    Ok(imports)
}
```

## Implementation Files

### Primary Implementation
- `src/reedcms/assets/js/bundler.rs` - JS bundler
- `src/reedcms/assets/js/resolver.rs` - Dependency resolver
- `src/reedcms/assets/js/minifier.rs` - JS minifier
- `src/reedcms/assets/js/tree_shake.rs` - Tree shaking
- `src/reedcms/assets/js/source_map.rs` - Source map generator

### Test Files
- `src/reedcms/assets/js/bundler.test.rs`
- `src/reedcms/assets/js/resolver.test.rs`
- `src/reedcms/assets/js/minifier.test.rs`
- `src/reedcms/assets/js/tree_shake.test.rs`
- `src/reedcms/assets/js/source_map.test.rs`

## File Structure
```
src/reedcms/assets/js/
├── bundler.rs             # JS bundler
├── bundler.test.rs        # Bundler tests
├── resolver.rs            # Dependency resolver
├── resolver.test.rs       # Resolver tests
├── minifier.rs            # JS minifier
├── minifier.test.rs       # Minifier tests
├── tree_shake.rs          # Tree shaking
├── tree_shake.test.rs     # Tree shake tests
├── source_map.rs          # Source maps
└── source_map.test.rs     # Source map tests
```

## Testing Requirements

### Unit Tests
- [ ] Test dependency resolution
- [ ] Test import parsing (ES6 and CommonJS)
- [ ] Test module wrapping
- [ ] Test comment removal
- [ ] Test whitespace removal
- [ ] Test console.log removal
- [ ] Test tree shaking
- [ ] Test source map generation

### Integration Tests
- [ ] Test complete bundle workflow
- [ ] Test circular dependency handling
- [ ] Test nested imports
- [ ] Test bundle all layouts
- [ ] Test minification size reduction

### Performance Tests
- [ ] Bundle 10 layouts: < 1s
- [ ] Minify 100KB JS: < 200ms
- [ ] Tree shaking: < 100ms
- [ ] Size reduction: 60-70% total

## Acceptance Criteria
- [ ] JS bundling functional for all variants
- [ ] Dependency resolution working (ES6 + CommonJS)
- [ ] Tree shaking removes unused code
- [ ] Minification working with 50-60% reduction
- [ ] Source maps generated correctly
- [ ] Console.log removal in PROD
- [ ] All layouts bundled successfully
- [ ] Output files written to public/js/
- [ ] All tests pass with 100% coverage
- [ ] Performance benchmarks met
- [ ] Documentation complete
- [ ] BBC English throughout

## Dependencies
- **Requires**: None (standalone)

## Blocks
- REED-09-02 (Asset Pipeline uses JS bundler)

## References
- Service Template: `_workbench/Tickets/templates/service-template.md`
- Service Test Template: `_workbench/Tickets/templates/service-template.test.md`
- Summary: Lines 1028-1030 in `project_summary.md`

## Notes
JavaScript bundling reduces HTTP requests and enables module-based development. Dependency resolution supports both ES6 modules and CommonJS for maximum compatibility. Tree shaking eliminates unused code for smaller bundles. Minification achieves ~60% size reduction. Source maps enable debugging of minified code. Console.log removal in PROD prevents logging overhead and information disclosure.
