# REED-05-02: Template Engine Setup

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
- **ID**: REED-05-02
- **Title**: MiniJinja Environment Configuration
- **Layer**: Template Layer (REED-05)
- **Priority**: High
- **Status**: Open
- **Complexity**: Medium
- **Dependencies**: REED-05-01

## Summary Reference
- **Section**: Template Engine Setup
- **Lines**: 992-995, 1394-1420 in project_summary.md
- **Key Concepts**: MiniJinja configuration, template loader, hot-reload for development

## Objective
Configure MiniJinja environment with template loader, filter registration, and hot-reload capability for development mode with static compilation for production.

## Requirements

### Implementation (`src/reedcms/templates/engine.rs`)

```rust
/// Initialises MiniJinja template engine.
///
/// ## Configuration
/// - Template directory: templates/layouts/
/// - Auto-escape: Enabled for HTML
/// - Strict mode: Enabled (undefined variables error)
/// - Filters: text, route, meta, config
///
/// ## Environment Detection
/// - DEV: Hot-reload enabled
/// - PROD: Static template loading
///
/// ## Performance
/// - Initialisation: < 50ms
/// - Template loading: < 10ms per template
///
/// ## Output
/// - Configured MiniJinja Environment
pub fn init_template_engine() -> ReedResult<Environment<'static>> {
    let mut env = Environment::new();

    // Set template loader
    env.set_loader(template_loader);

    // Register custom filters
    env.add_filter("text", make_text_filter());
    env.add_filter("route", make_route_filter());
    env.add_filter("meta", make_meta_filter());
    env.add_filter("config", make_config_filter());

    // Configure auto-escape for HTML
    env.set_auto_escape_callback(|name| {
        name.ends_with(".jinja") || name.ends_with(".html")
    });

    // Enable strict mode (undefined variables error)
    env.set_undefined_behavior(UndefinedBehavior::Strict);

    Ok(env)
}

/// Template loader function.
///
/// ## Process
/// 1. Resolve template path from name
/// 2. Check template existence
/// 3. Read template content
/// 4. Cache in memory (DEV: disable, PROD: enable)
///
/// ## Template Path Resolution
/// - Input: "knowledge.mouse"
/// - Path: templates/layouts/knowledge/knowledge.mouse.jinja
fn template_loader(name: &str) -> Result<Option<String>, Error> {
    let path = resolve_template_path(name)?;

    if !std::path::Path::new(&path).exists() {
        return Ok(None);
    }

    match std::fs::read_to_string(&path) {
        Ok(content) => Ok(Some(content)),
        Err(e) => Err(Error::new(
            ErrorKind::CannotDeserialize,
            format!("Failed to read template {}: {}", name, e)
        ))
    }
}

/// Resolves template name to file path.
///
/// ## Examples
/// - "knowledge.mouse" → "templates/layouts/knowledge/knowledge.mouse.jinja"
/// - "blog.touch" → "templates/layouts/blog/blog.touch.jinja"
fn resolve_template_path(name: &str) -> ReedResult<String> {
    // Split name into layout and variant
    let parts: Vec<&str> = name.split('.').collect();
    if parts.len() != 2 {
        return Err(ReedError::TemplateError {
            template: name.to_string(),
            reason: "Template name must be in format 'layout.variant'".to_string(),
        });
    }

    let layout = parts[0];
    let variant = parts[1];

    Ok(format!(
        "templates/layouts/{}/{}.{}.jinja",
        layout, layout, variant
    ))
}
```

### Template Loader (`src/reedcms/templates/loader.rs`)

