# REED-04-12: Reed.toml Project Configuration System

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
- **ID**: REED-04-12
- **Title**: Reed.toml Project Configuration System
- **Layer**: CLI Layer (REED-04)
- **Priority**: Medium
- **Status**: Open
- **Complexity**: Medium
- **Dependencies**: REED-04-01, REED-04-02, REED-02-01

## Summary Reference
- **Section**: Reed.toml Configuration System
- **Lines**: To be added to project_summary.md after implementation
- **Key Concepts**: Developer-facing configuration, TOML to CSV sync, project defaults

## Objective

Implement developer-facing `Reed.toml` configuration file in project root that serves as single source of truth for project defaults. Changes in Reed.toml are synced to `.reed/*.csv` files using existing CLI commands, maintaining clean separation between developer-facing configuration (TOML) and system-facing runtime data (CSV).

## Rationale

### Why TOML?

1. **Rust-Idiomatisch**: TOML is the standard for Rust projects (Cargo.toml, rustfmt.toml, clippy.toml)
2. **Type Safety**: Clear distinction between strings, integers, booleans, arrays
3. **Simple Syntax**: No whitespace sensitivity like YAML, no ambiguity
4. **Small Dependency**: `toml = "0.8"` is lightweight and well-maintained
5. **Security**: Simpler spec than YAML means fewer security concerns
6. **Developer Friendly**: Easy to read and write, familiar to Rust developers

### Problems with Current System

1. **Hidden Configuration**: `.reed/` directory is system-facing, not developer-friendly
2. **Scattered Defaults**: Project defaults spread across multiple CSV files
3. **No Documentation**: CSV files lack inline documentation for configuration options
4. **Manual Editing**: Developers must know CSV structure and keys
5. **No Validation**: Easy to create malformed configuration

### Benefits of Reed.toml

1. **Developer Experience**: Single file for all project configuration
2. **Git-Friendly**: Visible in project root, easy to version control
3. **Self-Documenting**: TOML format with comments and clear structure
4. **Type Safety**: TOML enforces valid data types (no string-to-bool confusion)
5. **Separation of Concerns**: 
   - `Reed.toml` = Developer-facing (project configuration)
   - `.reed/*.csv` = System-facing (runtime data)
6. **Existing Infrastructure**: Uses existing CLI commands and CSV handlers

## Reed.toml Structure

### Complete Example

```toml
# Reed.toml - ReedCMS Project Configuration
# This file is the single source of truth for project defaults
# Changes here are synced to .reed/ CSV files via `reed config:sync`

[project]
name = "vvoss.dev"
url = "https://vvoss.dev"
description = "Enterprise Software Architecture & Development"

# Language Configuration
[project.languages]
default = "de"                    # Default language for website
available = ["de", "en"]          # Available languages
fallback_chain = true             # Enable de → en fallback

# URL Structure
[project.routing]
url_prefix = true                 # Use /de/... /en/... structure
trailing_slash = true             # /wissen/ vs /wissen

# Template Configuration
[project.templates]
path = "templates/"               # Template directory
cache = true                      # Enable template caching
hot_reload = true                 # Hot reload in DEV mode

# Asset Configuration
[project.assets]
public_path = "public/"           # Public assets directory
bundle_strategy = "session_hash"  # session_hash | versioned | none
minify = true                     # Minify CSS/JS in PROD

# Build Configuration
[project.build]
target = "release"                # release | debug
optimize = true                   # Enable LTO and optimisations
upx_compress = false              # UPX compression (optional)

# Server Configuration
[server]
default_port = 8333               # Default HTTP port
workers = 4                       # Worker thread count (0 = auto)
socket_path = "/tmp/reed.sock"   # Unix socket path

# Site Protection (htaccess-style)
[server.auth]
enabled = false                   # Enable site-wide protection
username = "admin"                # Site access username
# Password set via: reed user:passwd admin

# Logging Configuration
[server.logging]
level = "info"                    # emerg | alert | crit | error | warn | notice | info | debug
# file = "/var/log/reed.log"     # Log file path (commented = stdout only)

# Development Configuration
[dev]
hot_reload = true                 # Enable hot reload
debug_mode = true                 # Enable debug output
profiling = false                 # Enable performance profiling
```

