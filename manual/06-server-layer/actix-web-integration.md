# Actix-Web Integration

> High-performance async HTTP server

---

## Overview

ReedCMS uses Actix-Web, a powerful Rust web framework built on the Actor model with async/await support.

---

## Why Actix-Web?

### Comparison

| Feature | Actix-Web | Axum | Rocket |
|---------|-----------|------|--------|
| Performance | ⚡⚡ Fastest | ⚡ Fast | ⚡ Fast |
| Async/Await | ✅ Yes | ✅ Yes | ⚠️ Limited |
| Middleware | ✅ Rich | ✅ Tower | ✅ Yes |
| Stability | ✅ Mature | ⚠️ Young | ✅ Mature |
| Documentation | ✅ Excellent | ✅ Good | ✅ Good |

**ReedCMS choice:** Actix-Web (performance + maturity)

---

## Server Configuration

### Basic Setup

```rust
use actix_web::{App, HttpServer, web};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(index))
    })
    .bind("127.0.0.1:8333")?
    .run()
    .await
}
```

### ReedCMS Setup

```rust
pub async fn start_http_server(port: u16, workers: Option<usize>) -> ReedResult<()> {
    let worker_count = workers.unwrap_or_else(num_cpus::get);
    
    HttpServer::new(|| {
        App::new()
            // Middleware
            .wrap(middleware::Logger::default())
            .wrap(middleware::Compress::default())
            .wrap(SiteProtection::new())
            
            // Routes
            .configure(configure_routes)
    })
    .workers(worker_count)
    .bind(format!("127.0.0.1:{}", port))?
    .run()
    .await
    .map_err(|e| ReedError::ConfigError {
        component: "http_server".to_string(),
        reason: format!("Server error: {}", e),
    })
}
```

---

## Worker Configuration

### Multi-Threading

```rust
.workers(8)  // 8 worker threads
```

**Recommendations:**
- Development: 2-4 workers
- Production: CPU core count
- High load: 2× CPU cores

**Default:** `num_cpus::get()` (automatic)

### Worker Behaviour

Each worker:
- Independent event loop
- Separate thread
- Handles multiple connections simultaneously
- Non-blocking I/O

**Capacity:**
- Single worker: ~1,000 connections
- 8 workers: ~8,000 connections

---

## Middleware Stack

### Logger Middleware

```rust
.wrap(middleware::Logger::default())
```

**Output:**
```
[INFO] 127.0.0.1 "GET /de/wissen HTTP/1.1" 200 15ms
```

**Custom format:**
```rust
.wrap(middleware::Logger::new("%a %r %s %T"))
```

### Compress Middleware

```rust
.wrap(middleware::Compress::default())
```

**Algorithms:**
- Brotli (best compression, modern browsers)
- Gzip (universal compatibility)
- Deflate (fallback)

**Automatic selection** based on `Accept-Encoding` header.

**Performance:**
- Brotli: ~30% better than gzip
- CPU cost: ~2ms overhead
- Network savings: 70-80% size reduction

### Custom Middleware

```rust
use actix_web::{
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    Error,
};

pub struct SiteProtection {
    password: Option<String>,
}

impl<S, B> Transform<S, ServiceRequest> for SiteProtection
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
{
    // Implementation
}
```

---

## Route Configuration

### Simple Routes

```rust
fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/")
            .route(web::get().to(index))
    );
    
    cfg.service(
        web::resource("/about")
            .route(web::get().to(about))
    );
}
```

### Catch-All Routes

```rust
cfg.service(
    web::resource("/{path:.*}")
        .route(web::get().to(handle_request))
);
```

**Matches:**
- `/de/wissen`
- `/en/knowledge`
- `/de/blog/post`
- Any URL structure

### Static Files

```rust
cfg.service(
    Files::new("/public", "public/")
        .show_files_listing()
);
```

---

## Request Handling

### Handler Function

```rust
async fn handle_request(
    req: HttpRequest,
    path: web::Path<String>
) -> Result<HttpResponse, Error> {
    // Extract data from request
    let user_agent = req.headers()
        .get("User-Agent")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    
    // Process request
    let html = generate_html(&path)?;
    
    // Return response
    Ok(HttpResponse::Ok()
        .content_type("text/html")
        .body(html))
}
```

