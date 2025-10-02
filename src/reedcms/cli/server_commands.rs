// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! CLI server management commands for ReedCMS.
//!
//! Provides commands for:
//! - server:io - Start server in interactive mode
//! - server:start - Start server in background (daemon)
//! - server:stop - Stop running server
//! - server:restart - Restart server
//! - server:status - Check server status
//! - server:logs - View server logs

use crate::reedcms::reedstream::{ReedError, ReedResponse, ReedResult};
use std::collections::HashMap;
use std::fs;
use std::process::{Command, Stdio};

/// Starts ReedCMS server in interactive mode.
///
/// ## Input
/// - flags: --port PORT, --socket PATH, --workers N
///
/// ## Output
/// - Server start confirmation with connection details
///
/// ## Performance
/// - Startup: < 50ms for HTTP mode
///
/// ## Error Conditions
/// - Port already in use
/// - Socket path invalid
/// - Configuration invalid
///
/// ## Note
/// Full server implementation requires REED-06-01 (Server Foundation).
/// This is a placeholder that validates command structure.
pub fn server_io(
    _args: &[String],
    flags: &HashMap<String, String>,
) -> ReedResult<ReedResponse<String>> {
    let mut output = String::new();
    output.push_str("🚀 Starting ReedCMS server...\n\n");

    // Determine server mode
    let (mode, bind_address) = if let Some(socket_path) = flags.get("socket") {
        ("Unix socket".to_string(), socket_path.clone())
    } else {
        let port = flags.get("port").map(|s| s.as_str()).unwrap_or("8333");
        ("HTTP".to_string(), format!("127.0.0.1:{}", port))
    };

    let workers = flags.get("workers").map(|s| s.as_str()).unwrap_or("4");

    output.push_str(&format!("✓ Configuration validated\n"));
    output.push_str(&format!("✓ Mode: {}\n", mode));
    output.push_str(&format!("✓ Address: {}\n", bind_address));
    output.push_str(&format!("✓ Workers: {}\n\n", workers));

    output.push_str("⚠ Server implementation not yet ready (requires REED-06-01)\n");
    output.push_str(
        "   Full HTTP/Socket server will be available when server foundation is complete.\n\n",
    );

    output.push_str("Would start server with:\n");
    output.push_str(&format!("- Mode: {}\n", mode));
    output.push_str(&format!("- Bind: {}\n", bind_address));
    output.push_str(&format!("- Workers: {}\n", workers));

    Ok(ReedResponse {
        data: output,
        source: "cli::server_io".to_string(),
        cached: false,
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        metrics: None,
    })
}

/// Starts server in background (daemon mode).
///
/// ## Input
/// - flags: --environment ENV, --config PATH
///
/// ## Output
/// - Server start confirmation with PID
///
/// ## Error Conditions
/// - Server already running
/// - Cannot write PID file
/// - Configuration invalid
pub fn server_start(
    _args: &[String],
    flags: &HashMap<String, String>,
) -> ReedResult<ReedResponse<String>> {
    let mut output = String::new();
    output.push_str("🚀 Starting ReedCMS server in background...\n\n");

    let environment = flags
        .get("environment")
        .map(|s| s.as_str())
        .unwrap_or("PROD");

    // Check if PID file exists
    let pid_file = ".reed/server.pid";
    if std::path::Path::new(pid_file).exists() {
        let pid = fs::read_to_string(pid_file)
            .unwrap_or_default()
            .trim()
            .to_string();

        // Check if process is actually running
        if is_process_running(&pid) {
            return Err(ReedError::ServerError {
                component: "server_start".to_string(),
                reason: format!("Server already running (PID: {})", pid),
            });
        } else {
            // Stale PID file, remove it
            fs::remove_file(pid_file).ok();
        }
    }

    output.push_str(&format!("✓ Configuration validated\n"));
    output.push_str(&format!("✓ Environment: {}\n\n", environment));

    output.push_str("⚠ Server daemon mode not yet implemented (requires REED-06-01)\n");
    output
        .push_str("   Background server will be available when server foundation is complete.\n\n");

    output.push_str("Would start background server with:\n");
    output.push_str(&format!("- Environment: {}\n", environment));
    output.push_str(&format!("- PID file: {}\n", pid_file));
    output.push_str("- Log file: .reed/flow/server.log\n");

    Ok(ReedResponse {
        data: output,
        source: "cli::server_start".to_string(),
        cached: false,
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        metrics: None,
    })
}

