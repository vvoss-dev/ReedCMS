// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Universal table abstraction for ReedBase.

use crate::error::{ReedError, ReedResult};
use crate::registry::get_or_create_user_code;
use crate::tables::csv_parser::parse_csv;
use crate::tables::types::{CsvRow, VersionInfo, WriteResult};
use fs2::FileExt;
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Universal table abstraction.
///
/// All tables (text, routes, meta, users, etc.) use identical structure.
///
/// ## Structure
/// ```text
/// .reed/tables/{name}/
/// ├── current.csv          # Active version
/// ├── {timestamp}.bsdiff   # Binary deltas (XZ compressed)
/// └── version.log          # Encoded metadata
/// ```
///
/// ## Performance
/// - read_current(): < 1ms (cached)
/// - write(): < 5ms (create delta + update)
/// - list_versions(): < 5ms (parse log)
///
/// ## Thread Safety
/// - Multiple readers: Yes (concurrent reads safe)
/// - Multiple writers: NO (use WriteSession from REED-19-06)
pub struct Table {
    base_path: PathBuf,
    name: String,
}

impl Table {
    /// Creates new table reference.
    ///
    /// Does NOT create table on disk, only creates reference.
    ///
    /// ## Input
    /// - `base_path`: Path to ReedBase directory
    /// - `name`: Table name
    ///
    /// ## Output
    /// - `Table`: Table reference
    ///
    /// ## Example Usage
    /// ```
    /// use reedbase::tables::Table;
    /// use std::path::Path;
    ///
    /// let table = Table::new(Path::new(".reed"), "text");
    /// ```
    pub fn new(base_path: &Path, name: &str) -> Self {
        Self {
            base_path: base_path.to_path_buf(),
            name: name.to_string(),
        }
    }

    /// Gets path to table directory.
    fn table_dir(&self) -> PathBuf {
        self.base_path.join("tables").join(&self.name)
    }

    /// Gets path to current.csv.
    ///
    /// ## Output
    /// - `PathBuf`: Full path to current.csv
    ///
    /// ## Performance
    /// - O(1), < 10ns
    pub fn current_path(&self) -> PathBuf {
        self.table_dir().join("current.csv")
    }

    /// Gets path to delta file.
    ///
    /// ## Input
    /// - `timestamp`: Version timestamp
    ///
    /// ## Output
    /// - `PathBuf`: Full path to {timestamp}.bsdiff
    pub fn delta_path(&self, timestamp: u64) -> PathBuf {
        self.table_dir().join(format!("{}.bsdiff", timestamp))
    }

    /// Gets path to version.log.
    ///
    /// ## Output
    /// - `PathBuf`: Full path to version.log
    pub fn log_path(&self) -> PathBuf {
        self.table_dir().join("version.log")
    }

    /// Checks if table exists on disk.
    ///
    /// ## Output
    /// - `bool`: True if current.csv exists
    ///
    /// ## Performance
    /// - < 100μs (file system check)
    pub fn exists(&self) -> bool {
        self.current_path().exists()
    }

    /// Initialises new table.
    ///
    /// Creates directory and initial current.csv.
    ///
    /// ## Input
    /// - `initial_content`: CSV content (with header)
    /// - `user`: Username for audit
    ///
    /// ## Output
    /// - `Result<()>`: Success or error
    ///
    /// ## Performance
    /// - < 20ms (create dir + write file + log)
    ///
    /// ## Error Conditions
    /// - TableAlreadyExists: Table already initialised
    /// - IoError: Cannot create files
    ///
    /// ## Example Usage
    /// ```no_run
    /// use reedbase::tables::Table;
    /// use std::path::Path;
    ///
    /// let table = Table::new(Path::new(".reed"), "text");
    /// table.init(b"key|value\nfoo|bar\n", "admin")?;
    /// # Ok::<(), reedbase::ReedError>(())
    /// ```
    pub fn init(&self, initial_content: &[u8], user: &str) -> ReedResult<()> {
        if self.exists() {
            return Err(ReedError::TableAlreadyExists {
                name: self.name.clone(),
            });
        }

        // Create table directory
        let table_dir = self.table_dir();
        fs::create_dir_all(&table_dir).map_err(|e| ReedError::IoError {
            operation: "create_table_dir".to_string(),
            reason: e.to_string(),
        })?;

        // Write initial current.csv
        fs::write(&self.current_path(), initial_content).map_err(|e| ReedError::IoError {
            operation: "write_initial_current".to_string(),
            reason: e.to_string(),
        })?;

        // Create timestamp for initial version
        let timestamp = Self::now_nanos();

        // Write initial delta (full content for rollback support)
        let delta_path = self.delta_path(timestamp);
        fs::write(&delta_path, initial_content).map_err(|e| ReedError::IoError {
            operation: "write_initial_delta".to_string(),
            reason: e.to_string(),
        })?;

        // Create initial version.log entry
        let user_code = get_or_create_user_code(user)?;
        let action_code = 5u8; // init

        let log_line = format!(
            "{}|{}|{}|{}\n",
            timestamp,
            action_code,
            user_code,
            initial_content.len()
        );

        fs::write(&self.log_path(), log_line).map_err(|e| ReedError::IoError {
            operation: "write_initial_log".to_string(),
            reason: e.to_string(),
        })?;

        Ok(())
    }

