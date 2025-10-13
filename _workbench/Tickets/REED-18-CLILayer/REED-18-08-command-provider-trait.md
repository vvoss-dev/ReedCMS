# REED-18-08: Command Provider Trait

## MANDATORY Development Standards

**CRITICAL**: Every implementation MUST follow these standards:

- **Language**: All documentation and code comments in BBC English
- **Principle**: KISS (Keep It Simple, Stupid) - minimal code and professional inline documentation
- **File Naming**: File name = Unique theme - crystal clear what single topic this file handles
- **Files**: One file = One responsibility
- **Functions**: One function = One distinctive job
- **Shared Functions**: Same patterns = One shared function
- **Testing**: Separate test files as `{name}_test.rs` - never inline `#[cfg(test)]` modules
- **Avoid**: Avoid Swiss Army knife functions
- **Avoid**: Generic file names like `handler.rs`, `utils.rs`
- **Templates**: See `_workbench/Tickets/templates/service-template.md` for complete implementation guide
- **Testing**: See `_workbench/Tickets/templates/service-template.test.md` for test structure

## Ticket Information
- **ID**: REED-18-08
- **Title**: Command Provider Trait
- **Layer**: CLI Layer (REED-18)
- **Priority**: Critical
- **Status**: Open
- **Complexity**: Medium
- **Dependencies**: REED-18-01 (Command Parser), REED-18-02 (Registry Loader)
- **Estimated Time**: 2 days

## Objective

Create the **CommandProvider trait** as the standard interface for adapters (reedbase, reedcms) to register their CLI commands with the router. This establishes the "docking mechanism" for all CLI commands.

## Requirements

### Core Functionality

- **Trait definition** for command registration
- **Router extension** to support command registration from adapters
- **CommandHandler type** that adapters can use
- **Auto-discovery support** via Rust feature flags
- **Metadata access** (adapter name, version, description)

### Architecture

```
┌─────────────────────────────────────────┐
│ reed binary (main.rs)                   │
│ - Discovers adapters via Cargo features │
│ - Calls .register_commands() on each    │
└─────────────────────────────────────────┘
                 ↓
┌─────────────────────────────────────────┐
│ CommandProvider Trait (reedcli)         │
│ - register_commands(&self, router)      │
│ - name() → &str                          │
│ - version() → &str                       │
└─────────────────────────────────────────┘
                 ↓
┌──────────────────────┬──────────────────┐
│ ReedBase Adapter     │ ReedCMS Adapter  │
│ impl CommandProvider │ impl...          │
│ - Registers 25 cmds  │ - Registers 37   │
└──────────────────────┴──────────────────┘
```

## Implementation Files

### Primary Implementation

**`reedcli/src/provider.rs`**

One file = CommandProvider trait only. NO other responsibilities.

```rust
// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Command provider trait for adapter registration.
//!
//! Defines the interface that adapters (reedbase, reedcms) must implement
//! to register their CLI commands with the router.

use crate::router::Router;
use crate::types::CliResult;

/// Command provider trait for adapters.
///
/// Adapters implement this trait to register their commands with the CLI router.
/// This creates a standardised "docking mechanism" for all CLI commands.
///
/// ## Implementation Requirements
/// - `register_commands()`: Register all commands with router
/// - `name()`: Return adapter identifier (e.g., "reedbase", "reedcms")
/// - `version()`: Return adapter version (use `env!("CARGO_PKG_VERSION")`)
///
/// ## Example Implementation
/// ```rust
/// use reedcli::CommandProvider;
///
/// pub struct ReedBaseAdapter;
///
/// impl CommandProvider for ReedBaseAdapter {
///     fn register_commands(&self, router: &mut Router) {
///         router.register("set", "text", crate::commands::set_text);
///         router.register("get", "text", crate::commands::get_text);
///     }
///     
///     fn name(&self) -> &str { "reedbase" }
///     fn version(&self) -> &str { env!("CARGO_PKG_VERSION") }
/// }
/// ```
pub trait CommandProvider {
    /// Register all commands with the router.
    ///
    /// ## Input
    /// - `router`: Mutable router to register commands with
    ///
    /// ## Performance
    /// - O(n) where n = number of commands
    /// - Target: < 1ms for 100 commands
    ///
    /// ## Error Conditions
    /// - None (registration is infallible)
    ///
    /// ## Example Usage
    /// ```rust
    /// let adapter = ReedBaseAdapter::new();
    /// let mut router = Router::new();
    /// adapter.register_commands(&mut router);
    /// ```
    fn register_commands(&self, router: &mut Router);
    
