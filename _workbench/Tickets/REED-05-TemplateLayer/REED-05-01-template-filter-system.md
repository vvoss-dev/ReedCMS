# REED-05-01: Template Filter System

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
- **ID**: REED-05-01
- **Title**: MiniJinja Template Filter System
- **Layer**: Template Layer (REED-05)
- **Priority**: High
- **Status**: Open
- **Complexity**: High
- **Dependencies**: REED-02-01

## Summary Reference
- **Section**: Template Filter System
- **Lines**: 1256-1323 in project_summary.md
- **Key Concepts**: MiniJinja filters for ReedBase data access, type-specific filters

## Objective
Implement MiniJinja template filters for type-specific ReedBase data access including text, route, meta, and config filters with environment-aware lookups.

## Requirements

### Filter Usage in Templates

```jinja
{# Text filter - retrieves text content #}
{{ "knowledge.page.title" | text("de") }}
{{ "knowledge.page.title" | text("auto") }}  {# Auto-detect language from context #}

{# Route filter - retrieves route URLs #}
{{ "knowledge" | route("en") }}              {# Output: /knowledge #}
{{ "knowledge" | route("de") }}              {# Output: /wissen #}

{# Meta filter - retrieves meta data #}
{{ "knowledge.cache.ttl" | meta }}           {# Output: 3600 #}

{# Config filter - auto-detects project./server. prefix #}
{{ "project.languages" | config }}           {# Auto-resolves to project.languages #}
{{ "server.auth.enabled" | config }}         {# Auto-resolves to server.auth.enabled #}
```

### Implementation Files

#### Text Filter (`src/reedcms/filters/text.rs`)

```rust
/// MiniJinja filter for text content retrieval.
///
/// ## Usage
/// {{ "key" | text("lang") }}
/// {{ "key@env" | text("lang") }}
///
/// ## Arguments
/// - key: Text key (e.g., "knowledge.title")
/// - language: Language code or "auto" for context detection
///
/// ## Environment Fallback
/// - Tries key@environment first
/// - Falls back to base key
///
/// ## Performance
/// - O(1) HashMap lookup
/// - < 100μs per filter call
pub fn make_text_filter() -> impl Filter {
    |key: String, lang: Option<String>| -> Result<String, Error> {
        let language = lang.unwrap_or_else(|| "auto".to_string());

        // Auto-detect language from template context
        let resolved_lang = if language == "auto" {
            detect_language_from_context()?
        } else {
            language
        };

        // Build ReedRequest
        let req = ReedRequest {
            key: key.clone(),
            language: Some(resolved_lang.clone()),
            environment: get_current_environment(),
            context: None,
        };

        // Call ReedBase get
        match reedbase::get::text(&req) {
            Ok(response) => Ok(response.data),
            Err(e) => Err(convert_reed_error_to_jinja(e, "text", &key))
        }
    }
}

/// Converts ReedError to MiniJinja Error with context.
///
/// ## Error Mapping
/// - NotFound → TemplateNotFound
/// - DataError → InvalidOperation  
/// - ConfigError → InvalidOperation
/// - IoError → InvalidOperation
/// - SystemError → InvalidOperation
///
/// ## Context Preservation
/// - Includes original error message
/// - Adds filter context (filter name, key)
/// - Preserves error chain for debugging
fn convert_reed_error_to_jinja(err: ReedError, filter: &str, key: &str) -> minijinja::Error {
    use minijinja::ErrorKind;
    
    match err {
        ReedError::NotFound { data_type, key: orig_key, context } => {
            let msg = format!(
                "Filter '{}': {} not found for key '{}' (context: {:?})",
                filter, data_type, orig_key, context
            );
            minijinja::Error::new(ErrorKind::TemplateNotFound, msg)
        }
        ReedError::DataError { component, reason } => {
            let msg = format!(
                "Filter '{}': Data error in {} for key '{}': {}",
                filter, component, key, reason
            );
            minijinja::Error::new(ErrorKind::InvalidOperation, msg)
        }
        ReedError::ConfigError { component, reason } => {
            let msg = format!(
                "Filter '{}': Config error in {} for key '{}': {}",
                filter, component, key, reason
            );
            minijinja::Error::new(ErrorKind::InvalidOperation, msg)
        }
        ReedError::IoError { operation, path, reason } => {
            let msg = format!(
                "Filter '{}': IO error during {} on '{}': {}",
                filter, operation, path, reason
            );
            minijinja::Error::new(ErrorKind::InvalidOperation, msg)
        }
        _ => {
            let msg = format!(
                "Filter '{}': Error for key '{}': {:?}",
                filter, key, err
            );
            minijinja::Error::new(ErrorKind::InvalidOperation, msg)
        }
    }
}

/// Detects language from template context.
fn detect_language_from_context() -> ReedResult<String>

/// Gets current environment from context.
fn get_current_environment() -> Option<String>
```

