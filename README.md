# ReedCMS

> A high-performance, CSV-based headless CMS built with Rust

## Overview

ReedCMS is a modern headless CMS that prioritises simplicity, performance, and developer experience. It uses CSV files for data storage with O(1) HashMap lookups, delivering sub-millisecond response times.

## Features

- **CSV-Based Storage**: Simple, portable, and version-control friendly
- **Sub-Millisecond Performance**: O(1) HashMap cache with ReedBase
- **Multi-Language Support**: Built-in i18n with language detection
- **Template System**: MiniJinja templates with hot-reload in development
- **RESTful API**: Full HTTP API for external integrations
- **Asset Pipeline**: CSS/JS bundling, minification, and cache busting
- **Security**: Argon2 password hashing, role-based permissions
- **Unix Socket Support**: Production-ready with nginx/apache integration
- **Monitoring**: Built-in performance profiling and health checks

## Architecture

ReedCMS follows a layered architecture:

1. **Foundation Layer**: Core communication (ReedStream)
2. **Data Layer**: CSV storage and caching (ReedBase)
3. **Security Layer**: Authentication and permissions
4. **CLI Layer**: Command-line interface
5. **Template Layer**: MiniJinja rendering
6. **Server Layer**: Actix-Web HTTP server
7. **API Layer**: RESTful endpoints
8. **Asset Layer**: CSS/JS bundling
9. **Build Layer**: Asset pipeline and compilation
10. **Monitor Layer**: Performance monitoring and debugging

## Quick Start

```bash
# 1. Clone and build
git clone https://github.com/vvoss-dev/ReedCMS.git
cd ReedCMS
cargo build --release

# 2. Install reed command system-wide
./scripts/setup.sh
# → Will ask for sudo password

# 3. Start development server
reed server:io --port 8333
# → Server runs on http://localhost:8333

# 4. Test commands
reed data:get knowledge.title@en
man reed
```

**Note**: The `reed` command is installed system-wide. Always use `reed`, never `./target/release/reed` directly.

## Documentation

Complete documentation is available in the `_workbench/Tickets/` directory, organised by layer with detailed implementation specifications.

## Development Status

🚧 **In Development** - ReedCMS is currently under active development. See the ticket system for implementation progress.

## License

Apache License 2.0 - see LICENSE file for details

## Author

Vivian Voss <ask@vvoss.dev>
