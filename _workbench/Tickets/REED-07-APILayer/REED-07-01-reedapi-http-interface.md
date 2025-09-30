# REED-07-01: ReedAPI HTTP Interface

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
- **ID**: REED-07-01
- **Title**: ReedAPI HTTP Interface
- **Layer**: API Layer (REED-07)
- **Priority**: High
- **Status**: Open
- **Complexity**: High
- **Dependencies**: REED-06-01, REED-02-01, REED-06-03

## Summary Reference
- **Section**: ReedAPI HTTP Interface
- **Lines**: 1016-1023, 1468-1504 in project_summary.md
- **Key Concepts**: RESTful API, JSON responses, batch operations, ReedBase access via HTTP

## Objective
Implement RESTful HTTP API for ReedBase data operations enabling external applications to interact with text, routes, meta, config, and system data via JSON endpoints with authentication and permission-based access control.

## Requirements

### API Endpoints

#### Base URL
```
/api/v1
```

#### Authentication
All endpoints require authentication via:
- HTTP Basic Auth
- Bearer Token

#### Endpoints Overview

**Get Operations**
- `GET /api/v1/text/{key}?lang={lang}&env={env}` - Get text value
- `GET /api/v1/route/{key}?lang={lang}` - Get route URL
- `GET /api/v1/meta/{key}` - Get meta value
- `GET /api/v1/config/{type}/{key}` - Get config value (type: project/server)

**Set Operations**
- `POST /api/v1/text` - Set text value
- `POST /api/v1/route` - Set route value
- `POST /api/v1/meta` - Set meta value
- `POST /api/v1/config/{type}` - Set config value

**Batch Operations**
- `POST /api/v1/batch/get` - Batch get multiple keys
- `POST /api/v1/batch/set` - Batch set multiple values

**List Operations**
- `GET /api/v1/list/text?pattern={pattern}` - List text keys
- `GET /api/v1/list/routes` - List all routes
- `GET /api/v1/list/layouts` - List all layouts

### Implementation (`src/reedcms/api/routes.rs`)

```rust
/// Configures API routes for Actix-Web.
///
/// ## Route Structure
/// - /api/v1/text/* - Text operations
/// - /api/v1/route/* - Route operations
/// - /api/v1/meta/* - Meta operations
/// - /api/v1/config/* - Config operations
/// - /api/v1/batch/* - Batch operations
/// - /api/v1/list/* - List operations
///
/// ## Authentication
/// All routes protected by AuthMiddleware
///
/// ## Permissions
/// - GET: Requires 'read' permission
/// - POST: Requires 'write' permission
pub fn configure_api_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1")
            .wrap(AuthMiddleware::authenticated())
            .service(configure_text_routes())
            .service(configure_route_routes())
            .service(configure_meta_routes())
            .service(configure_config_routes())
            .service(configure_batch_routes())
            .service(configure_list_routes())
    );
}

/// Configures text operation routes.
fn configure_text_routes() -> impl HttpServiceFactory {
    web::scope("/text")
        .route("/{key}", web::get().to(get_text_handler))
        .route("", web::post().to(set_text_handler))
}

/// Configures route operation routes.
fn configure_route_routes() -> impl HttpServiceFactory {
    web::scope("/route")
        .route("/{key}", web::get().to(get_route_handler))
        .route("", web::post().to(set_route_handler))
}

/// Configures meta operation routes.
fn configure_meta_routes() -> impl HttpServiceFactory {
    web::scope("/meta")
        .route("/{key}", web::get().to(get_meta_handler))
        .route("", web::post().to(set_meta_handler))
}

/// Configures config operation routes.
fn configure_config_routes() -> impl HttpServiceFactory {
    web::scope("/config")
        .route("/{type}/{key}", web::get().to(get_config_handler))
        .route("/{type}", web::post().to(set_config_handler))
}

/// Configures batch operation routes.
fn configure_batch_routes() -> impl HttpServiceFactory {
    web::scope("/batch")
        .route("/get", web::post().to(batch_get_handler))
        .route("/set", web::post().to(batch_set_handler))
}

/// Configures list operation routes.
fn configure_list_routes() -> impl HttpServiceFactory {
    web::scope("/list")
        .route("/text", web::get().to(list_text_handler))
        .route("/routes", web::get().to(list_routes_handler))
        .route("/layouts", web::get().to(list_layouts_handler))
}
```

