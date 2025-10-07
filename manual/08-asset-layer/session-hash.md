# Session Hash Generation

MD5-based cache-busting strategy for asset versioning.

## Purpose

- **Cache Busting**: Force browser reload when assets change
- **Version Control**: Single hash represents all CSS/JS files
- **Build Optimisation**: Deterministic hash generation

## Architecture

```
┌─────────────────────────────────────────────────────┐
│ Session Hash Generation Process                      │
├─────────────────────────────────────────────────────┤
│                                                       │
│  1. Discover Phase                                   │
│     ├─ Scan templates/ recursively                   │
│     ├─ Find all *.css files                          │
│     ├─ Find all *.js files                           │
│     └─ Sort alphabetically (deterministic)           │
│                                                       │
│  2. Concatenation Phase                              │
│     ├─ Read each file content                        │
│     ├─ Concatenate in sorted order                   │
│     └─ Build single byte vector                      │
│                                                       │
│  3. Hash Phase                                       │
│     ├─ MD5 hash over combined content                │
│     ├─ Convert to hex string (32 chars)              │
│     └─ Take first 8 characters                       │
│                                                       │
│  4. Storage Phase                                    │
│     ├─ Store in .reed/project.csv                    │
│     ├─ Key: project.session_hash                     │
│     └─ Value: a3f5b2c8                               │
│                                                       │
└─────────────────────────────────────────────────────┘
```

## Implementation

### Hash Generation

```rust
pub fn generate_session_hash() -> ReedResult<String> {
    // Discover all CSS/JS files
    let css_files = discover_css_files("templates/")?;
    let js_files = discover_js_files("templates/")?;
    
    // Combine and sort for deterministic hashing
    let mut all_files: Vec<PathBuf> = 
        css_files.into_iter()
                 .chain(js_files)
                 .collect();
    all_files.sort();
    
    // Read and concatenate content
    let mut combined_content = Vec::new();
    for file in &all_files {
        let content = fs::read(file)?;
        combined_content.extend_from_slice(&content);
    }
    
    // Generate MD5 hash (first 8 characters)
    let digest = md5::compute(&combined_content);
    let hash = format!("{:x}", digest);
    Ok(hash[..8].to_string())
}
```

### Storage

```rust
pub fn store_session_hash(hash: &str) -> ReedResult<()> {
    // Update .reed/project.csv
    let mut records = read_csv(".reed/project.csv")?;
    
    // Find or create project.session_hash key
    update_or_insert(&mut records, "project.session_hash", hash);
    
    write_csv(".reed/project.csv", &records)?;
    Ok(())
}
```

### Retrieval

```rust
pub fn get_session_hash() -> ReedResult<String> {
    let records = read_csv(".reed/project.csv")?;
    
    for record in records {
        if record.key == "project.session_hash" {
            return Ok(record.value);
        }
    }
    
    Err(ReedError::NotFound {
        resource: "project.session_hash",
        context: Some("project.csv"),
    })
}
```

## Usage in Templates

### CSS Bundle Reference

```html
<link rel="stylesheet" 
      href="/session/styles/{{ layout }}.{{ config('project.session_hash') }}.{{ variant }}.css">
```

### JavaScript Bundle Reference

```html
<script src="/session/scripts/{{ layout }}.{{ config('project.session_hash') }}.js"></script>
```

## File Naming Convention

```
Format: {layout}.{session_hash}.{variant}.{ext}

Examples:
├─ landing.a3f5b2c8.mouse.css
├─ landing.a3f5b2c8.touch.css
├─ landing.a3f5b2c8.reader.css
├─ landing.a3f5b2c8.js
├─ knowledge.b7e4f9d2.mouse.css
└─ knowledge.b7e4f9d2.js
```

## Performance

| Operation | Timing | Note |
|-----------|--------|------|
| File discovery | < 10ms | 100 files |
| Content read | < 30ms | 2 MB total |
| MD5 hash | < 10ms | Fast algorithm |
| CSV storage | < 5ms | Single write |
| **Total** | **< 60ms** | Full regeneration |

## Build Integration

### Automatic Regeneration

```bash
# During build
reed build:assets
→ Generates new session hash
→ Updates .reed/project.csv
→ Rebuilds all bundles with new hash
→ Cleans old bundles
```

### Manual Regeneration

```bash
# Force regeneration
reed assets:hash
→ Generates: a3f5b2c8
→ Stored in .reed/project.csv
```

## Cache Strategy

### Browser Cache

```http
Cache-Control: public, max-age=31536000, immutable
```

- **1-year cache**: CSS/JS bundles never change for given hash
- **Immutable**: Browser never revalidates
- **New hash**: Browser treats as new resource (cache miss)

### Server Cache

Session hash stored in ReedBase cache:
- **O(1) lookup**: `config('project.session_hash')`
- **No disk I/O**: Retrieved from HashMap
- **Environment-aware**: Separate hash per environment (@dev, @prod)

## Security Considerations

### Why MD5?

- **Not for security**: Used for cache-busting, not cryptography
- **Fast computation**: ~10ms for 2 MB
- **Sufficient collision resistance**: For versioning purposes
- **Industry standard**: Used by Webpack, Rollup, etc.

### Path Validation

```rust
// Prevent directory traversal
validate_path(file_path, "templates/")?;

// Only hash files within templates/
if !canonical_path.starts_with(canonical_base) {
    return Err(ReedError::SecurityViolation);
}
```

## Troubleshooting

### Hash Not Found

```
Error: NotFound { resource: "project.session_hash" }
```

**Solution**: Regenerate session hash

```bash
reed assets:hash
```

### Stale Bundles

**Symptom**: Browser serves old CSS/JS despite changes

**Cause**: Session hash not updated after file changes

**Solution**:
```bash
# Regenerate hash and rebuild
reed build:assets

# Or during development
reed build:watch  # Auto-regenerates on file changes
```

### Hash Mismatch

**Symptom**: 404 errors for asset bundles

**Cause**: Template references outdated hash

**Solution**:
```bash
# Ensure template uses config filter
{{ config('project.session_hash') }}  # ✓ Dynamic lookup
# NOT hardcoded hash:
a3f5b2c8  # ✗ Will break on hash change
```

## Related Documentation

- [CSS Bundler](css-bundler.md) - CSS bundling process
- [JS Bundler](js-bundler.md) - JavaScript bundling process
- [Static Server](static-server.md) - Asset serving with ETag caching

## CLI Reference

```bash
# Generate and store session hash
reed assets:hash

# View current hash
reed config:get project.session_hash

# Rebuild all assets with new hash
reed build:assets
```
