# REED-10-03: Debug Tools and Development Utilities

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
- **ID**: REED-10-03
- **Title**: Debug Tools and Development Utilities
- **Layer**: Monitor Layer (REED-10)
- **Priority**: Low
- **Status**: Open
- **Complexity**: Medium
- **Dependencies**: REED-10-01, REED-10-02

## Summary Reference
- **Section**: Debug Tools
- **Lines**: 1051-1053 in project_summary.md
- **Key Concepts**: Request inspector, cache viewer, route tester, template debugger

## Objective
Implement comprehensive debug tooling for development including request inspector, ReedBase cache viewer, route testing utilities, template debugging tools, and configuration inspector to streamline development and troubleshooting.

## Requirements

### Debug Tools Suite

**Request Inspector**
- View complete request details
- Header inspection
- Body inspection
- Response preview

**Cache Viewer**
- View ReedBase cache contents
- Cache statistics
- Cache invalidation
- Cache warming

**Route Tester**
- Test URL resolution
- View matched routes
- Parameter extraction
- Layout/language detection

**Template Debugger**
- Template variable inspection
- Context viewer
- Render error details
- Filter testing

**Config Inspector**
- View all configuration
- Environment overrides
- CSV file viewer
- Validation checks

### Implementation (`src/reedcms/debug/request_inspector.rs`)

```rust
/// Request inspector for debugging.
///
/// ## CLI Usage
/// ```bash
/// reed debug:request /knowledge
/// ```
///
/// ## Output
/// ```
/// 🔍 Request Inspector: /knowledge
///
/// URL Analysis:
///   Path: /knowledge
///   Route Match: knowledge (layout: knowledge, lang: en)
///   Parameters: {}
///
/// Headers:
///   User-Agent: curl/7.88.1
///   Accept: */*
///   Host: localhost:8333
///
/// Response Preview:
///   Status: 200 OK
///   Content-Type: text/html; charset=utf-8
///   Content-Length: 12345
///   Cache-Control: public, max-age=3600
///
/// Timing Breakdown:
///   Routing: 2.1ms
///   ReedBase: 8.3ms
///   Rendering: 32.4ms
///   Total: 42.8ms
/// ```
pub async fn inspect_request(url: &str) -> ReedResult<RequestInspection> {
    let mut inspection = RequestInspection::new(url);

    // 1. Analyse URL and routing
    inspection.analyse_url()?;

    // 2. Simulate request headers
    inspection.add_default_headers();

    // 3. Profile request execution
    inspection.profile_request().await?;

    // 4. Capture response details
    inspection.capture_response();

    Ok(inspection)
}

/// Request inspection data structure.
#[derive(Debug, Clone)]
pub struct RequestInspection {
    pub url: String,
    pub route_info: Option<RouteInfo>,
    pub headers: HashMap<String, String>,
    pub status: u16,
    pub response_headers: HashMap<String, String>,
    pub timing: HashMap<String, std::time::Duration>,
}

impl RequestInspection {
    pub fn new(url: &str) -> Self {
        Self {
            url: url.to_string(),
            route_info: None,
            headers: HashMap::new(),
            status: 0,
            response_headers: HashMap::new(),
            timing: HashMap::new(),
        }
    }

    /// Analyses URL and resolves route.
    fn analyse_url(&mut self) -> ReedResult<()> {
        let route_info = reedcms::routing::resolver::resolve_url(&self.url)?;
        self.route_info = Some(route_info);
        Ok(())
    }

    /// Adds default request headers.
    fn add_default_headers(&mut self) {
        self.headers.insert("User-Agent".to_string(), "ReedCMS-Debug".to_string());
        self.headers.insert("Accept".to_string(), "*/*".to_string());
        self.headers.insert("Host".to_string(), "localhost:8333".to_string());
    }

