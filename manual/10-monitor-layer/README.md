# Monitor Layer (10)

Provides production monitoring, performance profiling, and development debugging tools.

## Purpose

The Monitor Layer ensures system observability and debugging capabilities:

- **ReedMonitor**: FreeBSD-style syslog with metrics collection
- **Performance Profiler**: Detailed request timing and bottleneck detection
- **Debug Tools**: Development utilities for troubleshooting
- **Health Checks**: System health monitoring and alerting

## Architecture

```
┌─────────────────────────────────────────────────────┐
│ Monitor Layer Architecture                           │
├─────────────────────────────────────────────────────┤
│                                                       │
│  Production Monitoring                               │
│  ┌──────────────────────────────────────────┐       │
│  │ ReedMonitor (FreeBSD Syslog)             │       │
│  │  ├─ Log Levels: EMERG → DEBUG (RFC 5424) │       │
│  │  ├─ Output: Silent / Log / Forward / Both│       │
│  │  ├─ Format: {time} {host} reed[PID]: MSG │       │
│  │  └─ Rotation: 100MB, keep 10, gzip       │       │
│  └──────────────────────────────────────────┘       │
│           ↓                                           │
│  ┌──────────────────────────────────────────┐       │
│  │ Metrics Collection                        │       │
│  │  ├─ Request: Count, timing, status       │       │
│  │  ├─ ReedBase: Hit rate, lookup time      │       │
│  │  ├─ Templates: Render time, cache hits   │       │
│  │  ├─ System: Memory, CPU, connections     │       │
│  │  └─ Storage: In-memory, 24h rolling      │       │
│  └──────────────────────────────────────────┘       │
│           ↓                                           │
│  ┌──────────────────────────────────────────┐       │
│  │ Health Check Endpoints                    │       │
│  │  ├─ GET /health → Status + metrics       │       │
│  │  ├─ GET /metrics → Full snapshot         │       │
│  │  └─ Status: Healthy / Degraded / Unhealthy│      │
│  └──────────────────────────────────────────┘       │
│                                                       │
│  Development Tools                                   │
│  ┌──────────────────────────────────────────┐       │
│  │ Performance Profiler                      │       │
│  │  ├─ Span Tracking: Hierarchical timing   │       │
│  │  ├─ Bottleneck Detection: > 25% threshold│       │
│  │  ├─ Slow Query Tracker: > 100ms default  │       │
│  │  ├─ Flame Graphs: Visualisation support  │       │
│  │  └─ Zero-Cost: Disabled in production    │       │
│  └──────────────────────────────────────────┘       │
│           ↓                                           │
│  ┌──────────────────────────────────────────┐       │
│  │ Debug Tools                               │       │
│  │  ├─ Request Inspector: Full request dump │       │
│  │  ├─ Cache Viewer: ReedBase contents      │       │
│  │  ├─ Route Tester: URL resolution check   │       │
│  │  └─ Config Inspector: All settings view  │       │
│  └──────────────────────────────────────────┘       │
│                                                       │
└─────────────────────────────────────────────────────┘
```

## Key Components

### 1. ReedMonitor (FreeBSD Syslog)

**Purpose**: Production-grade logging with RFC 5424 compliance

**Log Format**:
```
Dec 15 14:23:01 server reedcms[1234]: INFO: Server started on 127.0.0.1:3000
Dec 15 14:23:02 server reedcms[1234]: INFO: METRIC[counter] requests_total: 42
Dec 15 14:23:03 server reedcms[1234]: ERROR: Database connection failed: timeout
```

**Log Levels** (RFC 5424):
```
0 - EMERG   Emergency: System unusable
1 - ALERT   Alert: Immediate action required
2 - CRIT    Critical: Critical conditions
3 - ERROR   Error: Error conditions
4 - WARN    Warning: Warning conditions
5 - NOTICE  Notice: Normal but significant
6 - INFO    Informational messages
7 - DEBUG   Debug-level messages
```

**Output Modes**:
- `Silent`: No output (metrics only)
- `Log`: Write to `.reed/flow/reedmonitor.log`
- `Forward`: Forward to system syslog/journald
- `Both`: Log file + forward

**See**: [Monitoring System](monitoring-system.md)

### 2. Metrics Collection

**Request Metrics**:
- Total request count
- Response times (min/max/avg/p50/p95/p99)
- Status code distribution (2xx/3xx/4xx/5xx)
- Requests per second (throughput)

