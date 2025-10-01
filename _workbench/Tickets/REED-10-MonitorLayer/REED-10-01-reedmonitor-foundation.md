# REED-10-01: ReedMonitor Foundation System

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
- **ID**: REED-10-01
- **Title**: ReedMonitor Foundation and Metrics Collection
- **Layer**: Monitor Layer (REED-10)
- **Priority**: Medium
- **Status**: Open
- **Complexity**: High
- **Dependencies**: REED-06-01

## Summary Reference
- **Section**: ReedMonitor System
- **Lines**: 1045-1047, 1545-1582 in project_summary.md
- **Key Concepts**: Performance monitoring, metrics collection, request tracking, system health

## Objective
Implement ReedMonitor foundation system with FreeBSD-style syslog output, performance metrics collection, request tracking, system resource monitoring, and health check endpoints for production operations.

## Requirements

### 1. FreeBSD-Style Logging System (`src/reedcms/monitor/syslog.rs`)

#### Log Format Specification
```
{timestamp} {hostname} {process}[{pid}]: {level}: {message}

Example:
Dec 15 14:23:01 server reedcms[1234]: INFO: Server started on 127.0.0.1:3000
Dec 15 14:23:02 server reedcms[1234]: INFO: METRIC[counter] requests_total: 42
Dec 15 14:23:03 server reedcms[1234]: ERROR: Database connection failed: timeout
```

#### Log Levels (RFC 5424)
```rust
pub enum LogLevel {
    EMERG = 0,   // Emergency: system is unusable
    ALERT = 1,   // Alert: action must be taken immediately
    CRIT = 2,    // Critical: critical conditions
    ERROR = 3,   // Error: error conditions
    WARN = 4,    // Warning: warning conditions
    NOTICE = 5,  // Notice: normal but significant condition
    INFO = 6,    // Informational: informational messages
    DEBUG = 7,   // Debug: debug-level messages
}
```

#### Output Modes
```rust
pub enum OutputMode {
    Silent,      // No output (metrics only)
    Log,         // Write to log file
    Forward,     // Forward to external system (syslog, journald)
    Both,        // Log file + forward
}
```

#### Implementation
```rust
/// FreeBSD-style syslog logger.
pub struct SysLogger {
    hostname: String,
    process_name: String,
    pid: u32,
    output_mode: OutputMode,
    log_file: Option<std::fs::File>,
    min_level: LogLevel,
}

impl SysLogger {
    /// Creates new syslog logger.
    pub fn new(output_mode: OutputMode, min_level: LogLevel) -> ReedResult<Self> {
        Ok(Self {
            hostname: get_hostname(),
            process_name: "reedcms".to_string(),
            pid: std::process::id(),
            output_mode,
            log_file: Self::open_log_file(&output_mode)?,
            min_level,
        })
    }

    /// Logs message at specified level.
    ///
    /// ## Performance
    /// - < 50μs for Silent mode
    /// - < 500μs for file write
    /// - < 1ms for external forward
    pub fn log(&mut self, level: LogLevel, message: &str) {
        if level as u8 > self.min_level as u8 {
            return; // Level filtered out
        }

        let timestamp = format_timestamp();
        let formatted = format!(
            "{} {} {}[{}]: {}: {}",
            timestamp,
            self.hostname,
            self.process_name,
            self.pid,
            level.as_str(),
            message
        );

        match self.output_mode {
            OutputMode::Silent => {},
            OutputMode::Log => self.write_to_file(&formatted),
            OutputMode::Forward => self.forward_to_syslog(&formatted),
            OutputMode::Both => {
                self.write_to_file(&formatted);
                self.forward_to_syslog(&formatted);
            }
        }
    }

    /// Logs metric in standard format.
    pub fn log_metric(&mut self, metric_type: &str, name: &str, value: &str) {
        let message = format!("METRIC[{}] {}: {}", metric_type, name, value);
        self.log(LogLevel::INFO, &message);
    }

    fn write_to_file(&mut self, message: &str) {
        if let Some(ref mut file) = self.log_file {
            use std::io::Write;
            let _ = writeln!(file, "{}", message);
        }
    }

    fn forward_to_syslog(&self, message: &str) {
        // Forward to system syslog
        #[cfg(target_os = "linux")]
        {
            // Use libc syslog
        }
        #[cfg(target_os = "macos")]
        {
            // Use unified logging system
        }
    }

    fn open_log_file(mode: &OutputMode) -> ReedResult<Option<std::fs::File>> {
        match mode {
            OutputMode::Log | OutputMode::Both => {
                let path = ".reed/flow/reedmonitor.log";
                std::fs::create_dir_all(".reed/flow")?;
                let file = std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(path)?;
                Ok(Some(file))
            }
            _ => Ok(None),
        }
    }
}

impl LogLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            LogLevel::EMERG => "EMERG",
            LogLevel::ALERT => "ALERT",
            LogLevel::CRIT => "CRIT",
            LogLevel::ERROR => "ERROR",
            LogLevel::WARN => "WARN",
            LogLevel::NOTICE => "NOTICE",
            LogLevel::INFO => "INFO",
            LogLevel::DEBUG => "DEBUG",
        }
    }
}

/// Gets system hostname.
fn get_hostname() -> String {
    hostname::get()
        .ok()
        .and_then(|h| h.into_string().ok())
        .unwrap_or_else(|| "localhost".to_string())
}

/// Formats timestamp in BSD syslog format.
fn format_timestamp() -> String {
    chrono::Local::now().format("%b %d %H:%M:%S").to_string()
}
```