### GET Handlers (`src/reedcms/api/get_handlers.rs`)

```rust
/// GET /api/v1/text/{key}
///
/// ## Query Parameters
/// - lang: Language code (optional)
/// - env: Environment (optional)
///
/// ## Response
/// ```json
/// {
///   "success": true,
///   "data": "Knowledge Base",
///   "key": "knowledge.title",
///   "language": "en",
///   "environment": "prod"
/// }
/// ```
///
/// ## Errors
/// - 404: Key not found
/// - 403: Permission denied
pub async fn get_text_handler(
    path: web::Path<String>,
    query: web::Query<HashMap<String, String>>,
    user: web::ReqData<AuthenticatedUser>,
) -> Result<HttpResponse, Error> {
    // Check read permission
    if !user.has_permission("text.read") {
        return Ok(HttpResponse::Forbidden().json(ApiError {
            success: false,
            error: "Permission denied".to_string(),
        }));
    }

    let key = path.into_inner();
    let language = query.get("lang").cloned();
    let environment = query.get("env").cloned();

    let req = ReedRequest {
        key: key.clone(),
        language,
        environment,
        context: None,
    };

    match reedbase::get::text(&req) {
        Ok(response) => Ok(HttpResponse::Ok().json(ApiResponse {
            success: true,
            data: response.data,
            key: Some(key),
            language: response.language,
            environment: response.environment,
        })),
        Err(_) => Ok(HttpResponse::NotFound().json(ApiError {
            success: false,
            error: format!("Text not found: {}", key),
        })),
    }
}

/// GET /api/v1/route/{key}
///
/// ## Query Parameters
/// - lang: Language code (optional)
///
/// ## Response
/// ```json
/// {
///   "success": true,
///   "data": "/knowledge",
///   "key": "knowledge",
///   "language": "en"
/// }
/// ```
pub async fn get_route_handler(
    path: web::Path<String>,
    query: web::Query<HashMap<String, String>>,
    user: web::ReqData<AuthenticatedUser>,
) -> Result<HttpResponse, Error> {
    if !user.has_permission("route.read") {
        return Ok(HttpResponse::Forbidden().json(ApiError {
            success: false,
            error: "Permission denied".to_string(),
        }));
    }

    let key = path.into_inner();
    let language = query.get("lang").cloned();

    let req = ReedRequest {
        key: key.clone(),
        language,
        environment: None,
        context: None,
    };

    match reedbase::get::route(&req) {
        Ok(response) => Ok(HttpResponse::Ok().json(ApiResponse {
            success: true,
            data: response.data,
            key: Some(key),
            language: response.language,
            environment: None,
        })),
        Err(_) => Ok(HttpResponse::NotFound().json(ApiError {
            success: false,
            error: format!("Route not found: {}", key),
        })),
    }
}

/// GET /api/v1/meta/{key}
///
/// ## Response
/// ```json
/// {
///   "success": true,
///   "data": "3600",
///   "key": "knowledge.cache.ttl"
/// }
/// ```
pub async fn get_meta_handler(
    path: web::Path<String>,
    user: web::ReqData<AuthenticatedUser>,
) -> Result<HttpResponse, Error> {
    if !user.has_permission("meta.read") {
        return Ok(HttpResponse::Forbidden().json(ApiError {
            success: false,
            error: "Permission denied".to_string(),
        }));
    }

    let key = path.into_inner();

    let req = ReedRequest {
        key: key.clone(),
        language: None,
        environment: None,
        context: None,
    };

    match reedbase::get::meta(&req) {
        Ok(response) => Ok(HttpResponse::Ok().json(ApiResponse {
            success: true,
            data: response.data,
            key: Some(key),
            language: None,
            environment: None,
        })),
        Err(_) => Ok(HttpResponse::NotFound().json(ApiError {
            success: false,
            error: format!("Meta not found: {}", key),
        })),
    }
}