    /// Profiles request execution.
    async fn profile_request(&mut self) -> ReedResult<()> {
        let profiler = Profiler::start("request");

        // Simulate routing
        let _routing = profiler.span("routing");
        std::thread::sleep(std::time::Duration::from_millis(2));
        drop(_routing);

        // Simulate ReedBase
        let _reedbase = profiler.span("reedbase");
        std::thread::sleep(std::time::Duration::from_millis(8));
        drop(_reedbase);

        // Simulate rendering
        let _render = profiler.span("rendering");
        std::thread::sleep(std::time::Duration::from_millis(32));
        drop(_render);

        let report = profiler.finish();

        for span in report.spans {
            self.timing.insert(span.name, span.duration);
        }
        self.timing.insert("total".to_string(), report.total_duration);

        Ok(())
    }

    /// Captures response details.
    fn capture_response(&mut self) {
        self.status = 200;
        self.response_headers.insert("Content-Type".to_string(), "text/html; charset=utf-8".to_string());
        self.response_headers.insert("Cache-Control".to_string(), "public, max-age=3600".to_string());
    }

    /// Formats inspection report.
    pub fn format(&self) -> String {
        let mut output = format!("🔍 Request Inspector: {}\n\n", self.url);

        // URL Analysis
        output.push_str("URL Analysis:\n");
        output.push_str(&format!("  Path: {}\n", self.url));

        if let Some(ref route) = self.route_info {
            output.push_str(&format!(
                "  Route Match: {} (layout: {}, lang: {})\n",
                self.url, route.layout, route.language
            ));
            output.push_str(&format!("  Parameters: {:?}\n", route.params));
        }
        output.push('\n');

        // Headers
        output.push_str("Headers:\n");
        for (key, value) in &self.headers {
            output.push_str(&format!("  {}: {}\n", key, value));
        }
        output.push('\n');

        // Response
        output.push_str("Response Preview:\n");
        output.push_str(&format!("  Status: {} OK\n", self.status));
        for (key, value) in &self.response_headers {
            output.push_str(&format!("  {}: {}\n", key, value));
        }
        output.push('\n');

        // Timing
        output.push_str("Timing Breakdown:\n");
        for (key, duration) in &self.timing {
            if key != "total" {
                output.push_str(&format!("  {}: {:.1}ms\n", key, duration.as_secs_f64() * 1000.0));
            }
        }
        if let Some(total) = self.timing.get("total") {
            output.push_str(&format!("  Total: {:.1}ms\n", total.as_secs_f64() * 1000.0));
        }

        output
    }
}
```

### Cache Viewer (`src/reedcms/debug/cache_viewer.rs`)

```rust
/// ReedBase cache viewer for debugging.
///
/// ## CLI Usage
/// ```bash
/// reed debug:cache
/// reed debug:cache text
/// reed debug:cache --search "knowledge"
/// ```
///
/// ## Output
/// ```
/// 📦 ReedBase Cache Contents
///
/// Text Cache (124 entries):
///   knowledge.title (en): "Knowledge Base"
///   knowledge.title (de): "Wissensdatenbank"
///   blog.title (en): "Blog"
///   ...
///
/// Route Cache (42 entries):
///   knowledge (en): /knowledge
///   knowledge (de): /wissen
///   ...
///
/// Cache Statistics:
///   Total entries: 312
///   Memory usage: ~1.2 MB
///   Hit rate: 94.3%
///   Misses: 18
/// ```
pub async fn view_cache(cache_type: Option<&str>, search: Option<&str>) -> ReedResult<CacheView> {
    let mut view = CacheView::new();

    // Load cache contents
    view.load_text_cache()?;
    view.load_route_cache()?;
    view.load_meta_cache()?;
    view.load_config_cache()?;

    // Apply filters
    if let Some(ct) = cache_type {
        view.filter_by_type(ct);
    }

    if let Some(s) = search {
        view.filter_by_search(s);
    }

    // Load statistics
    view.load_statistics();

    Ok(view)
}

