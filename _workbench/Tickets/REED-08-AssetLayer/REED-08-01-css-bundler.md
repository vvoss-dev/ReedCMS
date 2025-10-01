# REED-08-01: CSS Bundler and Minifier

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
- **ID**: REED-08-01
- **Title**: CSS Bundler and Minifier
- **Layer**: Asset Layer (REED-08)
- **Priority**: Medium
- **Status**: Open
- **Complexity**: Medium
- **Dependencies**: None

## Summary Reference
- **Section**: CSS Bundler
- **Lines**: 1025-1027 in project_summary.md
- **Key Concepts**: CSS bundling, minification, source maps, variant-specific bundles

## Objective
Implement CSS bundler that combines multiple CSS files per layout variant with session hash for cache-busting, minifies output, generates source maps for debugging, and produces optimised bundles for production deployment.

## Requirements

### Session Hash Strategy

**Purpose**: Cache-busting and bundle versioning  
**Implementation**: MD5 hash over all CSS/JS source files in project

**Hash Generation** (at build time or server startup):
```rust
/// Generates session hash for asset bundling.
///
/// ## Process
/// 1. Discover all CSS/JS files in templates/
/// 2. Read and concatenate file contents
/// 3. Generate MD5 hash
/// 4. Store in .reed/project.csv → project.session_hash
///
/// ## Performance
/// - Hash generation: < 50ms for 100 files
/// - Cached in project.csv for server runtime
///
/// ## Output
/// - `a3f5b2c8` (8-character hex string)
pub fn generate_session_hash() -> ReedResult<String> {
    let mut hasher = md5::Context::new();
    
    // Collect all CSS/JS files
    let css_files = discover_css_files("templates/")?;
    let js_files = discover_js_files("templates/")?;
    
    // Hash all file contents
    for file in css_files.iter().chain(js_files.iter()) {
        let content = std::fs::read(file)?;
        hasher.consume(&content);
    }
    
    // Generate 8-character hash
    let hash = format!("{:x}", hasher.compute());
    Ok(hash[..8].to_string())
}
```

**Storage**:
```csv
# .reed/project.csv
key|value|comment
project.session_hash|a3f5b2c8|Current asset bundle session hash
```

**Bundle Naming Convention**:
```
{layout}.{session_hash}.{variant}.css
{layout}.{session_hash}.js

Examples:
landing.a3f5b2c8.mouse.css
landing.a3f5b2c8.touch.css
landing.a3f5b2c8.reader.css
landing.a3f5b2c8.js

knowledge.a3f5b2c8.mouse.css
knowledge.a3f5b2c8.js
```

**Cleanup Strategy**:
- On new bundle generation, remove old bundles with different session hash
- Keeps `/public/session/` directory clean
- Only current session bundles remain

### Component Discovery and Asset Collection

**Purpose**: Automatic discovery of all CSS/JS files required for a layout

