// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! File watcher for automatic asset rebuilding.
//!
//! Monitors source files and triggers incremental rebuilds on changes.

use crate::reedcms::assets::css::bundler::{bundle_all_css, bundle_css};
use crate::reedcms::assets::js::bundler::{bundle_all_js, bundle_js};
use crate::reedcms::build::change_detect::{detect_rebuild_scope, RebuildScope};
use crate::reedcms::reedstream::{ReedError, ReedResult};
use notify::{DebouncedEvent, RecursiveMode, Watcher};
use std::sync::mpsc::channel;
use std::time::Duration;

/// Starts file watcher for development mode.
///
/// ## Watch Targets
/// - `assets/css/` - CSS files
/// - `assets/js/` - JavaScript files
/// - `templates/` - Template files (hot-reload)
/// - `.reed/` - Config files
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
/// ```text
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
/// ```
///
/// ## Error Conditions
/// - `ReedError::WatcherError`: Watcher initialization failed
/// - `ReedError::BuildError`: Rebuild failed
///
/// ## Example Usage
/// ```rust
/// use reedcms::build::watcher::start_watcher;
///
/// start_watcher()?;
/// ```
pub fn start_watcher() -> ReedResult<()> {
    println!("👀 Watching for file changes...");
    println!("  CSS: assets/css/");
    println!("  JS: assets/js/");
    println!("  Templates: templates/");
    println!("  Config: .reed/");
    println!("\nPress Ctrl+C to stop\n");

    // Create watcher with 300ms debounce
    let (tx, rx) = channel();
    let mut watcher =
        notify::watcher(tx, Duration::from_millis(300)).map_err(|e| ReedError::WatcherError {
            reason: format!("Failed to create watcher: {}", e),
        })?;

    // Watch directories
    watch_directory(&mut watcher, "assets/css")?;
    watch_directory(&mut watcher, "assets/js")?;
    watch_directory(&mut watcher, "templates")?;
    watch_directory(&mut watcher, ".reed")?;

    // Process events
    loop {
        match rx.recv() {
            Ok(event) => {
                if let Err(e) = handle_file_event(event) {
                    eprintln!("⚠ Event handling error: {:?}\n", e);
                }
            }
            Err(e) => {
                eprintln!("⚠ Watcher error: {:?}", e);
                break;
            }
        }
    }

    Ok(())
}

/// Watches a directory recursively.
///
/// ## Input
/// - `watcher`: Notify watcher
/// - `path`: Directory path
///
/// ## Output
/// - `ReedResult<()>`: Success indicator
///
/// ## Error Conditions
/// - `ReedError::WatcherError`: Directory watch failed
fn watch_directory<W: Watcher>(watcher: &mut W, path: &str) -> ReedResult<()> {
    let recursive = if path == ".reed" {
        RecursiveMode::NonRecursive
    } else {
        RecursiveMode::Recursive
    };

    watcher
        .watch(path, recursive)
        .map_err(|e| ReedError::WatcherError {
            reason: format!("Failed to watch {}: {}", path, e),
        })
}