#### Log File Management
```rust
/// Manages log file rotation and cleanup.
pub struct LogFileManager;

impl LogFileManager {
    /// Rotates log file if size exceeds limit.
    ///
    /// ## Rotation Strategy
    /// - Max size: 100MB per file
    /// - Keep last 10 files
    /// - Compress old files with gzip
    pub fn rotate_if_needed(log_path: &str) -> ReedResult<()> {
        let metadata = std::fs::metadata(log_path)?;
        if metadata.len() > 100 * 1024 * 1024 {
            Self::rotate_log(log_path)?;
        }
        Ok(())
    }

    fn rotate_log(log_path: &str) -> ReedResult<()> {
        let timestamp = chrono::Utc::now().format("%Y%m%d-%H%M%S");
        let rotated = format!("{}.{}.gz", log_path, timestamp);
        
        // Compress and rename
        let content = std::fs::read(log_path)?;
        let compressed = compress_gzip(&content)?;
        std::fs::write(&rotated, compressed)?;
        
        // Clear original file
        std::fs::write(log_path, "")?;
        
        // Cleanup old logs
        Self::cleanup_old_logs(log_path)?;
        
        Ok(())
    }

    fn cleanup_old_logs(log_path: &str) -> ReedResult<()> {
        // Keep only last 10 rotated files
        let dir = std::path::Path::new(log_path).parent().unwrap();
        let base_name = std::path::Path::new(log_path).file_name().unwrap();
        
        let mut log_files: Vec<_> = std::fs::read_dir(dir)?
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.file_name().to_string_lossy().starts_with(base_name.to_str().unwrap())
                    && e.file_name().to_string_lossy().ends_with(".gz")
            })
            .collect();
        
        log_files.sort_by_key(|e| e.metadata().unwrap().modified().unwrap());
        
        // Remove oldest files if more than 10
        if log_files.len() > 10 {
            for file in log_files.iter().take(log_files.len() - 10) {
                let _ = std::fs::remove_file(file.path());
            }
        }
        
        Ok(())
    }
}
```

### 2. Metrics Collection System

### Metrics Collected

#### Request Metrics
- Request count (total, per endpoint)
- Response times (min, max, avg, p50, p95, p99)
- Status code distribution (2xx, 3xx, 4xx, 5xx)
- Throughput (requests per second)

#### System Metrics
- Memory usage (RSS, heap)
- CPU usage
- Active connections
- Thread count
- Uptime

#### ReedBase Metrics
- Cache hit rate
- Lookup times
- CSV read times
- Data volume

#### Template Metrics
- Render times
- Cache hit rate
- Template errors