    /// Reads current version as bytes.
    ///
    /// ## Output
    /// - `Result<Vec<u8>>`: CSV content
    ///
    /// ## Performance
    /// - < 1ms for typical tables (< 100 KB)
    ///
    /// ## Error Conditions
    /// - TableNotFound: Table doesn't exist
    /// - IoError: Cannot read file
    ///
    /// ## Example Usage
    /// ```no_run
    /// use reedbase::tables::Table;
    /// use std::path::Path;
    ///
    /// let table = Table::new(Path::new(".reed"), "text");
    /// let content = table.read_current()?;
    /// # Ok::<(), reedbase::ReedError>(())
    /// ```
    pub fn read_current(&self) -> ReedResult<Vec<u8>> {
        if !self.exists() {
            return Err(ReedError::TableNotFound {
                name: self.name.clone(),
            });
        }

        fs::read(&self.current_path()).map_err(|e| ReedError::IoError {
            operation: "read_current".to_string(),
            reason: e.to_string(),
        })
    }

    /// Reads current version as parsed rows.
    ///
    /// ## Output
    /// - `Result<Vec<CsvRow>>`: Parsed CSV rows
    ///
    /// ## Performance
    /// - < 5ms for typical tables (< 1000 rows)
    ///
    /// ## Error Conditions
    /// - TableNotFound: Table doesn't exist
    /// - InvalidCsv: Parse error
    pub fn read_current_as_rows(&self) -> ReedResult<Vec<CsvRow>> {
        let content = self.read_current()?;
        parse_csv(&content)
    }

    /// Writes new version.
    ///
    /// Creates delta automatically, updates current.csv, logs to version.log.
    ///
    /// ## Input
    /// - `content`: New CSV content
    /// - `user`: Username for audit
    ///
    /// ## Output
    /// - `Result<WriteResult>`: Write metadata
    ///
    /// ## Performance
    /// - < 5ms typical (bsdiff + xz + write)
    ///
    /// ## Error Conditions
    /// - TableNotFound: Table doesn't exist (use init() first)
    /// - IoError: Cannot write files
    ///
    /// ## Example Usage
    /// ```no_run
    /// use reedbase::tables::Table;
    /// use std::path::Path;
    ///
    /// let table = Table::new(Path::new(".reed"), "text");
    /// let result = table.write(b"key|value\nfoo|baz\n", "admin")?;
    /// println!("Delta size: {} bytes", result.delta_size);
    /// # Ok::<(), reedbase::ReedError>(())
    /// ```
    pub fn write(&self, content: &[u8], user: &str) -> ReedResult<WriteResult> {
        if !self.exists() {
            return Err(ReedError::TableNotFound {
                name: self.name.clone(),
            });
        }

        // Acquire exclusive lock for write operation
        let lock_result = self.write_with_lock(content, user);

        lock_result
    }

