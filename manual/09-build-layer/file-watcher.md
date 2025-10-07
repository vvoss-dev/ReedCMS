# File Watcher

Auto-rebuilds assets during development with intelligent change detection and debouncing.

## Purpose

The file watcher dramatically improves development workflow:

- **Auto-Rebuild**: Detects file changes and rebuilds affected assets
- **Incremental**: Only rebuilds changed components (< 2s)
- **Hot-Reload**: Templates and configs reload without server restart
- **Debouncing**: Batches rapid changes to prevent wasted rebuilds
- **Real-Time Feedback**: Terminal output shows build progress

## Architecture

```
┌─────────────────────────────────────────────────────┐
│ File Watcher Flow                                   │
├─────────────────────────────────────────────────────┤
│                                                     │
│  File System Events                                 │
│  ├─ templates/**/*.{jinja,css,js}                   │
│  ├─ assets/css/**/*.css                             │
│  ├─ assets/js/**/*.js                               │
│  └─ .reed/*.csv                                     │
│           ↓                                         │
│  [1] Event Detection (notify crate)                 │
│  ├─ Write: File modified                            │
│  ├─ Create: File created                            │
│  ├─ Remove: File deleted                            │
│  └─ Rename: File renamed                            │
│           ↓                                         │
│  [2] Debouncing (300ms window)                      │
│  ├─ Collect: Multiple events                        │
│  ├─ Batch: Related changes                          │
│  └─ Trigger: Single rebuild                         │
│           ↓                                         │
│  [3] Change Detection                               │
│  ├─ Analyse: File path                              │
│  ├─ Determine: Rebuild scope                        │
│  └─ Decision: Full vs. increental                   │
│           ↓                                         │
│  [4] Rebuild Execution                              │
│  ├─ Core CSS → Rebuild all CSS                      │
│  ├─ Layout CSS → Rebuild specific layout            │
│  ├─ Core JS → Rebuild all JS                        │
│  ├─ Layout JS → Rebuild specific layout             │
│  ├─ Template → Hot-reload template                  │
│  └─ Config → Reload ReedBase cache                  │
│           ↓                                         │
│  [5] Feedback                                       │
│  └─ Terminal: Build status and timing               │
│                                                     │
└─────────────────────────────────────────────────────┘
```

## Watched Directories

### CSS Files

```
assets/css/
├── core/              → Full CSS rebuild
│   ├── reset.css
│   └── variables.css
├── components/        → Full CSS rebuild
│   ├── atoms/
│   ├── molecules/
│   └── organisms/
└── layouts/           → Incremental rebuild
    ├── knowledge/
    │   ├── knowledge.mouse.css    → Rebuild knowledge.mouse only
    │   └── knowledge.touch.css    → Rebuild knowledge.touch only
    └── blog/
        └── blog.mouse.css         → Rebuild blog.mouse only
```

### JavaScript Files

```
assets/js/
├── core/              → Full JS rebuild
│   ├── polyfills.js
│   └── utilities.js
├── components/        → Full JS rebuild
│   ├── organisms/
│   └── molecules/
└── layouts/           → Incremental rebuild
    ├── knowledge/
    │   └── knowledge.js           → Rebuild knowledge only
    └── blog/
        └── blog.js                → Rebuild blog only
```

### Templates

```
templates/
├── layouts/
│   ├── knowledge/
│   │   └── knowledge.mouse.jinja  → Hot-reload template
│   └── blog/
│       └── blog.touch.jinja       → Hot-reload template
└── components/
    ├── organisms/
    │   └── page-header/
    │       └── page-header.jinja  → Hot-reload template
    └── atoms/
        └── icon/
            └── icon.jinja         → Hot-reload template
```

### Configuration

```
.reed/
├── text.csv           → Reload ReedBase cache
├── routes.csv         → Reload ReedBase cache
├── meta.csv           → Reload ReedBase cache
└── project.csv        → Reload ReedBase cache
```

## Implementation

### Main Watcher Function

