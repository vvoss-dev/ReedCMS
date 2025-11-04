// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Write-Ahead Log (WAL) for crash safety in B+-Tree operations.
//!
//! Implements append-only log that records all mutations before they are applied
//! to the main B+-Tree file. On crash/restart, the WAL is replayed to reconstruct
//! any uncommitted changes, ensuring durability guarantees.
//!
//! ## WAL Format
//!
//! ```text
//! Entry format (variable length):
//! ┌─────────────────────────────────────────────────┐
//! │ Entry Type (1 byte: 1=Insert, 2=Delete)         │
//! ├─────────────────────────────────────────────────┤
//! │ Key Length (4 bytes, big-endian)                │
//! ├─────────────────────────────────────────────────┤
//! │ Key Data (variable)                             │
//! ├─────────────────────────────────────────────────┤
//! │ Value Length (4 bytes, only for Insert)         │
//! ├─────────────────────────────────────────────────┤
//! │ Value Data (variable, only for Insert)          │
//! ├─────────────────────────────────────────────────┤
//! │ CRC32 Checksum (4 bytes)                        │
//! └─────────────────────────────────────────────────┘
//! ```
//!
//! ## Crash Recovery
//!
//! 1. On startup, check if WAL exists
//! 2. If found, replay all valid entries
//! 3. Truncate WAL after successful replay
//! 4. Continue normal operations
//!
//! ## Performance
//!
//! - Log write: ~100μs (append + fsync)
//! - Replay (1000 entries): ~50ms
//! - Truncate: ~10ms
//!
//! ## Example Usage
//!
//! ```rust
//! use reedbase::btree::wal::{WriteAheadLog, WalEntry};
//!
//! // Open or create WAL
//! let mut wal = WriteAheadLog::open("index.wal")?;
//!
//! // Log mutation
//! wal.log_insert("key".to_string(), vec![1, 2, 3])?;
//!
//! // Sync to disk
//! wal.sync()?;
//!
//! // On restart: replay log
//! let entries = wal.replay()?;
//! for entry in entries {
//!     match entry {
//!         WalEntry::Insert { key, value } => {
//!             // Apply insert to B+-Tree
//!         }
//!         WalEntry::Delete { key } => {
//!             // Apply delete to B+-Tree
//!         }
//!     }
//! }
//!
//! // Clear log after successful replay
//! wal.truncate()?;
//! # Ok::<(), reedbase::ReedError>(())
//! ```

use crate::error::{ReedError, ReedResult};
use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::{BufReader, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

/// Entry type discriminator for WAL records.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EntryType {
    /// Insert or update operation.
    Insert = 1,
    /// Delete operation.
    Delete = 2,
}

impl EntryType {
    /// Convert byte to entry type.
    fn from_byte(byte: u8) -> ReedResult<Self> {
        match byte {
            1 => Ok(Self::Insert),
            2 => Ok(Self::Delete),
            _ => Err(ReedError::ParseError {
                reason: format!("Invalid WAL entry type: {}", byte),
            }),
        }
    }
}

/// WAL entry representing a single mutation.
///
/// ## Variants
/// - `Insert`: Add or update key-value pair
/// - `Delete`: Remove key
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WalEntry<K, V>
where
    K: Clone,
    V: Clone,
{
    /// Insert or update operation.
    Insert {
        /// Key to insert/update.
        key: K,
        /// Value to store.
        value: V,
    },
    /// Delete operation.
    Delete {
        /// Key to delete.
        key: K,
    },
}

/// Write-Ahead Log for B+-Tree durability.
///
/// Append-only log file that records all mutations before they are applied
/// to the main index. Enables crash recovery by replaying uncommitted changes.
///
/// ## File Format
/// - Extension: `.wal`
/// - Structure: Sequence of variable-length entries
/// - Each entry: [type, key_len, key, value_len?, value?, crc32]
///
/// ## Thread Safety
/// - Single writer (B+-Tree holds exclusive lock)
/// - Safe to read during replay (file not modified)
///
/// ## Durability Guarantees
/// - `log_*()` methods: Written to kernel buffer (not durable)
/// - `sync()`: Calls fsync() to ensure disk persistence
/// - After `sync()`, entry survives power loss
pub struct WriteAheadLog {
    /// Path to WAL file.
    path: PathBuf,

    /// File handle for append operations.
    file: File,
}