### Minimal Example

```toml
# Reed.toml - Minimal Configuration
[project]
name = "My ReedCMS Site"
url = "http://localhost:8333"

[project.languages]
default = "en"
available = ["en"]
```

## Implementation

### File Structure

```
src/reedcms/cli/
├── config_commands.rs          # CLI commands (config:init, config:sync, config:show, config:validate)
├── config_commands.test.rs     # Tests
```

```
src/reedcms/config/
├── toml_parser.rs              # TOML parsing and validation
├── toml_parser.test.rs         # Tests
├── toml_to_csv.rs              # Sync TOML → CSV using CLI commands
├── toml_to_csv.test.rs         # Tests
├── mod.rs                      # Module exports
```

### TOML Parser (`src/reedcms/config/toml_parser.rs`)

```rust
// Copyright 2025 Vivian Voss. Licensed under the Apache Licence, Version 2.0.
// SPDX-Licence-Identifier: Apache-2.0

//! Reed.toml parser and validator.
//!
//! Parses Reed.toml configuration file and validates structure.

use crate::reedcms::reedstream::{ReedError, ReedResult};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Complete Reed.toml configuration structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReedConfig {
    pub project: ProjectConfig,
    #[serde(default)]
    pub server: ServerConfig,
    #[serde(default)]
    pub dev: DevConfig,
}

/// Project configuration section.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub name: String,
    pub url: String,
    #[serde(default)]
    pub description: Option<String>,
    pub languages: LanguageConfig,
    #[serde(default)]
    pub routing: RoutingConfig,
    #[serde(default)]
    pub templates: TemplateConfig,
    #[serde(default)]
    pub assets: AssetConfig,
    #[serde(default)]
    pub build: BuildConfig,
}

/// Language configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageConfig {
    pub default: String,
    pub available: Vec<String>,
    #[serde(default = "default_true")]
    pub fallback_chain: bool,
}

/// Routing configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RoutingConfig {
    #[serde(default = "default_true")]
    pub url_prefix: bool,
    #[serde(default = "default_true")]
    pub trailing_slash: bool,
}

/// Template configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TemplateConfig {
    #[serde(default = "default_templates_path")]
    pub path: String,
    #[serde(default = "default_true")]
    pub cache: bool,
    #[serde(default = "default_true")]
    pub hot_reload: bool,
}

/// Asset configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AssetConfig {
    #[serde(default = "default_public_path")]
    pub public_path: String,
    #[serde(default = "default_session_hash")]
    pub bundle_strategy: String,
    #[serde(default = "default_true")]
    pub minify: bool,
}

/// Build configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BuildConfig {
    #[serde(default = "default_release")]
    pub target: String,
    #[serde(default = "default_true")]
    pub optimize: bool,
    #[serde(default)]
    pub upx_compress: bool,
}

/// Server configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ServerConfig {
    #[serde(default = "default_port")]
    pub default_port: u16,
    #[serde(default)]
    pub workers: usize,
    #[serde(default = "default_socket_path")]
    pub socket_path: String,
    #[serde(default)]
    pub auth: AuthConfig,
    #[serde(default)]
    pub logging: LoggingConfig,
}

/// Authentication configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AuthConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_admin")]
    pub username: String,
}

/// Logging configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LoggingConfig {
    #[serde(default = "default_info")]
    pub level: String,
    #[serde(default)]
    pub file: Option<String>,
}

/// Development configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DevConfig {
    #[serde(default = "default_true")]
    pub hot_reload: bool,
    #[serde(default = "default_true")]
    pub debug_mode: bool,
    #[serde(default)]
    pub profiling: bool,
}

// Default value functions
fn default_true() -> bool { true }
fn default_templates_path() -> String { "templates/".to_string() }
fn default_public_path() -> String { "public/".to_string() }
fn default_session_hash() -> String { "session_hash".to_string() }
fn default_release() -> String { "release".to_string() }
fn default_port() -> u16 { 8333 }
fn default_socket_path() -> String { "/tmp/reed.sock".to_string() }
fn default_admin() -> String { "admin".to_string() }
fn default_info() -> String { "info".to_string() }

/// Parses Reed.toml file.
///
/// ## Arguments
/// - path: Path to Reed.toml file
///
/// ## Returns
/// - ReedConfig structure
///
/// ## Errors
/// - File not found
/// - TOML parse error
/// - Validation error
pub fn parse_reed_toml<P: AsRef<Path>>(path: P) -> ReedResult<ReedConfig> {
    let content = std::fs::read_to_string(&path).map_err(|e| ReedError::IoError {
        operation: "read".to_string(),
        path: path.as_ref().display().to_string(),
        reason: e.to_string(),
    })?;

    let config: ReedConfig = toml::from_str(&content).map_err(|e| ReedError::ParseError {
        input: "Reed.toml".to_string(),
        reason: e.to_string(),
    })?;

    validate_config(&config)?;

    Ok(config)
}

/// Validates Reed.toml configuration.
///
/// ## Validation Rules
/// - Project name: 1-100 characters
/// - Project URL: Valid URL format
/// - Default language: Must be in available languages
/// - Available languages: At least one language
/// - Default port: 1-65535
///
/// ## Errors
/// - ValidationError with specific constraint
pub fn validate_config(config: &ReedConfig) -> ReedResult<()> {
    // Validate project name
    if config.project.name.is_empty() || config.project.name.len() > 100 {
        return Err(ReedError::ValidationError {
            field: "project.name".to_string(),
            value: config.project.name.clone(),
            constraint: "1-100 characters".to_string(),
        });
    }

    // Validate project URL
    if !config.project.url.starts_with("http://") && !config.project.url.starts_with("https://") {
        return Err(ReedError::ValidationError {
            field: "project.url".to_string(),
            value: config.project.url.clone(),
            constraint: "Must start with http:// or https://".to_string(),
        });
    }

    // Validate languages
    if config.project.languages.available.is_empty() {
        return Err(ReedError::ValidationError {
            field: "project.languages.available".to_string(),
            value: "[]".to_string(),
            constraint: "At least one language required".to_string(),
        });
    }

    // Validate default language is in available languages
    if !config.project.languages.available.contains(&config.project.languages.default) {
        return Err(ReedError::ValidationError {
            field: "project.languages.default".to_string(),
            value: config.project.languages.default.clone(),
            constraint: format!("Must be one of: {}", config.project.languages.available.join(", ")),
        });
    }

    // Validate port range
    if config.server.default_port == 0 {
        return Err(ReedError::ValidationError {
            field: "server.default_port".to_string(),
            value: config.server.default_port.to_string(),
            constraint: "1-65535".to_string(),
        });
    }

    Ok(())
}
```