```rust
pub async fn start_watcher() -> ReedResult<()> {
    println!("👀 Watching for file changes...");
    println!("  CSS: assets/css/");
    println!("  JS: assets/js/");
    println!("  Templates: templates/");
    println!("  Config: .reed/");
    println!("\nPress Ctrl+C to stop\n");

    // Create watcher with 300ms debounce
    let (tx, rx) = mpsc::channel();
    let mut watcher = notify::watcher(
        tx,
        Duration::from_millis(300)
    )?;

    // Watch directories recursively
    watcher.watch("assets/css", RecursiveMode::Recursive)?;
    watcher.watch("assets/js", RecursiveMode::Recursive)?;
    watcher.watch("templates", RecursiveMode::Recursive)?;
    watcher.watch(".reed", RecursiveMode::NonRecursive)?;

    // Process events
    loop {
        match rx.recv() {
            Ok(event) => {
                handle_file_event(event).await?;
            }
            Err(e) => {
                eprintln!("⚠ Watcher error: {:?}", e);
                break;
            }
        }
    }

    Ok(())
}
```

### Event Handling

```rust
async fn handle_file_event(event: DebouncedEvent)
    -> ReedResult<()>
{
    use DebouncedEvent::*;

    match event {
        Write(path) | Create(path) => {
            let timestamp = Local::now()
                .format("[%H:%M:%S]");
            println!("{} Changed: {}",
                timestamp,
                path.display()
            );

            handle_file_change(&path).await?;
        }
        Remove(path) => {
            let timestamp = Local::now()
                .format("[%H:%M:%S]");
            println!("{} Removed: {}",
                timestamp,
                path.display()
            );
        }
        _ => {}
    }

    Ok(())
}
```

## Change Detection

### Detection Logic

```rust
async fn handle_file_change(path: &Path) -> ReedResult<()> {
    let path_str = path.to_string_lossy();

    if path_str.starts_with("assets/css/") {
        handle_css_change(&path_str).await?;
    } else if path_str.starts_with("assets/js/") {
        handle_js_change(&path_str).await?;
    } else if path_str.starts_with("templates/") {
        handle_template_change(&path_str).await?;
    } else if path_str.starts_with(".reed/") {
        handle_config_change(&path_str).await?;
    }

    Ok(())
}
```

### CSS Change Handling

```rust
async fn handle_css_change(path: &str) -> ReedResult<()> {
    // Core or component CSS → Full rebuild
    if path.contains("assets/css/core/") ||
       path.contains("assets/css/components/")
    {
        println!("🔨 Rebuilding all CSS bundles...");
        let start = Instant::now();

        match bundle_all_css() {
            Ok(_) => {
                println!("✓ Rebuilt in {:.1}s\n",
                    start.elapsed().as_secs_f32());
            }
            Err(e) => {
                eprintln!("⚠ Rebuild failed: {:?}\n", e);
            }
        }
    }
    // Layout CSS → Incremental rebuild
    else if path.contains("assets/css/layouts/") {
        if let Some((layout, variant)) =
            extract_layout_variant(path, "css")
        {
            println!("🔨 Rebuilding {}.{}.css...",
                layout, variant);
            let start = Instant::now();

            match bundle_css(&layout, &variant) {
                Ok(_) => {
                    println!("✓ Rebuilt in {:.1}s\n",
                        start.elapsed().as_secs_f32());
                }
                Err(e) => {
                    eprintln!("⚠ Rebuild failed: {:?}\n", e);
                }
            }
        }
    }

    Ok(())
}
```

### JavaScript Change Handling

```rust
async fn handle_js_change(path: &str) -> ReedResult<()> {
    // Core or component JS → Full rebuild
    if path.contains("assets/js/core/") ||
       path.contains("assets/js/components/")
    {
        println!("🔨 Rebuilding all JS bundles...");
        let start = Instant::now();

        match bundle_all_js() {
            Ok(_) => {
                println!("✓ Rebuilt in {:.1}s\n",
                    start.elapsed().as_secs_f32());
            }
            Err(e) => {
                eprintln!("⚠ Rebuild failed: {:?}\n", e);
            }
        }
    }
    // Layout JS → Incremental rebuild
    else if path.contains("assets/js/layouts/") {
        if let Some((layout, _)) =
            extract_layout_variant(path, "js")
        {
            println!("🔨 Rebuilding {}.js...", layout);
            let start = Instant::now();

            match bundle_js(&layout, "mouse") {
                Ok(_) => {
                    println!("✓ Rebuilt in {:.1}s\n",
                        start.elapsed().as_secs_f32());
                }
                Err(e) => {
                    eprintln!("⚠ Rebuild failed: {:?}\n", e);
                }
            }
        }
    }

    Ok(())
}
```

