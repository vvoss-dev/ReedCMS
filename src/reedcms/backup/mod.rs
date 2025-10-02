// Copyright 2025 Vivian Voss. Licensed under the Apache Licence, Version 2.0.
// SPDX-Licence-Identifier: Apache-2.0

//! Backup System for ReedCMS
//!
//! Provides XZ-compressed backup creation, restoration, and management:
//! - Automatic backups before CSV modifications
//! - LZMA2 compression (~10x compression ratio)
//! - 32-backup retention with automatic cleanup
//! - ISO 8601 timestamp naming

mod cleanup;
mod create;
mod list;
mod restore;

pub use cleanup::cleanup_old_backups;
pub use create::create_backup;
pub use list::{list_backups, BackupInfo};
pub use restore::restore_backup;

#[cfg(test)]
mod cleanup_test;
#[cfg(test)]
mod create_test;
#[cfg(test)]
mod list_test;
#[cfg(test)]
mod restore_test;