### Implementation (`src/reedcms/monitor/core.rs`)

```rust
/// ReedMonitor core metrics collection system.
///
/// ## Metrics Storage
/// - In-memory time-series data
/// - Rolling window (last 24 hours)
/// - Aggregated statistics
///
/// ## Performance
/// - Metric recording: < 10μs
/// - Metric retrieval: < 1ms
/// - Memory usage: ~10MB for 24h of data
///
/// ## Thread Safety
/// - Lock-free metrics recording
/// - Atomic counters
/// - Thread-safe aggregation
pub struct ReedMonitor {
    metrics: Arc<RwLock<Metrics>>,
    start_time: std::time::Instant,
}

impl ReedMonitor {
    /// Creates new ReedMonitor instance.
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(RwLock::new(Metrics::new())),
            start_time: std::time::Instant::now(),
        }
    }

    /// Records request metric.
    ///
    /// ## Arguments
    /// - method: HTTP method
    /// - path: Request path
    /// - status: Response status code
    /// - duration: Request duration
    pub fn record_request(
        &self,
        method: &str,
        path: &str,
        status: u16,
        duration: std::time::Duration,
    ) {
        let mut metrics = self.metrics.write().unwrap();
        metrics.record_request(method, path, status, duration);
    }

    /// Records ReedBase lookup metric.
    pub fn record_reedbase_lookup(&self, key: &str, duration: std::time::Duration, hit: bool) {
        let mut metrics = self.metrics.write().unwrap();
        metrics.record_reedbase_lookup(key, duration, hit);
    }

    /// Records template render metric.
    pub fn record_template_render(&self, template: &str, duration: std::time::Duration) {
        let mut metrics = self.metrics.write().unwrap();
        metrics.record_template_render(template, duration);
    }

    /// Gets current metrics snapshot.
    pub fn get_snapshot(&self) -> MetricsSnapshot {
        let metrics = self.metrics.read().unwrap();
        metrics.snapshot(self.start_time.elapsed())
    }

    /// Gets system health status.
    pub fn get_health(&self) -> HealthStatus {
        let snapshot = self.get_snapshot();

        let status = if snapshot.error_rate > 0.05 {
            Health::Unhealthy
        } else if snapshot.avg_response_time > std::time::Duration::from_millis(500) {
            Health::Degraded
        } else {
            Health::Healthy
        };

        HealthStatus {
            status,
            uptime: snapshot.uptime,
            total_requests: snapshot.total_requests,
            error_rate: snapshot.error_rate,
            avg_response_time: snapshot.avg_response_time,
        }
    }

    /// Resets metrics (for testing).
    pub fn reset(&self) {
        let mut metrics = self.metrics.write().unwrap();
        *metrics = Metrics::new();
    }
}

/// Gets global ReedMonitor instance.
pub fn global_monitor() -> &'static ReedMonitor {
    use std::sync::OnceLock;
    static MONITOR: OnceLock<ReedMonitor> = OnceLock::new();
    MONITOR.get_or_init(|| ReedMonitor::new())
}
```

### Metrics Storage (`src/reedcms/monitor/metrics.rs`)

