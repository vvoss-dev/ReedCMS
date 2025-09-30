# REED-10-02: Performance Profiler and Analysis Tools

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
- **ID**: REED-10-02
- **Title**: Performance Profiler and Analysis Tools
- **Layer**: Monitor Layer (REED-10)
- **Priority**: Medium
- **Status**: Open
- **Complexity**: High
- **Dependencies**: REED-10-01

## Summary Reference
- **Section**: Performance Profiler
- **Lines**: 1048-1050 in project_summary.md
- **Key Concepts**: Request profiling, timing breakdown, bottleneck detection, flame graphs

## Objective
Implement performance profiler that provides detailed request timing breakdowns, identifies bottlenecks, generates flame graphs for visualisation, tracks slow queries, and enables performance optimisation through granular timing data.

## Requirements

### Profiling Levels

**Level 1: Request Overview**
- Total request time
- Status code
- Response size

**Level 2: Component Breakdown**
- URL routing time
- Template rendering time
- ReedBase lookups time
- Response building time

**Level 3: Detailed Trace**
- Individual function timings
- Database access times
- File I/O operations
- Network calls

### Implementation (`src/reedcms/profiler/core.rs`)

```rust
/// Performance profiler for detailed request analysis.
///
/// ## Usage
/// ```rust
/// let profiler = Profiler::start("handle_request");
///
/// let _route = profiler.span("routing");
/// // ... routing logic ...
/// drop(_route);
///
/// let _render = profiler.span("rendering");
/// // ... rendering logic ...
/// drop(_render);
///
/// let report = profiler.finish();
/// println!("{}", report);
/// ```
///
/// ## Features
/// - Nested span tracking
/// - Automatic timing
/// - Zero-cost when disabled
/// - Thread-safe
///
/// ## Performance
/// - Span start/end: < 1μs
/// - Report generation: < 100μs
pub struct Profiler {
    name: String,
    start_time: std::time::Instant,
    spans: Arc<Mutex<Vec<Span>>>,
    current_depth: Arc<AtomicUsize>,
}

impl Profiler {
    /// Starts new profiler for operation.
    pub fn start(name: &str) -> Self {
        Self {
            name: name.to_string(),
            start_time: std::time::Instant::now(),
            spans: Arc::new(Mutex::new(Vec::new())),
            current_depth: Arc::new(AtomicUsize::new(0)),
        }
    }

    /// Creates timed span.
    ///
    /// ## Usage
    /// Span is automatically closed when dropped.
    pub fn span(&self, name: &str) -> SpanGuard {
        let depth = self.current_depth.fetch_add(1, Ordering::SeqCst);

        SpanGuard {
            name: name.to_string(),
            start_time: std::time::Instant::now(),
            depth,
            spans: self.spans.clone(),
            current_depth: self.current_depth.clone(),
        }
    }

    /// Finishes profiling and generates report.
    pub fn finish(self) -> ProfileReport {
        let total_duration = self.start_time.elapsed();
        let spans = self.spans.lock().unwrap().clone();

        ProfileReport {
            name: self.name,
            total_duration,
            spans,
        }
    }
}

/// Span guard that automatically records timing on drop.
pub struct SpanGuard {
    name: String,
    start_time: std::time::Instant,
    depth: usize,
    spans: Arc<Mutex<Vec<Span>>>,
    current_depth: Arc<AtomicUsize>,
}

impl Drop for SpanGuard {
    fn drop(&mut self) {
        let duration = self.start_time.elapsed();
        let span = Span {
            name: self.name.clone(),
            duration,
            depth: self.depth,
        };

        self.spans.lock().unwrap().push(span);
        self.current_depth.fetch_sub(1, Ordering::SeqCst);
    }
}

/// Span timing information.
#[derive(Debug, Clone)]
pub struct Span {
    pub name: String,
    pub duration: std::time::Duration,
    pub depth: usize,
}

/// Profile report structure.
#[derive(Debug, Clone)]
pub struct ProfileReport {
    pub name: String,
    pub total_duration: std::time::Duration,
    pub spans: Vec<Span>,
}

