// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Concurrent write handling module.
//!
//! Provides file locking, write queuing, and coordination for concurrent writes.

pub mod lock;
pub mod queue;
pub mod types;

// Re-export public APIs
pub use lock::{acquire_lock, is_locked, wait_for_unlock, TableLock};
pub use queue::{count_pending, get_next_pending, queue_write, remove_from_queue};
pub use types::{CsvRow, PendingWrite, WriteOperation};

#[cfg(test)]
mod lock_test;
#[cfg(test)]
mod queue_test;
