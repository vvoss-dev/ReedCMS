# CSS Bundler

Combines, minifies, and optimises CSS files for production serving.

## Purpose

- **Bundle Optimisation**: Combine multiple CSS files into single bundle per layout/variant
- **Size Reduction**: Minify CSS with 60-70% size reduction
- **Source Maps**: Enable debugging with original source files
- **Cache Busting**: Session hash versioning

## Architecture

```
┌─────────────────────────────────────────────────────┐
│ CSS Bundling Pipeline                                │
├─────────────────────────────────────────────────────┤
│                                                       │
│  templates/layouts/landing/                          │
│  ├─ landing.mouse.css                                │
│  └─ Includes organisms/molecules/atoms               │
│           ↓                                           │
│  1. Discovery Phase                                  │
│     ├─ Scan layout file for dependencies            │
│     ├─ Resolve organisms (page-header, etc.)        │
│     ├─ Resolve molecules (nav-item, etc.)           │
│     └─ Resolve atoms (icons, buttons)               │
│           ↓                                           │
│  2. Concatenation Phase                              │
│     ├─ Layout CSS (base styles)                      │
│     ├─ Organism CSS (complex components)            │
│     ├─ Molecule CSS (component groups)              │
│     └─ Atom CSS (basic elements)                     │
│           ↓                                           │
│  3. Minification Phase                               │
│     ├─ Remove comments                               │
│     ├─ Remove whitespace                             │
│     ├─ Optimise selectors                            │
│     └─ Compress colours                              │
│           ↓                                           │
│  4. Source Map Generation                            │
│     ├─ Track original file locations                │
│     ├─ Map minified → source                         │
│     └─ Append source map comment                     │
│           ↓                                           │
│  5. Output Phase                                     │
│     └─ Write to public/session/styles/               │
│         └─ {layout}.{hash}.{variant}.css             │
│                                                       │
└─────────────────────────────────────────────────────┘
```

## Implementation

### Bundle Single Layout

```rust
pub fn bundle_css(layout: &str, variant: &str) 
    -> ReedResult<BundleResult> 
{
    // Get session hash
    let session_hash = get_session_hash()?;
    
    // Discover CSS files for layout
    let assets = discover_layout_assets(layout, variant)?;
    
    // Combine CSS content
    let mut combined_css = String::new();
    for css_path in &assets.css_files {
        let content = fs::read_to_string(css_path)?;
        combined_css.push_str(&content);
        combined_css.push('\n');
    }
    
    // Minify CSS
    let minified = minify_css(&combined_css)?;
    
    // Generate source map
    let source_map = SourceMap::new();
    let map_content = source_map.generate()?;
    
    // Write output
    let output_path = format!(
        "public/session/styles/{}.{}.{}.css",
        layout, session_hash, variant
    );
    write_css_file(&output_path, &minified)?;
    
    Ok(BundleResult {
        output_path,
        original_size: combined_css.len(),
        minified_size: minified.len(),
        reduction_percent: calculate_reduction(
            combined_css.len(), 
            minified.len()
        ),
    })
}
```

### Bundle All Layouts

```rust
pub fn bundle_all_css() -> ReedResult<Vec<BundleResult>> {
    let layouts = discover_layouts()?;
    let variants = vec!["mouse", "touch", "reader"];
    
    let mut results = Vec::new();
    
    for layout in &layouts {
        for variant in &variants {
            let result = bundle_css(layout, variant)?;
            results.push(result);
        }
    }
    
    // Clean old bundles with different session hash
    clean_old_bundles("public/session/styles", 
                      &get_session_hash()?)?;
    
    Ok(results)
}
```

## File Discovery

### Layout Assets

```rust
pub fn discover_layout_assets(layout: &str, variant: &str) 
    -> ReedResult<LayoutAssets> 
{
    let layout_css = format!(
        "templates/layouts/{}/{}.{}.css",
        layout, layout, variant
    );
    
    let mut css_files = vec![layout_css];
    
    // Parse layout for component includes
    let organisms = parse_includes(&layout_file)?;
    
    // Add organism CSS files
    for organism in &organisms {
        let org_css = format!(
            "templates/components/organisms/{}/{}.{}.css",
            organism, organism, variant
        );
        if Path::new(&org_css).exists() {
            css_files.push(org_css);
        }
    }
    
    Ok(LayoutAssets { css_files })
}
```

### Inclusion Order

**Critical**: CSS files must be combined in correct order to maintain cascade:

1. **Layout CSS** (lowest specificity)
2. **Organism CSS** (medium specificity)
3. **Molecule CSS** (higher specificity)
4. **Atom CSS** (highest specificity)

## Minification

### Algorithm

```rust
pub fn minify_css(css: &str) -> ReedResult<String> {
    let mut minified = String::new();
    
    // Remove comments (/* ... */)
    let no_comments = remove_comments(css);
    
    // Remove unnecessary whitespace
    let no_whitespace = remove_whitespace(&no_comments);
    
    // Optimise selectors
    let optimised = optimise_selectors(&no_whitespace);
    
    // Compress colours (#ffffff → #fff)
    let compressed = compress_colours(&optimised);
    
    Ok(compressed)
}
```

### Optimisations

| Optimisation | Before | After | Savings |
|--------------|--------|-------|---------|
| Comments | `/* Header styles */` | _(removed)_ | 100% |
| Whitespace | `body {\n  margin: 0;\n}` | `body{margin:0}` | ~50% |
| Colour codes | `colour: #ffffff;` | `colour:#fff` | 33% |
| Zero values | `margin: 0px;` | `margin:0` | 25% |
| **Total** | **10 KB** | **3 KB** | **~70%** |

