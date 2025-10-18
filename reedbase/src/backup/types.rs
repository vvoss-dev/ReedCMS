// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Backup and restore types.

use crate::error::ReedError;
use std::path::PathBuf;

/// Backup information.
#[derive(Debug, Clone)]
pub struct BackupInfo {
    /// Unix timestamp (seconds) when backup was created.
    pub timestamp: u64,

    /// Path to backup file.
    pub path: PathBuf,

    /// Size in bytes.
    pub size_bytes: u64,

    /// Size in megabytes.
    pub size_mb: f64,
}

/// Restore report.
#[derive(Debug, Clone)]
pub struct RestoreReport {
    /// Target timestamp requested.
    pub target_timestamp: u64,

    /// Tables that were restored (table_name, actual_timestamp).
    pub tables_restored: Vec<(String, u64)>,

    /// Tables that were skipped (didn't exist at target time).
    pub tables_skipped: Vec<String>,

    /// Tables that failed to restore.
    pub errors: Vec<(String, ReedError)>,
}

impl RestoreReport {
    /// Creates a new empty restore report.
    pub fn new(target_timestamp: u64) -> Self {
        Self {
            target_timestamp,
            tables_restored: Vec::new(),
            tables_skipped: Vec::new(),
            errors: Vec::new(),
        }
    }

    /// Checks if restore was successful (no errors).
    pub fn is_success(&self) -> bool {
        self.errors.is_empty()
    }

    /// Checks if restore was partial (some errors).
    pub fn is_partial(&self) -> bool {
        !self.errors.is_empty() && !self.tables_restored.is_empty()
    }

    /// Checks if restore completely failed.
    pub fn is_failure(&self) -> bool {
        !self.errors.is_empty() && self.tables_restored.is_empty()
    }
}