### TOML to CSV Sync (`src/reedcms/config/toml_to_csv.rs`)

```rust
// Copyright 2025 Vivian Voss. Licensed under the Apache Licence, Version 2.0.
// SPDX-Licence-Identifier: Apache-2.0

//! Syncs Reed.toml configuration to .reed/*.csv files.
//!
//! Uses existing CLI commands to update CSV files, ensuring consistency.

use crate::reedcms::cli::data_commands;
use crate::reedcms::config::toml_parser::ReedConfig;
use crate::reedcms::reedstream::ReedResult;

/// Syncs Reed.toml to .reed/*.csv files.
///
/// ## Process
/// 1. Parse Reed.toml
/// 2. Use existing CLI commands to set values
/// 3. Report changes
///
/// ## Uses Existing Infrastructure
/// - data_commands::set_project_value()
/// - data_commands::set_server_value()
///
/// ## Performance
/// - < 100ms for typical config
/// - Uses existing CSV atomic write operations
///
/// ## Output
/// - List of updated keys
pub fn sync_toml_to_csv(config: &ReedConfig) -> ReedResult<Vec<String>> {
    let mut updated_keys = Vec::new();

    // Project configuration
    set_and_track("project.name", &config.project.name, &mut updated_keys)?;
    set_and_track("project.url", &config.project.url, &mut updated_keys)?;
    
    if let Some(desc) = &config.project.description {
        set_and_track("project.description", desc, &mut updated_keys)?;
    }

    // Language configuration
    set_and_track("project.default_language", &config.project.languages.default, &mut updated_keys)?;
    set_and_track("project.languages", &config.project.languages.available.join(","), &mut updated_keys)?;

    // Routing configuration
    set_and_track("project.routing.url_prefix", &config.project.routing.url_prefix.to_string(), &mut updated_keys)?;
    set_and_track("project.routing.trailing_slash", &config.project.routing.trailing_slash.to_string(), &mut updated_keys)?;

    // Template configuration
    set_and_track("project.template.path", &config.project.templates.path, &mut updated_keys)?;
    set_and_track("project.template.cache", &config.project.templates.cache.to_string(), &mut updated_keys)?;
    set_and_track("project.template.hot_reload", &config.project.templates.hot_reload.to_string(), &mut updated_keys)?;

    // Asset configuration
    set_and_track("project.public.path", &config.project.assets.public_path, &mut updated_keys)?;
    set_and_track("project.asset.bundle_strategy", &config.project.assets.bundle_strategy, &mut updated_keys)?;
    set_and_track("project.asset.minify", &config.project.assets.minify.to_string(), &mut updated_keys)?;

    // Build configuration
    set_and_track("project.build.target", &config.project.build.target, &mut updated_keys)?;
    set_and_track("project.build.optimize", &config.project.build.optimize.to_string(), &mut updated_keys)?;
    set_and_track("project.build.upx_compress", &config.project.build.upx_compress.to_string(), &mut updated_keys)?;

    // Server configuration
    set_server_and_track("default_port", &config.server.default_port.to_string(), &mut updated_keys)?;
    set_server_and_track("workers", &config.server.workers.to_string(), &mut updated_keys)?;
    set_server_and_track("socket_path", &config.server.socket_path, &mut updated_keys)?;
    set_server_and_track("auth.enabled", &config.server.auth.enabled.to_string(), &mut updated_keys)?;
    set_server_and_track("auth.username", &config.server.auth.username, &mut updated_keys)?;
    set_server_and_track("logging.level", &config.server.logging.level, &mut updated_keys)?;
    
    if let Some(log_file) = &config.server.logging.file {
        set_server_and_track("logging.file", log_file, &mut updated_keys)?;
    }

    Ok(updated_keys)
}

/// Helper: Sets project value and tracks change.
fn set_and_track(key: &str, value: &str, updated: &mut Vec<String>) -> ReedResult<()> {
    // Use existing set:project command logic
    data_commands::set_project_value(key, value)?;
    updated.push(format!("project.{}", key));
    Ok(())
}

/// Helper: Sets server value and tracks change.
fn set_server_and_track(key: &str, value: &str, updated: &mut Vec<String>) -> ReedResult<()> {
    // Use existing set:server command logic
    data_commands::set_server_value(key, value)?;
    updated.push(format!("server.{}", key));
    Ok(())
}
```