    /// Performs an atomic read-modify-write operation under a single lock.
    ///
    /// This prevents Read-Modify-Write race conditions during concurrent operations.
    ///
    /// ## Input
    /// - `modify_fn`: Function that takes current content and returns new content
    /// - `user`: Username for audit trail
    ///
    /// ## Output
    /// - `Ok(WriteResult)`: Write succeeded
    /// - `Err(ReedError)`: Write failed
    ///
    /// ## Example
    /// ```no_run
    /// table.read_modify_write(|content| {
    ///     let mut new_content = content.to_vec();
    ///     new_content.extend_from_slice(b"new_row\n");
    ///     new_content
    /// }, "user123")?;
    /// ```
    pub fn read_modify_write<F>(&self, modify_fn: F, user: &str) -> ReedResult<WriteResult>
    where
        F: FnOnce(&[u8]) -> Vec<u8>,
    {
        if !self.exists() {
            return Err(ReedError::TableNotFound {
                name: self.name.clone(),
            });
        }

        let lock_path = self.table_dir().join(".lock");

        // Create lock file if it doesn't exist
        let lock_file = OpenOptions::new()
            .create(true)
            .write(true)
            .open(&lock_path)
            .map_err(|e| ReedError::IoError {
                operation: "create_lock_file".to_string(),
                reason: e.to_string(),
            })?;

        // Try to acquire exclusive lock with retry mechanism
        self.acquire_lock_with_retry(&lock_file)?;

        // Read current content
        let current_content = self.read_current().map_err(|e| {
            let _ = lock_file.unlock();
            e
        })?;

        // Apply modification function
        let new_content = modify_fn(&current_content);

        // Perform write operation
        let result = self.write_internal(&new_content, user);

        // Release lock (automatic on drop, but explicit unlock is clearer)
        let _ = lock_file.unlock();

        result
    }

    /// Internal write implementation with file locking.
    ///
    /// Acquires exclusive lock on table directory to prevent concurrent write conflicts.
    fn write_with_lock(&self, content: &[u8], user: &str) -> ReedResult<WriteResult> {
        let lock_path = self.table_dir().join(".lock");

        // Create lock file if it doesn't exist
        let lock_file = OpenOptions::new()
            .create(true)
            .write(true)
            .open(&lock_path)
            .map_err(|e| ReedError::IoError {
                operation: "create_lock_file".to_string(),
                reason: e.to_string(),
            })?;

        // Try to acquire exclusive lock with retry mechanism
        self.acquire_lock_with_retry(&lock_file)?;

        // Perform write operation
        let result = self.write_internal(content, user);

        // Release lock (automatic on drop, but explicit unlock is clearer)
        let _ = lock_file.unlock();

        result
    }

    /// Acquire exclusive lock with exponential backoff retry.
    fn acquire_lock_with_retry(&self, lock_file: &File) -> ReedResult<()> {
        const MAX_RETRIES: u32 = 50;
        const INITIAL_WAIT_MS: u64 = 5;
        const MAX_WAIT_MS: u64 = 100;

        for attempt in 0..MAX_RETRIES {
            match lock_file.try_lock_exclusive() {
                Ok(_) => return Ok(()),
                Err(_) if attempt < MAX_RETRIES - 1 => {
                    // Exponential backoff with cap: 5ms, 10ms, 20ms, 40ms, 80ms, 100ms (capped)
                    let wait_time = (INITIAL_WAIT_MS * 2u64.pow(attempt)).min(MAX_WAIT_MS);
                    std::thread::sleep(Duration::from_millis(wait_time));
                }
                Err(e) => {
                    return Err(ReedError::IoError {
                        operation: "acquire_write_lock".to_string(),
                        reason: format!(
                            "Failed to acquire lock after {} attempts: {}",
                            MAX_RETRIES, e
                        ),
                    });
                }
            }
        }

        Err(ReedError::IoError {
            operation: "acquire_write_lock".to_string(),
            reason: "Lock acquisition failed".to_string(),
        })
    }

