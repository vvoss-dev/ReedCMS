# Architecture Overview

## The 10-Layer Architecture

ReedCMS is organised into 10 distinct layers, each with a clear responsibility. Layers communicate through standardised interfaces and build upon each other.

## Layer Diagram

```
┌─────────────────────────────────────────────────────────────┐
│ 10. Monitor Layer                                           │
│     Logging │ Metrics │ Health │ Debug                      │
│                                                              │
│     Responsibilities:                                        │
│     - FreeBSD syslog format logging                         │
│     - Performance metrics collection                        │
│     - Health check endpoints                                │
│     - Development debugging tools                           │
└──────────────────────────┬──────────────────────────────────┘
┌──────────────────────────┴──────────────────────────────────┐
│ 09. Build Layer                                             │
│     Binary Compiler │ Asset Pipeline │ Deployment          │
│                                                              │
│     Responsibilities:                                        │
│     - Cargo integration for binary compilation              │
│     - Build-time asset processing                           │
│     - Production deployment preparation                     │
│     - Release optimisation                                  │
└──────────────────────────┬──────────────────────────────────┘
┌──────────────────────────┴──────────────────────────────────┐
│ 08. Asset Layer                                             │
│     CSS Bundler │ JS Bundler │ Static Server               │
│                                                              │
│     Responsibilities:                                        │
│     - Session-hash cache busting                            │
│     - CSS bundling and minification                         │
│     - JavaScript bundling                                   │
│     - Static file serving                                   │
└──────────────────────────┬──────────────────────────────────┘
┌──────────────────────────┴──────────────────────────────────┐
│ 07. API Layer                                               │
│     RESTful │ Security Matrix │ Rate Limiting               │
│                                                              │
│     Responsibilities:                                        │
│     - RESTful HTTP endpoints                                │
│     - Security matrix (permissions)                         │
│     - Sliding window rate limiting                          │
│     - API authentication                                    │
└──────────────────────────┬──────────────────────────────────┘
┌──────────────────────────┴──────────────────────────────────┐
│ 06. Server Layer                                            │
│     Actix-Web │ Routing │ Client Detection │ Response      │
│                                                              │
│     Responsibilities:                                        │
│     - HTTP server (Actix-Web)                               │
│     - URL routing from .reed/routes.csv                     │
│     - Client detection (mouse/touch/reader)                 │
│     - HTML response generation                              │
└──────────────────────────┬──────────────────────────────────┘
┌──────────────────────────┴──────────────────────────────────┐
│ 05. Template Layer                                          │
│     MiniJinja │ Atomic Design │ Filters │ Hot-Reload       │
│                                                              │
│     Responsibilities:                                        │
│     - MiniJinja template engine integration                 │
│     - Atomic Design component structure                     │
│     - Custom filters (text, route, meta)                    │
│     - Hot-reload in development mode                        │
└──────────────────────────┬──────────────────────────────────┘
┌──────────────────────────┴──────────────────────────────────┐
│ 04. CLI Layer                                               │
│     Parser │ Router │ Commands (Data │ User │ Role │...)   │
│                                                              │
│     Responsibilities:                                        │
│     - Command-line interface                                │
│     - namespace:action command parsing                      │
│     - Command routing and execution                         │
│     - All data and system operations                        │
└──────────────────────────┬──────────────────────────────────┘
┌──────────────────────────┴──────────────────────────────────┐
│ 03. Security Layer                                          │
│     Argon2 │ Users │ Roles │ Permissions │ Auth            │
│                                                              │
│     Responsibilities:                                        │
│     - Argon2 password hashing                               │
│     - User management (CRUD)                                │
│     - Role-based access control                             │
│     - Permission checking                                   │
└──────────────────────────┬──────────────────────────────────┘
┌──────────────────────────┴──────────────────────────────────┐
│ 02. Data Layer                                              │
│     ReedBase Cache │ CSV Storage │ Backup System           │
│                                                              │
│     Responsibilities:                                        │
│     - O(1) HashMap cache (ReedBase)                         │
│     - CSV file storage (.reed/*.csv)                        │
│     - XZ-compressed backups (32 kept)                       │
│     - Get/Set/List operations                               │
└──────────────────────────┬──────────────────────────────────┘
┌──────────────────────────┴──────────────────────────────────┐
│ 01. Foundation Layer                                        │
│     ReedStream │ ReedError │ ReedResponse │ ReedResult     │
│                                                              │
│     Responsibilities:                                        │
│     - Universal communication types                         │
│     - Error handling with ReedError                         │
│     - Response standardisation                              │
│     - Module trait definition                               │
└─────────────────────────────────────────────────────────────┘
```