### Language Detection Strategy

**Source of Truth**: URL path is the **single source of truth** for current language.

**Why URL over Cookie:**
- ✅ URL = user's **conscious choice** (via language switcher)
- ✅ Cookie = only **initial detection** (Accept-Language header)
- ✅ URL is bookmarkable, shareable, SEO-friendly
- ✅ No race conditions between cookie and URL

**Implementation:**

```rust
// Language is injected into filter at creation time from request URL
// In REED-06-02 Request Handler or REED-05-03 Context Builder:

// 1. Extract language from URL path
let current_lang = extract_lang_from_path(&req.path()); // /de/wissen → "de"

// 2. Create text filter with injected language
pub fn make_text_filter(current_lang: String) -> impl Filter + Send + Sync + 'static {
    move |key: &str, lang_param: Option<&str>| -> Result<String, minijinja::Error> {
        // Resolve 'auto' to current request language (from URL)
        let resolved_lang = match lang_param {
            Some("auto") => &current_lang,      // Use URL language
            Some(explicit) => explicit,          // Explicit override (e.g., "de" on English page)
            None => &current_lang,               // Default to URL language
        };
        
        let req = ReedRequest {
            key: key.to_string(),
            language: Some(resolved_lang.to_string()),
            environment: get_current_environment(),
            context: None,
            value: None,
            description: None,
        };
        
        match reedbase::get::text(&req) {
            Ok(response) => Ok(response.data),
            Err(err) => Err(convert_reed_error_to_jinja(err, "text", key)),
        }
    }
}
```

**Request Flow:**

1. **First Visit** (`/`):
   - Client detection (no `screen_info` cookie yet)
   - JavaScript collects screen data + sets cookie
   - Server detects language from `Accept-Language` header
   - Cookie `lang=de` set for future visits
   - Redirect to `/de/` (URL becomes source of truth)

2. **Subsequent Visits**:
   - URL language is used: `/de/wissen` → `current_lang = "de"`
   - Filter uses URL language for `text('auto')`
   - Cookie `lang` is **only checked on root path** (`/`) for redirect

3. **Language Switcher**:
   - User clicks "EN" on `/de/wissen`
   - Navigates to `/en/knowledge`
   - URL language changes, templates re-render with new language
   - Cookie remains `lang=de` (ignored for language, only for next root visit)

**Long-Term Behavior:**
- User returns after months → visits `/` → Cookie `lang=de` redirects to `/de/`
- This ensures returning users get their system language, not last visited URL