    /// Internal write implementation (called after lock is acquired).
    fn write_internal(&self, content: &[u8], user: &str) -> ReedResult<WriteResult> {
        let timestamp = Self::now_nanos();

        // Create binary delta using bsdiff
        let current_path = self.current_path();
        let delta_path = self.delta_path(timestamp);

        // Write new content to temp file for delta generation
        let temp_new_path = current_path.with_extension("new.tmp");
        fs::write(&temp_new_path, content).map_err(|e| ReedError::IoError {
            operation: "write_temp_new".to_string(),
            reason: e.to_string(),
        })?;

        // Generate binary delta (old -> new)
        let delta_info =
            crate::version::generate_delta(&current_path, &temp_new_path, &delta_path)?;
        let delta_size = delta_info.size as u64;

        // Clean up temp file
        let _ = fs::remove_file(&temp_new_path);

        // Update current.csv
        fs::write(&self.current_path(), content).map_err(|e| ReedError::IoError {
            operation: "write_current".to_string(),
            reason: e.to_string(),
        })?;

        // Append to version.log
        let user_code = get_or_create_user_code(user)?;
        let action_code = 2u8; // update

        let log_line = format!(
            "{}|{}|{}|{}\n",
            timestamp, action_code, user_code, delta_size
        );

        let mut log_file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_path())
            .map_err(|e| ReedError::IoError {
                operation: "open_log".to_string(),
                reason: e.to_string(),
            })?;

        log_file
            .write_all(log_line.as_bytes())
            .map_err(|e| ReedError::IoError {
                operation: "append_log".to_string(),
                reason: e.to_string(),
            })?;