    /// Return adapter name.
    ///
    /// ## Output
    /// - `&str`: Adapter identifier (lowercase, no spaces)
    ///
    /// ## Performance
    /// - O(1) operation
    /// - < 1μs typical
    ///
    /// ## Example Usage
    /// ```rust
    /// assert_eq!(adapter.name(), "reedbase");
    /// ```
    fn name(&self) -> &str;
    
    /// Return adapter version.
    ///
    /// ## Output
    /// - `&str`: Semantic version (e.g., "1.0.0")
    ///
    /// ## Performance
    /// - O(1) operation
    /// - < 1μs typical
    ///
    /// ## Example Usage
    /// ```rust
    /// assert_eq!(adapter.version(), "1.0.0");
    /// ```
    fn version(&self) -> &str;
    
    /// Return adapter description (optional).
    ///
    /// ## Output
    /// - `&str`: Human-readable description
    ///
    /// ## Performance
    /// - O(1) operation
    /// - < 1μs typical
    ///
    /// ## Example Usage
    /// ```rust
    /// assert_eq!(adapter.description(), "ReedBase database operations");
    /// ```
    fn description(&self) -> &str {
        ""
    }
}

/// Discover and register all available adapters.
///
/// ## Input
/// - `router`: Mutable router to register commands with
///
/// ## Output
/// - `CliResult<Vec<String>>`: List of registered adapter names
///
/// ## Performance
/// - O(n) where n = number of adapters
/// - Target: < 5ms for all adapters
///
/// ## Error Conditions
/// - None (individual adapter failures are logged, not fatal)
///
/// ## Example Usage
/// ```rust
/// let mut router = Router::new();
/// let adapters = discover_adapters(&mut router)?;
/// println!("Registered: {}", adapters.join(", "));
/// ```
pub fn discover_adapters(router: &mut Router) -> CliResult<Vec<String>> {
    let mut registered = Vec::new();
    
    // ReedBase adapter
    #[cfg(feature = "reedbase")]
    {
        let adapter = reedbase::ReedBaseAdapter::new();
        adapter.register_commands(router);
        registered.push(adapter.name().to_string());
    }
    
    // ReedCMS adapter
    #[cfg(feature = "reedcms")]
    {
        let adapter = reedcms::ReedCMSAdapter::new();
        adapter.register_commands(router);
        registered.push(adapter.name().to_string());
    }
    
    Ok(registered)
}
```

**`reedcli/src/router.rs`** (additions)

Extend existing router with adapter support:

```rust
// Add to existing router.rs

/// Router with command handlers.
pub struct Router {
    handlers: HashMap<(String, String), CommandHandler>,
    adapter_metadata: Vec<AdapterMetadata>, // NEW
}

/// Adapter metadata for tracking registered adapters.
#[derive(Debug, Clone)]
pub struct AdapterMetadata {
    pub name: String,
    pub version: String,
    pub description: String,
    pub command_count: usize,
}

impl Router {
    // ... existing methods ...
    
    /// Register commands from an adapter.
    ///
    /// ## Input
    /// - `adapter`: CommandProvider implementation
    ///
    /// ## Performance
    /// - O(n) where n = number of commands in adapter
    /// - Target: < 1ms for 100 commands
    ///
    /// ## Example Usage
    /// ```rust
    /// let mut router = Router::new();
    /// let adapter = ReedBaseAdapter::new();
    /// router.register_adapter(&adapter);
    /// ```
    pub fn register_adapter<P: CommandProvider>(&mut self, adapter: &P) {
        let initial_count = self.handlers.len();
        
        adapter.register_commands(self);
        
        let command_count = self.handlers.len() - initial_count;
        
        self.adapter_metadata.push(AdapterMetadata {
            name: adapter.name().to_string(),
            version: adapter.version().to_string(),
            description: adapter.description().to_string(),
            command_count,
        });
    }
    
    /// List registered adapters.
    ///
    /// ## Output
    /// - `&[AdapterMetadata]`: Adapter metadata slice
    ///
    /// ## Performance
    /// - O(1) operation
    /// - < 1μs typical
    ///
    /// ## Example Usage
    /// ```rust
    /// for adapter in router.adapters() {
    ///     println!("{} v{}: {} commands", 
    ///         adapter.name, adapter.version, adapter.command_count);
    /// }
    /// ```
    pub fn adapters(&self) -> &[AdapterMetadata] {
        &self.adapter_metadata
    }
}
```

**`reedcli/src/lib.rs`** (additions)

```rust
// Add to existing lib.rs
pub mod provider;

// Re-export for convenience
pub use provider::{CommandProvider, discover_adapters};
pub use router::{Router, AdapterMetadata};
```

### Test Files

**`reedcli/src/provider_test.rs`**

Separate test file. Each test = one specific behaviour.

```rust
// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

#[cfg(test)]
mod tests {
    use super::*;
    use crate::router::Router;
    use crate::types::CliResult;
    
