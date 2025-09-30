# REED-04-09: CLI Server Commands

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
- **ID**: REED-04-09
- **Title**: CLI Server Management Commands
- **Layer**: CLI Layer (REED-04)
- **Priority**: High
- **Status**: Open
- **Complexity**: Medium
- **Dependencies**: REED-04-01

## Summary Reference
- **Section**: CLI Server Management
- **Lines**: 1068-1076 in project_summary.md
- **Key Concepts**: Server lifecycle, HTTP/Unix socket modes, process management

## Objective
Implement complete server lifecycle management commands including start, stop, restart, status checking, and log viewing with support for both HTTP and Unix socket modes.

## Requirements

### Commands to Implement

```bash
# Start server modes
reed server:io                              # Start with default config
reed server:io --port 8333                  # HTTP mode with custom port
reed server:io --socket "/var/run/reedcms/web.sock"  # Unix socket mode

# Server lifecycle
reed server:start                           # Start in background
reed server:start --environment DEV         # Start with environment
reed server:stop                            # Stop running server
reed server:restart                         # Restart server
reed server:status                          # Check server status
reed server:logs                            # View server logs
reed server:logs --tail 50                  # Tail last 50 lines
reed server:logs --follow                   # Follow logs in real-time
```

### Implementation (`src/reedcms/cli/server_commands.rs`)

```rust
/// Starts ReedCMS server in interactive mode.
///
/// ## Flags
/// - --port: HTTP port (default: 8333)
/// - --socket: Unix socket path
/// - --workers: Number of worker threads
///
/// ## Modes
/// - HTTP mode: --port 8333
/// - Unix socket mode: --socket "/var/run/reedcms/web.sock"
///
/// ## Process
/// 1. Validate configuration
/// 2. Initialize ReedBase
/// 3. Load templates
/// 4. Start server
/// 5. Display access information
///
/// ## Output
/// 🚀 Starting ReedCMS server...
/// ✓ Configuration validated
/// ✓ ReedBase initialized (17 layouts, 1,250 text entries)
/// ✓ Templates loaded (51 files)
/// ✓ Server started successfully
///
/// Server information:
/// - Mode: HTTP
/// - Address: http://127.0.0.1:8333
/// - Workers: 4
/// - Environment: DEV
///
/// Press Ctrl+C to stop server...
pub fn server_io(flags: &HashMap<String, String>) -> ReedResult<ReedResponse<String>>

/// Starts server in background (daemon mode).
///
/// ## Flags
/// - --environment: Environment name (DEV, PROD)
/// - --config: Custom config file path
///
/// ## Process
/// 1. Check if server already running
/// 2. Validate configuration
/// 3. Fork to background
/// 4. Write PID file
/// 5. Start server process
///
/// ## Output
/// 🚀 Starting ReedCMS server in background...
/// ✓ Configuration validated
/// ✓ Server started (PID: 12345)
/// ✓ PID file: .reed/server.pid
/// ✓ Log file: .reed/flow/server.log
///
/// Server is now running in background.
/// Use 'reed server:status' to check status.
/// Use 'reed server:stop' to stop server.
pub fn server_start(flags: &HashMap<String, String>) -> ReedResult<ReedResponse<String>>

/// Stops running server.
///
/// ## Process
/// 1. Read PID from PID file
/// 2. Check if process exists
/// 3. Send SIGTERM signal
/// 4. Wait for graceful shutdown (max 30s)
/// 5. Force kill if necessary (SIGKILL)
/// 6. Remove PID file
///
/// ## Output
/// 🛑 Stopping ReedCMS server...
/// ✓ Found running server (PID: 12345)
/// ✓ Sent shutdown signal
/// ✓ Server stopped gracefully
/// ✓ PID file removed
///
/// Server stopped successfully.
pub fn server_stop() -> ReedResult<ReedResponse<String>>

/// Restarts server (stop + start).
///
/// ## Process
/// 1. Stop current server
/// 2. Wait for complete shutdown
/// 3. Start new server instance
///
/// ## Output
/// 🔄 Restarting ReedCMS server...
/// ✓ Stopping current server...
/// ✓ Server stopped (PID: 12345)
/// ✓ Starting new server...
/// ✓ Server started (PID: 12389)
///
/// Server restarted successfully.
pub fn server_restart() -> ReedResult<ReedResponse<String>>

/// Shows server status.
///
/// ## Checks
/// 1. PID file existence
/// 2. Process running
/// 3. Server responsiveness
/// 4. Resource usage
///
/// ## Output (running)
/// 🟢 ReedCMS Server: RUNNING
///
/// Process information:
/// - PID: 12345
/// - Uptime: 2h 15m 30s
/// - Started: 2024-01-20 10:00:00
///
/// Server information:
/// - Mode: HTTP
/// - Address: http://127.0.0.1:8333
/// - Environment: PROD
///
/// Resource usage:
/// - Memory: 125 MB
/// - CPU: 2.3%
/// - Requests/sec: 45
///
/// Health check: ✓ Healthy
///
/// ## Output (not running)
/// 🔴 ReedCMS Server: NOT RUNNING
/// No PID file found or process not responding.
pub fn server_status() -> ReedResult<ReedResponse<String>>

/// Views server logs.
///
/// ## Flags
/// - --tail: Number of lines to show (default: 100)
/// - --follow: Follow logs in real-time
/// - --level: Filter by log level (info, warn, error)
///
/// ## Output
/// 📋 Server logs (.reed/flow/server.log):
///
/// [2024-01-20 10:00:00] INFO: Server started on port 8333
/// [2024-01-20 10:00:05] INFO: GET /knowledge 200 (45ms)
/// [2024-01-20 10:00:12] INFO: GET /blog 200 (32ms)
/// [2024-01-20 10:00:20] WARN: Slow query detected: get_text (150ms)
/// [2024-01-20 10:00:25] ERROR: Template not found: missing.jinja
/// [2024-01-20 10:00:30] INFO: GET /about 200 (28ms)
///
/// Showing last 100 lines. Use --follow to tail logs.
pub fn server_logs(flags: &HashMap<String, String>) -> ReedResult<ReedResponse<String>>
```