### CLI Commands (`src/reedcms/cli/config_commands.rs`)

```rust
// Copyright 2025 Vivian Voss. Licensed under the Apache Licence, Version 2.0.
// SPDX-Licence-Identifier: Apache-2.0

//! Reed.toml configuration management CLI commands.

use crate::reedcms::config::{toml_parser, toml_to_csv};
use crate::reedcms::reedstream::ReedResult;

/// Initialises Reed.toml from existing .reed/*.csv files.
///
/// ## Command
/// `reed config:init`
///
/// ## Process
/// 1. Read current values from .reed/*.csv
/// 2. Generate Reed.toml with current configuration
/// 3. Write Reed.toml to project root
///
/// ## Output
/// ```
/// ✓ Created Reed.toml from current configuration
/// Edit Reed.toml and run `reed config:sync` to apply changes
/// ```
pub fn init_reed_toml() -> ReedResult<()> {
    println!("📝 Initialising Reed.toml from current configuration...");
    
    // TODO: Read current values from .reed/*.csv
    // TODO: Generate Reed.toml template
    // TODO: Write to project root
    
    println!("✓ Created Reed.toml");
    println!("  Edit Reed.toml and run `reed config:sync` to apply changes");
    
    Ok(())
}

/// Syncs Reed.toml → .reed/*.csv files.
///
/// ## Command
/// `reed config:sync`
///
/// ## Process
/// 1. Parse Reed.toml
/// 2. Validate configuration
/// 3. Sync to .reed/*.csv using existing CLI commands
/// 4. Report changes
///
/// ## Output
/// ```
/// 📝 Syncing Reed.toml → .reed/*.csv...
/// ✓ Updated project.name
/// ✓ Updated project.default_language
/// ✓ Updated server.default_port
/// ✓ Synced 12 configuration values
/// ```
pub fn sync_reed_toml() -> ReedResult<()> {
    println!("📝 Syncing Reed.toml → .reed/*.csv...");
    
    // Parse Reed.toml
    let config = toml_parser::parse_reed_toml("Reed.toml")?;
    
    // Sync to CSV
    let updated_keys = toml_to_csv::sync_toml_to_csv(&config)?;
    
    // Report changes
    for key in &updated_keys {
        println!("  ✓ Updated {}", key);
    }
    
    println!("✓ Synced {} configuration values", updated_keys.len());
    
    Ok(())
}