impl ProfileReport {
    /// Formats report as human-readable string.
    ///
    /// ## Output
    /// ```
    /// Profile: handle_request (45.2ms total)
    ///   routing: 2.1ms (4.6%)
    ///   reedbase_lookup: 8.3ms (18.4%)
    ///     cache_check: 0.5ms (1.1%)
    ///     csv_read: 7.8ms (17.3%)
    ///   template_render: 32.4ms (71.7%)
    ///     context_build: 1.2ms (2.7%)
    ///     template_load: 0.8ms (1.8%)
    ///     render: 30.4ms (67.3%)
    ///   response_build: 2.4ms (5.3%)
    /// ```
    pub fn format(&self) -> String {
        let mut output = String::new();

        output.push_str(&format!(
            "Profile: {} ({:.1}ms total)\n",
            self.name,
            self.total_duration.as_secs_f64() * 1000.0
        ));

        for span in &self.spans {
            let indent = "  ".repeat(span.depth + 1);
            let duration_ms = span.duration.as_secs_f64() * 1000.0;
            let percentage = (span.duration.as_secs_f64() / self.total_duration.as_secs_f64()) * 100.0;

            output.push_str(&format!(
                "{}{}: {:.1}ms ({:.1}%)\n",
                indent, span.name, duration_ms, percentage
            ));
        }

        output
    }

    /// Exports report as JSON.
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "name": self.name,
            "total_duration_ms": self.total_duration.as_secs_f64() * 1000.0,
            "spans": self.spans.iter().map(|s| {
                serde_json::json!({
                    "name": s.name,
                    "duration_ms": s.duration.as_secs_f64() * 1000.0,
                    "depth": s.depth
                })
            }).collect::<Vec<_>>()
        })
    }

    /// Identifies bottlenecks (spans > 25% of total time).
    pub fn bottlenecks(&self) -> Vec<&Span> {
        let threshold = self.total_duration.as_secs_f64() * 0.25;

        self.spans
            .iter()
            .filter(|s| s.duration.as_secs_f64() > threshold)
            .collect()
    }
}
```

### Request Profiling Middleware (`src/reedcms/profiler/middleware.rs`)

```rust
/// Profiling middleware for Actix-Web.
///
/// ## Usage
/// Enable via environment variable or config:
/// ```
/// REED_PROFILE=true
/// ```
///
/// ## Output
/// Profile data logged or sent to monitoring endpoint.
pub struct ProfilerMiddleware {
    enabled: bool,
}

impl ProfilerMiddleware {
    pub fn new() -> Self {
        let enabled = std::env::var("REED_PROFILE")
            .map(|v| v == "true" || v == "1")
            .unwrap_or(false);

        Self { enabled }
    }
}

impl<S, B> Transform<S, ServiceRequest> for ProfilerMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = ProfilerMiddlewareService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(ProfilerMiddlewareService {
            service,
            enabled: self.enabled,
        }))
    }
}

pub struct ProfilerMiddlewareService<S> {
    service: S,
    enabled: bool,
}

impl<S, B> Service<ServiceRequest> for ProfilerMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&self, ctx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(ctx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        if !self.enabled {
            return Box::pin(self.service.call(req));
        }

        let profiler = Profiler::start(&format!("request_{}", req.path()));

        // Store profiler in request extensions
        req.extensions_mut().insert(profiler);

        let fut = self.service.call(req);

        Box::pin(async move {
            let res = fut.await?;

            // Retrieve profiler and generate report
            if let Some(profiler) = res.request().extensions().get::<Profiler>() {
                let report = profiler.clone().finish();

                // Log profile report
                println!("{}", report.format());

                // Check for bottlenecks
                let bottlenecks = report.bottlenecks();
                if !bottlenecks.is_empty() {
                    eprintln!("⚠ Performance bottlenecks detected:");
                    for span in bottlenecks {
                        eprintln!("  - {}: {:.1}ms", span.name, span.duration.as_secs_f64() * 1000.0);
                    }
                }
            }

            Ok(res)
        })
    }
}
```

### Slow Query Tracker (`src/reedcms/profiler/slow_queries.rs`)

```rust
/// Tracks slow operations for analysis.
///
/// ## Threshold
/// - Default: 100ms
/// - Configurable via REED_SLOW_THRESHOLD
///
/// ## Storage
/// - Last 100 slow operations
/// - In-memory ring buffer
pub struct SlowQueryTracker {
    threshold: std::time::Duration,
    queries: Arc<Mutex<VecDeque<SlowQuery>>>,
}

