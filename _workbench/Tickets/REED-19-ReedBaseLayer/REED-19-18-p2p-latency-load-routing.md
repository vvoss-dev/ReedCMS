# REED-19-18: P2P Latency Measurement & Load-Based Query Routing

**Layer**: REED-19 (ReedBase Layer)  
**Status**: Planned  
**Priority**: High  
**Complexity**: High  
**Estimated Effort**: 12-16 days  

**Dependencies**:
- REED-19-16 (Database Registry) - MUST be completed first
- REED-19-17 (Multi-Location Sync) - MUST be completed first

**Related Tickets**:
- REED-19-12 (CLI/SQL Query Interface) - uses routing system
- REED-19-10 (Smart Indices) - routed queries must support indices

---

## Problem Statement

ReedBase needs **fully decentralized P2P query routing** with:

1. **NO master node** - all nodes are equal peers
2. **Self-measured latency** - each node measures latency to all peers every 30s
3. **Local-first routing** - always use local database unless overloaded
4. **Load-based forwarding** - forward queries to nearest healthy node when local exceeds thresholds
5. **Automatic failover** - detect unhealthy nodes and route around them
6. **Configurable thresholds** - CPU (default: 80%), Memory (default: 90%)
7. **Latency awareness** - choose nearest available node for forwarded queries
8. **Health monitoring** - continuous tracking of node availability and load

**Current limitations**:
- No P2P routing infrastructure
- No latency measurement system
- No load monitoring
- No query forwarding logic
- No failover mechanism

**User expectations**:
```bash
# Query local database
rdb db:query users_prod "SELECT * FROM users"
# → Executes locally if load < threshold

# Query when local is overloaded (CPU > 80%)
rdb db:query users_prod "SELECT * FROM users"
# → Automatically forwards to nearest healthy node
# → Returns: "Query forwarded to remote1 (45ms latency)"

# Show node status
rdb db:nodes users_prod
# →
# Location    Type     Status     CPU    Memory  Latency  Last Check
# local1      local    Healthy    45%    62%     -        -
# remote1     remote   Healthy    32%    55%     45ms     2s ago
# remote2     remote   Degraded   85%    78%     120ms    2s ago
# remote3     remote   Unhealthy  -      -       timeout  30s ago
```

---

## Solution Overview

Implement a **fully decentralized P2P routing system** with:

1. **Latency measurement daemon** running on each node
2. **Load monitoring** tracking local CPU and memory usage
3. **Health check protocol** between all peers
4. **Latency table** stored locally in `.reedbase/latency.csv`
5. **Load table** stored locally in `.reedbase/load.csv`
6. **Query router** with local-first + threshold-based forwarding
7. **Failover chain** with automatic retry to next-best node
8. **Configuration system** for thresholds and measurement intervals

---

## Architecture

### Core Components

```
┌────────────────────────────────────────────────────────────┐
│ P2P Latency & Load-Based Routing Architecture             │
├────────────────────────────────────────────────────────────┤
│                                                            │
│  Each Node Runs:                                           │
│                                                            │
│  ┌────────────────────────────────────────────────────┐   │
│  │ Measurement Daemon (30s interval)                  │   │
│  │  ├─ Latency Probe → All Peers                     │   │
│  │  ├─ Load Monitor → Local CPU/Memory               │   │
│  │  └─ Health Check → Peer Availability              │   │
│  └──────────────┬─────────────────────────────────────┘   │
│                 │                                          │
│                 ▼                                          │
│  ┌────────────────────────────────────────────────────┐   │
│  │ Local Storage                                      │   │
│  │  ├─ latency.csv  (peer_id, latency_ms, timestamp) │   │
│  │  └─ load.csv     (cpu_percent, mem_percent, ts)   │   │
│  └──────────────┬─────────────────────────────────────┘   │
│                 │                                          │
│                 ▼                                          │
│  ┌────────────────────────────────────────────────────┐   │
│  │ Query Router                                       │   │
│  │  ├─ Check Local Load                              │   │
│  │  ├─ IF overloaded → Find Best Peer                │   │
│  │  ├─ Forward Query via SSH/HTTP                    │   │
│  │  └─ Return Results to Client                      │   │
│  └────────────────────────────────────────────────────┘   │
│                                                            │
└────────────────────────────────────────────────────────────┘

┌────────────────────────────────────────────────────────────┐
│ Query Routing Decision Tree                                │
├────────────────────────────────────────────────────────────┤
│                                                            │
│  Query Received                                            │
│       │                                                    │
│       ▼                                                    │
│  Check Local Load                                          │
│       │                                                    │
│       ├─ CPU < threshold AND Memory < threshold           │
│       │  → Execute Locally                                │
│       │                                                    │
│       └─ CPU ≥ threshold OR Memory ≥ threshold            │
│          → Find Best Peer                                 │
│             │                                              │
│             ├─ Filter: Status = Healthy                   │
│             ├─ Filter: Load < threshold                   │
│             ├─ Sort: By latency (ascending)               │
│             └─ Select: First available                    │
│                │                                           │
│                ├─ Peer found                              │
│                │  → Forward Query                         │
│                │  → Return Results                        │
│                │                                           │
│                └─ No peer available                       │
│                   → Execute Locally (with warning)        │
│                                                            │
└────────────────────────────────────────────────────────────┘
```