**ReedBase Metrics**:
- Cache hit rate
- Lookup times (avg/p95/p99)
- Total lookups
- Cache size

**Template Metrics**:
- Render times (avg/p95/p99)
- Cache hits
- Template errors

**System Metrics**:
- Memory usage (RSS)
- CPU usage
- Active connections
- Uptime

**Storage**:
- In-memory time-series data
- Rolling 24-hour window
- Trim old data automatically
- ~10 MB memory usage

### 3. Performance Profiler

**Purpose**: Detailed request timing analysis

**Span Tracking**:
```rust
let profiler = Profiler::start("handle_request");

let _routing = profiler.span("routing");
// ... routing logic ...
drop(_routing);

let _render = profiler.span("template_render");
// ... rendering logic ...
drop(_render);

let report = profiler.finish();
println!("{}", report.format());
```

**Output**:
```
Profile: handle_request (45.2ms total)
  routing: 2.1ms (4.6%)
  reedbase_lookup: 8.3ms (18.4%)
    cache_check: 0.5ms (1.1%)
    csv_read: 7.8ms (17.3%)
  template_render: 32.4ms (71.7%)
    context_build: 1.2ms (2.7%)
    render: 30.4ms (67.3%)
  response_build: 2.4ms (5.3%)
```

**Features**:
- Nested span support
- Bottleneck detection (> 25% threshold)
- Slow query tracking (> 100ms default)
- Flame graph generation
- Zero-cost when disabled

**See**: [Performance Profiler](performance-profiler.md)

### 4. Debug Tools

**Request Inspector**:
```bash
reed debug:request /knowledge

# Output:
🔍 Request Inspector: /knowledge

URL Analysis:
  Path: /knowledge
  Route Match: knowledge (layout: knowledge, lang: en)

Headers:
  User-Agent: curl/7.88.1
  Accept: */*

Response Preview:
  Status: 200 OK
  Content-Type: text/html

Timing Breakdown:
  Routing: 2.1ms
  ReedBase: 8.3ms
  Rendering: 32.4ms
  Total: 42.8ms
```

**Cache Viewer**:
```bash
reed debug:cache

# Output:
📦 ReedBase Cache Contents

Text Cache (124 entries):
  knowledge.title (en): "Knowledge Base"
  blog.title (en): "Blog"

Cache Statistics:
  Total entries: 312
  Memory usage: ~1.2 MB
  Hit rate: 94.3%
```

**Route Tester**:
```bash
reed debug:route /knowledge

# Output:
🛣️  Route Test: /knowledge

Resolution:
  ✓ Match found
  Layout: knowledge
  Language: en

Template:
  Path: templates/layouts/knowledge/knowledge.mouse.jinja
  ✓ Template exists

Assets:
  CSS: ✓
  JS: ✓
```

**See**: [Debug Tools](debug-tools.md)

## Health Check System

### Health Endpoint

```bash
curl http://localhost:3000/health
```

**Response**:
```json
{
  "status": "healthy",
  "uptime": "3h 24m 15s",
  "total_requests": 15234,
  "error_rate": 0.023,
  "avg_response_time": "45ms"
}
```

### Metrics Endpoint

```bash
curl http://localhost:3000/metrics
```

**Response**:
```json
{
  "uptime": "3h 24m 15s",
  "total_requests": 15234,
  "avg_response_time": "45ms",
  "error_rate": 0.023,
  "requests_by_path": {
    "/knowledge": 8234,
    "/blog": 4123,
    "/": 2877
  },
  "status_codes": {
    "200": 14876,
    "404": 312,
    "500": 46
  },
  "reedbase_hit_rate": 0.943,
  "reedbase_avg_time": "12μs",
  "template_avg_time": "32ms",
  "memory_usage": 52428800
}
```

### Health Status Calculation

```
Healthy:
  - Error rate < 5%
  - Avg response time < 500ms

Degraded:
  - Error rate < 10%
  - Avg response time < 1000ms

Unhealthy:
  - Error rate ≥ 10%
  - OR avg response time ≥ 1000ms
```

## Performance Characteristics

