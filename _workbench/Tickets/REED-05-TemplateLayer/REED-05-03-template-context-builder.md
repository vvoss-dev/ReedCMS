# REED-05-03: Template Context Builder

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
- **ID**: REED-05-03
- **Title**: Template Context Building System
- **Layer**: Template Layer (REED-05)
- **Priority**: Medium
- **Status**: Open
- **Complexity**: Medium
- **Dependencies**: REED-05-01, REED-02-01

## Summary Reference
- **Section**: Template Context Builder
- **Lines**: 994 in project_summary.md
- **Key Concepts**: Context building with ReedBase data integration

## Objective
Implement template context building system that prepares all necessary data for template rendering including layout information, language settings, global variables, and ReedBase data access.

## Requirements

### Implementation (`src/reedcms/templates/context.rs`)

```rust
/// Builds template context for rendering.
///
/// ## Arguments
/// - layout: Layout name (e.g., "knowledge")
/// - language: Language code (e.g., "en", "de")
/// - environment: Environment name (e.g., "dev", "prod")
///
/// ## Context Variables
/// - layout: Current layout name
/// - lang: Current language code
/// - environment: Current environment
/// - variant: Template variant (mouse/touch/reader)
/// - request: Request information (URL, method, headers)
/// - globals: Global configuration values
///
/// ## Performance
/// - Context building: < 5ms
/// - Memory usage: < 1KB per context
///
/// ## Output
/// - MiniJinja Context object ready for rendering
pub fn build_context(
    layout: &str,
    language: &str,
    environment: &str,
    variant: &str
) -> ReedResult<Context> {
    let mut ctx = Context::new();

    // Core variables
    ctx.insert("layout", layout);
    ctx.insert("lang", language);
    ctx.insert("environment", environment);
    ctx.insert("variant", variant);

    // Add globals
    add_globals(&mut ctx)?;

    // Add layout-specific data
    add_layout_data(&mut ctx, layout, language)?;

    // Add request information (if available)
    if let Some(request_info) = get_request_info() {
        ctx.insert("request", request_info);
    }

    Ok(ctx)
}

/// Adds global variables to context.
///
/// ## Global Variables
/// - site_name: Project name from config
/// - site_url: Base URL from config
/// - languages: Available languages
/// - current_year: Current year for copyright
/// - version: ReedCMS version
///
/// ## Example Context
/// ```jinja
/// {{ site_name }}           {# ReedCMS Documentation #}
/// {{ languages | join(", ") }} {# en, de, fr #}
/// {{ current_year }}        {# 2025 #}
/// ```
pub fn add_globals(ctx: &mut Context) -> ReedResult<()> {
    // Site information from project config
    let site_name = get_config_value("project.name")?;
    let site_url = get_config_value("project.url")?;
    let languages = get_config_value("project.languages")?
        .split(',')
        .map(|s| s.trim().to_string())
        .collect::<Vec<String>>();

    ctx.insert("site_name", site_name);
    ctx.insert("site_url", site_url);
    ctx.insert("languages", languages);

    // System information
    ctx.insert("current_year", chrono::Utc::now().year());
    ctx.insert("version", env!("CARGO_PKG_VERSION"));

    Ok(())
}

/// Adds layout-specific data to context.
///
/// ## Layout Data
/// - layout_title: Title from text.csv
/// - layout_description: Description from text.csv
/// - layout_meta: Meta data from meta.csv
/// - navigation: Navigation items
///
/// ## Example Context
/// ```jinja
/// {{ layout_title }}        {# Knowledge Base #}
/// {{ layout_description }}  {# Comprehensive documentation #}
/// ```
pub fn add_layout_data(
    ctx: &mut Context,
    layout: &str,
    language: &str
) -> ReedResult<()> {
    // Layout title
    let title_key = format!("{}.title", layout);
    if let Ok(title) = get_text_value(&title_key, language) {
        ctx.insert("layout_title", title);
    }

    // Layout description
    let desc_key = format!("{}.description", layout);
    if let Ok(description) = get_text_value(&desc_key, language) {
        ctx.insert("layout_description", description);
    }

    // Layout meta data
    let cache_ttl_key = format!("{}.cache.ttl", layout);
    if let Ok(ttl) = get_meta_value(&cache_ttl_key) {
        ctx.insert("cache_ttl", ttl);
    }

    // Navigation items
    if let Ok(nav_items) = build_navigation(layout, language) {
        ctx.insert("navigation", nav_items);
    }

    Ok(())
}