/// GET /api/v1/config/{type}/{key}
///
/// ## Path Parameters
/// - type: "project" or "server"
/// - key: Config key
///
/// ## Response
/// ```json
/// {
///   "success": true,
///   "data": "en,de,fr",
///   "key": "languages",
///   "type": "project"
/// }
/// ```
pub async fn get_config_handler(
    path: web::Path<(String, String)>,
    user: web::ReqData<AuthenticatedUser>,
) -> Result<HttpResponse, Error> {
    if !user.has_permission("config.read") {
        return Ok(HttpResponse::Forbidden().json(ApiError {
            success: false,
            error: "Permission denied".to_string(),
        }));
    }

    let (config_type, key) = path.into_inner();
    let full_key = format!("{}.{}", config_type, key);

    let req = ReedRequest {
        key: full_key.clone(),
        language: None,
        environment: None,
        context: None,
    };

    let result = match config_type.as_str() {
        "project" => reedbase::get::project(&req),
        "server" => reedbase::get::server(&req),
        _ => {
            return Ok(HttpResponse::BadRequest().json(ApiError {
                success: false,
                error: "Invalid config type (use 'project' or 'server')".to_string(),
            }))
        }
    };

    match result {
        Ok(response) => Ok(HttpResponse::Ok().json(ApiConfigResponse {
            success: true,
            data: response.data,
            key: Some(key),
            config_type: Some(config_type),
        })),
        Err(_) => Ok(HttpResponse::NotFound().json(ApiError {
            success: false,
            error: format!("Config not found: {}", full_key),
        })),
    }
}
```

### SET Handlers (`src/reedcms/api/set_handlers.rs`)

```rust
/// POST /api/v1/text
///
/// ## Request Body
/// ```json
/// {
///   "key": "knowledge.title",
///   "value": "Knowledge Base",
///   "language": "en",
///   "description": "Page title"
/// }
/// ```
///
/// ## Response
/// ```json
/// {
///   "success": true,
///   "message": "Text set successfully"
/// }
/// ```
pub async fn set_text_handler(
    body: web::Json<SetTextRequest>,
    user: web::ReqData<AuthenticatedUser>,
) -> Result<HttpResponse, Error> {
    if !user.has_permission("text.write") {
        return Ok(HttpResponse::Forbidden().json(ApiError {
            success: false,
            error: "Permission denied".to_string(),
        }));
    }

    let req = ReedRequest {
        key: body.key.clone(),
        language: Some(body.language.clone()),
        environment: body.environment.clone(),
        context: Some(body.value.clone()),
    };

    match reedbase::set::text(&req, &body.description) {
        Ok(_) => Ok(HttpResponse::Ok().json(ApiSuccess {
            success: true,
            message: "Text set successfully".to_string(),
        })),
        Err(e) => Ok(HttpResponse::InternalServerError().json(ApiError {
            success: false,
            error: format!("Failed to set text: {}", e),
        })),
    }
}

/// POST /api/v1/route
///
/// ## Request Body
/// ```json
/// {
///   "key": "knowledge",
///   "route": "wissen",
///   "language": "de",
///   "layout": "knowledge"
/// }
/// ```
pub async fn set_route_handler(
    body: web::Json<SetRouteRequest>,
    user: web::ReqData<AuthenticatedUser>,
) -> Result<HttpResponse, Error> {
    if !user.has_permission("route.write") {
        return Ok(HttpResponse::Forbidden().json(ApiError {
            success: false,
            error: "Permission denied".to_string(),
        }));
    }

    let req = ReedRequest {
        key: body.key.clone(),
        language: Some(body.language.clone()),
        environment: None,
        context: Some(format!("{}:{}", body.layout, body.language)),
    };

    match reedbase::set::route(&req, &body.route) {
        Ok(_) => Ok(HttpResponse::Ok().json(ApiSuccess {
            success: true,
            message: "Route set successfully".to_string(),
        })),
        Err(e) => Ok(HttpResponse::InternalServerError().json(ApiError {
            success: false,
            error: format!("Failed to set route: {}", e),
        })),
    }
}

