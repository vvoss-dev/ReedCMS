// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Universal table abstraction for ReedBase.

use crate::error::{ReedError, ReedResult};
use crate::registry::get_or_create_user_code;
use crate::tables::csv_parser::parse_csv;
use crate::tables::types::{CsvRow, VersionInfo, WriteResult};
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

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

        let timestamp = Self::now_nanos();

        // Read old version (currently unused, will be needed for bsdiff in REED-19-03)
        let _old_content = self.read_current()?;

        // Create simple delta (for now, just store new content - bsdiff in REED-19-03)
        let delta_path = self.delta_path(timestamp);
        fs::write(&delta_path, content).map_err(|e| ReedError::IoError {
            operation: "write_delta".to_string(),
            reason: e.to_string(),
        })?;

        let delta_size = fs::metadata(&delta_path)
            .map_err(|e| ReedError::IoError {
                operation: "stat_delta".to_string(),
                reason: e.to_string(),
            })?
            .len();

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
        let versions = self.list_versions()?;
        if !versions.iter().any(|v| v.timestamp == timestamp) {
            return Err(ReedError::VersionNotFound { timestamp });
        }

        // Read delta (for now, deltas are full content - bsdiff in REED-19-03)
        let delta_path = self.delta_path(timestamp);
        let content = fs::read(&delta_path).map_err(|e| ReedError::DeltaCorrupted {
            timestamp,
            reason: e.to_string(),
        })?;

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
