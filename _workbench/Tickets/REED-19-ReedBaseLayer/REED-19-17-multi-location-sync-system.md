# REED-19-17: Multi-Location Sync System

**Layer**: REED-19 (ReedBase Layer)  
**Status**: Planned  
**Priority**: High  
**Complexity**: High  
**Estimated Effort**: 10-14 days  

**Dependencies**:
- REED-19-16 (Database Registry & Name Resolution) - MUST be completed first

**Related Tickets**:
- REED-19-18 (P2P Latency Measurement) - uses sync infrastructure
- REED-19-16 (Database Registry) - stores location information

---

## Problem Statement

ReedBase needs to support **multi-location database deployment** with automatic synchronisation across local and remote instances. The system must:

1. **Support array syntax** for specifying multiple locations during initialisation
2. **Handle interactive prompts** for each location's configuration
3. **Detect and install** ReedBase on remote systems (global or local mode)
4. **Synchronise database files** efficiently using rsync over SSH
5. **Support multiple sync topologies** (Hub-Spoke, Mesh, Custom)
6. **Integrate with the registry** to track all locations for each database
7. **Provide daemon process** for continuous synchronisation
8. **Handle network failures** gracefully with retry logic

**Current limitations**:
- No multi-location support (single database instance only)
- No remote synchronisation mechanism
- No topology configuration
- No sync daemon or automatic updates

**User expectations**:
```bash
# Array syntax for multiple locations
rdb db:init users_prod --global --local[3] --remote[8]

# Interactive prompts for each location
# → 3 prompts for local paths
# → 8 prompts for remote configurations (IP, path, SSH key)

# Result: Database synchronised across 12 locations
# Registry stores all locations with their metadata
```

---

## Solution Overview

Implement a **rsync-based multi-location sync system** with:

1. **Array syntax parser** for `--local[N]` and `--remote[N]` flags
2. **Interactive location collector** with validation and testing
3. **Remote detection system** to check ReedBase installation
4. **Remote installation logic** (global vs local mode)
5. **Sync topology manager** (Hub-Spoke, Mesh, Custom)
6. **Rsync-based file synchronisation** with delta compression
7. **Sync daemon process** with configurable intervals
8. **Registry integration** for location tracking
9. **Health monitoring** and automatic failover

---

## Architecture

### Core Components

```
┌────────────────────────────────────────────────────────────┐
│ Multi-Location Sync System Architecture                   │
├────────────────────────────────────────────────────────────┤
│                                                            │
│  User Command                                              │
│       │                                                    │
│       ▼                                                    │
│  Array Parser ──► Location Collector ──► Remote Detector  │
│       │                    │                     │        │
│       │                    ▼                     ▼        │
│       │            Interactive Prompts    Installation    │
│       │                    │              (global/local)  │
│       │                    ▼                     │        │
│       └──────► Topology Manager ◄───────────────┘        │
│                       │                                    │
│                       ▼                                    │
│              Sync Orchestrator                             │
│                       │                                    │
│         ┌─────────────┼─────────────┐                     │
│         ▼             ▼             ▼                     │
│    Rsync Engine   Daemon Mgr   Health Monitor            │
│         │             │             │                     │
│         └─────────────┴─────────────┘                     │
│                       │                                    │
│                       ▼                                    │
│              Database Registry                             │
│                                                            │
└────────────────────────────────────────────────────────────┘
```

### Data Structures

```rust
/// Location type (local or remote)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LocationType {
    Local,
    Remote,
}

/// Database location entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseLocation {
    pub location_id: String,
    pub location_type: LocationType,
    pub host: Option<String>,        // None for local, Some(IP) for remote
    pub path: PathBuf,
    pub ssh_key: Option<PathBuf>,    // None for local
    pub ssh_port: u16,               // Default: 22
    pub reedbase_mode: DatabaseMode, // Global or Local
    pub reedbase_version: String,
    pub created_at: String,
    pub last_sync: Option<String>,
    pub sync_status: SyncStatus,
}

/// Sync status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SyncStatus {
    Healthy,
    Degraded,
    Unreachable,
    Syncing,
}

/// Sync topology configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncTopology {
    /// Hub-Spoke: One primary location syncs to all others
    HubSpoke {
        hub_location_id: String,
    },
    /// Mesh: Every location syncs to every other location
    Mesh,
    /// Custom: User-defined sync pairs
    Custom {
        sync_pairs: Vec<(String, String)>, // (source, target)
    },
}

/// Sync configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConfig {
    pub database_name: String,
    pub topology: SyncTopology,
    pub sync_interval_seconds: u64, // Default: 60
    pub retry_attempts: u8,         // Default: 3
    pub retry_delay_seconds: u64,   // Default: 5
    pub rsync_options: Vec<String>, // Custom rsync flags
}

/// Array syntax parsed result
#[derive(Debug, Clone)]
pub struct LocationSpec {
    pub local_count: usize,
    pub remote_count: usize,
}

/// Location prompt response
#[derive(Debug, Clone)]
pub struct LocationInput {
    pub location_type: LocationType,
    pub host: Option<String>,
    pub path: PathBuf,
    pub ssh_key: Option<PathBuf>,
    pub ssh_port: u16,
    pub reedbase_mode: DatabaseMode,
}
```