/// Builds navigation items for layout.
///
/// ## Navigation Structure
/// ```rust
/// vec![
///     NavigationItem {
///         key: "home",
///         label: "Home",
///         url: "/",
///         active: false,
///     },
///     NavigationItem {
///         key: "knowledge",
///         label: "Knowledge",
///         url: "/knowledge",
///         active: true,
///     }
/// ]
/// ```
pub fn build_navigation(
    layout: &str,
    language: &str
) -> ReedResult<Vec<NavigationItem>> {
    let nav_keys = get_navigation_keys()?;
    let mut nav_items = Vec::new();

    for key in nav_keys {
        let label_key = format!("nav.{}", key);
        let label = get_text_value(&label_key, language)?;
        let url = get_route_value(&key, language)?;

        nav_items.push(NavigationItem {
            key: key.clone(),
            label,
            url: format!("/{}", url),
            active: key == layout,
        });
    }

    Ok(nav_items)
}

/// Navigation item structure
#[derive(Debug, Clone, Serialize)]
pub struct NavigationItem {
    pub key: String,
    pub label: String,
    pub url: String,
    pub active: bool,
}

/// Gets text value from ReedBase.
fn get_text_value(key: &str, language: &str) -> ReedResult<String> {
    let req = ReedRequest {
        key: key.to_string(),
        language: Some(language.to_string()),
        environment: None,
        context: None,
    };

    match reedbase::get::text(&req) {
        Ok(response) => Ok(response.data),
        Err(e) => Err(e)
    }
}

/// Gets route value from ReedBase.
fn get_route_value(key: &str, language: &str) -> ReedResult<String> {
    let req = ReedRequest {
        key: key.to_string(),
        language: Some(language.to_string()),
        environment: None,
        context: None,
    };

    match reedbase::get::route(&req) {
        Ok(response) => Ok(response.data),
        Err(e) => Err(e)
    }
}

/// Gets meta value from ReedBase.
fn get_meta_value(key: &str) -> ReedResult<String> {
    let req = ReedRequest {
        key: key.to_string(),
        language: None,
        environment: None,
        context: None,
    };

    match reedbase::get::meta(&req) {
        Ok(response) => Ok(response.data),
        Err(e) => Err(e)
    }
}

/// Gets config value from ReedBase.
fn get_config_value(key: &str) -> ReedResult<String> {
    // Try project first, then server
    if key.starts_with("project.") {
        let req = ReedRequest {
            key: key.to_string(),
            language: None,
            environment: None,
            context: None,
        };
        match reedbase::get::project(&req) {
            Ok(response) => return Ok(response.data),
            Err(_) => {}
        }
    }

    if key.starts_with("server.") {
        let req = ReedRequest {
            key: key.to_string(),
            language: None,
            environment: None,
            context: None,
        };
        match reedbase::get::server(&req) {
            Ok(response) => return Ok(response.data),
            Err(_) => {}
        }
    }

    Err(ReedError::NotFound {
        resource: key.to_string(),
        context: Some("Config value not found".to_string()),
    })
}

/// Gets navigation keys from registry.
fn get_navigation_keys() -> ReedResult<Vec<String>> {
    // TODO: Read from registry.csv
    Ok(vec![
        "home".to_string(),
        "knowledge".to_string(),
        "blog".to_string(),
        "about".to_string(),
    ])
}

/// Gets request information from thread-local storage.
fn get_request_info() -> Option<RequestInfo> {
    // TODO: Implement thread-local request storage
    None
}

/// Request information structure
#[derive(Debug, Clone, Serialize)]
pub struct RequestInfo {
    pub url: String,
    pub method: String,
    pub headers: HashMap<String, String>,
}
```

## Implementation Files

### Primary Implementation
- `src/reedcms/templates/context.rs` - Context building

### Test Files
- `src/reedcms/templates/context.test.rs` - Context tests

## Testing Requirements

### Unit Tests
- [ ] Test context building with valid data
- [ ] Test globals addition
- [ ] Test layout data addition
- [ ] Test navigation building
- [ ] Test missing data handling

### Integration Tests
- [ ] Test complete context with ReedBase
- [ ] Test context in actual template rendering
- [ ] Test multiple languages
- [ ] Test different layouts

### Performance Tests
- [ ] Context building: < 5ms
- [ ] Memory usage: < 1KB per context
- [ ] Navigation building: < 10ms

## Acceptance Criteria
- [ ] Context building functional
- [ ] Global variables populated
- [ ] Layout-specific data integrated
- [ ] Navigation items generated
- [ ] ReedBase integration working
- [ ] All tests pass with 100% coverage
- [ ] Performance benchmarks met
- [ ] Documentation complete
- [ ] BBC English throughout

## Dependencies
- **Requires**: REED-05-01 (Filters for data access), REED-02-01 (ReedBase)

## Blocks
- REED-06-04 (Response Builder needs context)

## References
- Service Template: `_workbench/Tickets/templates/service-template.md`
- Service Test Template: `_workbench/Tickets/templates/service-template.test.md`
- Summary: Line 994 in `project_summary.md`

## Notes
Template context is the bridge between ReedBase data and template rendering. Global variables reduce repetition in templates. Layout-specific data enables context-aware rendering. Navigation building provides automatic menu generation based on available layouts and routes.