/// Cache view structure.
#[derive(Debug, Clone)]
pub struct CacheView {
    pub text_entries: Vec<CacheEntry>,
    pub route_entries: Vec<CacheEntry>,
    pub meta_entries: Vec<CacheEntry>,
    pub config_entries: Vec<CacheEntry>,
    pub statistics: CacheStatistics,
}

impl CacheView {
    pub fn new() -> Self {
        Self {
            text_entries: Vec::new(),
            route_entries: Vec::new(),
            meta_entries: Vec::new(),
            config_entries: Vec::new(),
            statistics: CacheStatistics::default(),
        }
    }

    fn load_text_cache(&mut self) -> ReedResult<()> {
        // Access ReedBase text cache
        // Implementation depends on REED-02-01
        Ok(())
    }

    fn load_route_cache(&mut self) -> ReedResult<()> {
        Ok(())
    }

    fn load_meta_cache(&mut self) -> ReedResult<()> {
        Ok(())
    }

    fn load_config_cache(&mut self) -> ReedResult<()> {
        Ok(())
    }

    fn filter_by_type(&mut self, cache_type: &str) {
        match cache_type {
            "text" => {
                self.route_entries.clear();
                self.meta_entries.clear();
                self.config_entries.clear();
            }
            "route" => {
                self.text_entries.clear();
                self.meta_entries.clear();
                self.config_entries.clear();
            }
            _ => {}
        }
    }

    fn filter_by_search(&mut self, search: &str) {
        let filter = |entries: &mut Vec<CacheEntry>| {
            entries.retain(|e| e.key.contains(search) || e.value.contains(search));
        };

        filter(&mut self.text_entries);
        filter(&mut self.route_entries);
        filter(&mut self.meta_entries);
        filter(&mut self.config_entries);
    }

    fn load_statistics(&mut self) {
        // Get cache statistics from ReedMonitor
        let snapshot = global_monitor().get_snapshot();

        self.statistics = CacheStatistics {
            total_entries: self.total_entries(),
            memory_usage_mb: 1.2, // Estimated
            hit_rate: snapshot.reedbase_hit_rate,
            total_hits: 0,       // Would come from monitor
            total_misses: 0,     // Would come from monitor
        };
    }

    fn total_entries(&self) -> usize {
        self.text_entries.len()
            + self.route_entries.len()
            + self.meta_entries.len()
            + self.config_entries.len()
    }

    pub fn format(&self) -> String {
        let mut output = String::from("📦 ReedBase Cache Contents\n\n");

        if !self.text_entries.is_empty() {
            output.push_str(&format!("Text Cache ({} entries):\n", self.text_entries.len()));
            for entry in self.text_entries.iter().take(10) {
                output.push_str(&format!("  {} ({}): \"{}\"\n", entry.key, entry.language, entry.value));
            }
            if self.text_entries.len() > 10 {
                output.push_str(&format!("  ... and {} more\n", self.text_entries.len() - 10));
            }
            output.push('\n');
        }

        output.push_str("Cache Statistics:\n");
        output.push_str(&format!("  Total entries: {}\n", self.statistics.total_entries));
        output.push_str(&format!("  Memory usage: ~{:.1} MB\n", self.statistics.memory_usage_mb));
        output.push_str(&format!("  Hit rate: {:.1}%\n", self.statistics.hit_rate * 100.0));

        output
    }
}

#[derive(Debug, Clone)]
pub struct CacheEntry {
    pub key: String,
    pub value: String,
    pub language: String,
}