impl WriteAheadLog {
    /// Open or create Write-Ahead Log.
    ///
    /// ## Input
    /// - `path`: Path to WAL file (typically `index.wal`)
    ///
    /// ## Output
    /// - `Ok(WriteAheadLog)`: Successfully opened/created WAL
    /// - `Err(ReedError::IoError)`: File creation/open failed
    ///
    /// ## Performance
    /// - O(1) file open operation
    ///
    /// ## Error Conditions
    /// - Parent directory does not exist
    /// - Insufficient permissions
    /// - Disk full
    ///
    /// ## Example
    /// ```rust
    /// use reedbase::btree::wal::WriteAheadLog;
    ///
    /// let wal = WriteAheadLog::open("index.wal")?;
    /// # Ok::<(), reedbase::ReedError>(())
    /// ```
    pub fn open<P: AsRef<Path>>(path: P) -> ReedResult<Self> {
        let path = path.as_ref().to_path_buf();

        // Open in append mode (create if doesn't exist)
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .read(true)
            .open(&path)
            .map_err(|e| ReedError::IoError {
                operation: "open_wal".to_string(),
                reason: e.to_string(),
            })?;

        Ok(Self { path, file })
    }

    /// Log insert operation to WAL.
    ///
    /// ## Input
    /// - `key`: Key to insert/update
    /// - `value`: Value to store
    ///
    /// ## Output
    /// - `Ok(())`: Entry written to kernel buffer
    /// - `Err(ReedError)`: Write failed or serialisation error
    ///
    /// ## Performance
    /// - O(1) append operation (~100μs with fsync)
    ///
    /// ## Error Conditions
    /// - Disk full
    /// - I/O error
    /// - Serialisation error (key/value too large)
    ///
    /// ## Durability
    /// Entry is NOT durable until `sync()` is called.
    ///
    /// ## Example
    /// ```rust
    /// use reedbase::btree::wal::WriteAheadLog;
    ///
    /// let mut wal = WriteAheadLog::open("index.wal")?;
    /// wal.log_insert("key".to_string(), vec![1, 2, 3])?;
    /// wal.sync()?; // Ensure durability
    /// # Ok::<(), reedbase::ReedError>(())
    /// ```
    pub fn log_insert<K, V>(&mut self, key: K, value: V) -> ReedResult<()>
    where
        K: Serialize,
        V: Serialize,
    {
        // Serialise key
        let key_bytes = bincode::serialize(&key).map_err(|e| ReedError::SerializationError {
            reason: e.to_string(),
        })?;

        // Serialise value
        let value_bytes =
            bincode::serialize(&value).map_err(|e| ReedError::SerializationError {
                reason: e.to_string(),
            })?;

        // Build entry buffer
        let mut buffer = Vec::new();

        // Entry type (1 byte)
        buffer.push(EntryType::Insert as u8);

        // Key length (4 bytes, big-endian)
        buffer.extend_from_slice(&(key_bytes.len() as u32).to_be_bytes());

        // Key data
        buffer.extend_from_slice(&key_bytes);

        // Value length (4 bytes, big-endian)
        buffer.extend_from_slice(&(value_bytes.len() as u32).to_be_bytes());

        // Value data
        buffer.extend_from_slice(&value_bytes);

        // CRC32 checksum (over entire entry)
        let checksum = crc32fast::hash(&buffer);
        buffer.extend_from_slice(&checksum.to_be_bytes());

        // Write to file
        self.file
            .write_all(&buffer)
            .map_err(|e| ReedError::IoError {
                operation: "write_wal_insert".to_string(),
                reason: e.to_string(),
            })?;

        Ok(())
    }