```rust
/// Loads template by name.
///
/// ## Arguments
/// - name: Template name (e.g., "knowledge.mouse")
///
/// ## Process
/// 1. Check cache (PROD only)
/// 2. Load from disk
/// 3. Parse template
/// 4. Cache result (PROD only)
///
/// ## Performance
/// - Cached load: < 1ms
/// - Disk load: < 10ms
pub fn load_template(env: &Environment, name: &str) -> ReedResult<Template> {
    match env.get_template(name) {
        Ok(template) => Ok(template),
        Err(e) => Err(ReedError::TemplateError {
            template: name.to_string(),
            reason: format!("Template not found or invalid: {}", e),
        })
    }
}

/// Preloads all templates into cache.
///
/// ## Usage
/// Called at server startup in PROD mode
///
/// ## Process
/// 1. Discover all .jinja files
/// 2. Load and parse each template
/// 3. Cache in memory
///
/// ## Performance
/// - < 500ms for 50 templates
pub fn preload_templates(env: &mut Environment) -> ReedResult<usize> {
    let template_files = discover_template_files("templates/layouts/")?;
    let mut loaded_count = 0;

    for file_path in template_files {
        let template_name = extract_template_name(&file_path)?;
        if env.get_template(&template_name).is_ok() {
            loaded_count += 1;
        }
    }

    Ok(loaded_count)
}

/// Discovers all template files in directory.
fn discover_template_files(path: &str) -> ReedResult<Vec<String>>

/// Extracts template name from file path.
///
/// ## Example
/// - "templates/layouts/knowledge/knowledge.mouse.jinja" → "knowledge.mouse"
fn extract_template_name(path: &str) -> ReedResult<String>
```

### Hot-Reload System (`src/reedcms/templates/hot_reload.rs`)

```rust
/// Sets up hot-reload for development.
///
/// ## Process
/// 1. Watch templates/ directory
/// 2. Detect file changes
/// 3. Reload template
/// 4. Clear cache entry
///
/// ## Performance
/// - Change detection: < 100ms
/// - Template reload: < 10ms
///
/// ## Usage
/// Only enabled in DEV environment
pub fn setup_hot_reload(env: &mut Environment) -> ReedResult<AutoReloader> {
    if !is_dev_environment() {
        return Err(ReedError::ConfigError {
            component: "hot_reload".to_string(),
            reason: "Hot-reload only available in DEV environment".to_string(),
        });
    }

    let watcher = create_template_watcher()?;
    Ok(AutoReloader {
        environment: env,
        watcher,
    })
}

/// Creates file system watcher for templates.
fn create_template_watcher() -> ReedResult<Watcher>

/// Checks if running in DEV environment.
fn is_dev_environment() -> bool {
    std::env::var("REED_ENV")
        .unwrap_or_else(|_| "DEV".to_string())
        .to_uppercase() == "DEV"
}

/// Auto-reloader structure
pub struct AutoReloader<'a> {
    environment: &'a mut Environment<'a>,
    watcher: Watcher,
}

impl AutoReloader<'_> {
    /// Handles file change event.
    pub fn on_change(&mut self, path: &str) -> ReedResult<()> {
        let template_name = extract_template_name(path)?;

        // Clear cache for this template
        self.environment.clear_templates();

        println!("🔄 Template reloaded: {}", template_name);
        Ok(())
    }
}
```

### Server Integration and Startup Sequence

#### Global Template Engine Singleton (`src/reedcms/templates/engine.rs`)

```rust
use std::sync::OnceLock;

/// Global template engine singleton.
static TEMPLATE_ENGINE: OnceLock<Environment<'static>> = OnceLock::new();

/// Gets or initializes the global template engine.
///
/// ## Thread Safety
/// - OnceLock ensures single initialization
/// - Thread-safe access without locks after init
/// - Panic-free initialization
///
/// ## Usage
/// ```rust
/// let engine = get_template_engine()?;
/// let template = engine.get_template("layout.mouse.jinja")?;
/// ```
pub fn get_template_engine() -> ReedResult<&'static Environment<'static>> {
    TEMPLATE_ENGINE.get_or_try_init(|| {
        init_template_engine()
    })
}

/// Clears template cache (DEV mode hot-reload).
///
/// ## Note
/// This creates a new engine instance since Environment is immutable.
/// Only used in DEV mode for hot-reload functionality.
pub fn reload_template_engine() -> ReedResult<()> {
    // In production, this should be a no-op
    if !is_dev_environment() {
        return Ok(());
    }
    
    // Force re-initialization
    // Note: OnceLock doesn't support clearing, so this only works
    // if we use a different pattern for DEV mode
    Ok(())
}
```

#### Server Startup Integration (`src/reedcms/server/startup.rs`)

**This should be called from REED-06-01 server startup:**

```rust
use crate::templates::engine;