**Discovery Process**:
```rust
/// Discovers all assets required for a layout.
///
/// ## Process
/// 1. Parse layout template ({layout}.jinja)
/// 2. Extract {% include organism("...") %} statements
/// 3. Recursively discover organism dependencies (molecules, atoms)
/// 4. Collect all CSS/JS files from components
/// 5. Return ordered list of asset paths
///
/// ## Order
/// 1. Layout CSS/JS
/// 2. Organism CSS/JS (in inclusion order)
/// 3. Molecule CSS/JS (recursive dependencies)
/// 4. Atom CSS/JS (recursive dependencies)
///
/// ## Example
/// Layout: landing.jinja
/// Includes:
/// - organism("landing-hero")
///   → landing-hero.mouse.css
///   → landing-hero.mouse.js (if exists)
/// - organism("landing-problems")
///   → landing-problems.mouse.css
///   
/// Result: [
///   "templates/layouts/landing/landing.mouse.css",
///   "templates/components/organisms/landing-hero/landing-hero.mouse.css",
///   "templates/components/organisms/landing-problems/landing-problems.mouse.css",
/// ]
pub fn discover_layout_assets(layout: &str, variant: &str) -> ReedResult<LayoutAssets> {
    let template_path = format!("templates/layouts/{}/{}.jinja", layout, layout);
    let template_content = std::fs::read_to_string(&template_path)?;
    
    let mut css_files = Vec::new();
    let mut js_files = Vec::new();
    
    // 1. Add layout CSS/JS
    css_files.push(format!("templates/layouts/{}/{}.{}.css", layout, layout, variant));
    if let Some(js) = find_layout_js(layout) {
        js_files.push(js);
    }
    
    // 2. Extract organism includes
    let organisms = extract_organisms(&template_content)?;
    
    for organism in organisms {
        // Add organism assets
        css_files.push(format!(
            "templates/components/organisms/{}/{}.{}.css",
            organism, organism, variant
        ));
        if let Some(js) = find_organism_js(&organism) {
            js_files.push(js);
        }
        
        // 3. Recursively discover dependencies (molecules, atoms)
        let deps = discover_component_dependencies(&organism, variant)?;
        css_files.extend(deps.css_files);
        js_files.extend(deps.js_files);
    }
    
    Ok(LayoutAssets { css_files, js_files })
}

#[derive(Debug, Clone)]
pub struct LayoutAssets {
    pub css_files: Vec<String>,
    pub js_files: Vec<String>,
}
```

**Template Parsing**:
```rust
/// Extracts organism names from template content.
///
/// ## Pattern Matching
/// Matches: {% include organism("landing-hero") %}
/// Extracts: "landing-hero"
fn extract_organisms(template_content: &str) -> ReedResult<Vec<String>> {
    use regex::Regex;
    
    let re = Regex::new(r#"\{%\s*include\s+organism\("([^"]+)"\)\s*%\}"#)?;
    let mut organisms = Vec::new();
    
    for cap in re.captures_iter(template_content) {
        organisms.push(cap[1].to_string());
    }
    
    Ok(organisms)
}
```

### On-Demand Bundle Generation

**Strategy**: Bundles are generated on first request per layout  
**Performance**: < 100ms first request, < 1ms cached requests

**Request Flow**:
```
1. Request: GET /de/wissen
2. Route resolver: knowledge layout + de language
3. Template context builder checks: 
   - Does /public/session/styles/knowledge.a3f5b2c8.mouse.css exist?
4. If NO:
   - Discover assets for knowledge layout
   - Bundle CSS (all variants: mouse/touch/reader)
   - Bundle JS
   - Write to /public/session/
5. If YES:
   - Use existing bundle
6. Populate template context:
   - asset_css = "/public/session/styles/knowledge.a3f5b2c8.mouse.css"
   - asset_js = "/public/session/scripts/knowledge.a3f5b2c8.js"
7. Render template with asset paths
```

**Bundle Check**:
```rust
/// Checks if bundles exist for layout, generates if missing.
///
/// ## Performance
/// - Check: < 1ms (filesystem stat)
/// - Generation: < 100ms (first request only)
/// - Subsequent requests: Cached, no generation
pub fn ensure_bundles_exist(layout: &str, session_hash: &str) -> ReedResult<()> {
    let variants = ["mouse", "touch", "reader"];
    
    for variant in &variants {
        let css_path = format!(
            "public/session/styles/{}.{}.{}.css",
            layout, session_hash, variant
        );
        
        if !std::path::Path::new(&css_path).exists() {
            // Generate bundles for all variants at once
            generate_layout_bundles(layout, session_hash)?;
            break; // All variants generated together
        }
    }
    
    Ok(())
}
```

### CSS Directory Structure
```
assets/css/
├── core/
│   ├── reset.css           # CSS reset
│   ├── typography.css      # Font definitions
│   └── variables.css       # CSS variables
├── layouts/
│   ├── knowledge/
│   │   ├── knowledge.mouse.css
│   │   ├── knowledge.touch.css
│   │   └── knowledge.reader.css
│   └── blog/
│       ├── blog.mouse.css
│       ├── blog.touch.css
│       └── blog.reader.css
└── components/
    ├── navigation.css
    ├── footer.css
    └── forms.css
```