#[derive(Debug, Clone, Default)]
pub struct CacheStatistics {
    pub total_entries: usize,
    pub memory_usage_mb: f64,
    pub hit_rate: f64,
    pub total_hits: u64,
    pub total_misses: u64,
}
```

### Route Tester (`src/reedcms/debug/route_tester.rs`)

```rust
/// Route testing utility.
///
/// ## CLI Usage
/// ```bash
/// reed debug:route /knowledge
/// reed debug:route /de/wissen
/// reed debug:route /blog/my-post
/// ```
///
/// ## Output
/// ```
/// 🛣️  Route Test: /knowledge
///
/// Resolution:
///   ✓ Match found
///   Layout: knowledge
///   Language: en
///   Variant: mouse (from User-Agent)
///   Parameters: {}
///
/// Template:
///   Path: templates/layouts/knowledge/knowledge.mouse.jinja
///   ✓ Template exists
///
/// Assets:
///   CSS: public/css/knowledge.mouse.a7f3k9s2.css ✓
///   JS: public/js/knowledge.mouse.b4k7p2m9.js ✓
/// ```
pub async fn test_route(url: &str) -> ReedResult<RouteTest> {
    let mut test = RouteTest::new(url);

    // Test resolution
    test.test_resolution()?;

    // Check template existence
    test.check_template()?;

    // Check assets
    test.check_assets()?;

    Ok(test)
}

#[derive(Debug, Clone)]
pub struct RouteTest {
    pub url: String,
    pub route_info: Option<RouteInfo>,
    pub template_exists: bool,
    pub template_path: String,
    pub css_exists: bool,
    pub js_exists: bool,
    pub errors: Vec<String>,
}

impl RouteTest {
    pub fn new(url: &str) -> Self {
        Self {
            url: url.to_string(),
            route_info: None,
            template_exists: false,
            template_path: String::new(),
            css_exists: false,
            js_exists: false,
            errors: Vec::new(),
        }
    }

    fn test_resolution(&mut self) -> ReedResult<()> {
        match reedcms::routing::resolver::resolve_url(&self.url) {
            Ok(info) => {
                self.route_info = Some(info);
                Ok(())
            }
            Err(e) => {
                self.errors.push(format!("Route resolution failed: {:?}", e));
                Err(e)
            }
        }
    }

    fn check_template(&mut self) -> ReedResult<()> {
        if let Some(ref route) = self.route_info {
            self.template_path = format!(
                "templates/layouts/{}/{}.mouse.jinja",
                route.layout, route.layout
            );

            self.template_exists = std::path::Path::new(&self.template_path).exists();

            if !self.template_exists {
                self.errors.push(format!("Template not found: {}", self.template_path));
            }
        }
        Ok(())
    }

    fn check_assets(&mut self) -> ReedResult<()> {
        if let Some(ref route) = self.route_info {
            // Check CSS (with cache busting)
            let css_pattern = format!("public/css/{}.mouse.*.css", route.layout);
            self.css_exists = glob::glob(&css_pattern)
                .ok()
                .and_then(|mut g| g.next())
                .is_some();

            // Check JS (with cache busting)
            let js_pattern = format!("public/js/{}.mouse.*.js", route.layout);
            self.js_exists = glob::glob(&js_pattern)
                .ok()
                .and_then(|mut g| g.next())
                .is_some();
        }
        Ok(())
    }

    pub fn format(&self) -> String {
        let mut output = format!("🛣️  Route Test: {}\n\n", self.url);

        output.push_str("Resolution:\n");
        if let Some(ref route) = self.route_info {
            output.push_str("  ✓ Match found\n");
            output.push_str(&format!("  Layout: {}\n", route.layout));
            output.push_str(&format!("  Language: {}\n", route.language));
            output.push_str(&format!("  Parameters: {:?}\n", route.params));
        } else {
            output.push_str("  ✗ No match found\n");
        }
        output.push('\n');

        output.push_str("Template:\n");
        output.push_str(&format!("  Path: {}\n", self.template_path));
        if self.template_exists {
            output.push_str("  ✓ Template exists\n");
        } else {
            output.push_str("  ✗ Template missing\n");
        }
        output.push('\n');

        output.push_str("Assets:\n");
        output.push_str(&format!("  CSS: {}\n", if self.css_exists { "✓" } else { "✗" }));
        output.push_str(&format!("  JS: {}\n", if self.js_exists { "✓" } else { "✗" }));

        if !self.errors.is_empty() {
            output.push_str("\n⚠ Errors:\n");
            for error in &self.errors {
                output.push_str(&format!("  - {}\n", error));
            }
        }

        output
    }
}
```

### CLI Commands (`src/reedcms/cli/commands/debug.rs`)

```rust
/// CLI command: reed debug:request {url}
pub async fn execute_debug_request(url: &str) -> ReedResult<()> {
    let inspection = inspect_request(url).await?;
    println!("{}", inspection.format());
    Ok(())
}