### Extractors

```rust
// Path parameters
async fn handler(path: web::Path<(String, u32)>) -> HttpResponse {
    let (name, id) = path.into_inner();
    // ...
}

// Query parameters
async fn handler(query: web::Query<HashMap<String, String>>) -> HttpResponse {
    let value = query.get("key");
    // ...
}

// JSON body
async fn handler(json: web::Json<MyStruct>) -> HttpResponse {
    let data = json.into_inner();
    // ...
}

// Request object
async fn handler(req: HttpRequest) -> HttpResponse {
    let headers = req.headers();
    // ...
}
```

---

## Response Building

### Basic Response

```rust
HttpResponse::Ok()
    .content_type("text/html")
    .body(html)
```

### With Headers

```rust
HttpResponse::Ok()
    .content_type("text/html; charset=utf-8")
    .insert_header(("Cache-Control", "public, max-age=3600"))
    .insert_header(("X-Custom-Header", "value"))
    .body(html)
```

### Redirects

```rust
// 301 Permanent
HttpResponse::MovedPermanently()
    .insert_header(("Location", "/new-url"))
    .finish()

// 302 Temporary
HttpResponse::Found()
    .insert_header(("Location", "/temp-url"))
    .finish()
```

### Status Codes

```rust
HttpResponse::Ok()              // 200
HttpResponse::Created()         // 201
HttpResponse::NoContent()       // 204
HttpResponse::MovedPermanently() // 301
HttpResponse::Found()           // 302
HttpResponse::BadRequest()      // 400
HttpResponse::Unauthorized()    // 401
HttpResponse::NotFound()        // 404
HttpResponse::InternalServerError() // 500
```

---

## Error Handling

### Custom Error Type

```rust
impl actix_web::ResponseError for ReedError {
    fn error_response(&self) -> HttpResponse {
        match self {
            ReedError::NotFound { .. } => {
                HttpResponse::NotFound().json(json!({
                    "error": "Not found",
                    "details": self.to_string()
                }))
            }
            ReedError::AuthenticationFailed { .. } => {
                HttpResponse::Unauthorized().json(json!({
                    "error": "Authentication failed"
                }))
            }
            _ => {
                HttpResponse::InternalServerError().json(json!({
                    "error": "Internal server error"
                }))
            }
        }
    }
}
```

### Error Propagation

```rust
async fn handler() -> Result<HttpResponse, ReedError> {
    let data = load_data()?;  // Propagates ReedError
    let html = render(data)?;
    
    Ok(HttpResponse::Ok().body(html))
}
```

---

## Performance Tuning

### Keep-Alive

```rust
.keep_alive(Duration::from_secs(75))
```

**Benefits:**
- Reuse TCP connections
- Reduce handshake overhead
- Better performance for multiple requests

### Backlog

```rust
.backlog(8192)
```

**Purpose:** Maximum pending connections queue

### Client Timeout

```rust
.client_request_timeout(Duration::from_secs(30))
```

**Default:** 5 seconds

---

## Production Deployment

### Systemd Service

```ini
[Unit]
Description=ReedCMS Server
After=network.target

[Service]
Type=simple
User=www-data
WorkingDirectory=/var/www/reedcms
Environment="ENVIRONMENT=prod"
ExecStart=/usr/local/bin/reed server:io --port 8333
Restart=always

[Install]
WantedBy=multi-user.target
```

### Nginx Reverse Proxy

```nginx
upstream reedcms {
    server 127.0.0.1:8333;
}

server {
    listen 80;
    server_name example.com;
    
    location / {
        proxy_pass http://reedcms;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

---

## Troubleshooting

**Address already in use:**
```bash
# Find process
lsof -i :8333

# Kill process
kill $(lsof -t -i:8333)
```

**Permission denied:**
```bash
# Ports < 1024 require root
sudo reed server:io --port 80

# Or use port > 1024
reed server:io --port 8333
```

**Worker crashes:**
```bash
# Check logs
reed server:logs

# Reduce workers
reed server:io --workers 2
```

---

**See also:**
- [Request Handling](request-handling.md) - Request lifecycle
- [Routing](routing.md) - URL resolution
- [CLI Server Commands](../04-cli-layer/server-commands.md) - Server control