## Source Maps

### Generation

```rust
pub struct SourceMap {
    sources: Vec<String>,
    mappings: Vec<Mapping>,
}

impl SourceMap {
    pub fn add_source(&mut self, path: &str, content: &str) {
        self.sources.push(path.to_string());
        // Track line mappings
        self.mappings.push(Mapping {
            generated_line: self.current_line,
            original_file: path,
            original_line: 0,
        });
    }
    
    pub fn generate(&self) -> ReedResult<String> {
        // Generate source map JSON
        Ok(serde_json::to_string(&SourceMapJson {
            version: 3,
            sources: &self.sources,
            mappings: encode_mappings(&self.mappings),
        })?)
    }
}
```

### Source Map Comment

```css
/* Minified CSS content */
body{margin:0}header{background:#333}

/*# sourceMappingURL=landing.a3f5b2c8.mouse.css.map */
```

### Source Map File

```json
{
  "version": 3,
  "sources": [
    "templates/layouts/landing/landing.mouse.css",
    "templates/components/organisms/page-header/page-header.mouse.css"
  ],
  "mappings": "AAAA,KACE,SAAA,CAGF,OACE,iBAAA",
  "names": []
}
```

## Bundle Cleanup

### Old Bundle Removal

```rust
pub fn clean_old_bundles(dir: &str, current_hash: &str) 
    -> ReedResult<usize> 
{
    let entries = fs::read_dir(dir)?;
    let mut cleaned = 0;
    
    for entry in entries {
        let path = entry?.path();
        let filename = path.file_name()
                           .unwrap()
                           .to_string_lossy();
        
        // Match pattern: {layout}.{hash}.{variant}.css
        if filename.contains('.') && 
           !filename.contains(current_hash) 
        {
            fs::remove_file(&path)?;
            cleaned += 1;
        }
    }
    
    Ok(cleaned)
}
```

## Performance

| Operation | Timing | Note |
|-----------|--------|------|
| Discovery | < 10ms | Per layout |
| Concatenation | < 20ms | ~50 KB combined |
| Minification | < 50ms | 60-70% reduction |
| Source map | < 10ms | JSON generation |
| Write | < 10ms | Single file |
| **Total** | **< 100ms** | Per layout/variant |

### Batch Performance

```
10 layouts × 3 variants = 30 bundles
Total time: < 500ms
Average: ~17ms per bundle
```

## CLI Integration

### Build Command

```bash
# Bundle all CSS
reed build:css

# Output:
📦 Bundling CSS for landing.mouse...
  - Included: templates/layouts/landing/landing.mouse.css (5.2 KB)
  - Included: .../page-header/page-header.mouse.css (3.4 KB)
  → Output: public/session/styles/landing.a3f5b2c8.mouse.css (3.8 KB, -67%)
✓ Bundle complete

📦 Bundling CSS for landing.touch...
  ...

📊 Total CSS Bundle Statistics:
  Original size: 245.8 KB
  Minified size: 89.3 KB
  Size reduction: 64%
  Bundles created: 30
```

### Watch Mode

```bash
# Auto-rebuild on CSS changes
reed build:watch

# Watches:
# - templates/**/*.css
# - Rebuilds affected bundles only
# - Updates session hash if content changed
```

## Development vs Production

### Development Mode

```bash
# DEV mode: No minification, source maps inline
reed build:css --dev

# Output:
landing.a3f5b2c8.mouse.css  # Full CSS with comments
├─ Readable formatting
├─ Original comments preserved
└─ Inline source maps for debugging
```

### Production Mode

```bash
# PROD mode: Full minification, external source maps
reed build:css --prod

# Output:
landing.a3f5b2c8.mouse.css      # Minified CSS
landing.a3f5b2c8.mouse.css.map  # Source map (separate)
├─ All whitespace removed
├─ No comments
└─ Maximum compression
```

## Troubleshooting

### Bundle Not Found

```
Error: FileNotFound { path: "public/session/styles/landing.a3f5b2c8.mouse.css" }
```

**Solution**: Generate bundles

```bash
reed build:css
```

### Wrong Inclusion Order

**Symptom**: CSS rules not applying correctly

**Cause**: Component CSS loaded before layout CSS (wrong specificity)

**Solution**: Check `discover_layout_assets()` order

```rust
// CORRECT order:
css_files = [
    "layouts/landing/landing.mouse.css",      // Base
    "organisms/page-header/page-header.css",  // Components
    "atoms/icon/icon.css",                     // Details
];
```

### Minification Breaks CSS

**Symptom**: Layout broken after minification

**Cause**: Invalid CSS syntax (unclosed braces, missing semicolons)

**Solution**: Validate CSS before bundling

```bash
# Test without minification
reed build:css --dev

# If works in dev but not prod → syntax error
```

## Related Documentation

- [Session Hash](session-hash.md) - Cache-busting strategy
- [Static Server](static-server.md) - Serving bundled CSS
- [Atomic Design](../05-template-layer/atomic-design.md) - Component structure

## CLI Reference

```bash
# Bundle all CSS
reed build:css

# Bundle specific layout
reed build:css --layout=landing

# Bundle with variant
reed build:css --layout=landing --variant=mouse

# Development mode (no minification)
reed build:css --dev

# Watch mode (auto-rebuild)
reed build:watch
```
