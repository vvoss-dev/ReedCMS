# REED-06-02: URL Routing System

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
- **ID**: REED-06-02
- **Title**: URL → Layout + Language Resolution
- **Layer**: Server Layer (REED-06)
- **Priority**: High
- **Status**: Open
- **Complexity**: High
- **Dependencies**: REED-06-01, REED-02-01

## Summary Reference
- **Section**: URL Routing System
- **Lines**: 987-990 in project_summary.md
- **Key Concepts**: URL to layout resolution, language detection, pattern matching

## Objective
Implement URL resolution system that maps incoming URLs to layout + language combinations via .reed/routes.csv with pattern matching and parameter extraction.

## Requirements

### Routes Configuration (.reed/routes.csv)
```csv
route;layout;language;description
wissen;knowledge;de;German route for knowledge layout
knowledge;knowledge;en;English route for knowledge layout
blog;blog;de;German blog
blog;blog;en;English blog
```

### Implementation (`src/reedcms/routing/resolver.rs`)

```rust
/// Resolves URL to layout and language.
///
/// ## Arguments
/// - url: Request URL path (e.g., "/wissen", "/blog")
///
/// ## Process
/// 1. Strip leading slash
/// 2. Lookup route in routes.csv
/// 3. Return layout + language
/// 4. Fall back to 404 if not found
///
/// ## Performance
/// - O(1) HashMap lookup
/// - < 5ms per resolution
///
/// ## Output
/// - RouteInfo with layout, language, params
pub fn resolve_url(url: &str) -> ReedResult<RouteInfo> {
    // Strip leading slash
    let path = url.trim_start_matches('/');

    // Empty path defaults to home
    if path.is_empty() {
        return Ok(RouteInfo {
            layout: "home".to_string(),
            language: "en".to_string(),
            params: HashMap::new(),
        });
    }

    // Try exact match first
    if let Some(route_info) = lookup_exact_route(path)? {
        return Ok(route_info);
    }

    // Try pattern matching
    if let Some(route_info) = lookup_pattern_route(path)? {
        return Ok(route_info);
    }

    // 404 - Not found
    Err(ReedError::NotFound {
        resource: url.to_string(),
        context: Some("Route not found".to_string()),
    })
}

/// Looks up exact route match.
fn lookup_exact_route(path: &str) -> ReedResult<Option<RouteInfo>> {
    let req = ReedRequest {
        key: path.to_string(),
        language: None,
        environment: None,
        context: None,
    };

    match reedbase::get::route(&req) {
        Ok(response) => {
            // Parse response: "layout:language"
            let parts: Vec<&str> = response.data.split(':').collect();
            if parts.len() == 2 {
                Ok(Some(RouteInfo {
                    layout: parts[0].to_string(),
                    language: parts[1].to_string(),
                    params: HashMap::new(),
                }))
            } else {
                Ok(None)
            }
        }
        Err(_) => Ok(None)
    }
}

/// Looks up pattern-based route match.
///
/// ## Pattern Examples
/// - /blog/* → matches /blog/my-post
/// - /docs/:category/:page → extracts category and page params
fn lookup_pattern_route(path: &str) -> ReedResult<Option<RouteInfo>> {
    let routes = load_route_patterns()?;

    for (pattern, layout, language) in routes {
        if let Some(params) = match_pattern(&pattern, path) {
            return Ok(Some(RouteInfo {
                layout,
                language,
                params,
            }));
        }
    }

    Ok(None)
}

/// Matches URL against pattern and extracts parameters.
///
/// ## Examples
/// - Pattern: "/blog/:slug"
/// - URL: "/blog/my-post"
/// - Result: { "slug": "my-post" }
fn match_pattern(pattern: &str, path: &str) -> Option<HashMap<String, String>> {
    let pattern_parts: Vec<&str> = pattern.split('/').collect();
    let path_parts: Vec<&str> = path.split('/').collect();

    if pattern_parts.len() != path_parts.len() {
        return None;
    }

    let mut params = HashMap::new();

    for (pattern_part, path_part) in pattern_parts.iter().zip(path_parts.iter()) {
        if pattern_part.starts_with(':') {
            // Parameter
            let param_name = pattern_part.trim_start_matches(':');
            params.insert(param_name.to_string(), path_part.to_string());
        } else if *pattern_part == "*" {
            // Wildcard - matches anything
            continue;
        } else if pattern_part != path_part {
            // Literal doesn't match
            return None;
        }
    }

    Some(params)
}

/// Route information structure
#[derive(Debug, Clone)]
pub struct RouteInfo {
    pub layout: String,
    pub language: String,
    pub params: HashMap<String, String>,
}
```