---

## Implementation Plan

### Module Structure

```
src/reedcms/reedbase/
├── sync/
│   ├── mod.rs                    # Module exports
│   ├── array_parser.rs           # Parse --local[N] --remote[N]
│   ├── location_collector.rs     # Interactive location prompts
│   ├── remote_detector.rs        # Detect ReedBase on remote hosts
│   ├── remote_installer.rs       # Install ReedBase remotely
│   ├── topology_manager.rs       # Manage sync topologies
│   ├── rsync_engine.rs           # Rsync-based file sync
│   ├── sync_orchestrator.rs      # Coordinate sync operations
│   ├── daemon.rs                 # Background sync daemon
│   ├── health_monitor.rs         # Monitor location health
│   └── sync_test.rs              # Integration tests
```

---

## Detailed Implementation

### 1. Array Parser (`array_parser.rs`)

```rust
// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Array syntax parser for multi-location specifications.
//!
//! Parses command-line flags like `--local[3]` and `--remote[8]` into
//! structured LocationSpec with counts for interactive prompts.

use crate::reedcms::reedstream::{ReedError, ReedResult};
use std::collections::HashMap;

/// Parse array syntax from CLI arguments.
///
/// ## Input
/// - `args`: CLI arguments iterator
///
/// ## Output
/// - `Ok(LocationSpec)`: Parsed location counts
/// - `Err(ReedError::SyncError)`: Invalid syntax
///
/// ## Example
/// ```bash
/// --local[3] --remote[8]
/// → LocationSpec { local_count: 3, remote_count: 8 }
/// ```
pub fn parse_array_syntax(args: &[String]) -> ReedResult<LocationSpec> {
    let mut local_count = 0;
    let mut remote_count = 0;

    for arg in args {
        if let Some(count) = parse_flag_array(arg, "--local")? {
            local_count = count;
        } else if let Some(count) = parse_flag_array(arg, "--remote")? {
            remote_count = count;
        }
    }

    if local_count == 0 && remote_count == 0 {
        return Err(ReedError::SyncError {
            message: "At least one location (--local[N] or --remote[N]) required".to_string(),
        });
    }

    Ok(LocationSpec {
        local_count,
        remote_count,
    })
}

/// Parse single array flag like `--local[3]`.
fn parse_flag_array(arg: &str, flag: &str) -> ReedResult<Option<usize>> {
    if !arg.starts_with(flag) {
        return Ok(None);
    }

    // Format: --local[3] or --local (default: 1)
    if arg == flag {
        return Ok(Some(1));
    }

    // Extract number from brackets
    let bracket_start = arg.find('[').ok_or_else(|| ReedError::SyncError {
        message: format!("Invalid array syntax: {}", arg),
    })?;

    let bracket_end = arg.find(']').ok_or_else(|| ReedError::SyncError {
        message: format!("Invalid array syntax: {}", arg),
    })?;

    let count_str = &arg[bracket_start + 1..bracket_end];
    let count = count_str.parse::<usize>().map_err(|_| ReedError::SyncError {
        message: format!("Invalid count in array syntax: {}", count_str),
    })?;

    if count == 0 {
        return Err(ReedError::SyncError {
            message: "Location count must be at least 1".to_string(),
        });
    }

    if count > 99 {
        return Err(ReedError::SyncError {
            message: "Location count cannot exceed 99".to_string(),
        });
    }

    Ok(Some(count))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_local_array() {
        let args = vec!["--local[3]".to_string()];
        let spec = parse_array_syntax(&args).unwrap();
        assert_eq!(spec.local_count, 3);
        assert_eq!(spec.remote_count, 0);
    }

    #[test]
    fn test_parse_remote_array() {
        let args = vec!["--remote[8]".to_string()];
        let spec = parse_array_syntax(&args).unwrap();
        assert_eq!(spec.local_count, 0);
        assert_eq!(spec.remote_count, 8);
    }

    #[test]
    fn test_parse_both_arrays() {
        let args = vec!["--local[3]".to_string(), "--remote[8]".to_string()];
        let spec = parse_array_syntax(&args).unwrap();
        assert_eq!(spec.local_count, 3);
        assert_eq!(spec.remote_count, 8);
    }

    #[test]
    fn test_parse_default_single() {
        let args = vec!["--local".to_string()];
        let spec = parse_array_syntax(&args).unwrap();
        assert_eq!(spec.local_count, 1);
    }

    #[test]
    fn test_parse_invalid_syntax() {
        let args = vec!["--local[".to_string()];
        assert!(parse_array_syntax(&args).is_err());
    }

    #[test]
    fn test_parse_invalid_count() {
        let args = vec!["--local[abc]".to_string()];
        assert!(parse_array_syntax(&args).is_err());
    }

    #[test]
    fn test_parse_zero_count() {
        let args = vec!["--local[0]".to_string()];
        assert!(parse_array_syntax(&args).is_err());
    }

    #[test]
    fn test_parse_too_many_locations() {
        let args = vec!["--local[100]".to_string()];
        assert!(parse_array_syntax(&args).is_err());
    }
}
```

---

### 2. Location Collector (`location_collector.rs`)

```rust
// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Interactive location collector with validation.
//!
//! Prompts user for location details (path, SSH config) for each
//! local and remote location specified in array syntax.