```rust
/// Metrics storage structure.
pub struct Metrics {
    requests: RequestMetrics,
    reedbase: ReedBaseMetrics,
    templates: TemplateMetrics,
    system: SystemMetrics,
}

impl Metrics {
    pub fn new() -> Self {
        Self {
            requests: RequestMetrics::new(),
            reedbase: ReedBaseMetrics::new(),
            templates: TemplateMetrics::new(),
            system: SystemMetrics::new(),
        }
    }

    pub fn record_request(
        &mut self,
        method: &str,
        path: &str,
        status: u16,
        duration: std::time::Duration,
    ) {
        self.requests.record(method, path, status, duration);
    }

    pub fn record_reedbase_lookup(
        &mut self,
        key: &str,
        duration: std::time::Duration,
        hit: bool,
    ) {
        self.reedbase.record(key, duration, hit);
    }

    pub fn record_template_render(&mut self, template: &str, duration: std::time::Duration) {
        self.templates.record(template, duration);
    }

    pub fn snapshot(&self, uptime: std::time::Duration) -> MetricsSnapshot {
        MetricsSnapshot {
            uptime,
            total_requests: self.requests.total_count,
            avg_response_time: self.requests.avg_duration(),
            error_rate: self.requests.error_rate(),
            requests_by_path: self.requests.by_path.clone(),
            status_codes: self.requests.status_codes.clone(),
            reedbase_hit_rate: self.reedbase.hit_rate(),
            reedbase_avg_time: self.reedbase.avg_duration(),
            template_avg_time: self.templates.avg_duration(),
            memory_usage: self.system.memory_usage(),
        }
    }
}

/// Request metrics structure.
#[derive(Debug, Clone)]
pub struct RequestMetrics {
    total_count: u64,
    durations: Vec<std::time::Duration>,
    by_path: HashMap<String, u64>,
    status_codes: HashMap<u16, u64>,
}

impl RequestMetrics {
    pub fn new() -> Self {
        Self {
            total_count: 0,
            durations: Vec::new(),
            by_path: HashMap::new(),
            status_codes: HashMap::new(),
        }
    }

    pub fn record(
        &mut self,
        method: &str,
        path: &str,
        status: u16,
        duration: std::time::Duration,
    ) {
        self.total_count += 1;
        self.durations.push(duration);

        // Trim old durations (keep last 10000)
        if self.durations.len() > 10000 {
            self.durations.drain(0..5000);
        }

        // Track by path
        *self.by_path.entry(path.to_string()).or_insert(0) += 1;

        // Track status codes
        *self.status_codes.entry(status).or_insert(0) += 1;
    }

    pub fn avg_duration(&self) -> std::time::Duration {
        if self.durations.is_empty() {
            return std::time::Duration::from_secs(0);
        }

        let total: std::time::Duration = self.durations.iter().sum();
        total / self.durations.len() as u32
    }

    pub fn error_rate(&self) -> f64 {
        if self.total_count == 0 {
            return 0.0;
        }

        let errors: u64 = self
            .status_codes
            .iter()
            .filter(|(code, _)| **code >= 400)
            .map(|(_, count)| count)
            .sum();

        errors as f64 / self.total_count as f64
    }
}

/// ReedBase metrics structure.
#[derive(Debug, Clone)]
pub struct ReedBaseMetrics {
    total_lookups: u64,
    cache_hits: u64,
    durations: Vec<std::time::Duration>,
}

impl ReedBaseMetrics {
    pub fn new() -> Self {
        Self {
            total_lookups: 0,
            cache_hits: 0,
            durations: Vec::new(),
        }
    }

    pub fn record(&mut self, key: &str, duration: std::time::Duration, hit: bool) {
        self.total_lookups += 1;
        if hit {
            self.cache_hits += 1;
        }
        self.durations.push(duration);

        // Trim old durations
        if self.durations.len() > 10000 {
            self.durations.drain(0..5000);
        }
    }

    pub fn hit_rate(&self) -> f64 {
        if self.total_lookups == 0 {
            return 0.0;
        }
        self.cache_hits as f64 / self.total_lookups as f64
    }

    pub fn avg_duration(&self) -> std::time::Duration {
        if self.durations.is_empty() {
            return std::time::Duration::from_secs(0);
        }

        let total: std::time::Duration = self.durations.iter().sum();
        total / self.durations.len() as u32
    }
}

/// Template metrics structure.
#[derive(Debug, Clone)]
pub struct TemplateMetrics {
    render_count: u64,
    durations: Vec<std::time::Duration>,
}

impl TemplateMetrics {
    pub fn new() -> Self {
        Self {
            render_count: 0,
            durations: Vec::new(),
        }
    }

    pub fn record(&mut self, template: &str, duration: std::time::Duration) {
        self.render_count += 1;
        self.durations.push(duration);

        if self.durations.len() > 10000 {
            self.durations.drain(0..5000);
        }
    }

    pub fn avg_duration(&self) -> std::time::Duration {
        if self.durations.is_empty() {
            return std::time::Duration::from_secs(0);
        }

        let total: std::time::Duration = self.durations.iter().sum();
        total / self.durations.len() as u32
    }
}

/// System metrics structure.
#[derive(Debug, Clone)]
pub struct SystemMetrics;

impl SystemMetrics {
    pub fn new() -> Self {
        Self
    }

    pub fn memory_usage(&self) -> u64 {
        // Get RSS memory usage
        #[cfg(target_os = "linux")]
        {
            if let Ok(status) = std::fs::read_to_string("/proc/self/status") {
                for line in status.lines() {
                    if line.starts_with("VmRSS:") {
                        if let Some(kb) = line.split_whitespace().nth(1) {
                            if let Ok(kb_val) = kb.parse::<u64>() {
                                return kb_val * 1024; // Convert to bytes
                            }
                        }
                    }
                }
            }
        }

        0
    }
}

/// Metrics snapshot structure.
#[derive(Debug, Clone, Serialize)]
pub struct MetricsSnapshot {
    pub uptime: std::time::Duration,
    pub total_requests: u64,
    pub avg_response_time: std::time::Duration,
    pub error_rate: f64,
    pub requests_by_path: HashMap<String, u64>,
    pub status_codes: HashMap<u16, u64>,
    pub reedbase_hit_rate: f64,
    pub reedbase_avg_time: std::time::Duration,
    pub template_avg_time: std::time::Duration,
    pub memory_usage: u64,
}

/// Health status enum.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub enum Health {
    Healthy,
    Degraded,
    Unhealthy,
}

/// Health status structure.
#[derive(Debug, Clone, Serialize)]
pub struct HealthStatus {
    pub status: Health,
    pub uptime: std::time::Duration,
    pub total_requests: u64,
    pub error_rate: f64,
    pub avg_response_time: std::time::Duration,
}
```