        Ok(WriteResult {
            timestamp,
            delta_size,
            current_size: content.len() as u64,
        })
    }

    /// Lists all versions.
    ///
    /// Parses version.log and returns metadata for each version.
    ///
    /// ## Output
    /// - `Result<Vec<VersionInfo>>`: Version metadata (newest first)
    ///
    /// ## Performance
    /// - < 5ms for typical logs (< 100 versions)
    ///
    /// ## Error Conditions
    /// - TableNotFound: Table doesn't exist
    /// - LogCorrupted: version.log parse error
    ///
    /// ## Example Usage
    /// ```no_run
    /// use reedbase::tables::Table;
    /// use std::path::Path;
    ///
    /// let table = Table::new(Path::new(".reed"), "text");
    /// let versions = table.list_versions()?;
    /// for v in versions {
    ///     println!("Version {}: {} by {}", v.timestamp, v.action, v.user);
    /// }
    /// # Ok::<(), reedbase::ReedError>(())
    /// ```
    pub fn list_versions(&self) -> ReedResult<Vec<VersionInfo>> {
        if !self.exists() {
            return Err(ReedError::TableNotFound {
                name: self.name.clone(),
            });
        }

        let log_path = self.log_path();
        if !log_path.exists() {
            return Ok(Vec::new());
        }

        let file = File::open(&log_path).map_err(|e| ReedError::IoError {
            operation: "open_log".to_string(),
            reason: e.to_string(),
        })?;

        let reader = BufReader::new(file);
        let mut versions = Vec::new();

        for (line_num, line_result) in reader.lines().enumerate() {
            let line = line_result.map_err(|e| ReedError::LogCorrupted {
                reason: e.to_string(),
            })?;

            if line.trim().is_empty() {
                continue;
            }

            let parts: Vec<&str> = line.split('|').collect();
            if parts.len() < 4 {
                return Err(ReedError::LogCorrupted {
                    reason: format!("Invalid format at line {}", line_num + 1),
                });
            }

            let timestamp = parts[0]
                .parse::<u64>()
                .map_err(|_| ReedError::LogCorrupted {
                    reason: format!("Invalid timestamp at line {}", line_num + 1),
                })?;

            let action_code = parts[1]
                .parse::<u8>()
                .map_err(|_| ReedError::LogCorrupted {
                    reason: format!("Invalid action code at line {}", line_num + 1),
                })?;

            let user_code = parts[2]
                .parse::<u32>()
                .map_err(|_| ReedError::LogCorrupted {
                    reason: format!("Invalid user code at line {}", line_num + 1),
                })?;

            let delta_size = parts[3]
                .parse::<u64>()
                .map_err(|_| ReedError::LogCorrupted {
                    reason: format!("Invalid delta size at line {}", line_num + 1),
                })?;

            // Resolve codes to names
            let action = crate::registry::get_action_name(action_code)
                .unwrap_or_else(|_| format!("unknown({})", action_code));

            let user = crate::registry::get_username(user_code)
                .unwrap_or_else(|_| format!("unknown({})", user_code));

            versions.push(VersionInfo {
                timestamp,
                action,
                user,
                delta_size,
                message: None,
            });
        }

        // Reverse to get newest first
        versions.reverse();

        Ok(versions)
    }

    /// Rolls back to specific version.
    ///
    /// Reconstructs version from deltas and writes as current.
    ///
    /// ## Input
    /// - `timestamp`: Target version timestamp
    /// - `user`: Username for audit
    ///
    /// ## Output
    /// - `Result<()>`: Success or error
    ///
    /// ## Performance
    /// - < 100ms per 50 deltas (typical)
    ///
    /// ## Error Conditions
    /// - VersionNotFound: Timestamp not in log
    /// - DeltaCorrupted: Cannot apply delta
    ///
    /// ## Example Usage
    /// ```no_run
    /// use reedbase::tables::Table;
    /// use std::path::Path;
    ///
    /// let table = Table::new(Path::new(".reed"), "text");
    /// let versions = table.list_versions()?;
    /// table.rollback(versions[1].timestamp, "admin")?;
    /// # Ok::<(), reedbase::ReedError>(())
    /// ```
    pub fn rollback(&self, timestamp: u64, user: &str) -> ReedResult<()> {
        // Verify version exists
        let mut versions = self.list_versions()?;
        if !versions.iter().any(|v| v.timestamp == timestamp) {
            return Err(ReedError::VersionNotFound { timestamp });
        }

        // Versions are newest-first, reverse to get oldest-first for reconstruction
        versions.reverse();

        // Find target version index
        let target_idx = versions
            .iter()
            .position(|v| v.timestamp == timestamp)
            .ok_or(ReedError::VersionNotFound { timestamp })?;

        // Reconstruct version by applying deltas in sequence
        // Start with initial version (index 0) and apply deltas up to target
        let table_dir = self.table_dir();
        let mut reconstructed_path = table_dir.join("rollback.tmp");

        // First delta from init() is raw content (not a bsdiff delta)
        let first_delta_path = self.delta_path(versions[0].timestamp);
        fs::copy(&first_delta_path, &reconstructed_path).map_err(|e| ReedError::IoError {
            operation: "copy_init_delta".to_string(),
            reason: e.to_string(),
        })?;

        // Apply subsequent deltas to reach target version
        for i in 1..=target_idx {
            let prev_path = reconstructed_path.clone();
            let delta_path = self.delta_path(versions[i].timestamp);
            reconstructed_path = table_dir.join(format!("rollback_{}.tmp", i));

            crate::version::apply_delta(&prev_path, &delta_path, &reconstructed_path)?;
            let _ = fs::remove_file(&prev_path);
        }

        // Read reconstructed content
        let content = fs::read(&reconstructed_path).map_err(|e| ReedError::IoError {
            operation: "read_reconstructed".to_string(),
            reason: e.to_string(),
        })?;

        // Clean up temp file
        let _ = fs::remove_file(&reconstructed_path);

        // Write as new version
        self.write(&content, user)?;

        Ok(())
    }

    /// Deletes table and all versions.
    ///
    /// ## Input
    /// - `confirm`: Safety flag (must be true)
    ///
    /// ## Output
    /// - `Result<()>`: Success or error
    ///
    /// ## Error Conditions
    /// - NotConfirmed: confirm was false
    /// - IoError: Cannot delete files
    ///
    /// ## Example Usage
    /// ```no_run
    /// use reedbase::tables::Table;
    /// use std::path::Path;
    ///
    /// let table = Table::new(Path::new(".reed"), "old_table");
    /// table.delete(true)?; // DESTRUCTIVE!
    /// # Ok::<(), reedbase::ReedError>(())
    /// ```
    pub fn delete(&self, confirm: bool) -> ReedResult<()> {
        if !confirm {
            return Err(ReedError::NotConfirmed {
                operation: format!("delete table '{}'", self.name),
            });
        }

        let table_dir = self.table_dir();
        if table_dir.exists() {
            fs::remove_dir_all(&table_dir).map_err(|e| ReedError::IoError {
                operation: "delete_table".to_string(),
                reason: e.to_string(),
            })?;
        }

        Ok(())
    }

    /// Gets current timestamp in nanoseconds.
    fn now_nanos() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("System time before Unix epoch")
            .as_nanos() as u64
    }
}