use crate::reedcms::reedstream::{ReedError, ReedResult};
use std::io::{self, Write};
use std::path::PathBuf;

/// Collect location inputs interactively.
///
/// ## Input
/// - `spec`: LocationSpec with local/remote counts
///
/// ## Output
/// - `Ok(Vec<LocationInput>)`: All collected location inputs
/// - `Err(ReedError::SyncError)`: Invalid input or validation failure
///
/// ## Performance
/// - Interactive: No timing constraints
/// - Validates SSH connectivity before accepting remote locations
///
/// ## Example
/// ```rust
/// let spec = LocationSpec { local_count: 2, remote_count: 1 };
/// let locations = collect_locations(&spec)?;
/// ```
pub fn collect_locations(spec: &LocationSpec) -> ReedResult<Vec<LocationInput>> {
    let mut locations = Vec::new();

    // Collect local locations
    for i in 1..=spec.local_count {
        println!("\n=== Local Location {}/{} ===", i, spec.local_count);
        let input = collect_local_location()?;
        locations.push(input);
    }

    // Collect remote locations
    for i in 1..=spec.remote_count {
        println!("\n=== Remote Location {}/{} ===", i, spec.remote_count);
        let input = collect_remote_location()?;
        locations.push(input);
    }

    Ok(locations)
}

/// Collect single local location.
fn collect_local_location() -> ReedResult<LocationInput> {
    let path = prompt_path("Enter local path (e.g., /opt/reedbase/users_prod):")?;

    // Validate path exists or can be created
    if !path.exists() {
        let parent = path.parent().ok_or_else(|| ReedError::SyncError {
            message: format!("Invalid path: {}", path.display()),
        })?;

        if !parent.exists() {
            return Err(ReedError::SyncError {
                message: format!("Parent directory does not exist: {}", parent.display()),
            });
        }
    }

    Ok(LocationInput {
        location_type: LocationType::Local,
        host: None,
        path,
        ssh_key: None,
        ssh_port: 22,
        reedbase_mode: DatabaseMode::Local,
    })
}

/// Collect single remote location.
fn collect_remote_location() -> ReedResult<LocationInput> {
    let host = prompt("Enter remote host (IP or hostname):")?;

    // Ask for mode: global or local
    let mode_input = prompt("ReedBase mode (global/local) [default: global]:")?;
    let reedbase_mode = match mode_input.to_lowercase().as_str() {
        "local" | "l" => DatabaseMode::Local,
        _ => DatabaseMode::Global,
    };

    // Ask for path (only if local mode, or custom path for global)
    let path = if reedbase_mode == DatabaseMode::Local {
        prompt_path("Enter remote path (e.g., /home/user/project/.reedbase):")?
    } else {
        prompt_path("Enter database path (optional, press Enter for default):")?
    };

    // SSH configuration
    let ssh_key_str = prompt("Enter SSH key path (optional, press Enter for default):")?;
    let ssh_key = if ssh_key_str.is_empty() {
        None
    } else {
        Some(PathBuf::from(ssh_key_str))
    };

    let ssh_port_str = prompt("Enter SSH port [default: 22]:")?;
    let ssh_port = if ssh_port_str.is_empty() {
        22
    } else {
        ssh_port_str.parse().map_err(|_| ReedError::SyncError {
            message: "Invalid SSH port number".to_string(),
        })?
    };

    // Test SSH connectivity
    test_ssh_connection(&host, ssh_port, ssh_key.as_ref())?;

    Ok(LocationInput {
        location_type: LocationType::Remote,
        host: Some(host),
        path,
        ssh_key,
        ssh_port,
        reedbase_mode,
    })
}

/// Prompt user for string input.
fn prompt(message: &str) -> ReedResult<String> {
    print!("{} ", message);
    io::stdout().flush().map_err(|e| ReedError::SyncError {
        message: format!("Failed to flush stdout: {}", e),
    })?;

    let mut input = String::new();
    io::stdin().read_line(&mut input).map_err(|e| ReedError::SyncError {
        message: format!("Failed to read input: {}", e),
    })?;

    Ok(input.trim().to_string())
}