### Middleware Integration (`src/reedcms/monitor/middleware.rs`)

```rust
/// Monitoring middleware for Actix-Web.
///
/// ## Functionality
/// - Records request duration
/// - Tracks status codes
/// - Updates ReedMonitor metrics
///
/// ## Performance Overhead
/// - < 10μs per request
/// - Negligible impact on throughput
pub struct MonitorMiddleware;

impl<S, B> Transform<S, ServiceRequest> for MonitorMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = MonitorMiddlewareService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(MonitorMiddlewareService { service }))
    }
}

pub struct MonitorMiddlewareService<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for MonitorMiddlewareService<S>
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
        let start_time = std::time::Instant::now();
        let method = req.method().to_string();
        let path = req.path().to_string();

        let fut = self.service.call(req);

        Box::pin(async move {
            let res = fut.await?;
            let duration = start_time.elapsed();
            let status = res.status().as_u16();

            // Record metrics
            global_monitor().record_request(&method, &path, status, duration);

            Ok(res)
        })
    }
}
```

### Health Check Endpoint (`src/reedcms/monitor/health.rs`)

```rust
/// Health check endpoint handler.
///
/// ## Endpoint
/// GET /health
///
/// ## Response
/// ```json
/// {
///   "status": "healthy",
///   "uptime": "3h 24m 15s",
///   "total_requests": 15234,
///   "error_rate": 0.023,
///   "avg_response_time": "45ms"
/// }
/// ```
pub async fn health_check() -> Result<HttpResponse, Error> {
    let health = global_monitor().get_health();

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": format!("{:?}", health.status).to_lowercase(),
        "uptime": format_duration(health.uptime),
        "total_requests": health.total_requests,
        "error_rate": format!("{:.3}", health.error_rate),
        "avg_response_time": format!("{}ms", health.avg_response_time.as_millis())
    })))
}

/// Metrics endpoint handler.
///
/// ## Endpoint
/// GET /metrics
///
/// ## Response
/// Detailed metrics snapshot in JSON format.
pub async fn metrics_endpoint() -> Result<HttpResponse, Error> {
    let snapshot = global_monitor().get_snapshot();
    Ok(HttpResponse::Ok().json(snapshot))
}