/// CLI command: reed debug:cache [type] [--search term]
pub async fn execute_debug_cache(
    cache_type: Option<&str>,
    search: Option<&str>,
) -> ReedResult<()> {
    let view = view_cache(cache_type, search).await?;
    println!("{}", view.format());
    Ok(())
}

/// CLI command: reed debug:route {url}
pub async fn execute_debug_route(url: &str) -> ReedResult<()> {
    let test = test_route(url).await?;
    println!("{}", test.format());
    Ok(())
}

/// CLI command: reed debug:config
pub async fn execute_debug_config() -> ReedResult<()> {
    println!("⚙️  Configuration Inspector\n");

    println!("Project Configuration:");
    println!("  Name: {}", get_config_value("project.name")?);
    println!("  Languages: {}", get_config_value("project.languages")?);
    println!();

    println!("Server Configuration:");
    println!("  Port: {}", get_config_value("server.port").unwrap_or("8333".to_string()));
    println!();

    Ok(())
}

fn get_config_value(key: &str) -> ReedResult<String> {
    // Get from ReedBase
    Ok("value".to_string())
}
```

## Implementation Files

### Primary Implementation
- `src/reedcms/debug/request_inspector.rs` - Request inspector
- `src/reedcms/debug/cache_viewer.rs` - Cache viewer
- `src/reedcms/debug/route_tester.rs` - Route tester
- `src/reedcms/cli/commands/debug.rs` - CLI commands

### Test Files
- `src/reedcms/debug/request_inspector.test.rs`
- `src/reedcms/debug/cache_viewer.test.rs`
- `src/reedcms/debug/route_tester.test.rs`

## Testing Requirements

### Unit Tests
- [ ] Test request inspection data collection
- [ ] Test cache view filtering
- [ ] Test route testing logic
- [ ] Test format output

### Integration Tests
- [ ] Test request inspector with real routes
- [ ] Test cache viewer with populated cache
- [ ] Test route tester with various URLs
- [ ] Test config inspector

### Performance Tests
- [ ] Request inspection: < 50ms
- [ ] Cache viewing: < 100ms
- [ ] Route testing: < 10ms

## Acceptance Criteria
- [ ] Request inspector functional
- [ ] Cache viewer working
- [ ] Route tester implemented
- [ ] Config inspector functional
- [ ] All CLI commands working
- [ ] Clear error messages
- [ ] Formatted output readable
- [ ] All tests pass with 100% coverage
- [ ] Documentation complete
- [ ] BBC English throughout

## Dependencies
- **Requires**: REED-10-01 (Monitor), REED-10-02 (Profiler)

## Blocks
- None

## References
- Service Template: `_workbench/Tickets/templates/service-template.md`
- Service Test Template: `_workbench/Tickets/templates/service-template.test.md`
- Summary: Lines 1051-1053 in `project_summary.md`

## Notes
Debug tools accelerate development and troubleshooting. Request inspector provides comprehensive request/response analysis. Cache viewer enables verification of ReedBase data. Route tester validates URL resolution. Config inspector displays complete configuration. All tools designed for CLI usage with clear, formatted output. Essential for development workflow and production debugging.