### Server Management (`src/reedcms/cli/server_management.rs`)

```rust
/// Checks if server is running.
pub fn is_server_running() -> ReedResult<bool>

/// Reads PID from PID file.
pub fn read_pid() -> ReedResult<u32>

/// Writes PID to PID file.
pub fn write_pid(pid: u32) -> ReedResult<()>

/// Removes PID file.
pub fn remove_pid_file() -> ReedResult<()>

/// Checks if process with PID exists.
pub fn process_exists(pid: u32) -> bool

/// Sends signal to process.
pub fn send_signal(pid: u32, signal: Signal) -> ReedResult<()>

/// Waits for process to terminate.
pub fn wait_for_termination(pid: u32, timeout_secs: u64) -> ReedResult<bool>

/// Gets process resource usage.
pub fn get_process_stats(pid: u32) -> ReedResult<ProcessStats>

#[derive(Debug, Clone)]
pub struct ProcessStats {
    pub memory_mb: f32,
    pub cpu_percent: f32,
    pub uptime_seconds: u64,
}

#[derive(Debug, Clone, Copy)]
pub enum Signal {
    Term,
    Kill,
    Hup,
}
```

## Implementation Files

### Primary Implementation
- `src/reedcms/cli/server_commands.rs` - Server commands
- `src/reedcms/cli/server_management.rs` - Process management

### Test Files
- `src/reedcms/cli/server_commands.test.rs`
- `src/reedcms/cli/server_management.test.rs`

## Testing Requirements

### Unit Tests
- [ ] Test PID file operations
- [ ] Test process existence checks
- [ ] Test signal sending
- [ ] Test configuration validation
- [ ] Test log parsing

### Integration Tests
- [ ] Test server start/stop lifecycle
- [ ] Test restart operation
- [ ] Test status checking
- [ ] Test HTTP and socket modes
- [ ] Test graceful shutdown

### Edge Case Tests
- [ ] Test start when already running
- [ ] Test stop when not running
- [ ] Test orphaned PID file
- [ ] Test forced kill scenario
- [ ] Test log rotation

### Performance Tests
- [ ] Server startup: < 500ms
- [ ] Graceful shutdown: < 5s
- [ ] Status check: < 100ms
- [ ] Log display: < 200ms

## Acceptance Criteria
- [ ] HTTP mode working (--port)
- [ ] Unix socket mode working (--socket)
- [ ] Background daemon mode functional
- [ ] Graceful shutdown implemented
- [ ] PID file management working
- [ ] Status command comprehensive
- [ ] Log viewing functional (tail, follow)
- [ ] All tests pass with 100% coverage
- [ ] Documentation complete
- [ ] BBC English throughout

## Dependencies
- **Requires**: REED-04-01 (CLI Foundation)

## Blocks
- REED-06-01 (Server Foundation needs these commands)

## References
- Service Template: `_workbench/Tickets/templates/service-template.md`
- Service Test Template: `_workbench/Tickets/templates/service-template.test.md`
- Summary: Lines 1068-1076 in `project_summary.md`

## Notes
Server management commands are critical for production operations. Graceful shutdown ensures active requests complete before server stops. PID file management prevents multiple server instances. Log viewing commands enable real-time debugging without external tools. Unix socket mode enables integration with nginx/apache reverse proxies.