/// Initializes all required systems before starting HTTP server.
///
/// ## Initialization Order
/// 1. Template Engine (registers filters, sets up loader)
/// 2. Hot-reload watcher (DEV mode only)
/// 3. ReedBase cache (loads CSV files)
/// 4. Monitoring system (starts metrics collection)
///
/// ## Error Handling
/// - Fatal errors stop server startup
/// - Non-fatal errors logged as warnings
pub async fn initialize_systems() -> ReedResult<()> {
    println!("🚀 Initializing ReedCMS systems...");
    
    // 1. Initialize template engine
    println!("  📄 Loading template engine...");
    let _engine = engine::get_template_engine()?;
    println!("  ✓ Template engine ready");
    
    // 2. Start hot-reload watcher (DEV only)
    if is_dev_environment() {
        println!("  🔄 Starting template hot-reload...");
        engine::start_hot_reload()?;
        println!("  ✓ Hot-reload active");
    }
    
    // 3. Initialize ReedBase
    println!("  💾 Loading ReedBase cache...");
    crate::reedbase::init::initialize()?;
    println!("  ✓ ReedBase ready");
    
    // 4. Initialize monitoring
    println!("  📊 Starting monitoring system...");
    crate::monitor::core::initialize()?;
    println!("  ✓ Monitoring active");
    
    println!("✓ All systems initialized\n");
    Ok(())
}

fn is_dev_environment() -> bool {
    std::env::var("REED_ENV")
        .unwrap_or_else(|_| "DEV".to_string())
        .to_uppercase() == "DEV"
}
```

#### Integration in Server Main (`src/reedcms/server/main.rs`)

```rust
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize all systems
    if let Err(e) = startup::initialize_systems().await {
        eprintln!("❌ Startup failed: {:?}", e);
        std::process::exit(1);
    }
    
    // Start HTTP server
    HttpServer::new(|| {
        App::new()
            // Middleware
            .wrap(middleware::Logger::default())
            .wrap(monitor::middleware::MonitoringMiddleware::new())
            
            // Routes
            .configure(routes::configure)
    })
    .bind(("127.0.0.1", 3000))?
    .run()
    .await
}
```

## Implementation Files

### Primary Implementation
- `src/reedcms/templates/engine.rs` - Engine setup
- `src/reedcms/templates/loader.rs` - Template loader
- `src/reedcms/templates/hot_reload.rs` - Development hot-reload

### Test Files
- `src/reedcms/templates/engine.test.rs`
- `src/reedcms/templates/loader.test.rs`
- `src/reedcms/templates/hot_reload.test.rs`

## File Structure
```
src/reedcms/templates/
├── engine.rs           # MiniJinja setup
├── engine.test.rs      # Engine tests
├── loader.rs           # Template loader
├── loader.test.rs      # Loader tests
├── hot_reload.rs       # Hot-reload system
└── hot_reload.test.rs  # Hot-reload tests
```

## Testing Requirements

### Unit Tests
- [ ] Test engine initialisation
- [ ] Test template loader
- [ ] Test path resolution
- [ ] Test filter registration
- [ ] Test auto-escape configuration

### Integration Tests
- [ ] Test complete template loading
- [ ] Test hot-reload in DEV mode
- [ ] Test static loading in PROD mode
- [ ] Test template preloading

### Error Handling Tests
- [ ] Test missing template handling
- [ ] Test invalid template name
- [ ] Test malformed template content
- [ ] Test hot-reload errors

### Performance Tests
- [ ] Engine init: < 50ms
- [ ] Template load: < 10ms
- [ ] Hot-reload detection: < 100ms
- [ ] Preload 50 templates: < 500ms

## Acceptance Criteria
- [ ] MiniJinja environment configured
- [ ] Template loader working
- [ ] All filters registered (text, route, meta, config)
- [ ] Hot-reload for DEV environment functional
- [ ] Static template loading for PROD
- [ ] Template preloading working
- [ ] Auto-escape enabled for HTML
- [ ] Strict mode enforced
- [ ] All tests pass with 100% coverage
- [ ] Performance benchmarks met
- [ ] Documentation complete
- [ ] BBC English throughout

## Dependencies
- **Requires**: REED-05-01 (Template Filters)

## Blocks
- REED-05-03 (Context Builder needs engine)
- REED-06-01 (Server needs template engine)

## References
- Service Template: `_workbench/Tickets/templates/service-template.md`
- Service Test Template: `_workbench/Tickets/templates/service-template.test.md`
- Summary: Lines 992-995, 1394-1420 in `project_summary.md`

## Notes
The template engine is the core of ReedCMS rendering system. Hot-reload in DEV mode enables rapid development without server restarts. Static loading in PROD mode maximises performance by preloading all templates at startup. Strict mode catches template errors early by failing on undefined variables.