/// Shows current merged configuration.
///
/// ## Command
/// `reed config:show [--format json|toml|table]`
///
/// ## Process
/// 1. Parse Reed.toml
/// 2. Read current values from .reed/*.csv
/// 3. Merge and display
///
/// ## Output Formats
/// - table: Human-readable table (default)
/// - json: JSON output
/// - toml: TOML output
pub fn show_config(format: &str) -> ReedResult<()> {
    let config = toml_parser::parse_reed_toml("Reed.toml")?;
    
    match format {
        "json" => {
            let json = serde_json::to_string_pretty(&config).unwrap();
            println!("{}", json);
        }
        "toml" => {
            let toml = toml::to_string_pretty(&config).unwrap();
            println!("{}", toml);
        }
        _ => {
            // Table format (default)
            println!("Project Configuration:");
            println!("  Name: {}", config.project.name);
            println!("  URL: {}", config.project.url);
            println!("  Default Language: {}", config.project.languages.default);
            println!("  Available Languages: {}", config.project.languages.available.join(", "));
            println!("\nServer Configuration:");
            println!("  Port: {}", config.server.default_port);
            println!("  Workers: {}", config.server.workers);
        }
    }
    
    Ok(())
}

/// Validates Reed.toml syntax and configuration.
///
/// ## Command
/// `reed config:validate`
///
/// ## Process
/// 1. Parse Reed.toml
/// 2. Validate structure
/// 3. Validate values
/// 4. Report errors or success
///
/// ## Output
/// ```
/// ✓ Reed.toml is valid
/// ```
///
/// or
///
/// ```
/// ✗ Validation errors:
///   - project.languages.default: Must be one of: de, en
///   - server.default_port: 1-65535
/// ```
pub fn validate_reed_toml() -> ReedResult<()> {
    println!("🔍 Validating Reed.toml...");
    
    match toml_parser::parse_reed_toml("Reed.toml") {
        Ok(_) => {
            println!("✓ Reed.toml is valid");
            Ok(())
        }
        Err(e) => {
            println!("✗ Validation error: {}", e);
            Err(e)
        }
    }
}
```

## CLI Integration

### Router Update (`src/reedcms/cli/router.rs`)

```rust
// Add to command router
"config:init" => init_reed_toml(),
"config:sync" => sync_reed_toml(),
"config:show" => show_config("table"),
"config:validate" => validate_reed_toml(),
```

### Help Text (`src/reedcms/cli/help.rs`)

```
Configuration Management:
  config:init              Create Reed.toml from current .reed/*.csv files
  config:sync              Sync Reed.toml → .reed/*.csv files
  config:show              Show current merged configuration
    --format json|toml|table   Output format (default: table)
  config:validate          Validate Reed.toml syntax and configuration
```

## Usage Examples

### Initial Setup

```bash
# Create Reed.toml from existing configuration
reed config:init

# Edit Reed.toml with preferred editor
nano Reed.toml

# Sync changes to .reed/*.csv
reed config:sync
```

### Changing Default Language

```toml
# Reed.toml
[project.languages]
default = "en"  # Changed from "de"
available = ["de", "en"]
```

```bash
reed config:sync
# ✓ Updated project.default_language
```

### Viewing Current Configuration

```bash
# Table format (human-readable)
reed config:show

# JSON format (for scripts)
reed config:show --format json

# TOML format (for comparison)
reed config:show --format toml
```

### Validating Configuration

```bash
reed config:validate
# ✓ Reed.toml is valid

# or

# ✗ Validation error: project.languages.default: Must be one of: de, en
```

## Acceptance Criteria

- [ ] Reed.toml parser implemented with toml crate
- [ ] Configuration validation with descriptive errors
- [ ] TOML to CSV sync using existing CLI commands
- [ ] `reed config:init` creates Reed.toml from current .reed/*.csv
- [ ] `reed config:sync` syncs Reed.toml → .reed/*.csv
- [ ] `reed config:show` displays current configuration (table/json/toml)
- [ ] `reed config:validate` validates Reed.toml syntax and values
- [ ] Comprehensive tests for all commands
- [ ] Documentation in man pages
- [ ] Example Reed.toml in project root

## Files Created

```
src/reedcms/config/
├── toml_parser.rs          # TOML parsing and validation
├── toml_parser.test.rs     # Tests
├── toml_to_csv.rs          # Sync TOML → CSV
├── toml_to_csv.test.rs     # Tests
└── mod.rs                  # Module exports

src/reedcms/cli/
├── config_commands.rs      # CLI commands
└── config_commands.test.rs # Tests

Reed.toml                   # Example configuration in project root
```

## Dependencies

```toml
# Cargo.toml
toml = "0.8"  # TOML parsing and serialisation
```

## Performance Impact

- TOML parsing: < 5ms for typical config
- CSV sync: < 100ms (uses existing atomic write operations)
- Validation: < 1ms
- No runtime impact (config:sync is developer-facing only)

## Future Enhancements

- [ ] `reed config:diff` - Show differences between Reed.toml and .reed/*.csv
- [ ] `reed config:watch` - Auto-sync Reed.toml on file change
- [ ] `reed config:template` - Generate Reed.toml templates for different use cases
- [ ] IDE integration (VS Code extension with TOML validation)

## Related Tickets

- **REED-06-06**: Language System Fix (uses default language from Reed.toml)
- **REED-04-02**: CLI Data Commands (reused for CSV sync)

## Related Decisions

See `_workbench/Tickets/project_optimisations.md` for:
- **Decision D051**: Reed.toml as single source of truth for project configuration
- **Decision D052**: TOML → CSV sync using existing CLI commands
- **Decision D053**: Separation between developer-facing (TOML) and system-facing (CSV) configuration
- **Decision D054**: TOML chosen over YAML for Rust-idiomatic approach and type safety

## Notes

- Reed.toml is **optional** - ReedCMS works without it
- Existing `.reed/*.csv` files remain authoritative at runtime
- Reed.toml is **developer convenience** only
- No breaking changes to existing configuration system
- Clean separation of concerns: TOML = Input, CSV = Runtime
- TOML is the Rust standard (Cargo.toml, rustfmt.toml) - familiar to developers