    /// Log delete operation to WAL.
    ///
    /// ## Input
    /// - `key`: Key to delete
    ///
    /// ## Output
    /// - `Ok(())`: Entry written to kernel buffer
    /// - `Err(ReedError)`: Write failed or serialisation error
    ///
    /// ## Performance
    /// - O(1) append operation (~100μs with fsync)
    ///
    /// ## Error Conditions
    /// - Disk full
    /// - I/O error
    /// - Serialisation error (key too large)
    ///
    /// ## Durability
    /// Entry is NOT durable until `sync()` is called.
    ///
    /// ## Example
    /// ```rust
    /// use reedbase::btree::wal::WriteAheadLog;
    ///
    /// let mut wal = WriteAheadLog::open("index.wal")?;
    /// wal.log_delete("key".to_string())?;
    /// wal.sync()?; // Ensure durability
    /// # Ok::<(), reedbase::ReedError>(())
    /// ```
    pub fn log_delete<K>(&mut self, key: K) -> ReedResult<()>
    where
        K: Serialize,
    {
        // Serialise key
        let key_bytes = bincode::serialize(&key).map_err(|e| ReedError::SerializationError {
            reason: e.to_string(),
        })?;

        // Build entry buffer
        let mut buffer = Vec::new();

        // Entry type (1 byte)
        buffer.push(EntryType::Delete as u8);

        // Key length (4 bytes, big-endian)
        buffer.extend_from_slice(&(key_bytes.len() as u32).to_be_bytes());

        // Key data
        buffer.extend_from_slice(&key_bytes);

        // CRC32 checksum (over entire entry)
        let checksum = crc32fast::hash(&buffer);
        buffer.extend_from_slice(&checksum.to_be_bytes());

        // Write to file
        self.file
            .write_all(&buffer)
            .map_err(|e| ReedError::IoError {
                operation: "write_wal_delete".to_string(),
                reason: e.to_string(),
            })?;

        Ok(())
    }

    /// Replay all entries from WAL.
    ///
    /// Reads entire WAL file and returns all valid entries.
    /// Stops at first corrupted entry (partial write from crash).
    ///
    /// ## Output
    /// - `Ok(Vec<WalEntry>)`: All valid entries in order
    /// - `Err(ReedError)`: I/O error (not corruption)
    ///
    /// ## Performance
    /// - O(n) where n = number of entries (~50ms for 1000 entries)
    ///
    /// ## Error Conditions
    /// - I/O error reading file
    /// - Deserialisation error (should not happen with valid WAL)
    ///
    /// ## Corruption Handling
    /// - CRC32 mismatch: Stop replay (partial write, ignore rest)
    /// - Invalid entry type: Stop replay
    /// - Truncated entry: Stop replay
    ///
    /// Corruption is NOT an error (expected after crash during write).
    /// Valid entries before corruption are returned successfully.
    ///
    /// ## Example
    /// ```rust
    /// use reedbase::btree::wal::{WriteAheadLog, WalEntry};
    ///
    /// let wal = WriteAheadLog::open("index.wal")?;
    /// let entries = wal.replay()?;
    ///
    /// println!("Replaying {} operations", entries.len());
    /// for entry in entries {
    ///     // Apply to B+-Tree
    /// }
    /// # Ok::<(), reedbase::ReedError>(())
    /// ```
    pub fn replay<K, V>(&self) -> ReedResult<Vec<WalEntry<K, V>>>
    where
        K: Clone + for<'de> Deserialize<'de>,
        V: Clone + for<'de> Deserialize<'de>,
    {
        let mut entries = Vec::new();

        // Open file for reading (separate handle to avoid append-mode issues)
        let mut file = File::open(&self.path).map_err(|e| ReedError::IoError {
            operation: "open_wal_replay".to_string(),
            reason: e.to_string(),
        })?;

        // Ensure we're at the start of the file
        file.seek(SeekFrom::Start(0))
            .map_err(|e| ReedError::IoError {
                operation: "seek_wal_replay".to_string(),
                reason: e.to_string(),
            })?;

        let mut reader = BufReader::new(file);
        let mut buffer = Vec::new();

        loop {
            buffer.clear();

            // Read entry type (1 byte)
            let mut type_byte = [0u8; 1];
            match reader.read_exact(&mut type_byte) {
                Ok(_) => {}
                Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break, // End of file
                Err(e) => {
                    return Err(ReedError::IoError {
                        operation: "read_wal_type".to_string(),
                        reason: e.to_string(),
                    })
                }
            }

            let entry_type = match EntryType::from_byte(type_byte[0]) {
                Ok(t) => t,
                Err(_) => break, // Corrupted entry, stop replay
            };

            buffer.push(type_byte[0]);

            // Read key length (4 bytes)
            let mut len_bytes = [0u8; 4];
            if reader.read_exact(&mut len_bytes).is_err() {
                break; // Truncated entry
            }
            let key_len = u32::from_be_bytes(len_bytes) as usize;
            buffer.extend_from_slice(&len_bytes);

            // Read key data
            let mut key_bytes = vec![0u8; key_len];
            if reader.read_exact(&mut key_bytes).is_err() {
                break; // Truncated entry
            }
            buffer.extend_from_slice(&key_bytes);

            // Deserialise key
            let key: K = match bincode::deserialize(&key_bytes) {
                Ok(k) => k,
                Err(_) => break, // Corrupted key
            };

            // Read value if Insert entry
            let value = if entry_type == EntryType::Insert {
                // Read value length (4 bytes)
                let mut len_bytes = [0u8; 4];
                if reader.read_exact(&mut len_bytes).is_err() {
                    break; // Truncated entry
                }
                let value_len = u32::from_be_bytes(len_bytes) as usize;
                buffer.extend_from_slice(&len_bytes);

                // Read value data
                let mut value_bytes = vec![0u8; value_len];
                if reader.read_exact(&mut value_bytes).is_err() {
                    break; // Truncated entry
                }
                buffer.extend_from_slice(&value_bytes);

                // Deserialise value
                match bincode::deserialize(&value_bytes) {
                    Ok(v) => Some(v),
                    Err(_) => break, // Corrupted value
                }
            } else {
                None
            };

            // Read checksum (4 bytes)
            let mut checksum_bytes = [0u8; 4];
            if reader.read_exact(&mut checksum_bytes).is_err() {
                break; // Truncated entry
            }
            let stored_checksum = u32::from_be_bytes(checksum_bytes);

            // Validate checksum
            let computed_checksum = crc32fast::hash(&buffer);
            if stored_checksum != computed_checksum {
                break; // Corrupted entry (partial write)
            }

            // Add valid entry to results
            let entry = match entry_type {
                EntryType::Insert => WalEntry::Insert {
                    key,
                    value: value.unwrap(),
                },
                EntryType::Delete => WalEntry::Delete { key },
            };
            entries.push(entry);
        }

        Ok(entries)
    }

