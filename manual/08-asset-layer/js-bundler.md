# JavaScript Bundler

Bundles, tree-shakes, and minifies JavaScript files with dependency resolution.

## Purpose

- **Dependency Resolution**: Handle ES6/CommonJS imports automatically
- **Tree Shaking**: Remove unused exports (20% size reduction)
- **Minification**: Compress JavaScript (50-60% size reduction)
- **Module Isolation**: Prevent global scope pollution with IIFE wrapping
- **Source Maps**: Enable debugging with original source files

## Architecture

```
┌─────────────────────────────────────────────────────┐
│ JavaScript Bundling Pipeline                         │
├─────────────────────────────────────────────────────┤
│                                                       │
│  templates/layouts/landing/landing.js                │
│           ↓                                           │
│  1. Entry Point Analysis                             │
│     ├─ Parse import/require statements              │
│     ├─ Extract dependencies                          │
│     └─ Build dependency graph                        │
│           ↓                                           │
│  2. Dependency Resolution                            │
│     ├─ Resolve relative imports (./nav.js)          │
│     ├─ Resolve component imports                     │
│     ├─ Detect circular dependencies                 │
│     └─ Topological sort (correct order)             │
│           ↓                                           │
│  3. Module Wrapping                                  │
│     ├─ Wrap each module in IIFE                     │
│     ├─ Provide module/exports objects               │
│     └─ Maintain module isolation                     │
│           ↓                                           │
│  4. Tree Shaking                                     │
│     ├─ Detect unused exports                        │
│     ├─ Remove dead code                              │
│     └─ Optimise module imports                       │
│           ↓                                           │
│  5. Minification                                     │
│     ├─ Rename variables (a, b, c)                   │
│     ├─ Remove whitespace                             │
│     ├─ Remove comments                               │
│     └─ Optimise expressions                          │
│           ↓                                           │
│  6. Source Map Generation                            │
│     ├─ Track original locations                     │
│     ├─ Map minified → source                         │
│     └─ Append source map comment                     │
│           ↓                                           │
│  7. Output Phase                                     │
│     └─ Write to public/session/scripts/              │
│         └─ {layout}.{hash}.js                        │
│                                                       │
└─────────────────────────────────────────────────────┘
```

## Implementation

### Bundle Single Layout

```rust
pub fn bundle_js(layout: &str, variant: &str) 
    -> ReedResult<BundleResult> 
{
    let session_hash = get_session_hash()?;
    
    // Check if entry point exists
    let entry_point = format!(
        "templates/layouts/{}/{}.js", 
        layout, layout
    );
    
    if !Path::new(&entry_point).exists() {
        return Ok(BundleResult::empty());
    }
    
    let entry_content = fs::read_to_string(&entry_point)?;
    
    // Resolve dependencies
    let mut resolver = DependencyResolver::new("templates/");
    resolver.add_entry(&entry_point, &entry_content)?;
    let modules = resolver.resolve()?;
    
    // Combine modules with IIFE wrapping
    let mut combined_js = String::new();
    for module in &modules {
        combined_js.push_str(
            &wrap_module(&module.path, &module.content)
        );
    }
    
    // Tree shaking
    let shaken = tree_shake(&combined_js, &modules)?;
    
    // Minification
    let minified = minify_js(&shaken)?;
    
    // Source map
    let source_map = SourceMap::new();
    let map_content = source_map.generate()?;
    
    // Write output
    let output_path = format!(
        "public/session/scripts/{}.{}.js",
        layout, session_hash
    );
    write_js_file(&output_path, &minified)?;
    
    Ok(BundleResult {
        output_path,
        original_size: combined_js.len(),
        shaken_size: shaken.len(),
        minified_size: minified.len(),
        reduction_percent: calculate_reduction(
            combined_js.len(), 
            minified.len()
        ),
    })
}
```

## Dependency Resolution

### Import Parsing

```rust
pub struct DependencyResolver {
    modules: HashMap<String, Module>,
    base_path: PathBuf,
}

impl DependencyResolver {
    pub fn add_entry(&mut self, path: &str, content: &str) 
        -> ReedResult<()> 
    {
        // Parse ES6 imports
        let es6_imports = parse_es6_imports(content)?;
        // import { foo } from './module.js';
        
        // Parse CommonJS requires
        let cjs_requires = parse_cjs_requires(content)?;
        // const bar = require('./other.js');
        
        // Add module to graph
        self.modules.insert(path.to_string(), Module {
            path: path.to_string(),
            content: content.to_string(),
            dependencies: [es6_imports, cjs_requires].concat(),
        });
        
        Ok(())
    }
    
    pub fn resolve(&mut self) -> ReedResult<Vec<Module>> {
        // Topological sort for correct load order
        let sorted = self.topological_sort()?;
        
        // Detect circular dependencies
        self.check_circular_deps(&sorted)?;
        
        Ok(sorted)
    }
}
```

