# REED-06-01: Server Foundation

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
- **ID**: REED-06-01
- **Title**: Actix-Web HTTP Server Foundation
- **Layer**: Server Layer (REED-06)
- **Priority**: Critical
- **Status**: Open
- **Complexity**: High
- **Dependencies**: REED-01-01

## Summary Reference
- **Section**: Server Services
- **Lines**: 964-966, 1423-1444 in project_summary.md
- **Key Concepts**: Actix-Web HTTP server, Unix socket support, graceful shutdown

## Objective
Implement Actix-Web HTTP server foundation with Unix socket support, request handling pipeline, graceful shutdown, and PID tracking for production deployment.

## Requirements

### Unix Socket Configuration (.reed/server.csv)
```csv
service|bind_type|bind_address|socket_path|permissions|user|group
api|unix|/var/run/reedcms/api.sock||660|reedcms|www-data
web|unix|/var/run/reedcms/web.sock||660|reedcms|www-data
```

### Implementation Files

#### HTTP Server (`src/reedcms/server/http_server.rs`)

```rust
/// Starts HTTP server on specified port.
///
/// ## Arguments
/// - port: Port number (default: 8333)
/// - workers: Number of worker threads (default: num_cpus)
///
/// ## Process
/// 1. Initialize Actix-Web App
/// 2. Configure middleware
/// 3. Register routes
/// 4. Bind to address
/// 5. Start server
///
/// ## Performance
/// - Startup time: < 500ms
/// - Request handling: < 10ms average
/// - Concurrent connections: 10k+
///
/// ## Output
/// ```
/// Starting HTTP server on 127.0.0.1:8333
/// Worker threads: 4
/// Server started successfully
/// ```
pub async fn start_http_server(port: u16, workers: Option<usize>) -> ReedResult<()> {
    let worker_count = workers.unwrap_or_else(num_cpus::get);

    println!("🚀 Starting ReedCMS HTTP server...");
    println!("   Port: {}", port);
    println!("   Workers: {}", worker_count);

    let server = HttpServer::new(|| {
        App::new()
            .wrap(middleware::Logger::default())
            .wrap(middleware::Compress::default())
            .configure(configure_routes)
    })
    .workers(worker_count)
    .bind(format!("127.0.0.1:{}", port))?
    .run();

    println!("✓ Server started successfully");
    println!("  Access at: http://127.0.0.1:{}", port);

    server.await.map_err(|e| ReedError::ServerError {
        component: "http_server".to_string(),
        reason: format!("Server error: {}", e),
    })
}

/// Configures application routes.
fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/{path:.*}")
            .route(web::get().to(handle_request))
    );
}

/// Main request handler.
async fn handle_request(req: HttpRequest, path: web::Path<String>) -> Result<HttpResponse, Error> {
    // TODO: Implement request routing
    Ok(HttpResponse::Ok().body("ReedCMS"))
}
```

#### Unix Socket Server (`src/reedcms/server/socket_server.rs`)

```rust
/// Starts Unix socket server.
///
/// ## Arguments
/// - socket_path: Path to Unix socket file
/// - workers: Number of worker threads
///
/// ## Process
/// 1. Create socket directory
/// 2. Remove existing socket file
/// 3. Bind to socket path
/// 4. Set permissions (0o666)
/// 5. Start server
///
/// ## Socket Permissions
/// - Default: 0o666 (rw-rw-rw-)
/// - Allows nginx/apache access
/// - Can be customized via server.csv
///
/// ## Output
/// ```
/// Starting Unix socket server
/// Socket path: /var/run/reedcms/web.sock
/// Permissions: 0o666
/// Server started successfully
/// ```
pub async fn start_socket_server(socket_path: &str, workers: Option<usize>) -> ReedResult<()> {
    let worker_count = workers.unwrap_or_else(num_cpus::get);

    println!("🚀 Starting ReedCMS Unix socket server...");
    println!("   Socket: {}", socket_path);
    println!("   Workers: {}", worker_count);

    // Create socket directory if not exists
    if let Some(parent) = std::path::Path::new(socket_path).parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Remove existing socket file
    let _ = std::fs::remove_file(socket_path);

    let server = HttpServer::new(|| {
        App::new()
            .wrap(middleware::Logger::default())
            .wrap(middleware::Compress::default())
            .configure(configure_routes)
    })
    .workers(worker_count)
    .bind_uds(socket_path)?
    .run();

    // Set socket permissions
    set_socket_permissions(socket_path, 0o666)?;

    println!("✓ Server started successfully");
    println!("  Socket: {}", socket_path);

    server.await.map_err(|e| ReedError::ServerError {
        component: "socket_server".to_string(),
        reason: format!("Server error: {}", e),
    })
}