/// Prompt user for path input.
fn prompt_path(message: &str) -> ReedResult<PathBuf> {
    let input = prompt(message)?;
    Ok(PathBuf::from(input))
}

/// Test SSH connection to remote host.
fn test_ssh_connection(
    host: &str,
    port: u16,
    ssh_key: Option<&PathBuf>,
) -> ReedResult<()> {
    use std::process::Command;

    println!("Testing SSH connection to {}:{}...", host, port);

    let mut cmd = Command::new("ssh");
    cmd.arg("-p").arg(port.to_string());
    cmd.arg("-o").arg("ConnectTimeout=5");
    cmd.arg("-o").arg("BatchMode=yes");

    if let Some(key) = ssh_key {
        cmd.arg("-i").arg(key);
    }

    cmd.arg(host);
    cmd.arg("echo 'Connection successful'");

    let output = cmd.output().map_err(|e| ReedError::SyncError {
        message: format!("Failed to test SSH connection: {}", e),
    })?;

    if !output.status.success() {
        return Err(ReedError::SyncError {
            message: format!(
                "SSH connection test failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ),
        });
    }

    println!("✓ SSH connection successful");
    Ok(())
}
```

---

### 3. Remote Detector (`remote_detector.rs`)

```rust
// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Remote ReedBase detection system.
//!
//! Detects if ReedBase is installed on remote host and determines
//! installation mode (global or local).

use crate::reedcms::reedstream::{ReedError, ReedResult};
use std::path::PathBuf;
use std::process::Command;

/// Detection result
#[derive(Debug, Clone, PartialEq)]
pub enum DetectionResult {
    /// ReedBase found globally installed
    GlobalInstalled { version: String },
    /// ReedBase found locally in project
    LocalInstalled { version: String, path: PathBuf },
    /// ReedBase not found
    NotInstalled,
}

/// Detect ReedBase installation on remote host.
///
/// ## Input
/// - `host`: Remote hostname or IP
/// - `port`: SSH port
/// - `ssh_key`: Optional SSH key path
///
/// ## Output
/// - `Ok(DetectionResult)`: Detection result with version info
/// - `Err(ReedError::SyncError)`: SSH connection failed
///
/// ## Performance
/// - <5s typical (SSH connection + command execution)
///
/// ## Example
/// ```rust
/// let result = detect_remote_reedbase("10.0.0.5", 22, None)?;
/// match result {
///     DetectionResult::GlobalInstalled { version } => println!("Found v{}", version),
///     DetectionResult::NotInstalled => println!("Need to install"),
///     _ => {}
/// }
/// ```
pub fn detect_remote_reedbase(
    host: &str,
    port: u16,
    ssh_key: Option<&PathBuf>,
) -> ReedResult<DetectionResult> {
    // Check for global installation
    if let Ok(version) = check_global_reedbase(host, port, ssh_key) {
        return Ok(DetectionResult::GlobalInstalled { version });
    }

    // Check for local installation (if path provided)
    // TODO: Implement local detection logic

    Ok(DetectionResult::NotInstalled)
}

/// Check for globally installed ReedBase.
fn check_global_reedbase(
    host: &str,
    port: u16,
    ssh_key: Option<&PathBuf>,
) -> ReedResult<String> {
    let mut cmd = Command::new("ssh");
    cmd.arg("-p").arg(port.to_string());
    cmd.arg("-o").arg("ConnectTimeout=5");
    cmd.arg("-o").arg("BatchMode=yes");

    if let Some(key) = ssh_key {
        cmd.arg("-i").arg(key);
    }

    cmd.arg(host);
    cmd.arg("rdb --version");

    let output = cmd.output().map_err(|e| ReedError::SyncError {
        message: format!("Failed to execute remote command: {}", e),
    })?;

    if !output.status.success() {
        return Err(ReedError::SyncError {
            message: "ReedBase not found globally".to_string(),
        });
    }

    let version_output = String::from_utf8_lossy(&output.stdout);
    let version = parse_version(&version_output)?;

    Ok(version)
}

/// Parse version from `rdb --version` output.
fn parse_version(output: &str) -> ReedResult<String> {
    // Expected format: "rdb 1.0.0" or "ReedBase 1.0.0"
    let parts: Vec<&str> = output.trim().split_whitespace().collect();
    
    if parts.len() < 2 {
        return Err(ReedError::SyncError {
            message: format!("Invalid version output: {}", output),
        });
    }

    Ok(parts[1].to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_version() {
        let output = "rdb 1.0.0\n";
        let version = parse_version(output).unwrap();
        assert_eq!(version, "1.0.0");
    }

    #[test]
    fn test_parse_version_reedbase() {
        let output = "ReedBase 2.3.1\n";
        let version = parse_version(output).unwrap();
        assert_eq!(version, "2.3.1");
    }

    #[test]
    fn test_parse_version_invalid() {
        let output = "invalid\n";
        assert!(parse_version(output).is_err());
    }
}
```

---

### 4. Rsync Engine (`rsync_engine.rs`)

```rust
// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Rsync-based file synchronisation engine.
//!
//! Performs efficient delta-based synchronisation between database locations
//! using rsync over SSH for remote hosts.

use crate::reedcms::reedstream::{ReedError, ReedResult};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Sync direction
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SyncDirection {
    Push,
    Pull,
}

/// Rsync options
#[derive(Debug, Clone)]
pub struct RsyncOptions {
    pub archive: bool,           // -a flag
    pub compress: bool,          // -z flag
    pub delete: bool,            // --delete flag
    pub verbose: bool,           // -v flag
    pub exclude: Vec<String>,    // --exclude patterns
    pub custom: Vec<String>,     // Custom flags
}

impl Default for RsyncOptions {
    fn default() -> Self {
        Self {
            archive: true,
            compress: true,
            delete: true,
            verbose: false,
            exclude: vec![
                ".git/".to_string(),
                "*.tmp".to_string(),
                "*.lock".to_string(),
            ],
            custom: vec![],
        }
    }
}

/// Synchronise database between two locations.
///
/// ## Input
/// - `source`: Source location
/// - `target`: Target location
/// - `direction`: Push or Pull
/// - `options`: Rsync options
///
/// ## Output
/// - `Ok(SyncStats)`: Synchronisation statistics
/// - `Err(ReedError::SyncError)`: Rsync failed
///
/// ## Performance
/// - Depends on file size and network speed
/// - Rsync delta compression: typically <10% of full size
/// - LAN: ~100MB/s, WAN: depends on bandwidth
///
/// ## Example
/// ```rust
/// let source = DatabaseLocation { /* local */ };
/// let target = DatabaseLocation { /* remote */ };
/// let stats = sync_database(&source, &target, SyncDirection::Push, &RsyncOptions::default())?;
/// println!("Synced {} bytes in {}s", stats.bytes_transferred, stats.duration_seconds);
/// ```
pub fn sync_database(
    source: &DatabaseLocation,
    target: &DatabaseLocation,
    direction: SyncDirection,
    options: &RsyncOptions,
) -> ReedResult<SyncStats> {
    let start = std::time::Instant::now();

    let mut cmd = Command::new("rsync");

    // Add standard flags
    if options.archive {
        cmd.arg("-a");
    }
    if options.compress {
        cmd.arg("-z");
    }
    if options.delete {
        cmd.arg("--delete");
    }
    if options.verbose {
        cmd.arg("-v");
    }

    // Add exclude patterns
    for pattern in &options.exclude {
        cmd.arg("--exclude").arg(pattern);
    }

    // Add custom flags
    for flag in &options.custom {
        cmd.arg(flag);
    }

    // Add stats flag
    cmd.arg("--stats");

    // Build source and target paths
    let (source_path, target_path) = match direction {
        SyncDirection::Push => (
            build_path(source),
            build_path(target),
        ),
        SyncDirection::Pull => (
            build_path(target),
            build_path(source),
        ),
    };

    cmd.arg(&source_path);
    cmd.arg(&target_path);

    // Execute rsync
    let output = cmd.output().map_err(|e| ReedError::SyncError {
        message: format!("Failed to execute rsync: {}", e),
    })?;

    if !output.status.success() {
        return Err(ReedError::SyncError {
            message: format!(
                "Rsync failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ),
        });
    }

    // Parse rsync stats
    let stats_output = String::from_utf8_lossy(&output.stdout);
    let stats = parse_rsync_stats(&stats_output, start.elapsed().as_secs_f64())?;

    Ok(stats)
}

/// Build rsync path string (local or remote).
fn build_path(location: &DatabaseLocation) -> String {
    match &location.host {
        Some(host) => {
            // Remote path: user@host:path
            let ssh_spec = if location.ssh_port != 22 {
                format!("-p {}", location.ssh_port)
            } else {
                String::new()
            };

            format!("{}:{}", host, location.path.display())
        }
        None => {
            // Local path
            location.path.display().to_string()
        }
    }
}

/// Synchronisation statistics
#[derive(Debug, Clone)]
pub struct SyncStats {
    pub bytes_transferred: u64,
    pub files_transferred: u32,
    pub duration_seconds: f64,
    pub speed_mbps: f64,
}

/// Parse rsync stats from output.
fn parse_rsync_stats(output: &str, duration: f64) -> ReedResult<SyncStats> {
    let mut bytes_transferred = 0u64;
    let mut files_transferred = 0u32;

    for line in output.lines() {
        if line.contains("Total transferred file size:") {
            if let Some(bytes_str) = line.split(':').nth(1) {
                let bytes_str = bytes_str.trim().replace(",", "");
                bytes_transferred = bytes_str.parse().unwrap_or(0);
            }
        }
        if line.contains("Number of files transferred:") {
            if let Some(files_str) = line.split(':').nth(1) {
                let files_str = files_str.trim();
                files_transferred = files_str.parse().unwrap_or(0);
            }
        }
    }

    let speed_mbps = if duration > 0.0 {
        (bytes_transferred as f64 / 1_000_000.0) / duration
    } else {
        0.0
    };

    Ok(SyncStats {
        bytes_transferred,
        files_transferred,
        duration_seconds: duration,
        speed_mbps,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_local_path() {
        let location = DatabaseLocation {
            location_id: "local1".to_string(),
            location_type: LocationType::Local,
            host: None,
            path: PathBuf::from("/opt/reedbase/users"),
            ssh_key: None,
            ssh_port: 22,
            reedbase_mode: DatabaseMode::Global,
            reedbase_version: "1.0.0".to_string(),
            created_at: "2025-01-01".to_string(),
            last_sync: None,
            sync_status: SyncStatus::Healthy,
        };

        let path = build_path(&location);
        assert_eq!(path, "/opt/reedbase/users");
    }

    #[test]
    fn test_build_remote_path() {
        let location = DatabaseLocation {
            location_id: "remote1".to_string(),
            location_type: LocationType::Remote,
            host: Some("10.0.0.5".to_string()),
            path: PathBuf::from("/home/user/reedbase/users"),
            ssh_key: None,
            ssh_port: 22,
            reedbase_mode: DatabaseMode::Global,
            reedbase_version: "1.0.0".to_string(),
            created_at: "2025-01-01".to_string(),
            last_sync: None,
            sync_status: SyncStatus::Healthy,
        };

        let path = build_path(&location);
        assert_eq!(path, "10.0.0.5:/home/user/reedbase/users");
    }
}
```

---

### 5. Sync Orchestrator (`sync_orchestrator.rs`)

```rust
// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Sync orchestrator for topology-based synchronisation.
//!
//! Coordinates synchronisation operations across multiple locations
//! based on configured topology (Hub-Spoke, Mesh, Custom).

use crate::reedcms::reedstream::{ReedError, ReedResult};
use std::collections::HashMap;

/// Orchestrate synchronisation based on topology.
///
/// ## Input
/// - `database_name`: Database name
/// - `locations`: All registered locations
/// - `config`: Sync configuration with topology
///
/// ## Output
/// - `Ok(SyncReport)`: Summary of all sync operations
/// - `Err(ReedError::SyncError)`: Critical sync failure
///
/// ## Performance
/// - Hub-Spoke: O(N) time, N locations
/// - Mesh: O(N²) time, N×(N-1) sync pairs
/// - Custom: O(P) time, P = number of pairs
///
/// ## Example
/// ```rust
/// let config = SyncConfig {
///     topology: SyncTopology::HubSpoke { hub_location_id: "local1".to_string() },
///     ..Default::default()
/// };
/// let report = orchestrate_sync("users_prod", &locations, &config)?;
/// ```
pub fn orchestrate_sync(
    database_name: &str,
    locations: &[DatabaseLocation],
    config: &SyncConfig,
) -> ReedResult<SyncReport> {
    let start = std::time::Instant::now();
    let mut sync_operations = Vec::new();

    match &config.topology {
        SyncTopology::HubSpoke { hub_location_id } => {
            sync_operations = orchestrate_hub_spoke(locations, hub_location_id, config)?;
        }
        SyncTopology::Mesh => {
            sync_operations = orchestrate_mesh(locations, config)?;
        }
        SyncTopology::Custom { sync_pairs } => {
            sync_operations = orchestrate_custom(locations, sync_pairs, config)?;
        }
    }

    let duration = start.elapsed().as_secs_f64();

    Ok(SyncReport {
        database_name: database_name.to_string(),
        topology: config.topology.clone(),
        operations: sync_operations,
        total_duration_seconds: duration,
        timestamp: chrono::Utc::now().to_rfc3339(),
    })
}

/// Hub-Spoke: Hub syncs to all spokes.
fn orchestrate_hub_spoke(
    locations: &[DatabaseLocation],
    hub_id: &str,
    config: &SyncConfig,
) -> ReedResult<Vec<SyncOperation>> {
    let hub = locations.iter().find(|loc| loc.location_id == hub_id)
        .ok_or_else(|| ReedError::SyncError {
            message: format!("Hub location not found: {}", hub_id),
        })?;

    let mut operations = Vec::new();

    for spoke in locations.iter().filter(|loc| loc.location_id != hub_id) {
        let result = sync_with_retry(hub, spoke, SyncDirection::Push, config);
        operations.push(SyncOperation {
            source_id: hub.location_id.clone(),
            target_id: spoke.location_id.clone(),
            direction: SyncDirection::Push,
            result,
        });
    }

    Ok(operations)
}

/// Mesh: Every location syncs to every other location.
fn orchestrate_mesh(
    locations: &[DatabaseLocation],
    config: &SyncConfig,
) -> ReedResult<Vec<SyncOperation>> {
    let mut operations = Vec::new();

    for i in 0..locations.len() {
        for j in 0..locations.len() {
            if i == j {
                continue;
            }

            let source = &locations[i];
            let target = &locations[j];

            let result = sync_with_retry(source, target, SyncDirection::Push, config);
            operations.push(SyncOperation {
                source_id: source.location_id.clone(),
                target_id: target.location_id.clone(),
                direction: SyncDirection::Push,
                result,
            });
        }
    }

    Ok(operations)
}

/// Custom: User-defined sync pairs.
fn orchestrate_custom(
    locations: &[DatabaseLocation],
    sync_pairs: &[(String, String)],
    config: &SyncConfig,
) -> ReedResult<Vec<SyncOperation>> {
    let mut operations = Vec::new();
    let location_map: HashMap<String, &DatabaseLocation> = locations.iter()
        .map(|loc| (loc.location_id.clone(), loc))
        .collect();

    for (source_id, target_id) in sync_pairs {
        let source = location_map.get(source_id).ok_or_else(|| ReedError::SyncError {
            message: format!("Source location not found: {}", source_id),
        })?;

        let target = location_map.get(target_id).ok_or_else(|| ReedError::SyncError {
            message: format!("Target location not found: {}", target_id),
        })?;

        let result = sync_with_retry(source, target, SyncDirection::Push, config);
        operations.push(SyncOperation {
            source_id: source_id.clone(),
            target_id: target_id.clone(),
            direction: SyncDirection::Push,
            result,
        });
    }

    Ok(operations)
}

/// Sync with automatic retry logic.
fn sync_with_retry(
    source: &DatabaseLocation,
    target: &DatabaseLocation,
    direction: SyncDirection,
    config: &SyncConfig,
) -> Result<SyncStats, String> {
    let mut attempts = 0;
    let mut last_error = String::new();

    while attempts < config.retry_attempts {
        match crate::reedcms::reedbase::sync::rsync_engine::sync_database(
            source,
            target,
            direction,
            &RsyncOptions::default(),
        ) {
            Ok(stats) => return Ok(stats),
            Err(e) => {
                last_error = format!("{:?}", e);
                attempts += 1;
                if attempts < config.retry_attempts {
                    std::thread::sleep(std::time::Duration::from_secs(config.retry_delay_seconds));
                }
            }
        }
    }

    Err(format!("Failed after {} attempts: {}", attempts, last_error))
}

/// Sync report
#[derive(Debug, Clone)]
pub struct SyncReport {
    pub database_name: String,
    pub topology: SyncTopology,
    pub operations: Vec<SyncOperation>,
    pub total_duration_seconds: f64,
    pub timestamp: String,
}

/// Single sync operation result
#[derive(Debug, Clone)]
pub struct SyncOperation {
    pub source_id: String,
    pub target_id: String,
    pub direction: SyncDirection,
    pub result: Result<SyncStats, String>,
}
```

---

## CLI Commands

### 1. Initialize Database with Multiple Locations

```bash
# Array syntax
rdb db:init users_prod --global --local[3] --remote[8]

# Interactive prompts follow:
# === Local Location 1/3 ===
# Enter local path: /opt/reedbase/users_prod
#
# === Local Location 2/3 ===
# ...
#
# === Remote Location 1/8 ===
# Enter remote host: 10.0.0.5
# ReedBase mode (global/local) [default: global]: global
# Enter SSH key path (optional): ~/.ssh/id_rsa
# Enter SSH port [default: 22]: 22
# Testing SSH connection to 10.0.0.5:22...
# ✓ SSH connection successful
# ...

# Result:
✓ Database 'users_prod' initialized at 12 locations
✓ Registry updated: ~/.reedbase/registry.toml
✓ Sync configured: Hub-Spoke (hub: local1)
```

### 2. Configure Sync Topology

```bash
# Hub-Spoke (default)
rdb db:sync:config users_prod --topology hub-spoke --hub local1

# Mesh (all-to-all)
rdb db:sync:config users_prod --topology mesh

# Custom pairs
rdb db:sync:config users_prod --topology custom --pairs "local1→remote1,local1→remote2"
```

### 3. Manual Sync

```bash
# Sync once
rdb db:sync users_prod

# Sync with verbose output
rdb db:sync users_prod --verbose

# Result:
Syncing database 'users_prod'...
Topology: Hub-Spoke (hub: local1)

[1/11] local1 → remote1 ... ✓ 2.4 MB transferred in 1.2s (2.0 MB/s)
[2/11] local1 → remote2 ... ✓ 0.8 MB transferred in 0.5s (1.6 MB/s)
...
[11/11] local1 → remote8 ... ✓ 1.2 MB transferred in 0.8s (1.5 MB/s)

✓ Sync completed: 11 operations, 18.6 MB total, 12.3s
```

### 4. Start Sync Daemon

```bash
# Start background sync daemon
rdb db:sync:start users_prod --interval 60

# Result:
✓ Sync daemon started for 'users_prod'
  PID: 12345
  Interval: 60s
  Log: ~/.reedbase/sync/users_prod.log
```

### 5. Monitor Sync Status

```bash
# Show sync status
rdb db:sync:status users_prod

# Result:
Database: users_prod
Topology: Hub-Spoke (hub: local1)
Daemon: Running (PID: 12345)
Interval: 60s
Last sync: 2025-01-14 10:23:45 UTC (23s ago)

Location Status:
  local1   (hub)    Healthy    -        -         -
  local2   (spoke)  Healthy    12ms     1.2MB     23s ago
  local3   (spoke)  Healthy    8ms      0.8MB     23s ago
  remote1  (spoke)  Healthy    45ms     2.4MB     23s ago
  remote2  (spoke)  Degraded   120ms    1.1MB     5m ago
  ...
```

---

## Testing Strategy

### Unit Tests
- Array syntax parsing (valid/invalid inputs)
- Location validation (paths, SSH config)
- Rsync command building (local/remote paths)
- Stats parsing (rsync output)

### Integration Tests
- Multi-location initialization workflow
- Remote detection (mocked SSH)
- Sync orchestration (all topologies)
- Daemon lifecycle (start/stop/restart)

### Performance Tests
- Rsync speed (LAN vs WAN)
- Mesh topology scaling (N²)
- Retry logic overhead

---

## Performance Targets

| Operation | Target | Measurement |
|-----------|--------|-------------|
| Array syntax parsing | <1ms | Time to parse |
| SSH connection test | <5s | Time to verify |
| Rsync LAN speed | >100MB/s | Throughput |
| Rsync WAN speed | >1MB/s | Throughput |
| Hub-Spoke sync (10 nodes) | <30s | Total time |
| Mesh sync (5 nodes) | <60s | Total time |
| Daemon startup | <100ms | Time to start |

---

## Documentation Requirements

1. **User Guide**:
   - Multi-location initialization workflow
   - Topology selection guide
   - SSH configuration examples
   - Troubleshooting sync issues

2. **Administrator Guide**:
   - Remote installation procedures
   - Sync topology design patterns
   - Performance tuning (rsync options)
   - Network requirements

3. **Developer Guide**:
   - Rsync integration architecture
   - Topology orchestration logic
   - Retry and error handling
   - Testing sync operations

---

## Error Conditions

| Error | Cause | Resolution |
|-------|-------|------------|
| `SyncError: Invalid array syntax` | Malformed `--local[N]` flag | Fix syntax: `--local[3]` |
| `SyncError: SSH connection failed` | Network or auth issue | Check SSH config, test manually |
| `SyncError: ReedBase not found` | Not installed on remote | Install ReedBase first |
| `SyncError: Rsync failed` | Permission or path issue | Check paths and permissions |
| `SyncError: Hub location not found` | Invalid hub ID | Verify hub location exists |

---

## Dependencies

**External crates**:
- None (uses standard `std::process::Command` for SSH and rsync)

**Internal modules**:
- `reedbase::registry` - Database registry
- `reedbase::db::init` - Database initialization
- `reedstream` - Error types

**System dependencies**:
- `ssh` command (OpenSSH client)
- `rsync` command (rsync 3.0+)

---

## Acceptance Criteria

- [ ] Array syntax parser handles `--local[N]` and `--remote[N]`
- [ ] Interactive prompts collect all location details
- [ ] SSH connectivity tested before accepting remote locations
- [ ] Remote ReedBase detection works (global and local modes)
- [ ] Rsync engine syncs files with delta compression
- [ ] Hub-Spoke topology implemented and tested
- [ ] Mesh topology implemented and tested
- [ ] Custom topology supports arbitrary sync pairs
- [ ] Sync daemon runs in background with configurable interval
- [ ] Sync status command shows location health
- [ ] Retry logic with exponential backoff
- [ ] All tests pass (unit, integration, performance)
- [ ] Documentation complete (user + admin guides)

---

## Future Enhancements

- **Conflict resolution**: Detect and resolve file conflicts
- **Compression options**: Allow user to configure rsync compression level
- **Bandwidth limiting**: Add `--bwlimit` option for rate limiting
- **Sync scheduling**: Cron-like scheduling for daemon
- **Webhook notifications**: Notify on sync completion or failure
- **Web UI**: Visual sync status dashboard