| Operation | Timing | Note |
|-----------|--------|------|
| **Syslog (Silent mode)** | < 50μs | No I/O |
| **Syslog (File write)** | < 500μs | Append to log |
| **Syslog (Forward)** | < 1ms | System syslog |
| **Metric recording** | < 10μs | Atomic operation |
| **Metric retrieval** | < 1ms | Read lock + clone |
| **Health check** | < 1ms | Snapshot + calculation |
| **Profiler span** | < 1μs | Start/stop timing |
| **Profile report** | < 100μs | Format generation |
| **Request inspection** | < 50ms | Full analysis |
| **Cache viewing** | < 100ms | Snapshot + filter |
| **Route testing** | < 10ms | URL resolution |

## Log File Management

### Rotation Strategy

```
Log file: .reed/flow/reedmonitor.log

Trigger: File size > 100 MB
Action:
  1. Compress with gzip
  2. Rename: reedmonitor.log.20250115-143218.gz
  3. Clear original file
  4. Keep last 10 rotated files
  5. Delete older files
```

### Example

```
.reed/flow/
├── reedmonitor.log                        # Active (42 MB)
├── reedmonitor.log.20250115-143218.gz    # Archived (8 MB)
├── reedmonitor.log.20250115-102341.gz    # Archived (9 MB)
├── reedmonitor.log.20250114-231045.gz    # Archived (7 MB)
...
└── reedmonitor.log.20250110-154523.gz    # Archived (8 MB, oldest kept)
```

## CLI Commands

### Monitoring

```bash
# View logs
reed monitor:logs
reed monitor:logs --level=ERROR
reed monitor:logs --follow

# View metrics
reed monitor:metrics
reed monitor:metrics --refresh=1s

# Health check
reed monitor:health
```

### Profiling

```bash
# Profile request
reed profile:request /knowledge

# View slow queries
reed profile:slow
reed profile:slow --threshold=200ms

# Generate flame graph
reed profile:flamegraph > profile.txt
flamegraph.pl profile.txt > flame.svg
```

### Debugging

```bash
# Request inspector
reed debug:request /knowledge
reed debug:request /de/wissen

# Cache viewer
reed debug:cache
reed debug:cache text
reed debug:cache --search="knowledge"

# Route tester
reed debug:route /knowledge
reed debug:route /blog/my-post

# Config inspector
reed debug:config
```

## Integration with Other Layers

### Middleware Integration (Layer 06 - Server)

```rust
use actix_web::App;
use reedcms::monitor::middleware::MonitorMiddleware;

App::new()
    .wrap(MonitorMiddleware)  // Records all requests
    .service(...)
```

**Automatic Recording**:
- Request method, path, status
- Response time
- User agent detection
- Error tracking

### ReedBase Integration (Layer 02 - Data)

```rust
use reedcms::monitor::global_monitor;

// In ReedBase lookup
let start = Instant::now();
let result = cache.get(key);
let duration = start.elapsed();

global_monitor().record_reedbase_lookup(
    key,
    duration,
    result.is_some()  // Cache hit
);
```

### Template Integration (Layer 05 - Template)

```rust
use reedcms::monitor::global_monitor;

// In template render
let start = Instant::now();
let html = template.render(context)?;
let duration = start.elapsed();

global_monitor().record_template_render(
    template_name,
    duration
);
```

## Production Deployment

### Environment Configuration

```bash
# Logging
export REED_LOG_MODE=both           # Log + forward to syslog
export REED_LOG_LEVEL=INFO          # INFO and above

# Profiling
export REED_PROFILE=false            # Disabled in production
export REED_SLOW_THRESHOLD=200       # Track queries > 200ms

# Monitoring
export REED_METRICS_ENABLED=true     # Enable metrics collection
export REED_HEALTH_ENDPOINT=/health  # Health check URL
```

### Systemd Integration

```ini
# /etc/systemd/system/reedcms.service

[Unit]
Description=ReedCMS Web Server
After=network.target

[Service]
Type=simple
User=reedcms
WorkingDirectory=/opt/reedcms
Environment="REED_LOG_MODE=forward"
Environment="REED_LOG_LEVEL=INFO"
ExecStart=/opt/reedcms/reedcms server:start
Restart=always
StandardOutput=journal
StandardError=journal
SyslogIdentifier=reedcms

[Install]
WantedBy=multi-user.target
```

### Load Balancer Health Check

```nginx
# nginx.conf

upstream reedcms_backend {
    server 127.0.0.1:3000;
    
    # Health check every 5s
    check interval=5000 rise=2 fall=3 timeout=1000 type=http;
    check_http_send "GET /health HTTP/1.0\r\n\r\n";
    check_http_expect_alive http_2xx http_3xx;
}
```