/// Request structure for setting text.
#[derive(Debug, Deserialize)]
pub struct SetTextRequest {
    pub key: String,
    pub value: String,
    pub language: String,
    pub environment: Option<String>,
    pub description: String,
}

/// Request structure for setting route.
#[derive(Debug, Deserialize)]
pub struct SetRouteRequest {
    pub key: String,
    pub route: String,
    pub language: String,
    pub layout: String,
}
```

### Batch Operations (`src/reedcms/api/batch_handlers.rs`)

```rust
/// POST /api/v1/batch/get
///
/// ## Request Body
/// ```json
/// {
///   "requests": [
///     {
///       "type": "text",
///       "key": "knowledge.title",
///       "language": "en"
///     },
///     {
///       "type": "route",
///       "key": "knowledge",
///       "language": "de"
///     }
///   ]
/// }
/// ```
///
/// ## Response
/// ```json
/// {
///   "success": true,
///   "results": [
///     {
///       "success": true,
///       "data": "Knowledge Base",
///       "key": "knowledge.title"
///     },
///     {
///       "success": true,
///       "data": "wissen",
///       "key": "knowledge"
///     }
///   ]
/// }
/// ```
///
/// ## Performance
/// - Batch of 100 gets: < 50ms
/// - Parallel execution
pub async fn batch_get_handler(
    body: web::Json<BatchGetRequest>,
    user: web::ReqData<AuthenticatedUser>,
) -> Result<HttpResponse, Error> {
    if !user.has_permission("batch.read") {
        return Ok(HttpResponse::Forbidden().json(ApiError {
            success: false,
            error: "Permission denied".to_string(),
        }));
    }

    let mut results = Vec::new();

    for request in &body.requests {
        let result = match request.request_type.as_str() {
            "text" => execute_text_get(request),
            "route" => execute_route_get(request),
            "meta" => execute_meta_get(request),
            _ => Err(format!("Invalid request type: {}", request.request_type)),
        };

        results.push(match result {
            Ok(data) => BatchGetResult {
                success: true,
                data: Some(data),
                error: None,
                key: request.key.clone(),
            },
            Err(e) => BatchGetResult {
                success: false,
                data: None,
                error: Some(e),
                key: request.key.clone(),
            },
        });
    }

    Ok(HttpResponse::Ok().json(BatchGetResponse {
        success: true,
        results,
    }))
}

/// Executes text get operation.
fn execute_text_get(request: &BatchGetItem) -> Result<String, String> {
    let req = ReedRequest {
        key: request.key.clone(),
        language: request.language.clone(),
        environment: request.environment.clone(),
        context: None,
    };

    reedbase::get::text(&req)
        .map(|r| r.data)
        .map_err(|e| format!("{:?}", e))
}

/// Batch get request structure.
#[derive(Debug, Deserialize)]
pub struct BatchGetRequest {
    pub requests: Vec<BatchGetItem>,
}

#[derive(Debug, Deserialize)]
pub struct BatchGetItem {
    #[serde(rename = "type")]
    pub request_type: String,
    pub key: String,
    pub language: Option<String>,
    pub environment: Option<String>,
}

/// Batch get response structure.
#[derive(Debug, Serialize)]
pub struct BatchGetResponse {
    pub success: bool,
    pub results: Vec<BatchGetResult>,
}

#[derive(Debug, Serialize)]
pub struct BatchGetResult {
    pub success: bool,
    pub data: Option<String>,
    pub error: Option<String>,
    pub key: String,
}
```

### Response Structures (`src/reedcms/api/responses.rs`)

```rust
/// Standard API success response.
#[derive(Debug, Serialize)]
pub struct ApiResponse {
    pub success: bool,
    pub data: String,
    pub key: Option<String>,
    pub language: Option<String>,
    pub environment: Option<String>,
}

/// API success message response.
#[derive(Debug, Serialize)]
pub struct ApiSuccess {
    pub success: bool,
    pub message: String,
}