### Template Hot-Reload

```rust
async fn handle_template_change(path: &str)
    -> ReedResult<()>
{
    println!("🔄 Reloading template...");

    // Clear template cache (MiniJinja reloads automatically)
    match clear_template_cache() {
        Ok(_) => {
            println!("✓ Template reloaded\n");
        }
        Err(e) => {
            eprintln!("⚠ Template reload failed: {:?}\n", e);
        }
    }

    Ok(())
}

fn clear_template_cache() -> ReedResult<()> {
    // Implementation depends on Layer 05
    // MiniJinja auto-reloads in DEV mode
    Ok(())
}
```

### Config Reload

```rust
async fn handle_config_change(path: &str)
    -> ReedResult<()>
{
    println!("🔄 Reloading configuration...");

    // Reload ReedBase cache for changed CSV
    match reload_reedbase_cache() {
        Ok(_) => {
            println!("✓ Configuration reloaded\n");
        }
        Err(e) => {
            eprintln!("⚠ Config reload failed: {:?}\n", e);
        }
    }

    Ok(())
}

fn reload_reedbase_cache() -> ReedResult<()> {
    // Clear and reload HashMap cache
    // Implementation depends on Layer 02
    Ok(())
}
```

## Path Parsing

### Layout/Variant Extraction

```rust
fn extract_layout_variant(path: &str, asset_type: &str)
    -> Option<(String, String)>
{
    // Pattern: assets/{type}/layouts/{layout}/{layout}.{variant}.{ext}
    let pattern = format!("assets/{}/layouts/", asset_type);

    if let Some(start) = path.find(&pattern) {
        let after = &path[start + pattern.len()..];
        let parts: Vec<&str> = after.split('/').collect();

        if parts.len() >= 2 {
            let layout = parts[0].to_string();
            let filename = parts[1];

            // Extract variant from filename
            // knowledge.mouse.css → mouse
            let name_parts: Vec<&str> = filename.split('.').collect();
            if name_parts.len() >= 2 {
                let variant = name_parts[1].to_string();
                return Some((layout, variant));
            }
        }
    }

    None
}
```

**Examples**:
```rust
extract_layout_variant(
    "assets/css/layouts/knowledge/knowledge.mouse.css",
    "css"
)
// → Some(("knowledge", "mouse"))

extract_layout_variant(
    "assets/js/layouts/blog/blog.js",
    "js"
)
// → Some(("blog", "mouse"))  // JS is variant-independent
```

## Debouncing

### Purpose

Prevent rapid repeated rebuilds when multiple files change:

```
Without debouncing:
Save file 1 → Rebuild (2s)
Save file 2 → Rebuild (2s)
Save file 3 → Rebuild (2s)
Total: 6s

With debouncing (300ms):
Save file 1 → Wait...
Save file 2 → Wait...
Save file 3 → Wait...
300ms elapsed → Rebuild once (2s)
Total: 2.3s
```

### Implementation

```rust
pub struct Debouncer {
    delay: Duration,
    last_event: Arc<Mutex<Option<Instant>>>,
}

impl Debouncer {
    pub fn new(delay_ms: u64) -> Self {
        Self {
            delay: Duration::from_millis(delay_ms),
            last_event: Arc::new(Mutex::new(None)),
        }
    }

    pub fn should_process(&self) -> bool {
        let mut last = self.last_event.lock().unwrap();
        let now = Instant::now();

        if let Some(last_time) = *last {
            if now.duration_since(last_time) < self.delay {
                return false;  // Within debounce window
            }
        }

        *last = Some(now);
        true  // Process event
    }
}
```

## CLI Integration

### Start Watcher

```bash
# Start file watcher
reed build:watch

# Output:
👀 Watching for file changes...
  CSS: assets/css/
  JS: assets/js/
  Templates: templates/
  Config: .reed/

Press Ctrl+C to stop

[12:34:56] Changed: assets/css/layouts/blog/blog.mouse.css
🔨 Rebuilding blog.mouse.css...
✓ Rebuilt in 1.2s

[12:35:42] Changed: templates/layouts/knowledge/knowledge.jinja
🔄 Reloading template...
✓ Template reloaded
```

