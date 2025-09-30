# REED-09-03: File Watcher and Auto-Rebuild System

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
- **ID**: REED-09-03
- **Title**: File Watcher and Auto-Rebuild System
- **Layer**: Build Layer (REED-09)
- **Priority**: Medium
- **Status**: Open
- **Complexity**: Medium
- **Dependencies**: REED-09-02

## Summary Reference
- **Section**: File Watcher
- **Lines**: 1041-1043 in project_summary.md
- **Key Concepts**: File system monitoring, incremental rebuilds, change detection, debouncing

## Objective
Implement file watcher system that monitors source files for changes, detects which assets need rebuilding, triggers incremental builds with debouncing to prevent redundant builds, and provides real-time feedback during development.

## Requirements

### Watched Directories

```
assets/css/           → Trigger CSS rebuild
assets/js/            → Trigger JS rebuild
templates/            → Trigger template reload (DEV mode)
.reed/                → Trigger config reload
```

### Change Detection Strategy

```
CSS Change:
  assets/css/core/reset.css
  └─> Rebuild all CSS bundles (core CSS affects all)

Layout CSS Change:
  assets/css/layouts/knowledge/knowledge.mouse.css
  └─> Rebuild knowledge.mouse.css only

JS Change:
  assets/js/layouts/blog/blog.touch.js
  └─> Rebuild blog.touch.js only

Template Change:
  templates/layouts/knowledge/knowledge.mouse.jinja
  └─> Reload template (hot-reload in DEV)

Config Change:
  .reed/text.csv
  └─> Reload ReedBase cache
```

### Implementation (`build/watcher.rs`)

