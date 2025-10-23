// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! File locking for concurrent write coordination.
//!
//! Uses advisory file locks for cross-process synchronisation.

use crate::error::{ReedError, ReedResult};
use fs2::FileExt;
use std::fs::{File, OpenOptions};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

/// Acquires exclusive lock on table.
///
/// ## Input
/// - `base_path`: Path to ReedBase directory
/// - `table_name`: Table name
/// - `timeout`: Maximum time to wait for lock
///
/// ## Output
/// - `ReedResult<TableLock>`: Lock handle (RAII - auto-releases on drop)
///
/// ## Performance
/// - < 10ms if lock available immediately
/// - Up to `timeout` if lock held by another process
///
/// ## Error Conditions
/// - LockTimeout: Could not acquire lock within timeout
/// - IoError: Cannot create lock file
///
/// ## Example Usage
/// ```no_run
/// use reedbase::concurrent::acquire_lock;
/// use std::time::Duration;
/// use std::path::Path;
///
/// let lock = acquire_lock(Path::new(".reed"), "users", Duration::from_secs(30))?;
/// // Lock held - perform write
/// // Lock automatically released when `lock` drops
/// # Ok::<(), reedbase::ReedError>(())
/// ```
pub fn acquire_lock(
    base_path: &Path,
    table_name: &str,
    timeout: Duration,
) -> ReedResult<TableLock> {
    let lock_path = base_path.join("tables").join(table_name).join("write.lock");

    // Ensure table directory exists
    if let Some(parent) = lock_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| ReedError::IoError {
            operation: "create_lock_dir".to_string(),
            reason: e.to_string(),
        })?;
    }

    let lock_file = OpenOptions::new()
        .create(true)
        .write(true)
        .open(&lock_path)
        .map_err(|e| ReedError::IoError {
            operation: "create_lock_file".to_string(),
            reason: e.to_string(),
        })?;

    let start = Instant::now();

    loop {
        match lock_file.try_lock_exclusive() {
            Ok(()) => {
                return Ok(TableLock {
                    file: lock_file,
                    path: lock_path,
                    table_name: table_name.to_string(),
                });
            }
            Err(_) if start.elapsed() < timeout => {
                std::thread::sleep(Duration::from_millis(100));
            }
            Err(_) => {
                return Err(ReedError::LockTimeout {
                    table: table_name.to_string(),
                    timeout_secs: timeout.as_secs(),
                });
            }
        }
    }
}

/// Table lock handle (RAII).
///
/// Lock is automatically released when this struct is dropped.
pub struct TableLock {
    file: File,
    path: PathBuf,
    table_name: String,
}

impl Drop for TableLock {
    /// Releases lock on drop.
    ///
    /// ## Performance
    /// - < 1ms typical
    fn drop(&mut self) {
        let _ = self.file.unlock();
    }
}

/// Checks if table is locked.
///
/// ## Input
/// - `base_path`: Path to ReedBase directory
/// - `table_name`: Table name
///
/// ## Output
/// - `ReedResult<bool>`: True if locked
///
/// ## Performance
/// - < 5ms typical
///
/// ## Error Conditions
/// - IoError: Cannot access lock file
///
/// ## Example Usage
/// ```no_run
/// use reedbase::concurrent::is_locked;
/// use std::path::Path;
///
/// if is_locked(Path::new(".reed"), "users")? {
///     println!("Table is currently locked");
/// }
/// # Ok::<(), reedbase::ReedError>(())
/// ```
pub fn is_locked(base_path: &Path, table_name: &str) -> ReedResult<bool> {
    let lock_path = base_path.join("tables").join(table_name).join("write.lock");

    let lock_file = match OpenOptions::new().read(true).open(&lock_path) {
        Ok(f) => f,
        Err(_) => return Ok(false), // Lock file doesn't exist
    };

    match lock_file.try_lock_exclusive() {
        Ok(()) => {
            let _ = lock_file.unlock();
            Ok(false)
        }
        Err(_) => Ok(true),
    }
}

/// Waits for lock to be released.
///
/// ## Input
/// - `base_path`: Path to ReedBase directory
/// - `table_name`: Table name
/// - `timeout`: Maximum time to wait
///
/// ## Output
/// - `ReedResult<()>`: Ok when lock released
///
/// ## Performance
/// - Variable (depends on lock holder)
/// - Up to `timeout`
///
/// ## Error Conditions
/// - LockTimeout: Lock not released within timeout
///
/// ## Example Usage
/// ```no_run
/// use reedbase::concurrent::wait_for_unlock;
/// use std::time::Duration;
/// use std::path::Path;
///
/// wait_for_unlock(Path::new(".reed"), "users", Duration::from_secs(60))?;
/// // Lock now available
/// # Ok::<(), reedbase::ReedError>(())
/// ```
pub fn wait_for_unlock(base_path: &Path, table_name: &str, timeout: Duration) -> ReedResult<()> {
    let start = Instant::now();

    while is_locked(base_path, table_name)? {
        if start.elapsed() >= timeout {
            return Err(ReedError::LockTimeout {
                table: table_name.to_string(),
                timeout_secs: timeout.as_secs(),
            });
        }
        std::thread::sleep(Duration::from_millis(100));
    }

    Ok(())
}