### Output Structure
```
public/session/
├── styles/
│   ├── landing.a3f5b2c8.mouse.css       # Bundled and minified
│   ├── landing.a3f5b2c8.mouse.css.map   # Source map
│   ├── landing.a3f5b2c8.touch.css
│   ├── landing.a3f5b2c8.touch.css.map
│   ├── landing.a3f5b2c8.reader.css
│   ├── landing.a3f5b2c8.reader.css.map
│   ├── knowledge.a3f5b2c8.mouse.css
│   └── knowledge.a3f5b2c8.mouse.css.map
└── scripts/
    ├── landing.a3f5b2c8.js
    ├── landing.a3f5b2c8.js.map
    ├── knowledge.a3f5b2c8.js
    └── knowledge.a3f5b2c8.js.map
```

### Template Integration

**Context Variables** (populated by context builder in REED-05-03):
```rust
// In template context
context.insert("asset_css", asset_css_path);
context.insert("asset_js", asset_js_path);

// Example values:
// asset_css = "/public/session/styles/landing.a3f5b2c8.mouse.css"
// asset_js = "/public/session/scripts/landing.a3f5b2c8.js"
```

**Template Usage**:
```jinja
<!DOCTYPE html>
<html lang="{{ client.lang }}">
<head>
    <meta charset="UTF-8">
    <title>{{ "page.title" | text(client.lang) }}</title>
    <link rel="stylesheet" href="{{ asset_css }}">
</head>
<body>
    <!-- Content -->
    <script src="{{ asset_js }}" defer></script>
</body>
</html>
```

**Benefit**: Simple template usage, automatic variant resolution, cache-busting via session hash

### Implementation (`src/reedcms/assets/css/bundler.rs`)

```rust
/// Bundles and minifies CSS files for layouts.
///
/// ## Process
/// 1. Discover layout CSS files
/// 2. Resolve @import statements
/// 3. Combine files in correct order (core → components → layout)
/// 4. Minify CSS
/// 5. Generate source maps
/// 6. Write to public/css/
///
/// ## Order of Inclusion
/// 1. Core CSS (reset, variables, typography)
/// 2. Component CSS (navigation, footer, forms)
/// 3. Layout-specific CSS
///
/// ## Performance
/// - Bundle time: < 500ms for 10 layouts
/// - Minification: ~70% size reduction
/// - Source map generation: < 100ms
///
/// ## Output
/// ```
/// Bundling CSS for knowledge.mouse...
/// - Included: reset.css (2.3 KB)
/// - Included: variables.css (1.1 KB)
/// - Included: navigation.css (3.4 KB)
/// - Included: knowledge.mouse.css (5.2 KB)
/// → Output: public/css/knowledge.mouse.css (3.8 KB, -67%)
/// ✓ Bundle complete
/// ```
pub fn bundle_css(layout: &str, variant: &str) -> ReedResult<BundleResult> {
    println!("📦 Bundling CSS for {}.{}...", layout, variant);

    // 1. Collect CSS files
    let mut css_files = Vec::new();
    css_files.extend(collect_core_css()?);
    css_files.extend(collect_component_css()?);
    css_files.push(collect_layout_css(layout, variant)?);

    // 2. Combine CSS content
    let mut combined_css = String::new();
    let mut source_map = SourceMap::new();

    for (file_path, css_content) in css_files {
        println!("  - Included: {} ({} KB)", file_path, css_content.len() / 1024);
        source_map.add_source(&file_path);
        combined_css.push_str(&css_content);
        combined_css.push('\n');
    }

    let original_size = combined_css.len();

    // 3. Minify CSS
    let minified = minify_css(&combined_css)?;
    let minified_size = minified.len();

    // 4. Generate source map
    let source_map_content = source_map.generate()?;

    // 5. Write output files
    let output_path = format!("public/css/{}.{}.css", layout, variant);
    let source_map_path = format!("{}.map", output_path);

    write_css_file(&output_path, &minified)?;
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
        minified_size,
        reduction_percent: reduction,
    })
}