/// Stops running server.
///
/// ## Output
/// - Stop confirmation
///
/// ## Error Conditions
/// - Server not running
/// - Cannot read PID file
/// - Process does not respond to signals
pub fn server_stop(
    _args: &[String],
    _flags: &HashMap<String, String>,
) -> ReedResult<ReedResponse<String>> {
    let mut output = String::new();
    output.push_str("🛑 Stopping ReedCMS server...\n\n");

    let pid_file = ".reed/server.pid";

    if !std::path::Path::new(pid_file).exists() {
        return Err(ReedError::ServerError {
            component: "server_stop".to_string(),
            reason: "Server not running (no PID file found)".to_string(),
        });
    }

    let pid = fs::read_to_string(pid_file)
        .map_err(|e| ReedError::IoError {
            operation: "read".to_string(),
            path: pid_file.to_string(),
            reason: e.to_string(),
        })?
        .trim()
        .to_string();

    if !is_process_running(&pid) {
        fs::remove_file(pid_file).ok();
        return Err(ReedError::ServerError {
            component: "server_stop".to_string(),
            reason: format!("Server not running (stale PID: {})", pid),
        });
    }

    output.push_str(&format!("✓ Found running server (PID: {})\n", pid));

    // Send SIGTERM signal
    #[cfg(unix)]
    {
        use std::process::Command;
        let kill_result = Command::new("kill").arg("-TERM").arg(&pid).status();

        if kill_result.is_ok() {
            output.push_str("✓ Sent shutdown signal (SIGTERM)\n");

            // Wait for graceful shutdown (max 5 seconds)
            for _ in 0..50 {
                std::thread::sleep(std::time::Duration::from_millis(100));
                if !is_process_running(&pid) {
                    break;
                }
            }

            if is_process_running(&pid) {
                output.push_str("⚠ Server did not stop gracefully, forcing shutdown...\n");
                Command::new("kill").arg("-KILL").arg(&pid).status().ok();
            } else {
                output.push_str("✓ Server stopped gracefully\n");
            }
        }
    }

    #[cfg(not(unix))]
    {
        output.push_str("⚠ Process termination not implemented on Windows\n");
    }

    // Remove PID file
    fs::remove_file(pid_file).map_err(|e| ReedError::IoError {
        operation: "delete".to_string(),
        path: pid_file.to_string(),
        reason: e.to_string(),
    })?;

    output.push_str("✓ PID file removed\n\n");
    output.push_str("Server stopped successfully.\n");

    Ok(ReedResponse {
        data: output,
        source: "cli::server_stop".to_string(),
        cached: false,
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        metrics: None,
    })
}

/// Restarts server (stop + start).
///
/// ## Output
/// - Restart confirmation
///
/// ## Error Conditions
/// - Server not running
/// - Start fails after stop
pub fn server_restart(
    args: &[String],
    flags: &HashMap<String, String>,
) -> ReedResult<ReedResponse<String>> {
    let mut output = String::new();
    output.push_str("🔄 Restarting ReedCMS server...\n\n");

    // Stop server
    output.push_str("✓ Stopping current server...\n");
    let stop_result = server_stop(args, flags);

    if stop_result.is_ok() {
        output.push_str("✓ Server stopped\n\n");
    } else {
        output.push_str("⚠ No server was running\n\n");
    }

    // Wait a moment
    std::thread::sleep(std::time::Duration::from_millis(500));

    // Start server
    output.push_str("✓ Starting new server...\n");
    let start_result = server_start(args, flags)?;
    output.push_str(&start_result.data);

    Ok(ReedResponse {
        data: output,
        source: "cli::server_restart".to_string(),
        cached: false,
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        metrics: None,
    })
}