impl SlowQueryTracker {
    pub fn new() -> Self {
        let threshold_ms = std::env::var("REED_SLOW_THRESHOLD")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(100);

        Self {
            threshold: std::time::Duration::from_millis(threshold_ms),
            queries: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    /// Records operation if it exceeds threshold.
    pub fn record(&self, operation: &str, duration: std::time::Duration, context: String) {
        if duration >= self.threshold {
            let query = SlowQuery {
                operation: operation.to_string(),
                duration,
                context,
                timestamp: chrono::Utc::now(),
            };

            let mut queries = self.queries.lock().unwrap();
            queries.push_back(query);

            // Keep only last 100
            if queries.len() > 100 {
                queries.pop_front();
            }
        }
    }

    /// Gets all slow queries.
    pub fn get_slow_queries(&self) -> Vec<SlowQuery> {
        self.queries.lock().unwrap().iter().cloned().collect()
    }

    /// Gets slow queries for specific operation.
    pub fn get_by_operation(&self, operation: &str) -> Vec<SlowQuery> {
        self.queries
            .lock()
            .unwrap()
            .iter()
            .filter(|q| q.operation == operation)
            .cloned()
            .collect()
    }
}

/// Slow query record.
#[derive(Debug, Clone, Serialize)]
pub struct SlowQuery {
    pub operation: String,
    pub duration: std::time::Duration,
    pub context: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Gets global slow query tracker.
pub fn global_slow_tracker() -> &'static SlowQueryTracker {
    use std::sync::OnceLock;
    static TRACKER: OnceLock<SlowQueryTracker> = OnceLock::new();
    TRACKER.get_or_init(|| SlowQueryTracker::new())
}
```

### Flame Graph Generator (`src/reedcms/profiler/flamegraph.rs`)

```rust
/// Generates flame graph data from profile report.
///
/// ## Format
/// Collapsed stack format for flame graph tools:
/// ```
/// request;routing 2100
/// request;reedbase_lookup 8300
/// request;reedbase_lookup;cache_check 500
/// request;reedbase_lookup;csv_read 7800
/// request;template_render 32400
/// ```
///
/// ## Usage
/// ```bash
/// reed profile:flamegraph > profile.txt
/// flamegraph.pl profile.txt > flame.svg
/// ```
pub fn generate_flamegraph_data(report: &ProfileReport) -> String {
    let mut output = String::new();
    let mut stack = Vec::new();

    for span in &report.spans {
        // Build stack path
        while stack.len() > span.depth {
            stack.pop();
        }
        stack.push(&span.name);

        // Generate line
        let path = stack.join(";");
        let samples = (span.duration.as_micros() / 100) as u64; // Convert to samples

        output.push_str(&format!("{} {}\n", path, samples));
    }

    output
}

/// Generates flame graph SVG directly.
///
/// ## Output
/// SVG file suitable for viewing in browser.
pub fn generate_svg(report: &ProfileReport, width: u32, height: u32) -> String {
    let data = generate_flamegraph_data(report);

    // Simplified SVG generation
    // Full implementation would use proper flame graph rendering
    format!(
        r#"<svg width="{}" height="{}" xmlns="http://www.w3.org/2000/svg">
    <text x="10" y="20">Flame Graph: {}</text>
    <text x="10" y="40">Total: {:.1}ms</text>
</svg>"#,
        width,
        height,
        report.name,
        report.total_duration.as_secs_f64() * 1000.0
    )
}
```

### CLI Commands (`src/reedcms/cli/commands/profile.rs`)

```rust
/// CLI command: reed profile:request {url}
///
/// ## Usage
/// ```bash
/// reed profile:request /knowledge
/// ```
///
/// ## Output
/// Detailed profile report with timing breakdown.
pub async fn execute_profile_request(url: &str) -> ReedResult<()> {
    println!("📊 Profiling request: {}\n", url);

    let profiler = Profiler::start("request");

    // Simulate request
    let _routing = profiler.span("routing");
    std::thread::sleep(std::time::Duration::from_millis(2));
    drop(_routing);

    let _reedbase = profiler.span("reedbase_lookup");
    std::thread::sleep(std::time::Duration::from_millis(8));
    drop(_reedbase);

    let _render = profiler.span("template_render");
    std::thread::sleep(std::time::Duration::from_millis(32));
    drop(_render);

    let report = profiler.finish();
    println!("{}", report.format());

    // Show bottlenecks
    let bottlenecks = report.bottlenecks();
    if !bottlenecks.is_empty() {
        println!("\n⚠ Bottlenecks:");
        for span in bottlenecks {
            println!("  - {}: {:.1}ms", span.name, span.duration.as_secs_f64() * 1000.0);
        }
    }

    Ok(())
}

/// CLI command: reed profile:slow
///
/// ## Usage
/// ```bash
/// reed profile:slow
/// ```
///
/// ## Output
/// List of slow queries/operations.
pub async fn execute_profile_slow() -> ReedResult<()> {
    println!("🐌 Slow Operations Report\n");

    let queries = global_slow_tracker().get_slow_queries();

    if queries.is_empty() {
        println!("No slow operations recorded.");
        return Ok(());
    }

    println!("Found {} slow operations:\n", queries.len());

    for query in queries {
        println!("📍 {}", query.operation);
        println!("   Duration: {:.1}ms", query.duration.as_secs_f64() * 1000.0);
        println!("   Context: {}", query.context);
        println!("   Time: {}", query.timestamp.format("%Y-%m-%d %H:%M:%S"));
        println!();
    }

    Ok(())
}

/// CLI command: reed profile:flamegraph
///
/// ## Usage
/// ```bash
/// reed profile:flamegraph > profile.txt
/// ```
pub async fn execute_profile_flamegraph() -> ReedResult<()> {
    // Generate sample profile
    let profiler = Profiler::start("sample");
    // ... record spans ...
    let report = profiler.finish();

    let data = generate_flamegraph_data(&report);
    print!("{}", data);

    Ok(())
}
```

## Implementation Files

### Primary Implementation
- `src/reedcms/profiler/core.rs` - Profiler core
- `src/reedcms/profiler/middleware.rs` - Profiling middleware
- `src/reedcms/profiler/slow_queries.rs` - Slow query tracker
- `src/reedcms/profiler/flamegraph.rs` - Flame graph generator
- `src/reedcms/cli/commands/profile.rs` - CLI commands

### Test Files
- `src/reedcms/profiler/core.test.rs`
- `src/reedcms/profiler/middleware.test.rs`
- `src/reedcms/profiler/slow_queries.test.rs`
- `src/reedcms/profiler/flamegraph.test.rs`

## Testing Requirements

### Unit Tests
- [ ] Test profiler span tracking
- [ ] Test nested spans
- [ ] Test report generation
- [ ] Test bottleneck detection
- [ ] Test slow query recording
- [ ] Test flame graph data generation

### Integration Tests
- [ ] Test profiler with actual requests
- [ ] Test middleware integration
- [ ] Test slow query tracking
- [ ] Test profile report accuracy

### Performance Tests
- [ ] Span start/end: < 1μs
- [ ] Report generation: < 100μs
- [ ] Profiler overhead: < 0.1% request time
- [ ] Memory usage: < 1KB per profile

## Acceptance Criteria
- [ ] Profiler core implemented
- [ ] Span tracking functional
- [ ] Nested span support working
- [ ] Profile reports generated correctly
- [ ] Bottleneck detection implemented
- [ ] Slow query tracking functional
- [ ] Flame graph generation working
- [ ] Middleware integration complete
- [ ] CLI commands functional
- [ ] Zero-cost when disabled
- [ ] All tests pass with 100% coverage
- [ ] Performance benchmarks met
- [ ] Documentation complete
- [ ] BBC English throughout

## Dependencies
- **Requires**: REED-10-01 (ReedMonitor Foundation)

## Blocks
- REED-10-03 (Debug Tools use profiler data)

## References
- Service Template: `_workbench/Tickets/templates/service-template.md`
- Service Test Template: `_workbench/Tickets/templates/service-template.test.md`
- Summary: Lines 1048-1050 in `project_summary.md`

## Notes
Performance profiler provides granular timing data essential for optimisation. Span-based tracking enables hierarchical timing analysis. Automatic bottleneck detection highlights problem areas. Slow query tracking identifies persistent performance issues. Flame graph visualisation enables intuitive performance analysis. Zero-cost when disabled ensures no production overhead.