### Import Examples

#### ES6 Imports

```javascript
// Relative import
import { navigation } from './components/navigation.js';

// Component import
import { Button } from '../../components/atoms/button/button.js';

// Default import
import Header from '../organisms/page-header/page-header.js';
```

#### CommonJS Requires

```javascript
// Relative require
const utils = require('./utils.js');

// Component require
const Icon = require('../../components/atoms/icon/icon.js');
```

## Module Wrapping

### IIFE Pattern

```javascript
// Original module
export function greet(name) {
    return `Hello, ${name}!`;
}

// Wrapped module
(function(module, exports) {
    function greet(name) {
        return `Hello, ${name}!`;
    }
    exports.greet = greet;
})({exports: {}}, {});
```

### Purpose

- **Scope Isolation**: Variables don't pollute global scope
- **Module System**: Provides `module` and `exports` objects
- **Compatibility**: Works with both ES6 and CommonJS patterns

## Tree Shaking

### Algorithm

```rust
pub fn tree_shake(js: &str, modules: &[Module]) 
    -> ReedResult<String> 
{
    // Build export/import graph
    let exports = find_all_exports(js)?;
    let imports = find_all_imports(js)?;
    
    // Mark used exports
    let mut used = HashSet::new();
    for import in &imports {
        used.insert(import.name.clone());
    }
    
    // Remove unused exports
    let mut shaken = js.to_string();
    for export in &exports {
        if !used.contains(&export.name) {
            shaken = remove_export(&shaken, export)?;
        }
    }
    
    Ok(shaken)
}
```

### Example

```javascript
// Before tree shaking (5 KB)
export function usedFunction() { /* ... */ }
export function unusedFunction() { /* ... */ }  // Not imported
export const unusedConst = 42;                  // Not imported

// After tree shaking (4 KB, -20%)
export function usedFunction() { /* ... */ }
```

## Minification

### Algorithm

```rust
pub fn minify_js(js: &str) -> ReedResult<String> {
    // 1. Remove comments
    let no_comments = remove_js_comments(js);
    
    // 2. Rename variables to shortest names
    let renamed = rename_variables(&no_comments);
    
    // 3. Remove whitespace
    let no_whitespace = remove_whitespace(&renamed);
    
    // 4. Optimise expressions
    let optimised = optimise_expressions(&no_whitespace);
    
    Ok(optimised)
}
```

### Optimisations

| Optimisation | Before | After | Savings |
|--------------|--------|-------|---------|
| Comments | `// Header logic` | _(removed)_ | 100% |
| Variable names | `const headerElement = ...` | `const a=...` | ~60% |
| Whitespace | `function foo() {\n  ...\n}` | `function foo(){...}` | ~40% |
| Expressions | `true === value` | `value` | ~50% |
| **Total** | **20 KB** | **8 KB** | **~60%** |

### Variable Renaming

```javascript
// Before minification
function calculateTotalPrice(itemPrice, quantity, taxRate) {
    const subtotal = itemPrice * quantity;
    const tax = subtotal * taxRate;
    const total = subtotal + tax;
    return total;
}

// After minification
function calculateTotalPrice(a,b,c){const d=a*b,e=d*c;return d+e}
```

## Performance

| Operation | Timing | Note |
|-----------|--------|------|
| Parse imports | < 20ms | Per entry point |
| Resolve deps | < 30ms | ~10 modules |
| Wrap modules | < 10ms | IIFE wrapping |
| Tree shaking | < 40ms | Dead code removal |
| Minification | < 80ms | Variable renaming |
| Source map | < 10ms | JSON generation |
| Write | < 10ms | Single file |
| **Total** | **< 200ms** | Per layout |

### Batch Performance

```
10 layouts = 10 bundles
Total time: < 1s
Average: ~100ms per bundle
```

## Variant Independence

**Critical**: Unlike CSS, JavaScript is **not variant-specific**.