    // Mock adapter for testing
    struct MockAdapter {
        name: &'static str,
        version: &'static str,
    }
    
    impl CommandProvider for MockAdapter {
        fn register_commands(&self, router: &mut Router) {
            router.register("test", "command", |_args, _flags| {
                Ok(ReedResponse::ok("test"))
            });
        }
        
        fn name(&self) -> &str {
            self.name
        }
        
        fn version(&self) -> &str {
            self.version
        }
    }
    
    #[test]
    fn test_adapter_registration() {
        let mut router = Router::new();
        let adapter = MockAdapter {
            name: "test-adapter",
            version: "1.0.0",
        };
        
        router.register_adapter(&adapter);
        
        // Verify command was registered
        assert!(router.has_command("test", "command"));
        
        // Verify metadata was recorded
        let adapters = router.adapters();
        assert_eq!(adapters.len(), 1);
        assert_eq!(adapters[0].name, "test-adapter");
        assert_eq!(adapters[0].version, "1.0.0");
        assert_eq!(adapters[0].command_count, 1);
    }
    
    #[test]
    fn test_multiple_adapter_registration() {
        let mut router = Router::new();
        
        let adapter1 = MockAdapter {
            name: "adapter1",
            version: "1.0.0",
        };
        
        let adapter2 = MockAdapter {
            name: "adapter2",
            version: "2.0.0",
        };
        
        router.register_adapter(&adapter1);
        router.register_adapter(&adapter2);
        
        let adapters = router.adapters();
        assert_eq!(adapters.len(), 2);
        assert_eq!(adapters[0].name, "adapter1");
        assert_eq!(adapters[1].name, "adapter2");
    }
    
    #[test]
    fn test_adapter_metadata() {
        let adapter = MockAdapter {
            name: "test",
            version: "1.2.3",
        };
        
        assert_eq!(adapter.name(), "test");
        assert_eq!(adapter.version(), "1.2.3");
        assert_eq!(adapter.description(), ""); // Default empty
    }
}
```

## Performance Requirements

| Operation | Target |
|-----------|--------|
| Register single command | < 10μs |
| Register 100 commands | < 1ms |
| Discover all adapters | < 5ms |
| Get adapter metadata | < 1μs |

## Error Conditions

None - registration is infallible. Individual adapter failures can be logged but do not prevent CLI startup.

## Integration Pattern

**`src/main.rs`** (example usage):

```rust
use reedcli::{Router, discover_adapters, run_with_router};

fn main() {
    let mut router = Router::new();
    
    // Auto-discover and register all adapters
    match discover_adapters(&mut router) {
        Ok(adapters) => {
            println!("Registered adapters: {}", adapters.join(", "));
        }
        Err(e) => {
            eprintln!("Warning: Adapter discovery failed: {}", e);
        }
    }
    
    // Run CLI with populated router
    if let Err(e) = run_with_router(router, std::env::args().collect()) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
```

## Acceptance Criteria

- [ ] CommandProvider trait with 4 methods (register_commands, name, version, description)
- [ ] Router.register_adapter() method
- [ ] Router.adapters() method for metadata access
- [ ] AdapterMetadata struct tracks name/version/command_count
- [ ] discover_adapters() function with feature flag support
- [ ] Mock adapter test implementation
- [ ] Test: Single adapter registration
- [ ] Test: Multiple adapter registration
- [ ] Test: Adapter metadata tracking
- [ ] Test: Command count tracking
- [ ] All tests pass with 100% coverage
- [ ] Performance targets met
- [ ] All code in BBC English
- [ ] All functions have complete documentation
- [ ] No Swiss Army knife functions
- [ ] Separate test file as `provider_test.rs`

## Dependencies

**Requires**: 
- REED-18-01 (Command Parser - for command routing)
- REED-18-02 (Registry Loader - router structure)

**Blocks**: 
- REED-18-09 (ReedBase Adapter)
- REED-18-10 (ReedCMS Adapter)

## References
- Service Template: `_workbench/Tickets/templates/service-template.md`
- Service Test Template: `_workbench/Tickets/templates/service-template.test.md`
- REED-18-00: Layer Overview

## Notes

**Design Philosophy**:
- **Simple trait** - only 4 methods needed
- **Cargo features** - adapters enabled/disabled at compile time
- **Metadata tracking** - know which adapters are active
- **Infallible registration** - missing adapters don't crash CLI

**NOT in scope**:
- Dynamic loading (plugins) - compile-time only
- Command conflicts - first registration wins
- Adapter dependencies - handled by Cargo

**Future enhancements** (NOT this ticket):
- Adapter health checks
- Command namespacing/conflicts
- Hot-reload of adapters