/// Formats duration as human-readable string.
fn format_duration(duration: std::time::Duration) -> String {
    let secs = duration.as_secs();
    let hours = secs / 3600;
    let minutes = (secs % 3600) / 60;
    let seconds = secs % 60;

    if hours > 0 {
        format!("{}h {}m {}s", hours, minutes, seconds)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, seconds)
    } else {
        format!("{}s", seconds)
    }
}
```

## Implementation Files

### Primary Implementation
- `src/reedcms/monitor/syslog.rs` - FreeBSD syslog logger
- `src/reedcms/monitor/log_manager.rs` - Log file rotation and cleanup
- `src/reedcms/monitor/core.rs` - Monitor core
- `src/reedcms/monitor/metrics.rs` - Metrics storage
- `src/reedcms/monitor/middleware.rs` - Actix middleware
- `src/reedcms/monitor/health.rs` - Health check endpoints

### Test Files
- `src/reedcms/monitor/syslog.test.rs`
- `src/reedcms/monitor/log_manager.test.rs`
- `src/reedcms/monitor/core.test.rs`
- `src/reedcms/monitor/metrics.test.rs`
- `src/reedcms/monitor/middleware.test.rs`
- `src/reedcms/monitor/health.test.rs`

## Testing Requirements

### Unit Tests
- [ ] Test FreeBSD syslog format generation
- [ ] Test all 8 log levels (EMERG through DEBUG)
- [ ] Test log level filtering
- [ ] Test output mode switching (Silent/Log/Forward/Both)
- [ ] Test hostname and PID integration
- [ ] Test log file rotation
- [ ] Test log file cleanup (keep last 10)
- [ ] Test metrics recording
- [ ] Test metrics aggregation
- [ ] Test health status calculation
- [ ] Test duration averaging
- [ ] Test error rate calculation
- [ ] Test cache hit rate calculation

### Integration Tests
- [ ] Test syslog logger with actual file writes
- [ ] Test log rotation trigger at 100MB
- [ ] Test log compression (gzip)
- [ ] Test middleware integration
- [ ] Test health endpoint
- [ ] Test metrics endpoint
- [ ] Test concurrent metric recording
- [ ] Test memory trimming
- [ ] Test FreeBSD logging from all layers (Data, Template, Server)

### Performance Tests
- [ ] Silent mode logging: < 50μs
- [ ] File write logging: < 500μs
- [ ] External forward: < 1ms
- [ ] Metric recording: < 10μs
- [ ] Metric retrieval: < 1ms
- [ ] Middleware overhead: < 10μs
- [ ] Log file rotation: < 100ms
- [ ] Memory usage: < 10MB for 24h data

## Acceptance Criteria
- [ ] FreeBSD syslog format implemented with all 8 log levels
- [ ] Hostname and PID integration working
- [ ] All 4 output modes functional (Silent/Log/Forward/Both)
- [ ] Log file rotation at 100MB implemented
- [ ] Log file cleanup (keep last 10) working
- [ ] Log compression (gzip) functional
- [ ] ReedMonitor core implemented
- [ ] Request metrics collection working
- [ ] ReedBase metrics collection working
- [ ] Template metrics collection working
- [ ] System metrics collection working
- [ ] Middleware integration functional
- [ ] Health check endpoint working
- [ ] Metrics endpoint working
- [ ] Thread-safe metrics recording
- [ ] All tests pass with 100% coverage
- [ ] Performance benchmarks met (syslog + metrics)
- [ ] Documentation complete
- [ ] BBC English throughout

## Dependencies
- **Requires**: REED-06-01 (Server Foundation)

## Blocks
- REED-10-02 (Performance Profiler uses monitor data)
- REED-10-03 (Debug Tools use monitor data)

## References
- Service Template: `_workbench/Tickets/templates/service-template.md`
- Service Test Template: `_workbench/Tickets/templates/service-template.test.md`
- Summary: Lines 1045-1047, 1545-1582 in `project_summary.md`

## Notes
ReedMonitor provides essential visibility into production system behaviour. Low-overhead metrics collection ensures minimal performance impact. Health checks enable load balancer integration and alerting. Request tracking helps identify performance bottlenecks. ReedBase metrics reveal cache effectiveness. Thread-safe recording enables concurrent access without contention.