**Filter Parameters:**
- `text('auto')` - Uses URL language (most common case)
- `text('de')` - Forces German text (even on English page)
- `text('en')` - Forces English text (even on German page)
```

#### Route Filter (`src/reedcms/filters/route.rs`)

```rust
/// MiniJinja filter for route URL retrieval.
///
/// ## Usage
/// {{ "layout_key" | route("lang") }}
///
/// ## Arguments
/// - key: Layout key (e.g., "knowledge", "blog")
/// - language: Language code
///
/// ## Output
/// - Route URL (e.g., "/knowledge", "/wissen")
///
/// ## Performance
/// - O(1) HashMap lookup
/// - < 100μs per filter call
pub fn make_route_filter() -> impl Filter {
    |key: String, lang: Option<String>| -> Result<String, Error> {
        let language = lang.unwrap_or_else(|| detect_language_from_context().unwrap_or_else(|_| "en".to_string()));

        let req = ReedRequest {
            key: key.clone(),
            language: Some(language.clone()),
            environment: get_current_environment(),
            context: None,
        };

        match reedbase::get::route(&req) {
            Ok(response) => Ok(format!("/{}", response.data)),
            Err(e) => Err(convert_reed_error_to_jinja(e, "route", &key))
        }
    }
}
```

#### Meta Filter (`src/reedcms/filters/meta.rs`)

```rust
/// MiniJinja filter for meta data retrieval.
///
/// ## Usage
/// {{ "key" | meta }}
///
/// ## Arguments
/// - key: Meta key (e.g., "layout.cache.ttl")
///
/// ## Output
/// - Meta value (string)
///
/// ## Performance
/// - O(1) HashMap lookup
/// - < 100μs per filter call
pub fn make_meta_filter() -> impl Filter {
    |key: String| -> Result<String, Error> {
        let req = ReedRequest {
            key: key.clone(),
            language: None,
            environment: get_current_environment(),
            context: None,
        };

        match reedbase::get::meta(&req) {
            Ok(response) => Ok(response.data),
            Err(e) => Err(convert_reed_error_to_jinja(e, "meta", &key))
        }
    }
}
```

#### Config Filter (`src/reedcms/filters/config.rs`)

```rust
/// MiniJinja filter for configuration retrieval.
///
/// ## Usage
/// {{ "languages" | config }}           {# Auto-resolves to project.languages #}
/// {{ "auth.enabled" | config }}        {# Auto-resolves to server.auth.enabled #}
///
/// ## Auto-Detection Logic
/// 1. Try project.{key}
/// 2. Try server.{key}
/// 3. Return error if neither found
///
/// ## Arguments
/// - key: Config key (without project./server. prefix)
///
/// ## Output
/// - Configuration value (string)
///
/// ## Performance
/// - 2x O(1) HashMap lookups (worst case)
/// - < 200μs per filter call
pub fn make_config_filter() -> impl Filter {
    |key: String| -> Result<String, Error> {
        // Try project first
        let project_key = format!("project.{}", key);
        let req_project = ReedRequest {
            key: project_key.clone(),
            language: None,
            environment: get_current_environment(),
            context: None,
        };

        if let Ok(response) = reedbase::get::project(&req_project) {
            return Ok(response.data);
        }

        // Try server second
        let server_key = format!("server.{}", key);
        let req_server = ReedRequest {
            key: server_key.clone(),
            language: None,
            environment: get_current_environment(),
            context: None,
        };

        match reedbase::get::server(&req_server) {
            Ok(response) => Ok(response.data),
            Err(e) => Err(convert_reed_error_to_jinja(
                e,
                "config",
                &format!("{} (tried project.{} and server.{})", key, key, key)
            ))
        }
    }
}
```

## Implementation Files

### Primary Implementation
- `src/reedcms/filters/text.rs` - Text filter
- `src/reedcms/filters/route.rs` - Route filter
- `src/reedcms/filters/meta.rs` - Meta filter
- `src/reedcms/filters/config.rs` - Config filter

### Test Files
- `src/reedcms/filters/text.test.rs`
- `src/reedcms/filters/route.test.rs`
- `src/reedcms/filters/meta.test.rs`
- `src/reedcms/filters/config.test.rs`

## File Structure
```
src/reedcms/filters/
├── text.rs           # Text filter
├── text.test.rs      # Text tests
├── route.rs          # Route filter
├── route.test.rs     # Route tests
├── meta.rs           # Meta filter
├── meta.test.rs      # Meta tests
├── config.rs         # Config filter
└── config.test.rs    # Config tests
```

## Testing Requirements

### Unit Tests
- [ ] Test text filter with valid key
- [ ] Test text filter with environment suffix
- [ ] Test text filter with auto language detection
- [ ] Test route filter with multiple languages
- [ ] Test meta filter
- [ ] Test config filter auto-detection (project/server)

### Integration Tests
- [ ] Test filters in actual Jinja templates
- [ ] Test environment fallback in filters
- [ ] Test error messages for missing keys
- [ ] Test performance with cache

### Error Handling Tests
- [ ] Test NotFound errors
- [ ] Test invalid language codes
- [ ] Test missing environment data

### Performance Tests
- [ ] Text filter: < 100μs
- [ ] Route filter: < 100μs
- [ ] Meta filter: < 100μs
- [ ] Config filter: < 200μs

## Acceptance Criteria
- [ ] All four filters implemented (text, route, meta, config)
- [ ] Filters integrated with MiniJinja
- [ ] ReedBase data access working
- [ ] Environment fallback functional
- [ ] Auto language detection working ("auto" parameter)
- [ ] Config auto-detection (project./server.) working
- [ ] All tests pass with 100% coverage
- [ ] Performance benchmarks met
- [ ] Documentation complete
- [ ] BBC English throughout

## Dependencies
- **Requires**: REED-02-01 (ReedBase for data access)

## Blocks
- REED-05-02 (Template Engine needs these filters)

## References
- Service Template: `_workbench/Tickets/templates/service-template.md`
- Service Test Template: `_workbench/Tickets/templates/service-template.test.md`
- Summary: Lines 1256-1323 in `project_summary.md`

## Notes
Template filters are the primary interface between templates and ReedBase data. The "auto" language detection simplifies templates by inferring language from request context. Config filter auto-detection (project./server.) reduces verbosity in templates. All filters must maintain O(1) performance through HashMap lookups.