### Data Structures

```rust
/// Node status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NodeStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

/// Latency measurement entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyEntry {
    pub peer_id: String,
    pub latency_ms: f64,
    pub timestamp: String,
    pub status: NodeStatus,
}

/// Load measurement entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadEntry {
    pub cpu_percent: f32,
    pub memory_percent: f32,
    pub timestamp: String,
}

/// Node health information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeHealth {
    pub location_id: String,
    pub status: NodeStatus,
    pub latency_ms: Option<f64>,
    pub cpu_percent: Option<f32>,
    pub memory_percent: Option<f32>,
    pub last_check: String,
}

/// Routing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingConfig {
    pub database_name: String,
    pub cpu_threshold_percent: f32,      // Default: 80.0
    pub memory_threshold_percent: f32,   // Default: 90.0
    pub measurement_interval_seconds: u64, // Default: 30
    pub health_check_timeout_seconds: u64, // Default: 5
    pub unhealthy_threshold_failures: u8,  // Default: 3
}

impl Default for RoutingConfig {
    fn default() -> Self {
        Self {
            database_name: String::new(),
            cpu_threshold_percent: 80.0,
            memory_threshold_percent: 90.0,
            measurement_interval_seconds: 30,
            health_check_timeout_seconds: 5,
            unhealthy_threshold_failures: 3,
        }
    }
}

/// Query routing decision
#[derive(Debug, Clone)]
pub enum RoutingDecision {
    /// Execute locally
    Local,
    /// Forward to specific peer
    Forward {
        peer_id: String,
        latency_ms: f64,
    },
    /// No healthy peers, execute locally with warning
    LocalOverloaded {
        cpu_percent: f32,
        memory_percent: f32,
    },
}

/// Query routing result
#[derive(Debug, Clone)]
pub struct RoutingResult {
    pub decision: RoutingDecision,
    pub execution_time_ms: f64,
    pub rows_returned: usize,
    pub forwarded: bool,
}
```

---

## Implementation Plan

### Module Structure

```
reedbase/src/
├── routing/
│   ├── mod.rs                    # Module exports
│   ├── latency_probe.rs          # Latency measurement
│   ├── load_monitor.rs           # CPU/memory monitoring
│   ├── health_check.rs           # Peer health checking
│   ├── measurement_daemon.rs     # Background measurement daemon
│   ├── latency_table.rs          # Latency CSV storage
│   ├── load_table.rs             # Load CSV storage
│   ├── query_router.rs           # Query routing logic
│   ├── peer_selector.rs          # Best peer selection
│   ├── query_forwarder.rs        # Remote query execution
│   └── routing_test.rs           # Integration tests
```

---

## Detailed Implementation

### 1. Latency Probe (`latency_probe.rs`)

```rust
// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Latency measurement for P2P nodes.
//!
//! Measures round-trip latency to all peer nodes using ICMP ping
//! and SSH echo for reachability verification.

use crate::reedcms::reedstream::{ReedError, ReedResult};
use std::time::{Duration, Instant};
use std::process::Command;

/// Measure latency to remote peer.
///
/// ## Input
/// - `peer`: DatabaseLocation of peer node
/// - `timeout`: Timeout duration for measurement
///
/// ## Output
/// - `Ok(f64)`: Latency in milliseconds
/// - `Err(ReedError::RoutingError)`: Measurement failed (timeout, unreachable)
///
/// ## Performance
/// - <5s per measurement (configurable timeout)
/// - Uses ICMP ping for speed
/// - Falls back to SSH echo if ping unavailable
///
/// ## Example
/// ```rust
/// let peer = DatabaseLocation { host: Some("10.0.0.5".to_string()), ... };
/// let latency_ms = measure_latency(&peer, Duration::from_secs(5))?;
/// println!("Latency to peer: {}ms", latency_ms);
/// ```
pub fn measure_latency(
    peer: &DatabaseLocation,
    timeout: Duration,
) -> ReedResult<f64> {
    let host = peer.host.as_ref().ok_or_else(|| ReedError::RoutingError {
        message: "Cannot measure latency to local location".to_string(),
    })?;

    // Try ICMP ping first (faster)
    if let Ok(latency) = ping_latency(host, timeout) {
        return Ok(latency);
    }

    // Fallback to SSH echo
    ssh_echo_latency(host, peer.ssh_port, peer.ssh_key.as_ref(), timeout)
}