/// API error response.
#[derive(Debug, Serialize)]
pub struct ApiError {
    pub success: bool,
    pub error: String,
}

/// Config-specific API response.
#[derive(Debug, Serialize)]
pub struct ApiConfigResponse {
    pub success: bool,
    pub data: String,
    pub key: Option<String>,
    pub config_type: Option<String>,
}
```

## Implementation Files

### Primary Implementation
- `src/reedcms/api/routes.rs` - Route configuration
- `src/reedcms/api/get_handlers.rs` - GET operation handlers
- `src/reedcms/api/set_handlers.rs` - POST/PUT operation handlers
- `src/reedcms/api/batch_handlers.rs` - Batch operation handlers
- `src/reedcms/api/list_handlers.rs` - List operation handlers
- `src/reedcms/api/responses.rs` - Response structures

### Test Files
- `src/reedcms/api/routes.test.rs`
- `src/reedcms/api/get_handlers.test.rs`
- `src/reedcms/api/set_handlers.test.rs`
- `src/reedcms/api/batch_handlers.test.rs`
- `src/reedcms/api/list_handlers.test.rs`

## File Structure
```
src/reedcms/api/
├── routes.rs                # Route configuration
├── routes.test.rs           # Route tests
├── get_handlers.rs          # GET handlers
├── get_handlers.test.rs     # GET tests
├── set_handlers.rs          # SET handlers
├── set_handlers.test.rs     # SET tests
├── batch_handlers.rs        # Batch handlers
├── batch_handlers.test.rs   # Batch tests
├── list_handlers.rs         # List handlers
├── list_handlers.test.rs    # List tests
├── responses.rs             # Response structures
└── responses.test.rs        # Response tests
```

## Testing Requirements

### Unit Tests
- [ ] Test all GET handlers with valid keys
- [ ] Test all SET handlers with valid data
- [ ] Test batch get with multiple requests
- [ ] Test batch set with multiple operations
- [ ] Test permission checking in handlers
- [ ] Test error responses for missing keys
- [ ] Test invalid request body handling

### Integration Tests
- [ ] Test complete API workflow (auth → get → set)
- [ ] Test batch operations end-to-end
- [ ] Test authentication with API endpoints
- [ ] Test permission-based access control
- [ ] Test content negotiation (JSON)
- [ ] Test rate limiting with API calls

### API Tests
- [ ] Test all endpoints with curl/HTTP client
- [ ] Test CORS headers (if enabled)
- [ ] Test JSON parsing and validation
- [ ] Test error status codes (404, 403, 500)

### Performance Tests
- [ ] GET operation: < 10ms
- [ ] SET operation: < 20ms
- [ ] Batch get (100 items): < 50ms
- [ ] Concurrent requests: 1000/sec sustained

## Acceptance Criteria
- [ ] All GET endpoints functional
- [ ] All SET endpoints functional
- [ ] Batch operations working
- [ ] List operations implemented
- [ ] Authentication required for all endpoints
- [ ] Permission checking enforced
- [ ] JSON request/response format correct
- [ ] Error responses proper (404, 403, 500)
- [ ] All tests pass with 100% coverage
- [ ] Performance benchmarks met
- [ ] Documentation complete
- [ ] BBC English throughout

## Dependencies
- **Requires**: REED-06-01 (Server), REED-02-01 (ReedBase), REED-06-03 (Authentication)

## Blocks
- REED-07-02 (API Security Matrix uses these endpoints)

## References
- Service Template: `_workbench/Tickets/templates/service-template.md`
- Service Test Template: `_workbench/Tickets/templates/service-template.test.md`
- Summary: Lines 1016-1023, 1468-1504 in `project_summary.md`

## Notes
ReedAPI provides programmatic access to ReedBase data for external applications and frontend JavaScript. All endpoints require authentication to prevent unauthorized access. Permission system enables fine-grained control over read/write operations per data type. Batch operations reduce HTTP overhead for bulk data access. JSON format enables easy integration with modern web frameworks and mobile applications.