```
CSS (variant-specific):
├─ landing.a3f5b2c8.mouse.css
├─ landing.a3f5b2c8.touch.css
└─ landing.a3f5b2c8.reader.css

JS (variant-independent):
└─ landing.a3f5b2c8.js  # Works for all variants
```

**Reason**: Device detection and responsive behaviour handled via CSS media queries and feature detection in JavaScript, not separate bundles.

## CLI Integration

### Build Command

```bash
# Bundle all JavaScript
reed build:js

# Output:
📦 Bundling JS for landing.mouse...
  - Resolved: 5 modules
  - Included: templates/layouts/landing/landing.js (8.4 KB)
  - Included: .../navigation/navigation.js (6.1 KB)
  → Output: public/session/scripts/landing.a3f5b2c8.js (4.2 KB, -70%)
✓ Bundle complete

📦 Bundling JS for knowledge.mouse...
  ⓘ No JavaScript file for knowledge.mouse

📊 Total JS Bundle Statistics:
  Original size: 124.6 KB
  Minified size: 49.8 KB
  Size reduction: 60%
  Bundles created: 7
```

### Watch Mode

```bash
# Auto-rebuild on JS changes
reed build:watch

# Watches:
# - templates/**/*.js
# - Rebuilds affected bundles only
# - Updates session hash if content changed
```

## Development vs Production

### Development Mode

```bash
# DEV mode: No minification, readable output
reed build:js --dev

# Output:
landing.a3f5b2c8.js
├─ Original variable names
├─ Preserved comments
├─ Readable formatting
└─ Inline source maps
```

### Production Mode

```bash
# PROD mode: Full minification
reed build:js --prod

# Output:
landing.a3f5b2c8.js        # Minified
landing.a3f5b2c8.js.map    # Source map
├─ Short variable names (a, b, c)
├─ No comments
├─ No whitespace
└─ Maximum compression
```

## Troubleshooting

### Circular Dependency Detected

```
Error: CircularDependency { 
    modules: ["a.js", "b.js", "a.js"] 
}
```

**Solution**: Refactor modules to break circular imports

```javascript
// BAD: Circular dependency
// a.js
import { b } from './b.js';
export const a = () => b();

// b.js
import { a } from './a.js';  // Circular!
export const b = () => a();

// GOOD: Extract shared logic
// shared.js
export const shared = () => { /* ... */ };

// a.js
import { shared } from './shared.js';
export const a = () => shared();

// b.js
import { shared } from './shared.js';
export const b = () => shared();
```

### Module Not Found

```
Error: ModuleNotFound { 
    path: "./components/navigation.js",
    context: "templates/layouts/landing/landing.js"
}
```

**Solution**: Check import path relative to layout file

```javascript
// From: templates/layouts/landing/landing.js

// WRONG: Absolute path
import { Nav } from '/components/organisms/navigation/navigation.js';

// CORRECT: Relative path
import { Nav } from '../../components/organisms/navigation/navigation.js';
```

### Tree Shaking Removes Used Code

**Symptom**: Runtime error `function is not defined`

**Cause**: Function imported but not detected by tree shaker

**Solution**: Use explicit exports/imports

```javascript
// BAD: Dynamic require (not detectable)
const moduleName = 'navigation';
const Nav = require(`./${moduleName}.js`);

// GOOD: Static import (detectable)
import { Nav } from './navigation.js';
```

### Minification Breaks Code

**Symptom**: JavaScript errors after minification

**Cause**: Code relies on function/variable names (reflection, eval)

**Solution**: Mark functions as minification-safe

```javascript
// BAD: Relies on function name
function myFunction() { /* ... */ }
console.log(myFunction.name);  // "myFunction" → "a" (broken!)

// GOOD: Use string literal
function myFunction() { /* ... */ }
const FUNCTION_NAME = 'myFunction';
console.log(FUNCTION_NAME);  // Always "myFunction"
```

## Related Documentation

- [Session Hash](session-hash.md) - Cache-busting strategy
- [CSS Bundler](css-bundler.md) - CSS bundling process
- [Static Server](static-server.md) - Serving bundled JavaScript

## CLI Reference

```bash
# Bundle all JavaScript
reed build:js

# Bundle specific layout
reed build:js --layout=landing

# Development mode (no minification)
reed build:js --dev

# Watch mode (auto-rebuild)
reed build:watch

# View bundle analysis
reed build:js --analyse
```
