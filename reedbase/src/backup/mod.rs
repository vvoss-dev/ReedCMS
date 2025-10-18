// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Backup and point-in-time recovery module.
//!
//! This module provides backup creation, listing, and point-in-time recovery
//! using standard tools (tar) and existing version.log infrastructure.

mod create;
mod list;
mod restore;
mod types;

pub use create::create_backup;
pub use list::list_backups;
pub use restore::restore_point_in_time;
pub use types::{BackupInfo, RestoreReport};

#[cfg(test)]
mod tests;