## Troubleshooting

### High Memory Usage

**Symptom**: Memory usage > 100 MB

**Cause**: Metrics accumulation without trimming

**Solution**: Metrics auto-trim after 10,000 entries

```rust
// Automatically handled
if self.durations.len() > 10000 {
    self.durations.drain(0..5000);  // Keep recent 5000
}
```

### Log File Growing Too Large

**Symptom**: Log file > 100 MB not rotating

**Solution**: Manual rotation trigger

```bash
# Trigger rotation manually
reed monitor:rotate-logs

# Or restart service (rotation on startup)
systemctl restart reedcms
```

### Profiler Overhead

**Symptom**: Requests slower with profiling enabled

**Cause**: Profiler adds ~0.1% overhead

**Solution**: Disable in production

```bash
# Disable profiling
export REED_PROFILE=false

# Or enable only for specific requests
curl -H "X-ReedCMS-Profile: true" http://localhost:3000/knowledge
```

### Missing Metrics

**Symptom**: `/metrics` endpoint returns empty data

**Cause**: Metrics not recorded in middleware

**Solution**: Ensure middleware is registered

```rust
// In server setup
App::new()
    .wrap(MonitorMiddleware)  // Must be here
    .service(...)
```

## Related Documentation

- [Monitoring System](monitoring-system.md) - ReedMonitor and syslog details
- [Performance Profiler](performance-profiler.md) - Profiling and bottleneck detection
- [Debug Tools](debug-tools.md) - Development utilities

## Implementation Files

```
src/reedcms/monitor/
├── syslog.rs              # FreeBSD syslog logger
├── log_manager.rs         # Log rotation and cleanup
├── core.rs                # Monitor core
├── metrics.rs             # Metrics storage
├── middleware.rs          # Actix middleware
└── health.rs              # Health check endpoints

src/reedcms/profiler/
├── core.rs                # Profiler core
├── middleware.rs          # Profiling middleware
├── slow_queries.rs        # Slow query tracker
└── flamegraph.rs          # Flame graph generator

src/reedcms/debug/
├── request_inspector.rs   # Request inspector
├── cache_viewer.rs        # Cache viewer
└── route_tester.rs        # Route tester
```

## Security Considerations

### Log Sensitivity

**Don't Log**:
- Passwords or hashes
- Session tokens
- API keys
- Personal data (GDPR)

**Safe to Log**:
- Request paths (without query params with sensitive data)
- Status codes
- Timing data
- Error messages (sanitised)

### Metrics Endpoint Access

```rust
// Restrict /metrics to localhost only
.service(
    web::resource("/metrics")
        .guard(guard::Host("localhost"))
        .route(web::get().to(metrics_endpoint))
)
```

### Debug Tools in Production

**Recommendation**: Disable debug commands in production

```bash
# Development
export REED_DEBUG_ENABLED=true

# Production
export REED_DEBUG_ENABLED=false
```

## Best Practices

### 1. Log Levels

```
EMERG/ALERT/CRIT: System-critical issues only
ERROR: Errors that need attention
WARN: Potential issues, recoverable
NOTICE: Significant events (startup, shutdown)
INFO: Normal operations (requests, metrics)
DEBUG: Development only, disabled in production
```

### 2. Metric Recording

```rust
// ✓ GOOD: Low overhead
global_monitor().record_request(method, path, status, duration);

// ✗ BAD: Don't record in hot loops
for item in large_collection {
    global_monitor().record_something();  // Too much overhead
}
```

### 3. Profiling

```rust
// ✓ GOOD: Conditional profiling
if cfg!(debug_assertions) || std::env::var("REED_PROFILE").is_ok() {
    let profiler = Profiler::start("operation");
    // ...
}

// ✗ BAD: Always enabled
let profiler = Profiler::start("operation");  // Production overhead
```

## CLI Reference

```bash
# Monitoring
reed monitor:logs [--level=LEVEL] [--follow]
reed monitor:metrics [--refresh=DURATION]
reed monitor:health
reed monitor:rotate-logs

# Profiling
reed profile:request URL
reed profile:slow [--threshold=MS]
reed profile:flamegraph

# Debugging
reed debug:request URL
reed debug:cache [TYPE] [--search=TERM]
reed debug:route URL
reed debug:config
```