/// Sets Unix socket file permissions.
fn set_socket_permissions(path: &str, mode: u32) -> ReedResult<()> {
    use std::os::unix::fs::PermissionsExt;

    let permissions = std::fs::Permissions::from_mode(mode);
    std::fs::set_permissions(path, permissions).map_err(|e| ReedError::IoError {
        operation: "set_permissions".to_string(),
        path: path.to_string(),
        reason: e.to_string(),
    })
}
```

#### Server Configuration (`src/reedcms/server/config.rs`)

```rust
/// Loads server configuration from .reed/server.csv.
///
/// ## Configuration Fields
/// - service: Service name (api, web)
/// - bind_type: Binding type (http, unix)
/// - bind_address: IP:Port for HTTP
/// - socket_path: Socket path for Unix
/// - permissions: Socket permissions (octal)
/// - user: Unix user owner
/// - group: Unix group owner
pub fn load_server_config() -> ReedResult<ServerConfig> {
    let config_path = ".reed/server.csv";

    if !std::path::Path::new(config_path).exists() {
        return Ok(ServerConfig::default());
    }

    // Parse CSV configuration
    let entries = read_server_csv(config_path)?;

    Ok(ServerConfig {
        bind_type: entries.get("bind_type").cloned(),
        bind_address: entries.get("bind_address").cloned(),
        socket_path: entries.get("socket_path").cloned(),
        workers: entries.get("workers").and_then(|s| s.parse().ok()),
    })
}

#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub bind_type: Option<String>,
    pub bind_address: Option<String>,
    pub socket_path: Option<String>,
    pub workers: Option<usize>,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            bind_type: Some("http".to_string()),
            bind_address: Some("127.0.0.1:8333".to_string()),
            socket_path: None,
            workers: None,
        }
    }
}
```

#### Graceful Shutdown (`src/reedcms/server/shutdown.rs`)

```rust
/// Handles graceful server shutdown.
///
/// ## Process
/// 1. Capture SIGTERM/SIGINT signals
/// 2. Stop accepting new connections
/// 3. Wait for active requests to complete (max 30s)
/// 4. Shutdown server
/// 5. Cleanup resources
///
/// ## Timeout
/// - Default: 30 seconds
/// - Configurable via server.csv
pub async fn graceful_shutdown(server_handle: ServerHandle) -> ReedResult<()> {
    use tokio::signal;

    // Wait for shutdown signal
    let ctrl_c = signal::ctrl_c();
    tokio::select! {
        _ = ctrl_c => {
            println!("📉 Shutdown signal received");
        }
    }

    println!("🛑 Stopping server...");
    println!("   Waiting for active requests to complete...");

    // Graceful shutdown with timeout
    tokio::time::timeout(
        std::time::Duration::from_secs(30),
        server_handle.stop(true)
    )
    .await
    .map_err(|_| ReedError::ServerError {
        component: "shutdown".to_string(),
        reason: "Graceful shutdown timeout exceeded".to_string(),
    })??;

    println!("✓ Server stopped gracefully");

    Ok(())
}
```

## Implementation Files

### Primary Implementation
- `src/reedcms/server/http_server.rs` - HTTP server
- `src/reedcms/server/socket_server.rs` - Unix socket server
- `src/reedcms/server/config.rs` - Configuration
- `src/reedcms/server/shutdown.rs` - Graceful shutdown

### Test Files
- `src/reedcms/server/http_server.test.rs`
- `src/reedcms/server/socket_server.test.rs`
- `src/reedcms/server/config.test.rs`
- `src/reedcms/server/shutdown.test.rs`

## Testing Requirements

### Unit Tests
- [ ] Test server configuration loading
- [ ] Test socket permission setting
- [ ] Test graceful shutdown logic
- [ ] Test route configuration

### Integration Tests
- [ ] Test HTTP server startup and requests
- [ ] Test Unix socket server with test client
- [ ] Test graceful shutdown with active connections
- [ ] Test server restart

### Performance Tests
- [ ] Server startup: < 500ms
- [ ] Request handling: < 10ms average
- [ ] Concurrent connections: 10k+ supported
- [ ] Graceful shutdown: < 5s

## Acceptance Criteria
- [ ] HTTP server starts on specified port
- [ ] Unix socket server works with nginx
- [ ] Socket permissions correctly set (0o666)
- [ ] Graceful shutdown with active request completion
- [ ] PID tracking functional
- [ ] Worker thread configuration working
- [ ] All tests pass with 100% coverage
- [ ] Documentation complete
- [ ] BBC English throughout

## Dependencies
- **Requires**: REED-01-01 (ReedStream)

## Blocks
- REED-06-02 (Routing needs server foundation)
- REED-06-03 (Authentication needs server)
- REED-06-04 (Response builder needs server)

## References
- Service Template: `_workbench/Tickets/templates/service-template.md`
- Service Test Template: `_workbench/Tickets/templates/service-template.test.md`
- Summary: Lines 964-966, 1423-1444 in `project_summary.md`

## Notes
The server foundation is critical for production deployment. Unix socket support enables integration with nginx/apache reverse proxies for better performance and security. Graceful shutdown ensures no requests are lost during server restarts. Socket permissions (0o666) allow web server processes to communicate with ReedCMS.