    /// Truncate WAL file (clear all entries).
    ///
    /// Called after successful replay to start fresh log.
    /// Also used periodically to prevent unbounded growth.
    ///
    /// ## Output
    /// - `Ok(())`: File truncated successfully
    /// - `Err(ReedError::IoError)`: Truncate failed
    ///
    /// ## Performance
    /// - O(1) syscall (~10ms)
    ///
    /// ## Error Conditions
    /// - I/O error
    /// - File handle closed/invalid
    ///
    /// ## Example
    /// ```rust
    /// use reedbase::btree::wal::WriteAheadLog;
    ///
    /// let mut wal = WriteAheadLog::open("index.wal")?;
    /// let entries = wal.replay()?;
    ///
    /// // Apply entries to B+-Tree...
    ///
    /// // Clear WAL after successful replay
    /// wal.truncate()?;
    /// # Ok::<(), reedbase::ReedError>(())
    /// ```
    pub fn truncate(&mut self) -> ReedResult<()> {
        self.file.set_len(0).map_err(|e| ReedError::IoError {
            operation: "truncate_wal".to_string(),
            reason: e.to_string(),
        })?;

        // Seek back to start for next write
        self.file
            .seek(SeekFrom::Start(0))
            .map_err(|e| ReedError::IoError {
                operation: "seek_wal".to_string(),
                reason: e.to_string(),
            })?;

        Ok(())
    }

    /// Sync WAL to disk (fsync).
    ///
    /// Ensures all buffered writes are persisted to disk.
    /// After successful sync, logged entries survive power loss.
    ///
    /// ## Output
    /// - `Ok(())`: All data synced to disk
    /// - `Err(ReedError::IoError)`: Sync failed
    ///
    /// ## Performance
    /// - ~10-50ms (depends on disk/filesystem)
    /// - Use batching to amortise cost (sync after N operations)
    ///
    /// ## Error Conditions
    /// - I/O error
    /// - Disk full (delayed error from earlier write)
    ///
    /// ## Example
    /// ```rust
    /// use reedbase::btree::wal::WriteAheadLog;
    ///
    /// let mut wal = WriteAheadLog::open("index.wal")?;
    ///
    /// // Batch writes
    /// for i in 0..100 {
    ///     wal.log_insert(format!("key{}", i), vec![i as u8])?;
    /// }
    ///
    /// // Single sync for entire batch
    /// wal.sync()?;
    /// # Ok::<(), reedbase::ReedError>(())
    /// ```
    pub fn sync(&mut self) -> ReedResult<()> {
        self.file.sync_all().map_err(|e| ReedError::IoError {
            operation: "sync_wal".to_string(),
            reason: e.to_string(),
        })
    }
}