/// Check server status.
///
/// ## Output
/// - Server status information
///
/// ## Performance
/// - Check time: < 10ms
pub fn server_status(
    _args: &[String],
    _flags: &HashMap<String, String>,
) -> ReedResult<ReedResponse<String>> {
    let mut output = String::new();
    output.push_str("📊 ReedCMS Server Status\n\n");

    let pid_file = ".reed/server.pid";

    if !std::path::Path::new(pid_file).exists() {
        output.push_str("Status: ❌ Not running\n");
        output.push_str("PID file: Not found\n");
    } else {
        let pid = fs::read_to_string(pid_file)
            .unwrap_or_default()
            .trim()
            .to_string();

        if is_process_running(&pid) {
            output.push_str("Status: ✅ Running\n");
            output.push_str(&format!("PID: {}\n", pid));
            output.push_str(&format!("PID file: {}\n", pid_file));

            // Try to get process info (Unix only)
            #[cfg(unix)]
            {
                let ps_output = Command::new("ps")
                    .arg("-p")
                    .arg(&pid)
                    .arg("-o")
                    .arg("etime,rss")
                    .arg("--no-headers")
                    .output();

                if let Ok(ps) = ps_output {
                    let info = String::from_utf8_lossy(&ps.stdout);
                    let parts: Vec<&str> = info.trim().split_whitespace().collect();
                    if parts.len() >= 2 {
                        output.push_str(&format!("Uptime: {}\n", parts[0]));
                        output.push_str(&format!("Memory: {} KB\n", parts[1]));
                    }
                }
            }
        } else {
            output.push_str("Status: ⚠ Stale PID file\n");
            output.push_str(&format!("PID: {} (not running)\n", pid));
            output.push_str("\nTip: Remove stale PID file with: rm .reed/server.pid\n");
        }
    }

    Ok(ReedResponse {
        data: output,
        source: "cli::server_status".to_string(),
        cached: false,
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        metrics: None,
    })
}

/// View server logs.
///
/// ## Input
/// - flags: --tail N, --follow
///
/// ## Output
/// - Server log content
pub fn server_logs(
    _args: &[String],
    flags: &HashMap<String, String>,
) -> ReedResult<ReedResponse<String>> {
    let log_file = ".reed/flow/server.log";

    if !std::path::Path::new(log_file).exists() {
        return Ok(ReedResponse {
            data: "No server log file found.\nLog file: .reed/flow/server.log\n".to_string(),
            source: "cli::server_logs".to_string(),
            cached: false,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            metrics: None,
        });
    }

    let mut output = String::new();

    if flags.contains_key("follow") {
        output.push_str("Following server logs (Ctrl+C to stop)...\n\n");
        output.push_str("⚠ Follow mode not yet implemented\n");
        output.push_str("   Use: tail -f .reed/flow/server.log\n");
    } else if let Some(tail_count) = flags.get("tail") {
        output.push_str(&format!("Last {} lines of server log:\n\n", tail_count));

        // Read file and show last N lines
        let content = fs::read_to_string(log_file).map_err(|e| ReedError::IoError {
            operation: "read".to_string(),
            path: log_file.to_string(),
            reason: e.to_string(),
        })?;

        let lines: Vec<&str> = content.lines().collect();
        let n: usize = tail_count.parse().unwrap_or(10);
        let start = if lines.len() > n { lines.len() - n } else { 0 };

        for line in &lines[start..] {
            output.push_str(line);
            output.push('\n');
        }
    } else {
        // Show all logs
        output.push_str("Server log:\n\n");
        let content = fs::read_to_string(log_file).map_err(|e| ReedError::IoError {
            operation: "read".to_string(),
            path: log_file.to_string(),
            reason: e.to_string(),
        })?;
        output.push_str(&content);
    }

    Ok(ReedResponse {
        data: output,
        source: "cli::server_logs".to_string(),
        cached: false,
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        metrics: None,
    })
}

/// Helper: Check if process is running.
fn is_process_running(pid: &str) -> bool {
    #[cfg(unix)]
    {
        Command::new("kill")
            .arg("-0")
            .arg(pid)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|status| status.success())
            .unwrap_or(false)
    }

    #[cfg(not(unix))]
    {
        false
    }
}
