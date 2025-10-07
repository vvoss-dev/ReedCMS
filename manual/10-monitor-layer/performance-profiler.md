# Performance Profiler

Detailed request timing analysis with bottleneck detection and flame graphs.

## Purpose

- **Span Tracking**: Hierarchical timing measurement
- **Bottleneck Detection**: Identify operations > 25% total time
- **Slow Query Tracking**: Log operations exceeding threshold (default 100ms)
- **Flame Graphs**: Visual performance analysis
- **Zero-Cost**: Disabled in production (no overhead)

## Usage

```rust
let profiler = Profiler::start("handle_request");

let _routing = profiler.span("routing");
// routing logic
drop(_routing);

let _render = profiler.span("template_render");
// rendering logic
drop(_render);

let report = profiler.finish();
println!("{}", report.format());
```

## Output Example

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

## CLI Commands

```bash
# Profile specific request
reed profile:request /knowledge

# View slow operations
reed profile:slow
reed profile:slow --threshold=200ms

# Generate flame graph data
reed profile:flamegraph > profile.txt
```

## Performance

- Span start/end: < 1μs
- Report generation: < 100μs
- Overhead when enabled: < 0.1%
- Overhead when disabled: 0%

## See README.md for complete implementation details.
