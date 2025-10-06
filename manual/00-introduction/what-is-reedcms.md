# What is ReedCMS?

## Overview

ReedCMS is a high-performance headless Content Management System built with Rust that uses CSV files for data storage and delivers sub-millisecond response times through an in-memory HashMap cache.

## Core Concept

```
ReedCMS = CSV Storage + O(1) Cache + CLI Interface + Template Engine
```

Instead of traditional databases, ReedCMS stores all data in simple, human-readable CSV files. At runtime, these files are loaded into memory for instant access, combining the simplicity of flat files with database-level performance.

## Key Characteristics

### 1. CSV-Based Storage

All data lives in pipe-delimited CSV files:

```csv
# .reed/text.csv
key|value|description
page.title@en|Welcome|Homepage title
page.title@de|Willkommen|German homepage title
```

**Benefits:**
- Human-readable and editable
- Version control friendly (Git-compatible)
- No database installation required
- Portable across systems
- Zero configuration

### 2. Sub-Millisecond Performance

Despite using CSV files, ReedCMS achieves database-level performance:

| Operation | Performance | Method |
|-----------|-------------|--------|
| Text lookup | < 100μs | O(1) HashMap cache |
| Route resolution | < 50μs | Pre-loaded HashMap |
| Template render | < 50ms | MiniJinja with hot-reload |
| Full page response | < 100ms | Complete render cycle |

**How:** CSV files are loaded once at startup into HashMap caches with RwLock for thread-safe concurrent access.

### 3. CLI-First Design

Every operation is available via the `reed` command:

```bash
# Set content
reed data:set page.title@en "Welcome" --desc "Homepage title"

# Get content
reed data:get page.title@en

# Start server
reed server:io --port 8333

# Create user
reed user:create admin --email admin@example.com
```

The CLI is the primary interface, making automation and scripting trivial.

### 4. Headless Architecture

ReedCMS provides:
- RESTful API for external applications
- Template rendering for traditional websites
- Both HTTP server and Unix socket modes

You can use ReedCMS as:
- Pure API backend (headless mode)
- Traditional website CMS (template mode)
- Hybrid (API + rendered pages)

### 5. Environment-Aware

Built-in support for different environments:

```csv
# Development-specific values
server.debug@dev|true
server.port@dev|8333

# Production-specific values
server.debug@prod|false
server.socket@prod|/tmp/reed.sock
```

Automatic fallback chain: `key@lang@env` → `key@lang` → `key@env` → `key`

## Architecture

ReedCMS is organised into 10 distinct layers:

```
┌─────────────────────────────────────┐
│ 10. Monitor    │ Logging & Metrics  │
├─────────────────────────────────────┤
│ 09. Build      │ Compilation        │
├─────────────────────────────────────┤
│ 08. Asset      │ CSS/JS Bundling    │
├─────────────────────────────────────┤
│ 07. API        │ RESTful Endpoints  │
├─────────────────────────────────────┤
│ 06. Server     │ HTTP/Socket Server │
├─────────────────────────────────────┤
│ 05. Template   │ MiniJinja Engine   │
├─────────────────────────────────────┤
│ 04. CLI        │ Command Interface  │
├─────────────────────────────────────┤
│ 03. Security   │ Auth & Permissions │
├─────────────────────────────────────┤
│ 02. Data       │ CSV & Cache        │
├─────────────────────────────────────┤
│ 01. Foundation │ Core Types         │
└─────────────────────────────────────┘
```

Each layer has a clear responsibility and communicates through the universal ReedStream interface.

## Use Cases

### 1. Traditional Website

Use ReedCMS like a traditional CMS:
- Store content in CSV files
- Design templates with MiniJinja
- Render pages server-side
- Deploy via HTTP or Unix socket

### 2. Headless API

Use ReedCMS as pure API backend:
- Store structured data in CSV
- Access via RESTful API
- Integrate with React, Vue, etc.
- Mobile app backend

### 3. Static Site Generator

Use ReedCMS for static generation:
- Manage content via CLI
- Render pages at build time
- Deploy static HTML
- No runtime server needed

### 4. Internal Tool

Use ReedCMS for internal applications:
- Quick setup (no database)
- CLI automation
- Git-based workflow
- Team collaboration

## Who Should Use ReedCMS?

### Ideal For

- **Small to medium websites** (< 100,000 pages)
- **Content-focused sites** (blogs, documentation, portfolios)
- **Teams using Git** (version-controlled content)
- **Rust enthusiasts** (modern, fast, safe)
- **CLI lovers** (script everything)

### Not Ideal For

- **High-write workloads** (thousands of writes/second)
- **Massive datasets** (millions of entries)
- **Real-time collaboration** (simultaneous edits)
- **Complex transactions** (multi-table updates)

## Philosophy

ReedCMS is built on three principles:

### 1. KISS (Keep It Simple, Stupid)

- CSV files instead of database schemas
- Flat file structure instead of complex relationships
- One function = one job
- Clear naming over clever abstractions

### 2. Performance Through Caching

- Load once, access millions of times
- O(1) lookups for everything
- Memory-efficient with RwLock
- Sub-millisecond response times

### 3. Developer Experience

- CLI-first design
- Comprehensive documentation
- Clear error messages
- Hot-reload in development

## Getting Started

```bash
# 1. Clone and build
git clone https://github.com/vvoss-dev/ReedCMS.git
cd ReedCMS
cargo build --release

# 2. Install reed command
./scripts/setup.sh

# 3. Start development server
reed server:io --port 8333

# 4. Access your site
open http://localhost:8333
```

See [Getting Started](getting-started.md) for detailed instructions.

## Next Steps

- [Architecture Overview](architecture-overview.md) - Understand the 10-layer structure
- [Core Philosophy](core-philosophy.md) - Deep dive into design decisions
- [Getting Started](getting-started.md) - Step-by-step tutorial
- [CLI Layer](../04-cli-layer/README.md) - Learn the commands

## Summary

ReedCMS combines the simplicity of CSV files with database-level performance, wrapped in a professional CLI interface and built with Rust for safety and speed. It's perfect for content-focused websites where developer experience and version control matter more than complex database features.