## Layer Descriptions

### Layer 01: Foundation

**Purpose:** Provides universal communication types used by all other layers.

**Key Components:**
- `ReedStream` - Communication interface
- `ReedError` - Error variants with context
- `ReedResponse<T>` - Standardised response wrapper
- `ReedResult<T>` - Type alias for `Result<T, ReedError>`
- `ReedModule` - Trait all modules implement

**Why First:** All layers depend on these core types for communication.

[→ Foundation Layer Documentation](../01-foundation-layer/README.md)

---

### Layer 02: Data

**Purpose:** CSV storage with O(1) cache for sub-millisecond lookups.

**Key Components:**
- `ReedBase` - Central HashMap cache with RwLock
- CSV files in `.reed/` directory
- Backup system (XZ-compressed, 32 kept)
- Get/Set/List operations

**Data Flow:**
```
Request → ReedBase Cache → CSV File (on miss) → Cache Update → Response
```

**Performance:** < 100μs for cached lookups, < 50ms for cache misses

[→ Data Layer Documentation](../02-data-layer/README.md)

---

### Layer 03: Security

**Purpose:** User authentication, role management, and permissions.

**Key Components:**
- Argon2 password hashing (RFC 9106)
- User CRUD operations
- Role-based access control (RBAC)
- Permission matrix

**Security Flow:**
```
Request → Auth Check → Permission Check → Execute → Response
```

[→ Security Layer Documentation](../03-security-layer/README.md)

---

### Layer 04: CLI

**Purpose:** Command-line interface for all system operations.

**Key Components:**
- Command parser (`namespace:action` format)
- Command router
- Data commands (text, route, meta)
- User/Role commands
- Layout/Migration commands

**Command Structure:**
```bash
reed <namespace>:<action> [args] [flags]
```

**Examples:**
```bash
reed data:get page.title@en
reed user:create admin --email admin@example.com
reed server:io --port 8333
```

[→ CLI Layer Documentation](../04-cli-layer/README.md)

---

### Layer 05: Template

**Purpose:** MiniJinja template rendering with Atomic Design structure.

**Key Components:**
- MiniJinja engine integration
- Atomic Design (atoms/molecules/organisms)
- Custom filters (text, route, meta)
- Hot-reload in development
- Variant system (mouse/touch/reader)

**Template Flow:**
```
Request → Route → Layout → Components → Render → HTML
```

[→ Template Layer Documentation](../05-template-layer/README.md)

---

### Layer 06: Server

**Purpose:** HTTP server with routing and client detection.

**Key Components:**
- Actix-Web framework
- Route resolution from `.reed/routes.csv`
- Client detection (User-Agent, screen width)
- HTML response generation

**Request Flow:**
```
HTTP Request → Route Match → Client Detect → Template → Response
```

**Modes:**
- HTTP server (`--port 8333`)
- Unix socket (`--socket /tmp/reed.sock`)

[→ Server Layer Documentation](../06-server-layer/README.md)

---

### Layer 07: API

**Purpose:** RESTful API with security matrix and rate limiting.

**Key Components:**
- RESTful endpoints (`/api/v1/*`)
- Security matrix (permission checking)
- Sliding window rate limiting
- API key authentication

**API Structure:**
```
GET    /api/v1/text/get/:key
POST   /api/v1/text/set
GET    /api/v1/user/list
```

[→ API Layer Documentation](../07-api-layer/README.md)

---

### Layer 08: Asset

**Purpose:** CSS/JS bundling with cache busting.

**Key Components:**
- Session-hash generation (cache busting)
- CSS bundling and minification
- JavaScript bundling
- Static file serving

**Bundle Flow:**
```
Components → Discover CSS/JS → Bundle → Minify → Session Hash → Serve
```

[→ Asset Layer Documentation](../08-asset-layer/README.md)

---

### Layer 09: Build

**Purpose:** Binary compilation and deployment preparation.

**Key Components:**
- Cargo integration
- Asset pipeline execution
- Release optimisation
- Deployment packaging

**Build Process:**
```
Source Code → Cargo Build → Asset Pipeline → Optimise → Package
```

[→ Build Layer Documentation](../09-build-layer/README.md)

---

### Layer 10: Monitor

**Purpose:** Logging, metrics, and system health.

**Key Components:**
- FreeBSD syslog format logging
- Performance metrics collection
- Health check endpoints
- Development debugging tools

**Monitoring Flow:**
```
Event → Log → Metrics → Health Check → Dashboard
```