### Language Detection (`src/reedcms/routing/language.rs`)

```rust
/// Detects language from request.
///
/// ## Detection Order
/// 1. URL path (e.g., /en/page)
/// 2. Accept-Language header
/// 3. Default language from config
///
/// ## Examples
/// - "/en/knowledge" → "en"
/// - "/de/wissen" → "de"
/// - Accept-Language: de-DE,de;q=0.9,en;q=0.8 → "de"
pub fn detect_language(req: &HttpRequest) -> String {
    // Try URL path first
    if let Some(lang) = extract_language_from_path(req.path()) {
        return lang;
    }

    // Try Accept-Language header
    if let Some(lang) = parse_accept_language_header(req) {
        return lang;
    }

    // Fall back to default
    get_default_language().unwrap_or_else(|| "en".to_string())
}

/// Extracts language from URL path.
///
/// ## Examples
/// - "/en/knowledge" → Some("en")
/// - "/de/wissen" → Some("de")
/// - "/knowledge" → None
fn extract_language_from_path(path: &str) -> Option<String> {
    let parts: Vec<&str> = path.trim_start_matches('/').split('/').collect();

    if parts.len() >= 1 && parts[0].len() == 2 {
        // Check if valid language code
        if is_valid_language_code(parts[0]) {
            return Some(parts[0].to_string());
        }
    }

    None
}

/// Parses Accept-Language header.
fn parse_accept_language_header(req: &HttpRequest) -> Option<String> {
    req.headers()
        .get("Accept-Language")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| {
            // Parse "de-DE,de;q=0.9,en;q=0.8"
            s.split(',')
                .next()
                .and_then(|first| {
                    first.split('-').next().map(|lang| lang.trim().to_string())
                })
        })
}

/// Checks if language code is valid.
fn is_valid_language_code(code: &str) -> bool {
    let valid_languages = get_supported_languages().unwrap_or_default();
    valid_languages.contains(&code.to_string())
}

/// Gets supported languages from config.
fn get_supported_languages() -> ReedResult<Vec<String>> {
    let req = ReedRequest {
        key: "project.languages".to_string(),
        language: None,
        environment: None,
        context: None,
    };

    match reedbase::get::project(&req) {
        Ok(response) => {
            Ok(response.data
                .split(',')
                .map(|s| s.trim().to_string())
                .collect())
        }
        Err(_) => Ok(vec!["en".to_string(), "de".to_string()])
    }
}

/// Gets default language from config.
fn get_default_language() -> Option<String> {
    let req = ReedRequest {
        key: "project.default_language".to_string(),
        language: None,
        environment: None,
        context: None,
    };

    reedbase::get::project(&req).ok().map(|r| r.data)
}
```

## Implementation Files

### Primary Implementation
- `src/reedcms/routing/resolver.rs` - URL resolver
- `src/reedcms/routing/language.rs` - Language detection
- `src/reedcms/routing/patterns.rs` - Pattern matching

### Test Files
- `src/reedcms/routing/resolver.test.rs`
- `src/reedcms/routing/language.test.rs`
- `src/reedcms/routing/patterns.test.rs`

## Testing Requirements

### Unit Tests
- [ ] Test URL resolution for exact matches
- [ ] Test pattern matching with parameters
- [ ] Test language detection from URL
- [ ] Test Accept-Language header parsing
- [ ] Test 404 handling

### Integration Tests
- [ ] Test complete routing workflow
- [ ] Test with actual routes.csv
- [ ] Test multilingual routing
- [ ] Test parameter extraction

### Performance Tests
- [ ] URL resolution: < 5ms
- [ ] Language detection: < 1ms
- [ ] Pattern matching: < 10ms

## Acceptance Criteria
- [ ] URL → Layout resolution working
- [ ] Language detection from URL functional
- [ ] Accept-Language header parsing working
- [ ] Pattern matching for dynamic routes
- [ ] Parameter extraction implemented
- [ ] .reed/routes.csv integration complete
- [ ] 404 handling correct
- [ ] All tests pass with 100% coverage
- [ ] Documentation complete
- [ ] BBC English throughout

## Dependencies
- **Requires**: REED-06-01 (Server Foundation), REED-02-01 (ReedBase)

## Blocks
- REED-06-04 (Response Builder needs routing)

## References
- Service Template: `_workbench/Tickets/templates/service-template.md`
- Summary: Lines 987-990 in `project_summary.md`

## Notes
URL routing is the entry point for all requests. Language detection enables automatic multilingual routing. Pattern matching allows dynamic routes with parameters. The system prioritizes exact matches over pattern matches for performance.