/// Handles file system event.
///
/// ## Input
/// - `event`: Debounced file system event
///
/// ## Output
/// - `ReedResult<()>`: Success indicator
///
/// ## Performance
/// - < 10ms for event processing
fn handle_file_event(event: DebouncedEvent) -> ReedResult<()> {
    use DebouncedEvent::*;

    match event {
        Write(path) | Create(path) => {
            let timestamp = chrono::Local::now().format("[%H:%M:%S]");
            let path_str = path.to_string_lossy().to_string();

            println!("{} Changed: {}", timestamp, path_str);

            handle_file_change(&path_str)?;
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
///
/// ## Input
/// - `path`: Changed file path
///
/// ## Output
/// - `ReedResult<()>`: Success indicator
///
/// ## Performance
/// - Incremental rebuild: < 2s
/// - Full rebuild: < 10s
fn handle_file_change(path: &str) -> ReedResult<()> {
    let scope = detect_rebuild_scope(path);

    match scope {
        RebuildScope::AllCss => rebuild_all_css()?,
        RebuildScope::SpecificCss { layout, variant } => rebuild_specific_css(&layout, &variant)?,
        RebuildScope::AllJs => rebuild_all_js()?,
        RebuildScope::SpecificJs { layout, variant } => rebuild_specific_js(&layout, &variant)?,
        RebuildScope::Template { path: _ } => reload_template()?,
        RebuildScope::Config { path: _ } => reload_config()?,
        RebuildScope::None => {}
    }

    Ok(())
}

/// Rebuilds all CSS bundles.
///
/// ## Output
/// - `ReedResult<()>`: Success indicator
///
/// ## Performance
/// - < 10s for 10 layouts
fn rebuild_all_css() -> ReedResult<()> {
    println!("🔨 Rebuilding all CSS bundles...");
    let start = std::time::Instant::now();

    match bundle_all_css() {
        Ok(results) => {
            println!(
                "✓ Rebuilt {} bundles in {:.1}s\n",
                results.len(),
                start.elapsed().as_secs_f32()
            );
            Ok(())
        }
        Err(e) => {
            eprintln!("⚠ Rebuild failed: {:?}\n", e);
            Err(e)
        }
    }
}

/// Rebuilds specific CSS bundle.
///
/// ## Input
/// - `layout`: Layout name
/// - `variant`: Variant name
///
/// ## Output
/// - `ReedResult<()>`: Success indicator
///
/// ## Performance
/// - < 2s for single layout
fn rebuild_specific_css(layout: &str, variant: &str) -> ReedResult<()> {
    println!("🔨 Rebuilding {}.{}.css...", layout, variant);
    let start = std::time::Instant::now();

    match bundle_css(layout, variant) {
        Ok(_) => {
            println!("✓ Rebuilt in {:.1}s\n", start.elapsed().as_secs_f32());
            Ok(())
        }
        Err(e) => {
            eprintln!("⚠ Rebuild failed: {:?}\n", e);
            Err(e)
        }
    }
}

/// Rebuilds all JS bundles.
///
/// ## Output
/// - `ReedResult<()>`: Success indicator
///
/// ## Performance
/// - < 10s for 10 layouts
fn rebuild_all_js() -> ReedResult<()> {
    println!("🔨 Rebuilding all JS bundles...");
    let start = std::time::Instant::now();

    match bundle_all_js() {
        Ok(results) => {
            println!(
                "✓ Rebuilt {} bundles in {:.1}s\n",
                results.len(),
                start.elapsed().as_secs_f32()
            );
            Ok(())
        }
        Err(e) => {
            eprintln!("⚠ Rebuild failed: {:?}\n", e);
            Err(e)
        }
    }
}

/// Rebuilds specific JS bundle.
///
/// ## Input
/// - `layout`: Layout name
/// - `variant`: Variant name
///
/// ## Output
/// - `ReedResult<()>`: Success indicator
///
/// ## Performance
/// - < 2s for single layout
fn rebuild_specific_js(layout: &str, variant: &str) -> ReedResult<()> {
    println!("🔨 Rebuilding {}.{}.js...", layout, variant);
    let start = std::time::Instant::now();

    match bundle_js(layout, variant) {
        Ok(_) => {
            println!("✓ Rebuilt in {:.1}s\n", start.elapsed().as_secs_f32());
            Ok(())
        }
        Err(e) => {
            eprintln!("⚠ Rebuild failed: {:?}\n", e);
            Err(e)
        }
    }
}

/// Reloads template (hot-reload).
///
/// ## Output
/// - `ReedResult<()>`: Success indicator
///
/// ## Performance
/// - < 100ms
///
/// ## Note
/// Template hot-reload depends on REED-05-02 implementation.
/// Currently a no-op placeholder.
fn reload_template() -> ReedResult<()> {
    println!("🔄 Template changed (hot-reload in REED-05-02)");
    println!("✓ Change detected\n");
    Ok(())
}

/// Reloads configuration.
///
/// ## Output
/// - `ReedResult<()>`: Success indicator
///
/// ## Performance
/// - < 100ms
///
/// ## Note
/// Config reload depends on REED-02-01 implementation.
/// Currently a no-op placeholder.
fn reload_config() -> ReedResult<()> {
    println!("🔄 Config changed (reload in REED-02-01)");
    println!("✓ Change detected\n");
    Ok(())
}