[→ Monitor Layer Documentation](../10-monitor-layer/README.md)

---

## Communication Patterns

### Vertical Communication (Between Layers)

All layers communicate through `ReedStream` types:

```rust
// Request format
pub struct ReedRequest {
    pub key: String,
    pub language: Option<String>,
    pub environment: Option<String>,
    // ...
}

// Response format
pub struct ReedResponse<T> {
    pub data: T,
    pub source: String,
    pub cached: bool,
    pub timestamp: u64,
    pub metrics: Option<ResponseMetrics>,
}

// Result type
pub type ReedResult<T> = Result<T, ReedError>;
```

**Example:**
```rust
// Layer 04 (CLI) calls Layer 02 (Data)
let response = get_text("page.title", "en")?;
println!("{}", response.data);
```

### Horizontal Communication (Within Layer)

Modules within a layer communicate directly:

```rust
// Within Security Layer
pub mod users;
pub mod roles;
pub mod permissions;

// roles.rs uses users.rs
use super::users::get_user;
```

## Data Flow Example

**User Request → HTML Response:**

```
┌─────────────────────────────────────────────────────────┐
│ 1. HTTP Request: GET /knowledge                         │
└────────────┬────────────────────────────────────────────┘
             │
             ▼
┌─────────────────────────────────────────────────────────┐
│ 2. Server Layer: Route resolution                       │
│    - Check .reed/routes.csv                             │
│    - Match "/knowledge" → layout "knowledge"            │
└────────────┬────────────────────────────────────────────┘
             │
             ▼
┌─────────────────────────────────────────────────────────┐
│ 3. Server Layer: Client detection                       │
│    - Check User-Agent                                   │
│    - Determine variant: "mouse"                         │
└────────────┬────────────────────────────────────────────┘
             │
             ▼
┌─────────────────────────────────────────────────────────┐
│ 4. Template Layer: Load layout                          │
│    - Load knowledge.mouse.jinja                         │
│    - Discover components                                │
└────────────┬────────────────────────────────────────────┘
             │
             ▼
┌─────────────────────────────────────────────────────────┐
│ 5. Template Layer: Build context                        │
│    - Call text filter → Data Layer                      │
│    - Call route filter → Data Layer                     │
│    - Call meta filter → Data Layer                      │
└────────────┬────────────────────────────────────────────┘
             │
             ▼
┌─────────────────────────────────────────────────────────┐
│ 6. Data Layer: Fetch content                            │
│    - ReedBase cache lookup (O(1))                       │
│    - Return text/route/meta data                        │
└────────────┬────────────────────────────────────────────┘
             │
             ▼
┌─────────────────────────────────────────────────────────┐
│ 7. Template Layer: Render HTML                          │
│    - MiniJinja renders with context                     │
│    - Generate complete HTML                             │
└────────────┬────────────────────────────────────────────┘
             │
             ▼
┌─────────────────────────────────────────────────────────┐
│ 8. Server Layer: Send response                          │
│    - Add headers                                        │
│    - Return HTML to client                              │
└─────────────────────────────────────────────────────────┘
```

## Design Principles

### 1. Separation of Concerns

Each layer has ONE clear responsibility. No layer reaches across boundaries.

### 2. Standardised Communication

All layers use `ReedResult<ReedResponse<T>>` for consistency.

### 3. Bottom-Up Dependencies

Higher layers depend on lower layers, never the reverse.

### 4. Module Isolation

Each module within a layer is independent and testable.

### 5. Performance by Design

- Layer 02 provides O(1) lookups
- Caching at every level
- Minimal allocations in hot paths

## File Organisation

```
src/reedcms/
├── reedstream.rs          # Layer 01: Foundation types
├── reed/                  # Dispatchers (cross-layer)
│   ├── reedbase.rs
│   ├── reedcli.rs
│   └── reedserver.rs
├── reedbase/              # Layer 02: Data operations
├── users/                 # Layer 03: Security
├── cli/                   # Layer 04: CLI commands
├── templates/             # Layer 05: Template engine
├── server/                # Layer 06: HTTP server
├── api/                   # Layer 07: RESTful API
├── assets/                # Layer 08: Asset bundling
├── build/                 # Layer 09: Build system
└── monitor/               # Layer 10: Monitoring
```

## Next Steps

- [Core Philosophy](core-philosophy.md) - Understand the "why"
- [Foundation Layer](../01-foundation-layer/README.md) - Start at the bottom
- [Data Layer](../02-data-layer/README.md) - Learn the cache system