/// Measure latency using ICMP ping.
fn ping_latency(host: &str, timeout: Duration) -> ReedResult<f64> {
    let start = Instant::now();

    let output = Command::new("ping")
        .arg("-c").arg("1")
        .arg("-W").arg(timeout.as_secs().to_string())
        .arg(host)
        .output()
        .map_err(|e| ReedError::RoutingError {
            message: format!("Failed to execute ping: {}", e),
        })?;

    if !output.status.success() {
        return Err(ReedError::RoutingError {
            message: "Ping failed".to_string(),
        });
    }

    let elapsed = start.elapsed().as_secs_f64() * 1000.0;
    Ok(elapsed)
}

/// Measure latency using SSH echo.
fn ssh_echo_latency(
    host: &str,
    port: u16,
    ssh_key: Option<&PathBuf>,
    timeout: Duration,
) -> ReedResult<f64> {
    let start = Instant::now();

    let mut cmd = Command::new("ssh");
    cmd.arg("-p").arg(port.to_string());
    cmd.arg("-o").arg(format!("ConnectTimeout={}", timeout.as_secs()));
    cmd.arg("-o").arg("BatchMode=yes");

    if let Some(key) = ssh_key {
        cmd.arg("-i").arg(key);
    }

    cmd.arg(host);
    cmd.arg("echo 'pong'");

    let output = cmd.output().map_err(|e| ReedError::RoutingError {
        message: format!("Failed to execute SSH: {}", e),
    })?;

    if !output.status.success() {
        return Err(ReedError::RoutingError {
            message: "SSH echo failed".to_string(),
        });
    }

    let elapsed = start.elapsed().as_secs_f64() * 1000.0;
    Ok(elapsed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_measure_local_error() {
        let local = DatabaseLocation {
            location_id: "local1".to_string(),
            location_type: LocationType::Local,
            host: None,
            ..Default::default()
        };

        let result = measure_latency(&local, Duration::from_secs(5));
        assert!(result.is_err());
    }
}
```

---

### 2. Load Monitor (`load_monitor.rs`)

```rust
// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! System load monitoring for query routing decisions.
//!
//! Monitors local CPU and memory usage to determine if queries
//! should be executed locally or forwarded to peers.

use crate::reedcms::reedstream::{ReedError, ReedResult};
use std::fs;

/// Get current system load (CPU and memory).
///
/// ## Output
/// - `Ok((cpu_percent, memory_percent))`: Current load percentages
/// - `Err(ReedError::RoutingError)`: Failed to read system stats
///
/// ## Performance
/// - <1ms (reads from /proc filesystem on Linux)
///
/// ## Platform Support
/// - Linux: /proc/stat and /proc/meminfo
/// - macOS: sysctl (future implementation)
/// - Windows: WMI (future implementation)
///
/// ## Example
/// ```rust
/// let (cpu, mem) = get_system_load()?;
/// println!("CPU: {}%, Memory: {}%", cpu, mem);
/// ```
pub fn get_system_load() -> ReedResult<(f32, f32)> {
    #[cfg(target_os = "linux")]
    {
        let cpu = get_cpu_usage_linux()?;
        let mem = get_memory_usage_linux()?;
        Ok((cpu, mem))
    }

    #[cfg(target_os = "macos")]
    {
        let cpu = get_cpu_usage_macos()?;
        let mem = get_memory_usage_macos()?;
        Ok((cpu, mem))
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    {
        Err(ReedError::RoutingError {
            message: "System load monitoring not supported on this platform".to_string(),
        })
    }
}

#[cfg(target_os = "linux")]
fn get_cpu_usage_linux() -> ReedResult<f32> {
    use std::thread;
    use std::time::Duration;

    // Read /proc/stat twice with 100ms interval
    let stat1 = read_proc_stat()?;
    thread::sleep(Duration::from_millis(100));
    let stat2 = read_proc_stat()?;

    // Calculate CPU usage percentage
    let total_delta = (stat2.total - stat1.total) as f32;
    let idle_delta = (stat2.idle - stat1.idle) as f32;

    if total_delta == 0.0 {
        return Ok(0.0);
    }

    let usage = ((total_delta - idle_delta) / total_delta) * 100.0;
    Ok(usage.max(0.0).min(100.0))
}

#[cfg(target_os = "linux")]
fn get_memory_usage_linux() -> ReedResult<f32> {
    let meminfo = fs::read_to_string("/proc/meminfo").map_err(|e| ReedError::RoutingError {
        message: format!("Failed to read /proc/meminfo: {}", e),
    })?;

    let mut mem_total = 0u64;
    let mut mem_available = 0u64;

    for line in meminfo.lines() {
        if line.starts_with("MemTotal:") {
            mem_total = parse_meminfo_line(line)?;
        } else if line.starts_with("MemAvailable:") {
            mem_available = parse_meminfo_line(line)?;
        }
    }

    if mem_total == 0 {
        return Err(ReedError::RoutingError {
            message: "MemTotal not found in /proc/meminfo".to_string(),
        });
    }

    let mem_used = mem_total.saturating_sub(mem_available);
    let usage = (mem_used as f32 / mem_total as f32) * 100.0;
    Ok(usage.max(0.0).min(100.0))
}

#[cfg(target_os = "linux")]
struct CpuStat {
    total: u64,
    idle: u64,
}

#[cfg(target_os = "linux")]
fn read_proc_stat() -> ReedResult<CpuStat> {
    let stat = fs::read_to_string("/proc/stat").map_err(|e| ReedError::RoutingError {
        message: format!("Failed to read /proc/stat: {}", e),
    })?;

    let first_line = stat.lines().next().ok_or_else(|| ReedError::RoutingError {
        message: "/proc/stat is empty".to_string(),
    })?;

    let parts: Vec<&str> = first_line.split_whitespace().collect();
    if parts.len() < 5 {
        return Err(ReedError::RoutingError {
            message: "Invalid /proc/stat format".to_string(),
        });
    }

    let user: u64 = parts[1].parse().unwrap_or(0);
    let nice: u64 = parts[2].parse().unwrap_or(0);
    let system: u64 = parts[3].parse().unwrap_or(0);
    let idle: u64 = parts[4].parse().unwrap_or(0);

    let total = user + nice + system + idle;

    Ok(CpuStat { total, idle })
}

#[cfg(target_os = "linux")]
fn parse_meminfo_line(line: &str) -> ReedResult<u64> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 2 {
        return Err(ReedError::RoutingError {
            message: format!("Invalid meminfo line: {}", line),
        });
    }

    parts[1].parse().map_err(|e| ReedError::RoutingError {
        message: format!("Failed to parse memory value: {}", e),
    })
}

#[cfg(target_os = "macos")]
fn get_cpu_usage_macos() -> ReedResult<f32> {
    // TODO: Implement using sysctl
    Ok(0.0)
}

#[cfg(target_os = "macos")]
fn get_memory_usage_macos() -> ReedResult<f32> {
    // TODO: Implement using sysctl vm.swapusage
    Ok(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_system_load() {
        let result = get_system_load();
        #[cfg(any(target_os = "linux", target_os = "macos"))]
        {
            assert!(result.is_ok());
            let (cpu, mem) = result.unwrap();
            assert!(cpu >= 0.0 && cpu <= 100.0);
            assert!(mem >= 0.0 && mem <= 100.0);
        }
    }
}
```

---

### 3. Query Router (`query_router.rs`)

```rust
// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Query router with local-first and load-based forwarding.
//!
//! Routes queries based on local system load and peer latency.
//! Always prefers local execution unless thresholds exceeded.

use crate::reedcms::reedstream::{ReedError, ReedResult};
use std::path::PathBuf;

/// Route query based on system load and peer availability.
///
/// ## Input
/// - `database_name`: Database name
/// - `query`: ReedQL query string
/// - `config`: Routing configuration with thresholds
///
/// ## Output
/// - `Ok(RoutingResult)`: Query execution result
/// - `Err(ReedError::RoutingError)`: Routing failed
///
/// ## Performance
/// - Load check: <1ms
/// - Local execution: depends on query
/// - Remote execution: +latency overhead
///
/// ## Decision Logic
/// 1. Check local load (CPU and memory)
/// 2. IF below thresholds → execute locally
/// 3. ELSE → find best peer (healthy, low latency)
/// 4. IF peer found → forward query
/// 5. ELSE → execute locally with warning
///
/// ## Example
/// ```rust
/// let config = RoutingConfig::default();
/// let result = route_query("users_prod", "SELECT * FROM users", &config)?;
/// match result.decision {
///     RoutingDecision::Local => println!("Executed locally"),
///     RoutingDecision::Forward { peer_id, latency_ms } => {
///         println!("Forwarded to {} ({}ms)", peer_id, latency_ms);
///     }
///     RoutingDecision::LocalOverloaded { cpu_percent, memory_percent } => {
///         println!("WARNING: Local overloaded (CPU: {}%, Mem: {}%)", cpu_percent, memory_percent);
///     }
/// }
/// ```
pub fn route_query(
    database_name: &str,
    query: &str,
    config: &RoutingConfig,
) -> ReedResult<RoutingResult> {
    let start = std::time::Instant::now();

    // Step 1: Check local load
    let (cpu_percent, memory_percent) = crate::reedcms::reedbase::routing::load_monitor::get_system_load()?;

    // Step 2: Decide routing
    let decision = if cpu_percent < config.cpu_threshold_percent
        && memory_percent < config.memory_threshold_percent
    {
        // Local execution (under threshold)
        RoutingDecision::Local
    } else {
        // Find best peer
        match find_best_peer(database_name, config)? {
            Some((peer_id, latency_ms)) => RoutingDecision::Forward {
                peer_id,
                latency_ms,
            },
            None => RoutingDecision::LocalOverloaded {
                cpu_percent,
                memory_percent,
            },
        }
    };

    // Step 3: Execute based on decision
    let (rows_returned, forwarded) = match &decision {
        RoutingDecision::Local => {
            let rows = execute_local(database_name, query)?;
            (rows, false)
        }
        RoutingDecision::Forward { peer_id, .. } => {
            let rows = forward_query(database_name, query, peer_id)?;
            (rows, true)
        }
        RoutingDecision::LocalOverloaded { .. } => {
            eprintln!("WARNING: Local system overloaded, executing anyway");
            let rows = execute_local(database_name, query)?;
            (rows, false)
        }
    };

    let execution_time_ms = start.elapsed().as_secs_f64() * 1000.0;

    Ok(RoutingResult {
        decision,
        execution_time_ms,
        rows_returned,
        forwarded,
    })
}

/// Find best peer for query forwarding.
///
/// ## Selection Criteria
/// 1. Status = Healthy (not Degraded or Unhealthy)
/// 2. Load < threshold (CPU and memory)
/// 3. Lowest latency
fn find_best_peer(
    database_name: &str,
    config: &RoutingConfig,
) -> ReedResult<Option<(String, f64)>> {
    // Load latency table
    let latency_table = crate::reedcms::reedbase::routing::latency_table::load_latency_table(database_name)?;

    // Filter healthy peers with low load
    let mut candidates: Vec<(String, f64)> = latency_table
        .iter()
        .filter(|entry| entry.status == NodeStatus::Healthy)
        .filter(|entry| {
            // Check peer load (requires querying peer's load table)
            // For now, assume all healthy peers are below threshold
            // TODO: Implement remote load querying
            true
        })
        .map(|entry| (entry.peer_id.clone(), entry.latency_ms))
        .collect();

    // Sort by latency (ascending)
    candidates.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

    // Return first (lowest latency)
    Ok(candidates.first().cloned())
}

/// Execute query locally.
fn execute_local(database_name: &str, query: &str) -> ReedResult<usize> {
    // TODO: Integrate with ReedQL query executor
    // For now, return placeholder
    Ok(0)
}

/// Forward query to remote peer.
fn forward_query(database_name: &str, query: &str, peer_id: &str) -> ReedResult<usize> {
    // TODO: Implement query forwarding via SSH or HTTP API
    // For now, return placeholder
    Ok(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_route_query_local() {
        let config = RoutingConfig {
            database_name: "test".to_string(),
            cpu_threshold_percent: 100.0, // Never forward
            memory_threshold_percent: 100.0,
            ..Default::default()
        };

        let result = route_query("test", "SELECT * FROM users", &config);
        // Test will fail until execute_local is implemented
        // assert!(result.is_ok());
    }
}
```

---

### 4. Measurement Daemon (`measurement_daemon.rs`)

```rust
// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Background daemon for continuous latency and load measurement.
//!
//! Runs as a background process, measuring latency to all peers
//! and local system load at configurable intervals (default: 30s).

use crate::reedcms::reedstream::{ReedError, ReedResult};
use std::thread;
use std::time::Duration;
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};

/// Start measurement daemon in background.
///
/// ## Input
/// - `database_name`: Database name
/// - `config`: Routing configuration
///
/// ## Output
/// - `Ok(DaemonHandle)`: Handle to control daemon
/// - `Err(ReedError::RoutingError)`: Failed to start daemon
///
/// ## Behavior
/// - Spawns background thread
/// - Measures latency to all peers every N seconds
/// - Measures local load every N seconds
/// - Writes results to latency.csv and load.csv
/// - Continues until stopped
///
/// ## Example
/// ```rust
/// let config = RoutingConfig::default();
/// let handle = start_measurement_daemon("users_prod", config)?;
/// 
/// // ... do other work ...
/// 
/// handle.stop()?;
/// ```
pub fn start_measurement_daemon(
    database_name: &str,
    config: RoutingConfig,
) -> ReedResult<DaemonHandle> {
    let running = Arc::new(AtomicBool::new(true));
    let running_clone = running.clone();
    let db_name = database_name.to_string();

    let thread = thread::spawn(move || {
        measurement_loop(&db_name, &config, running_clone);
    });

    Ok(DaemonHandle {
        thread: Some(thread),
        running,
    })
}

/// Daemon handle for control.
pub struct DaemonHandle {
    thread: Option<thread::JoinHandle<()>>,
    running: Arc<AtomicBool>,
}

impl DaemonHandle {
    /// Stop the daemon.
    pub fn stop(mut self) -> ReedResult<()> {
        self.running.store(false, Ordering::Relaxed);

        if let Some(thread) = self.thread.take() {
            thread.join().map_err(|_| ReedError::RoutingError {
                message: "Failed to join daemon thread".to_string(),
            })?;
        }

        Ok(())
    }

    /// Check if daemon is running.
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }
}

/// Main measurement loop.
fn measurement_loop(database_name: &str, config: &RoutingConfig, running: Arc<AtomicBool>) {
    while running.load(Ordering::Relaxed) {
        // Measure latency to all peers
        if let Err(e) = measure_all_peers(database_name, config) {
            eprintln!("ERROR: Latency measurement failed: {:?}", e);
        }

        // Measure local load
        if let Err(e) = measure_local_load(database_name) {
            eprintln!("ERROR: Load measurement failed: {:?}", e);
        }

        // Sleep until next measurement
        thread::sleep(Duration::from_secs(config.measurement_interval_seconds));
    }
}

/// Measure latency to all peers and update latency table.
fn measure_all_peers(database_name: &str, config: &RoutingConfig) -> ReedResult<()> {
    // Load registry to get all locations
    let registry = crate::reedcms::reedbase::registry::load_registry()?;
    let db_entry = registry.find_database(database_name)?;

    // Get all remote locations
    let locations = crate::reedcms::reedbase::registry::get_database_locations(&db_entry)?;
    let remote_locations: Vec<_> = locations.iter()
        .filter(|loc| loc.location_type == LocationType::Remote)
        .collect();

    let mut latency_entries = Vec::new();
    let timeout = Duration::from_secs(config.health_check_timeout_seconds);

    for location in remote_locations {
        let latency_result = crate::reedcms::reedbase::routing::latency_probe::measure_latency(
            location,
            timeout,
        );

        let (latency_ms, status) = match latency_result {
            Ok(latency) => {
                let status = if latency < 200.0 {
                    NodeStatus::Healthy
                } else {
                    NodeStatus::Degraded
                };
                (latency, status)
            }
            Err(_) => {
                // Unreachable
                (0.0, NodeStatus::Unhealthy)
            }
        };

        latency_entries.push(LatencyEntry {
            peer_id: location.location_id.clone(),
            latency_ms,
            timestamp: chrono::Utc::now().to_rfc3339(),
            status,
        });
    }

    // Write to latency table
    crate::reedcms::reedbase::routing::latency_table::write_latency_table(
        database_name,
        &latency_entries,
    )?;

    Ok(())
}

/// Measure local load and update load table.
fn measure_local_load(database_name: &str) -> ReedResult<()> {
    let (cpu_percent, memory_percent) = crate::reedcms::reedbase::routing::load_monitor::get_system_load()?;

    let entry = LoadEntry {
        cpu_percent,
        memory_percent,
        timestamp: chrono::Utc::now().to_rfc3339(),
    };

    crate::reedcms::reedbase::routing::load_table::append_load_entry(database_name, &entry)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_daemon_start_stop() {
        let config = RoutingConfig {
            database_name: "test".to_string(),
            measurement_interval_seconds: 1, // Fast for testing
            ..Default::default()
        };

        let handle = start_measurement_daemon("test", config).unwrap();
        assert!(handle.is_running());

        thread::sleep(Duration::from_millis(100));
        assert!(handle.is_running());

        handle.stop().unwrap();
    }
}
```

---

## CLI Commands

### 1. Start Measurement Daemon

```bash
# Start daemon for database
rdb db:measure:start users_prod

# Start with custom interval
rdb db:measure:start users_prod --interval 60

# Result:
✓ Measurement daemon started for 'users_prod'
  PID: 12345
  Interval: 30s
  Thresholds: CPU 80%, Memory 90%
  Log: ~/.reedbase/routing/users_prod.log
```

### 2. Show Node Status

```bash
# Show all nodes for database
rdb db:nodes users_prod

# Result:
Database: users_prod
Measurement Interval: 30s
Thresholds: CPU 80%, Memory 90%

Location    Type     Status     CPU    Memory  Latency  Last Check
────────────────────────────────────────────────────────────────────
local1      local    Healthy    45%    62%     -        -
remote1     remote   Healthy    32%    55%     45ms     2s ago
remote2     remote   Degraded   85%    78%     120ms    2s ago
remote3     remote   Unhealthy  -      -       timeout  30s ago
remote4     remote   Healthy    28%    45%     35ms     2s ago
────────────────────────────────────────────────────────────────────

Summary:
  Total: 5 nodes
  Healthy: 3 (60%)
  Degraded: 1 (20%)
  Unhealthy: 1 (20%)
```

### 3. Configure Routing Thresholds

```bash
# Set custom thresholds
rdb db:routing:config users_prod --cpu-threshold 85 --memory-threshold 95

# Set measurement interval
rdb db:routing:config users_prod --interval 60

# Result:
✓ Routing configuration updated for 'users_prod'
  CPU Threshold: 85%
  Memory Threshold: 95%
  Measurement Interval: 60s
```

### 4. Manual Query with Routing Info

```bash
# Execute query with routing details
rdb db:query users_prod "SELECT * FROM users" --verbose

# Result (local execution):
Routing Decision: Local
  CPU: 45% (threshold: 80%)
  Memory: 62% (threshold: 90%)

Query: SELECT * FROM users
Rows: 1,234
Execution Time: 12.3ms

# Result (forwarded):
Routing Decision: Forwarded to remote1
  Reason: Local CPU 85% exceeds threshold 80%
  Remote Latency: 45ms
  Remote Load: CPU 32%, Memory 55%

Query: SELECT * FROM users
Rows: 1,234
Execution Time: 67.8ms (45ms network + 22.8ms query)
```

### 5. View Latency History

```bash
# Show latency measurements for past hour
rdb db:latency users_prod --hours 1

# Result:
Latency History: users_prod (past 1 hour)

remote1:
  Current: 45ms (Healthy)
  Average: 47ms
  Min: 42ms (10:23:15)
  Max: 55ms (10:45:30)
  
remote2:
  Current: 120ms (Degraded)
  Average: 118ms
  Min: 110ms (10:15:00)
  Max: 135ms (10:32:45)
```

---

## Testing Strategy

### Unit Tests
- Latency measurement (mocked ping/SSH)
- Load monitoring (CPU/memory reading)
- Query routing logic (decision tree)
- Peer selection (filtering and sorting)

### Integration Tests
- End-to-end routing (local + remote execution)
- Daemon lifecycle (start/stop/restart)
- Latency table updates (CSV writes)
- Load table updates (CSV appends)

### Performance Tests
- Measurement overhead (<100ms per cycle)
- Query forwarding latency (baseline + overhead)
- Daemon CPU usage (<1% average)

---

## Performance Targets

| Operation | Target | Measurement |
|-----------|--------|-------------|
| Load check | <1ms | Time to read CPU/memory |
| Latency measurement (ping) | <50ms | Round-trip time |
| Latency measurement (SSH) | <200ms | Round-trip time |
| Routing decision | <5ms | Time to select peer |
| Query forwarding overhead | <10% | Latency penalty |
| Daemon CPU usage | <1% | Average over 1 hour |
| Measurement cycle | <30s | All peers + local |

---

## Documentation Requirements

1. **User Guide**:
   - Routing concepts (local-first, load-based)
   - Threshold configuration
   - Node status interpretation
   - Troubleshooting connectivity issues

2. **Administrator Guide**:
   - Daemon management
   - Performance tuning (thresholds, intervals)
   - Network requirements (ICMP, SSH)
   - Monitoring and alerting

3. **Developer Guide**:
   - Routing architecture
   - Peer selection algorithm
   - Query forwarding protocol
   - Testing routing logic

---

## Error Conditions

| Error | Cause | Resolution |
|-------|-------|------------|
| `RoutingError: Failed to measure latency` | Network unreachable | Check connectivity, firewall rules |
| `RoutingError: No healthy peers available` | All peers down/overloaded | Execute locally with warning |
| `RoutingError: Load monitoring not supported` | Unsupported OS | Implement platform-specific load reading |
| `RoutingError: Latency table not found` | Daemon not started | Start measurement daemon first |
| `RoutingError: Query forwarding failed` | SSH/API error | Check remote connectivity |

---

## Dependencies

**External crates**:
- `chrono = "0.4"` - Timestamp generation

**Internal modules**:
- `reedbase::registry` - Database locations
- `reedbase::sync` - Location information
- `reedstream` - Error types

**System dependencies**:
- `ping` command (ICMP)
- `ssh` command (SSH echo)
- `/proc` filesystem (Linux load monitoring)
- `sysctl` command (macOS load monitoring)

---

## Metrics & Observability

### Performance Metrics

| Metric | Type | Unit | Target | P99 Alert | Collection Point |
|--------|------|------|--------|-----------|------------------|
| peer_latency | Histogram | ms | <50 | >200 | latency.rs:measure() |
| query_forward_rate | Gauge | % | <20 | >50 | router.rs:route() |
| local_cpu_percent | Gauge | % | <80 | >90 | load.rs:measure_cpu() |
| local_memory_percent | Gauge | % | <90 | >95 | load.rs:measure_memory() |
| routing_decision_time | Histogram | μs | <100 | >1000 | router.rs:select_peer() |
| measurement_errors | Counter | count | <5% | >20% | latency.rs:measure() |

### Alert Rules

**CRITICAL Alerts:**
- `local_cpu_percent > 90%` for 5 minutes → "Local node overloaded - queries will forward"
- `query_forward_rate > 50%` for 10 minutes → "Excessive query forwarding - local node issues"

**WARNING Alerts:**
- `peer_latency p99 > 200ms` for 10 minutes → "High peer latency - network issues"
- `routing_decision_time p99 > 1ms` for 5 minutes → "Router slow - check peer count"

### Implementation

```rust
use crate::reedbase::metrics::global as metrics;
use std::time::Instant;

pub fn route_query(query: &Query) -> ReedResult<QueryResult> {
    let start = Instant::now();
    
    let local_load = measure_local_load()?;
    let should_forward = local_load.cpu > CPU_THRESHOLD || local_load.memory > MEMORY_THRESHOLD;
    
    let result = if should_forward {
        let peer = select_best_peer()?;
        forward_to_peer(&peer, query)?
    } else {
        execute_locally(query)?
    };
    
    metrics().record(Metric {
        name: "routing_decision_time".to_string(),
        value: start.elapsed().as_nanos() as f64 / 1000.0,
        unit: MetricUnit::Microseconds,
        tags: hashmap!{ "forwarded" => should_forward.to_string() },
    });
    
    let forward_rate = if should_forward { 100.0 } else { 0.0 };
    metrics().record(Metric {
        name: "query_forward_rate".to_string(),
        value: forward_rate,
        unit: MetricUnit::Percent,
        tags: hashmap!{},
    });
    
    Ok(result)
}

pub fn measure_latency(peer: &Peer) -> ReedResult<Duration> {
    let start = Instant::now();
    let latency = ping_peer(peer)?;
    
    metrics().record(Metric {
        name: "peer_latency".to_string(),
        value: latency.as_millis() as f64,
        unit: MetricUnit::Milliseconds,
        tags: hashmap!{ "peer" => &peer.name },
    });
    
    Ok(latency)
}
```

### Collection Strategy

- **Sampling**: All operations
- **Aggregation**: 1-minute rolling window
- **Storage**: `.reedbase/metrics/p2p.csv`
- **Retention**: 7 days raw, 90 days aggregated

### Why These Metrics Matter

**peer_latency**: Network performance
- Directly affects query forwarding decisions
- High latency = slow remote queries
- Helps identify network issues

**query_forward_rate**: Load distribution
- Low rate (<20%) = healthy local node
- High rate = local node overloaded or failing
- Indicates when to scale local resources

**local_cpu_percent**: Resource monitoring
- Triggers query forwarding when high
- Prevents local node degradation
- Guides capacity planning

**routing_decision_time**: Routing overhead
- Should be negligible (<100μs)
- Slow decisions add latency to all queries
- Indicates algorithmic or data structure issues

## Acceptance Criteria

- [ ] Latency measurement using ping and SSH echo
- [ ] Load monitoring for CPU and memory (Linux + macOS)
- [ ] Query router with local-first logic
- [ ] Threshold-based forwarding decision
- [ ] Peer selection by latency and status
- [ ] Measurement daemon runs in background
- [ ] Latency table stored in .reedbase/latency.csv
- [ ] Load table stored in .reedbase/load.csv
- [ ] CLI commands for daemon control
- [ ] CLI commands for node status display
- [ ] Configurable thresholds and intervals
- [ ] All tests pass (unit, integration, performance)
- [ ] Documentation complete (user + admin guides)

---

## Future Enhancements

- **HTTP API for query forwarding**: Replace SSH with REST API
- **Load sharing**: Distribute queries across multiple peers
- **Predictive routing**: ML-based load prediction
- **Geographic awareness**: Consider datacenter location
- **Cost optimization**: Route to cheaper compute regions
- **Caching layer**: Cache query results across nodes
- **Real-time dashboard**: Web UI for live node monitoring