```rust
/// Starts file watcher for development mode.
///
/// ## Watch Targets
/// - assets/css/ - CSS files
/// - assets/js/ - JavaScript files
/// - templates/ - Template files (hot-reload)
/// - .reed/ - Config files
///
/// ## Debouncing
/// - 300ms delay after last change
/// - Prevents multiple rapid rebuilds
/// - Batches related changes
///
/// ## Performance
/// - Change detection: < 10ms
/// - Incremental rebuild: < 2s
/// - Template reload: < 100ms
///
/// ## Output
/// ```
/// 👀 Watching for file changes...
///   CSS: assets/css/
///   JS: assets/js/
///   Templates: templates/
///   Config: .reed/
///
/// Press Ctrl+C to stop
///
/// [12:34:56] Changed: assets/css/layouts/knowledge/knowledge.mouse.css
/// 🔨 Rebuilding knowledge.mouse.css...
/// ✓ Rebuilt in 1.2s
///
/// [12:35:12] Changed: templates/layouts/blog/blog.touch.jinja
/// 🔄 Reloading template...
/// ✓ Template reloaded
/// ```
pub async fn start_watcher() -> ReedResult<()> {
    println!("👀 Watching for file changes...");
    println!("  CSS: assets/css/");
    println!("  JS: assets/js/");
    println!("  Templates: templates/");
    println!("  Config: .reed/");
    println!("\nPress Ctrl+C to stop\n");

    // Create watcher
    let (tx, rx) = std::sync::mpsc::channel();
    let mut watcher = notify::watcher(tx, std::time::Duration::from_millis(300))
        .map_err(|e| ReedError::WatcherError {
            reason: e.to_string(),
        })?;

    // Watch directories
    watcher.watch("assets/css", notify::RecursiveMode::Recursive)
        .map_err(|e| ReedError::WatcherError { reason: e.to_string() })?;

    watcher.watch("assets/js", notify::RecursiveMode::Recursive)
        .map_err(|e| ReedError::WatcherError { reason: e.to_string() })?;

    watcher.watch("templates", notify::RecursiveMode::Recursive)
        .map_err(|e| ReedError::WatcherError { reason: e.to_string() })?;

    watcher.watch(".reed", notify::RecursiveMode::NonRecursive)
        .map_err(|e| ReedError::WatcherError { reason: e.to_string() })?;

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

/// Handles file system event.
async fn handle_file_event(event: notify::DebouncedEvent) -> ReedResult<()> {
    use notify::DebouncedEvent::*;

    match event {
        Write(path) | Create(path) => {
            let timestamp = chrono::Local::now().format("[%H:%M:%S]");
            println!("{} Changed: {}", timestamp, path.display());

            handle_file_change(&path).await?;
        }
        Remove(path) => {
            let timestamp = chrono::Local::now().format("[%H:%M:%S]");
            println!("{} Removed: {}", timestamp, path.display());
        }
        _ => {}
    }

    Ok(())
}

/// Handles file change based on path.
async fn handle_file_change(path: &std::path::Path) -> ReedResult<()> {
    let path_str = path.to_string_lossy().to_string();

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

/// Handles CSS file change.
async fn handle_css_change(path: &str) -> ReedResult<()> {
    // Determine rebuild scope
    if path.contains("assets/css/core/") || path.contains("assets/css/components/") {
        // Core or component CSS changed - rebuild all
        println!("🔨 Rebuilding all CSS bundles...");
        let start = std::time::Instant::now();

        match reedcms::assets::css::bundler::bundle_all_css() {
            Ok(_) => {
                println!("✓ Rebuilt in {:.1}s\n", start.elapsed().as_secs_f32());
            }
            Err(e) => {
                eprintln!("⚠ Rebuild failed: {:?}\n", e);
            }
        }
    } else if path.contains("assets/css/layouts/") {
        // Layout CSS changed - rebuild specific layout/variant
        if let Some((layout, variant)) = extract_layout_variant_from_path(path, "css") {
            println!("🔨 Rebuilding {}.{}.css...", layout, variant);
            let start = std::time::Instant::now();

            match reedcms::assets::css::bundler::bundle_css(&layout, &variant) {
                Ok(_) => {
                    println!("✓ Rebuilt in {:.1}s\n", start.elapsed().as_secs_f32());
                }
                Err(e) => {
                    eprintln!("⚠ Rebuild failed: {:?}\n", e);
                }
            }
        }
    }

    Ok(())
}

/// Handles JavaScript file change.
async fn handle_js_change(path: &str) -> ReedResult<()> {
    // Determine rebuild scope
    if path.contains("assets/js/core/") || path.contains("assets/js/components/") {
        // Core or component JS changed - rebuild all
        println!("🔨 Rebuilding all JS bundles...");
        let start = std::time::Instant::now();

        match reedcms::assets::js::bundler::bundle_all_js() {
            Ok(_) => {
                println!("✓ Rebuilt in {:.1}s\n", start.elapsed().as_secs_f32());
            }
            Err(e) => {
                eprintln!("⚠ Rebuild failed: {:?}\n", e);
            }
        }
    } else if path.contains("assets/js/layouts/") {
        // Layout JS changed - rebuild specific layout/variant
        if let Some((layout, variant)) = extract_layout_variant_from_path(path, "js") {
            println!("🔨 Rebuilding {}.{}.js...", layout, variant);
            let start = std::time::Instant::now();

            match reedcms::assets::js::bundler::bundle_js(&layout, &variant) {
                Ok(_) => {
                    println!("✓ Rebuilt in {:.1}s\n", start.elapsed().as_secs_f32());
                }
                Err(e) => {
                    eprintln!("⚠ Rebuild failed: {:?}\n", e);
                }
            }
        }
    }

    Ok(())
}

/// Handles template file change.
async fn handle_template_change(path: &str) -> ReedResult<()> {
    println!("🔄 Reloading template...");

    // In DEV mode, templates are hot-reloaded automatically
    // Just clear the template cache
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

/// Handles config file change.
async fn handle_config_change(path: &str) -> ReedResult<()> {
    println!("🔄 Reloading configuration...");

    // Reload ReedBase cache for changed CSV file
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

/// Extracts layout and variant from file path.
///
/// ## Examples
/// - assets/css/layouts/knowledge/knowledge.mouse.css → ("knowledge", "mouse")
/// - assets/js/layouts/blog/blog.touch.js → ("blog", "touch")
fn extract_layout_variant_from_path(path: &str, asset_type: &str) -> Option<(String, String)> {
    let pattern = format!("assets/{}/layouts/", asset_type);

    if let Some(start) = path.find(&pattern) {
        let after = &path[start + pattern.len()..];
        let parts: Vec<&str> = after.split('/').collect();

        if parts.len() >= 2 {
            let layout = parts[0].to_string();
            let filename = parts[1];

            // Extract variant from filename (e.g., knowledge.mouse.css → mouse)
            let name_parts: Vec<&str> = filename.split('.').collect();
            if name_parts.len() >= 2 {
                let variant = name_parts[1].to_string();
                return Some((layout, variant));
            }
        }
    }

    None
}

/// Clears template cache for hot-reload.
fn clear_template_cache() -> ReedResult<()> {
    // Access template engine and clear cache
    // Implementation depends on REED-05-02
    Ok(())
}

/// Reloads ReedBase cache.
fn reload_reedbase_cache() -> ReedResult<()> {
    // Clear and reload ReedBase HashMap cache
    // Implementation depends on REED-02-01
    Ok(())
}
```

### Debouncing (`build/debounce.rs`)

```rust
/// Debounces file system events.
///
/// ## Strategy
/// - Collects events for 300ms
/// - Batches related changes
/// - Triggers single rebuild
///
/// ## Benefits
/// - Prevents rapid repeated rebuilds
/// - Handles multiple file saves
/// - Improves performance
pub struct Debouncer {
    delay: std::time::Duration,
    last_event: std::sync::Arc<std::sync::Mutex<Option<std::time::Instant>>>,
}

impl Debouncer {
    /// Creates new debouncer with delay.
    pub fn new(delay_ms: u64) -> Self {
        Self {
            delay: std::time::Duration::from_millis(delay_ms),
            last_event: std::sync::Arc::new(std::sync::Mutex::new(None)),
        }
    }

    /// Checks if event should be processed.
    ///
    /// ## Returns
    /// - true if enough time has passed since last event
    /// - false if within debounce window
    pub fn should_process(&self) -> bool {
        let mut last = self.last_event.lock().unwrap();
        let now = std::time::Instant::now();

        if let Some(last_time) = *last {
            if now.duration_since(last_time) < self.delay {
                return false;
            }
        }

        *last = Some(now);
        true
    }

    /// Resets debouncer.
    pub fn reset(&self) {
        let mut last = self.last_event.lock().unwrap();
        *last = None;
    }
}
```

### Change Detection (`build/change_detect.rs`)

```rust
/// Detects what needs rebuilding based on changed file.
///
/// ## Detection Rules
/// - Core CSS change → Rebuild all CSS
/// - Layout CSS change → Rebuild specific layout CSS
/// - Core JS change → Rebuild all JS
/// - Layout JS change → Rebuild specific layout JS
/// - Template change → Reload template
/// - Config change → Reload config
pub fn detect_rebuild_scope(path: &str) -> RebuildScope {
    if path.starts_with("assets/css/core/") || path.starts_with("assets/css/components/") {
        RebuildScope::AllCss
    } else if path.starts_with("assets/css/layouts/") {
        if let Some((layout, variant)) = extract_layout_variant(path, "css") {
            RebuildScope::SpecificCss { layout, variant }
        } else {
            RebuildScope::AllCss
        }
    } else if path.starts_with("assets/js/core/") || path.starts_with("assets/js/components/") {
        RebuildScope::AllJs
    } else if path.starts_with("assets/js/layouts/") {
        if let Some((layout, variant)) = extract_layout_variant(path, "js") {
            RebuildScope::SpecificJs { layout, variant }
        } else {
            RebuildScope::AllJs
        }
    } else if path.starts_with("templates/") {
        RebuildScope::Template { path: path.to_string() }
    } else if path.starts_with(".reed/") {
        RebuildScope::Config { path: path.to_string() }
    } else {
        RebuildScope::None
    }
}

/// Rebuild scope enum.
#[derive(Debug, Clone, PartialEq)]
pub enum RebuildScope {
    AllCss,
    SpecificCss { layout: String, variant: String },
    AllJs,
    SpecificJs { layout: String, variant: String },
    Template { path: String },
    Config { path: String },
    None,
}

/// Extracts layout and variant from path.
fn extract_layout_variant(path: &str, asset_type: &str) -> Option<(String, String)> {
    // Implementation same as in watcher.rs
    None
}
```

### CLI Command (`src/reedcms/cli/commands/build_watch.rs`)

```rust
/// CLI command: reed build:watch
///
/// ## Usage
/// ```bash
/// reed build:watch
/// ```
///
/// ## Description
/// Starts file watcher and rebuilds assets on changes.
/// Ideal for development workflow.
pub async fn execute_build_watch() -> ReedResult<()> {
    println!("🚀 Starting ReedCMS build watcher...\n");

    // Initial build
    println!("📦 Running initial build...");
    match build::pipeline::run_pipeline(build::pipeline::BuildMode::Full).await {
        Ok(report) => {
            println!("✓ Initial build complete\n");
        }
        Err(e) => {
            eprintln!("⚠ Initial build failed: {:?}\n", e);
            eprintln!("Continuing with watcher anyway...\n");
        }
    }

    // Start watcher
    build::watcher::start_watcher().await?;

    Ok(())
}
```

## Implementation Files

### Primary Implementation
- `build/watcher.rs` - File watcher
- `build/debounce.rs` - Event debouncing
- `build/change_detect.rs` - Change detection
- `src/reedcms/cli/commands/build_watch.rs` - CLI command

### Test Files
- `build/watcher.test.rs`
- `build/debounce.test.rs`
- `build/change_detect.test.rs`

## Testing Requirements

### Unit Tests
- [ ] Test layout/variant extraction from path
- [ ] Test rebuild scope detection
- [ ] Test debouncing logic
- [ ] Test change detection rules

### Integration Tests
- [ ] Test watcher with real file changes
- [ ] Test incremental CSS rebuild
- [ ] Test incremental JS rebuild
- [ ] Test template hot-reload
- [ ] Test config reload
- [ ] Test debouncing prevents rapid rebuilds

### Performance Tests
- [ ] Change detection: < 10ms
- [ ] Incremental rebuild: < 2s
- [ ] Template reload: < 100ms
- [ ] Config reload: < 100ms

## Acceptance Criteria
- [ ] File watcher monitors all target directories
- [ ] Change detection working correctly
- [ ] Incremental rebuilds for layout-specific changes
- [ ] Full rebuilds for core changes
- [ ] Template hot-reload functional
- [ ] Config reload functional
- [ ] Debouncing prevents rapid rebuilds
- [ ] Real-time feedback in terminal
- [ ] CLI command `reed build:watch` working
- [ ] All tests pass with 100% coverage
- [ ] Performance benchmarks met
- [ ] Documentation complete
- [ ] BBC English throughout

## Dependencies
- **Requires**: REED-09-02 (Asset Pipeline)

## Blocks
- None (final Build Layer ticket)

## References
- Service Template: `_workbench/Tickets/templates/service-template.md`
- Service Test Template: `_workbench/Tickets/templates/service-template.test.md`
- Summary: Lines 1041-1043 in `project_summary.md`

## Notes
File watcher dramatically improves development workflow by automating rebuilds. Incremental rebuilds save time by only rebuilding changed assets. Debouncing prevents wasted rebuilds from rapid file changes. Change detection intelligently determines rebuild scope. Template hot-reload eliminates server restarts during development. Real-time feedback keeps developer informed of build status.