### Initial Build

```rust
pub async fn execute_build_watch() -> ReedResult<()> {
    println!("🚀 Starting ReedCMS build watcher...\n");

    // Run initial full build
    println!("📦 Running initial build...");
    match run_pipeline(BuildMode::Full).await {
        Ok(report) => {
            println!("✓ Initial build complete\n");
        }
        Err(e) => {
            eprintln!("⚠ Initial build failed: {:?}\n", e);
            eprintln!("Continuing with watcher anyway...\n");
        }
    }

    // Start watcher
    start_watcher().await?;

    Ok(())
}
```

## Performance

| Operation | Timing | Note |
|-----------|--------|------|
| **Change detection** | < 10ms | File system event → decision |
| **Incremental CSS rebuild** | < 2s | Single layout/variant |
| **Incremental JS rebuild** | < 2s | Single layout |
| **Template hot-reload** | < 100ms | No rebuild needed |
| **Config reload** | < 100ms | HashMap refresh |
| **Full CSS rebuild** | ~8s | All layouts × variants |
| **Full JS rebuild** | ~4s | All layouts |

## Rebuild Scope Matrix

| Changed File | Rebuild Scope | Reason |
|-------------|---------------|--------|
| `assets/css/core/reset.css` | All CSS | Core affects all layouts |
| `assets/css/components/atoms/icon/icon.css` | All CSS | Shared component |
| `assets/css/layouts/blog/blog.mouse.css` | `blog.mouse.css` only | Layout-specific |
| `assets/js/core/utilities.js` | All JS | Core affects all layouts |
| `assets/js/components/organisms/nav/nav.js` | All JS | Shared component |
| `assets/js/layouts/knowledge/knowledge.js` | `knowledge.js` only | Layout-specific |
| `templates/layouts/blog/blog.jinja` | Hot-reload | No rebuild |
| `.reed/text.csv` | Config reload | No rebuild |

## Troubleshooting

### Watcher Not Detecting Changes

**Cause**: File system watch limit exceeded (Linux)

**Solution**: Increase inotify limit

```bash
# Check current limit
cat /proc/sys/fs/inotify/max_user_watches

# Increase limit
echo 524288 | sudo tee /proc/sys/fs/inotify/max_user_watches

# Make permanent
echo "fs.inotify.max_user_watches=524288" | \
  sudo tee /etc/sysctl.d/99-inotify.conf
```

### Changes Triggering Multiple Rebuilds

**Cause**: Debouncing not working, or delay too short

**Solution**: Increase debounce delay

```rust
// Change from 300ms to 500ms
let mut watcher = notify::watcher(
    tx,
    Duration::from_millis(500)  // Increased delay
)?;
```

### Rebuild Too Slow

**Problem**: Incremental rebuild takes > 5s

**Cause**: Full rebuild triggered instead of incremental

**Solution**: Check rebuild scope detection

```bash
# Expected output (incremental):
🔨 Rebuilding blog.mouse.css...
✓ Rebuilt in 1.2s

# Unexpected output (full rebuild):
🔨 Rebuilding all CSS bundles...
✓ Rebuilt in 8.7s
```

### Template Not Reloading

**Cause**: MiniJinja hot-reload disabled or cache not clearing

**Solution**: Enable hot-reload in dev mode

```rust
// In template engine setup (Layer 05)
let env = Environment::new();
env.set_auto_reload(true);  // Enable hot-reload
```

## Related Documentation

- [Build Pipeline](build-pipeline.md) - Asset orchestration
- [Binary Compiler](binary-compiler.md) - Release builds
- [MiniJinja Integration](../05-template-layer/minijinja-integration.md) - Template hot-reload
- [ReedBase Cache](../02-data-layer/reedbase-cache.md) - Config reload

## CLI Reference

```bash
# Development workflow
reed build:watch             # Start file watcher
reed build:watch --verbose   # Show detailed logs
reed build:watch --debounce=500  # Custom debounce (ms)

# Manual rebuilds
reed build:assets            # Full rebuild
reed build:css --layout=blog # Rebuild specific CSS
reed build:js --layout=blog  # Rebuild specific JS
```