/// Bundles CSS for all layouts and variants.
///
/// ## Process
/// 1. Discover all layouts from templates/layouts/
/// 2. For each layout, bundle all variants
/// 3. Report total size savings
pub fn bundle_all_css() -> ReedResult<Vec<BundleResult>> {
    let layouts = discover_layouts()?;
    let variants = vec!["mouse", "touch", "reader"];

    let mut results = Vec::new();
    let mut total_original = 0;
    let mut total_minified = 0;

    for layout in layouts {
        for variant in &variants {
            match bundle_css(&layout, variant) {
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

    println!("\n📊 Total CSS Bundle Statistics:");
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
    pub minified_size: usize,
    pub reduction_percent: u32,
}

/// Collects core CSS files.
fn collect_core_css() -> ReedResult<Vec<(String, String)>> {
    let core_files = vec!["reset.css", "variables.css", "typography.css"];
    collect_css_files("assets/css/core", &core_files)
}

/// Collects component CSS files.
fn collect_component_css() -> ReedResult<Vec<(String, String)>> {
    let component_files = vec!["navigation.css", "footer.css", "forms.css"];
    collect_css_files("assets/css/components", &component_files)
}

/// Collects layout-specific CSS file.
fn collect_layout_css(layout: &str, variant: &str) -> ReedResult<(String, String)> {
    let file_path = format!("assets/css/layouts/{}/{}.{}.css", layout, layout, variant);
    let content = std::fs::read_to_string(&file_path).map_err(|e| ReedError::IoError {
        operation: "read".to_string(),
        path: file_path.clone(),
        reason: e.to_string(),
    })?;

    Ok((file_path, content))
}

/// Collects multiple CSS files from directory.
fn collect_css_files(dir: &str, files: &[&str]) -> ReedResult<Vec<(String, String)>> {
    let mut result = Vec::new();

    for file in files {
        let file_path = format!("{}/{}", dir, file);
        match std::fs::read_to_string(&file_path) {
            Ok(content) => result.push((file_path, content)),
            Err(_) => {
                eprintln!("⚠ CSS file not found: {}", file_path);
            }
        }
    }

    Ok(result)
}

/// Discovers all layouts from templates directory.
fn discover_layouts() -> ReedResult<Vec<String>> {
    let templates_dir = "templates/layouts";
    let mut layouts = Vec::new();

    let entries = std::fs::read_dir(templates_dir).map_err(|e| ReedError::IoError {
        operation: "read_dir".to_string(),
        path: templates_dir.to_string(),
        reason: e.to_string(),
    })?;

    for entry in entries {
        let entry = entry.map_err(|e| ReedError::IoError {
            operation: "read_entry".to_string(),
            path: templates_dir.to_string(),
            reason: e.to_string(),
        })?;

        if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
            if let Some(name) = entry.file_name().to_str() {
                layouts.push(name.to_string());
            }
        }
    }

    Ok(layouts)
}
```

### CSS Minifier (`src/reedcms/assets/css/minifier.rs`)

```rust
/// Minifies CSS content.
///
/// ## Minification Steps
/// 1. Remove comments (/* ... */)
/// 2. Remove whitespace (spaces, tabs, newlines)
/// 3. Remove unnecessary semicolons
/// 4. Shorten hex colours (#ffffff → #fff)
/// 5. Remove units from zero values (0px → 0)
///
/// ## Performance
/// - Minification: < 10ms per KB
/// - Size reduction: ~60-70%
///
/// ## Examples
/// ```css
/// /* Input */
/// body {
///     margin: 0px;
///     padding: 0px;
///     background: #ffffff;
/// }
///
/// /* Output */
/// body{margin:0;padding:0;background:#fff}
/// ```
pub fn minify_css(css: &str) -> ReedResult<String> {
    let mut result = css.to_string();

    // 1. Remove comments
    result = remove_comments(&result);

    // 2. Remove whitespace
    result = remove_whitespace(&result);

    // 3. Remove unnecessary semicolons
    result = remove_unnecessary_semicolons(&result);

    // 4. Shorten hex colours
    result = shorten_hex_colours(&result);

    // 5. Remove units from zero values
    result = remove_zero_units(&result);

    Ok(result)
}

/// Removes CSS comments.
fn remove_comments(css: &str) -> String {
    let mut result = String::new();
    let mut in_comment = false;
    let mut chars = css.chars().peekable();

    while let Some(c) = chars.next() {
        if !in_comment && c == '/' && chars.peek() == Some(&'*') {
            in_comment = true;
            chars.next(); // Skip '*'
        } else if in_comment && c == '*' && chars.peek() == Some(&'/') {
            in_comment = false;
            chars.next(); // Skip '/'
        } else if !in_comment {
            result.push(c);
        }
    }

    result
}

/// Removes unnecessary whitespace.
fn remove_whitespace(css: &str) -> String {
    css.split_whitespace().collect::<Vec<_>>().join("")
}

/// Removes unnecessary semicolons before closing braces.
fn remove_unnecessary_semicolons(css: &str) -> String {
    css.replace(";}", "}")
}

/// Shortens hex colours from 6 to 3 digits where possible.
///
/// ## Examples
/// - #ffffff → #fff
/// - #000000 → #000
/// - #ff0000 → #f00
fn shorten_hex_colours(css: &str) -> String {
    use regex::Regex;

    let re = Regex::new(r"#([0-9a-fA-F])\1([0-9a-fA-F])\2([0-9a-fA-F])\3").unwrap();
    re.replace_all(css, "#$1$2$3").to_string()
}

/// Removes units from zero values.
///
/// ## Examples
/// - 0px → 0
/// - 0em → 0
/// - 0% → 0%  (percentage kept)
fn remove_zero_units(css: &str) -> String {
    use regex::Regex;

    let re = Regex::new(r"\b0(px|em|rem|vh|vw|pt|cm|mm|in)\b").unwrap();
    re.replace_all(css, "0").to_string()
}
```

### Source Map Generator (`src/reedcms/assets/css/source_map.rs`)

```rust
/// Generates source maps for CSS bundles.
///
/// ## Source Map Format
/// JSON format following Source Map v3 specification
///
/// ## Purpose
/// - Debugging minified CSS in browser DevTools
/// - Maps minified positions to original source files
/// - Essential for development
pub struct SourceMap {
    sources: Vec<String>,
    mappings: Vec<Mapping>,
}

impl SourceMap {
    /// Creates new source map.
    pub fn new() -> Self {
        Self {
            sources: Vec::new(),
            mappings: Vec::new(),
        }
    }

    /// Adds source file to map.
    pub fn add_source(&mut self, path: &str) {
        self.sources.push(path.to_string());
    }

    /// Generates source map JSON.
    ///
    /// ## Output Format
    /// ```json
    /// {
    ///   "version": 3,
    ///   "sources": ["reset.css", "navigation.css"],
    ///   "names": [],
    ///   "mappings": "AAAA,CAAC,CAAC,CAAC"
    /// }
    /// ```
    pub fn generate(&self) -> ReedResult<String> {
        let map = serde_json::json!({
            "version": 3,
            "sources": self.sources,
            "names": [],
            "mappings": self.encode_mappings()
        });

        serde_json::to_string_pretty(&map).map_err(|e| ReedError::SerializationError {
            data_type: "source_map".to_string(),
            reason: e.to_string(),
        })
    }

    /// Encodes mappings in Base64 VLQ format.
    fn encode_mappings(&self) -> String {
        // Simplified implementation - full VLQ encoding would be more complex
        String::new()
    }
}

/// Mapping entry structure.
#[derive(Debug, Clone)]
struct Mapping {
    generated_line: u32,
    generated_column: u32,
    source_index: u32,
    source_line: u32,
    source_column: u32,
}
```

### File Writers (`src/reedcms/assets/css/writer.rs`)

```rust
/// Writes CSS file to disk.
pub fn write_css_file(path: &str, content: &str) -> ReedResult<()> {
    // Create directory if needed
    if let Some(parent) = std::path::Path::new(path).parent() {
        std::fs::create_dir_all(parent).map_err(|e| ReedError::IoError {
            operation: "create_dir".to_string(),
            path: parent.display().to_string(),
            reason: e.to_string(),
        })?;
    }

    std::fs::write(path, content).map_err(|e| ReedError::IoError {
        operation: "write".to_string(),
        path: path.to_string(),
        reason: e.to_string(),
    })
}

/// Writes source map file to disk.
pub fn write_source_map(path: &str, content: &str) -> ReedResult<()> {
    write_css_file(path, content)
}
```

## Implementation Files

### Primary Implementation
- `src/reedcms/assets/css/bundler.rs` - CSS bundler
- `src/reedcms/assets/css/minifier.rs` - CSS minifier
- `src/reedcms/assets/css/source_map.rs` - Source map generator
- `src/reedcms/assets/css/writer.rs` - File writers

### Test Files
- `src/reedcms/assets/css/bundler.test.rs`
- `src/reedcms/assets/css/minifier.test.rs`
- `src/reedcms/assets/css/source_map.test.rs`
- `src/reedcms/assets/css/writer.test.rs`

## File Structure
```
src/reedcms/assets/css/
├── bundler.rs             # CSS bundler
├── bundler.test.rs        # Bundler tests
├── minifier.rs            # CSS minifier
├── minifier.test.rs       # Minifier tests
├── source_map.rs          # Source map generator
├── source_map.test.rs     # Source map tests
├── writer.rs              # File writers
└── writer.test.rs         # Writer tests
```

## Testing Requirements

### Unit Tests
- [ ] Test core CSS collection
- [ ] Test component CSS collection
- [ ] Test layout CSS collection
- [ ] Test comment removal
- [ ] Test whitespace removal
- [ ] Test hex colour shortening
- [ ] Test zero unit removal
- [ ] Test source map generation

### Integration Tests
- [ ] Test complete bundle workflow
- [ ] Test bundle all layouts
- [ ] Test minification size reduction
- [ ] Test source map accuracy
- [ ] Test output file creation

### Performance Tests
- [ ] Bundle 10 layouts: < 500ms
- [ ] Minify 100KB CSS: < 100ms
- [ ] Source map generation: < 100ms
- [ ] Size reduction: 60-70%

## Acceptance Criteria
- [ ] CSS bundling functional for all variants
- [ ] Minification working with 60-70% reduction
- [ ] Source maps generated correctly
- [ ] Core → components → layout order enforced
- [ ] All layouts bundled successfully
- [ ] Output files written to public/css/
- [ ] All tests pass with 100% coverage
- [ ] Performance benchmarks met
- [ ] Documentation complete
- [ ] BBC English throughout

## Dependencies
- **Requires**: None (standalone)

## Blocks
- REED-09-02 (Asset Pipeline uses CSS bundler)

## References
- Service Template: `_workbench/Tickets/templates/service-template.md`
- Service Test Template: `_workbench/Tickets/templates/service-template.test.md`
- Summary: Lines 1025-1027 in `project_summary.md`

## Notes
CSS bundling reduces HTTP requests and file sizes for faster page loads. Minification achieves ~70% size reduction without quality loss. Source maps enable debugging of minified CSS in browser DevTools. Variant-specific bundles (mouse/touch/reader) optimise CSS for different devices. Bundle order (core → components → layout) ensures proper CSS cascade and specificity.
