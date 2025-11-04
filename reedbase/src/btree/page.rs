// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! B+-Tree page management for disk-based index storage.
//!
//! Implements fixed-size pages (4KB) with headers, CRC32 validation, and memory-mapped I/O.
//! Pages contain either internal B+-Tree nodes (keys + child pointers) or leaf nodes
//! (keys + values + next leaf pointer).
//!
//! ## Page Layout
//! ```text
//! ┌─────────────────────────────────────────────────┐
//! │ PageHeader (32 bytes)                           │
//! ├─────────────────────────────────────────────────┤
//! │                                                 │
//! │ Data Section (4064 bytes)                       │
//! │                                                 │
//! │ - Internal node: [keys, child pointers]        │
//! │ - Leaf node: [keys, values]                     │
//! │                                                 │
//! └─────────────────────────────────────────────────┘
//! Total: 4096 bytes (4KB)
//! ```
//!
//! ## CRC32 Protection
//! - Checksum calculated over data section only (4064 bytes)
//! - Validates data integrity on read
//! - Detects corruption from partial writes, hardware errors, etc.
//!
//! ## Performance Characteristics
//! - Fixed 4KB page size matches typical filesystem block size
//! - Memory-mapped I/O for zero-copy reads
//! - CRC32 validation: ~2μs per page (modern CPUs with SSE4.2)
//! - Sequential leaf access via linked list for efficient range queries

use crate::btree::types::{NodeType, PageId, BTREE_MAGIC};
use crate::error::{ReedError, ReedResult};
use memmap2::{Mmap, MmapMut};

/// Page size in bytes (4KB).
///
/// Matches common filesystem block size for optimal I/O performance.
/// Larger pages reduce tree height but increase memory pressure and
/// I/O cost for small updates.
pub const PAGE_SIZE: usize = 4096;

/// Page header size in bytes.
///
/// 32-byte header leaves 4064 bytes for data storage.
const HEADER_SIZE: usize = 32;

/// Data section size in bytes.
const DATA_SIZE: usize = PAGE_SIZE - HEADER_SIZE; // 4064 bytes

/// Page header structure (32 bytes).
///
/// Fixed-size header at start of every page containing metadata and checksums.
/// Padding reserved for future extensions (e.g., timestamps, versioning).
///
/// ## Memory Layout
/// ```text
/// Offset | Size | Field
/// -------|------|-------
/// 0      | 4    | magic (BTREE_MAGIC constant)
/// 4      | 1    | page_type (NodeType as u8)
/// 5      | 2    | num_keys (u16 big-endian)
/// 7      | 4    | next_page (PageId, 0 = none)
/// 11     | 4    | checksum (CRC32 of data)
/// 15     | 17   | _padding (reserved)
/// ```
#[repr(C)]
#[derive(Debug, Clone)]
pub struct PageHeader {
    /// Magic bytes for format validation (BTREE_MAGIC = 0xB7EE_7EE1).
    pub magic: u32,

    /// Node type discriminator (Internal = 0, Leaf = 1).
    pub page_type: u8,

    /// Number of keys stored in data section.
    pub num_keys: u16,

    /// Next page in leaf linked list (0 if none).
    ///
    /// Only used for leaf pages to chain sequential access.
    /// Internal nodes set this to 0.
    pub next_page: u32,

    /// CRC32 checksum of data section (4064 bytes).
    ///
    /// Calculated using `crc32fast::hash()` over entire data section.
    pub checksum: u32,

    /// Reserved padding for future use.
    ///
    /// Must be zeroed. May be used for:
    /// - Timestamps (8 bytes)
    /// - Version numbers (2 bytes)
    /// - Flags (1 byte)
    /// - Additional pointers (4 bytes)
    #[allow(dead_code)]
    _padding: [u8; 17],
}

impl PageHeader {
    /// Serialise header to 32-byte array.
    ///
    /// ## Output
    /// - 32-byte array with fields in big-endian format
    ///
    /// ## Performance
    /// - O(1) constant time (32 bytes)
    fn to_bytes(&self) -> [u8; HEADER_SIZE] {
        let mut bytes = [0u8; HEADER_SIZE];

        // Offset 0: magic (4 bytes, big-endian)
        bytes[0..4].copy_from_slice(&self.magic.to_be_bytes());

        // Offset 4: page_type (1 byte)
        bytes[4] = self.page_type;

        // Offset 5: num_keys (2 bytes, big-endian)
        bytes[5..7].copy_from_slice(&self.num_keys.to_be_bytes());

        // Offset 7: next_page (4 bytes, big-endian)
        bytes[7..11].copy_from_slice(&self.next_page.to_be_bytes());

        // Offset 11: checksum (4 bytes, big-endian)
        bytes[11..15].copy_from_slice(&self.checksum.to_be_bytes());

        // Offset 15: padding (17 bytes, already zeroed)
        bytes[15..32].copy_from_slice(&self._padding);

        bytes
    }

    /// Deserialise header from 32-byte slice.
    ///
    /// ## Input
    /// - `bytes`: 32-byte slice containing header data
    ///
    /// ## Output
    /// - `Ok(PageHeader)`: Successfully parsed header
    /// - `Err(ReedError)`: Invalid magic bytes or malformed data
    ///
    /// ## Performance
    /// - O(1) constant time (32 bytes)
    ///
    /// ## Error Conditions
    /// - Slice length != 32 bytes
    /// - Magic bytes != BTREE_MAGIC
    fn from_bytes(bytes: &[u8]) -> ReedResult<Self> {
        if bytes.len() != HEADER_SIZE {
            return Err(ReedError::ParseError {
                reason: format!("Header must be {} bytes, got {}", HEADER_SIZE, bytes.len()),
            });
        }

        // Parse magic
        let magic = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        if magic != BTREE_MAGIC {
            return Err(ReedError::ParseError {
                reason: format!(
                    "Invalid magic bytes: expected 0x{:X}, got 0x{:X}",
                    BTREE_MAGIC, magic
                ),
            });
        }

        // Parse page_type
        let page_type = bytes[4];

        // Parse num_keys
        let num_keys = u16::from_be_bytes([bytes[5], bytes[6]]);

        // Parse next_page
        let next_page = u32::from_be_bytes([bytes[7], bytes[8], bytes[9], bytes[10]]);

        // Parse checksum
        let checksum = u32::from_be_bytes([bytes[11], bytes[12], bytes[13], bytes[14]]);

        // Parse padding
        let mut padding = [0u8; 17];
        padding.copy_from_slice(&bytes[15..32]);

        Ok(Self {
            magic,
            page_type,
            num_keys,
            next_page,
            checksum,
            _padding: padding,
        })
    }
}

/// Page structure representing a single B+-Tree node on disk.
///
/// Fixed 4KB size with 32-byte header and 4064-byte data section.
/// Supports both internal nodes (keys + child pointers) and leaf nodes
/// (keys + values).
///
/// ## Thread Safety
/// - Individual pages are not thread-safe (use external synchronisation)
/// - Memory-mapped files require coordination for concurrent access
///
/// ## Example Usage
/// ```rust
/// use reedbase::btree::page::{Page, PAGE_SIZE};
/// use reedbase::btree::types::PageId;
/// use memmap2::MmapMut;
///
/// // Create new internal node page
/// let page = Page::new_internal(0);
///
/// // Set data and write to mmap
/// let mut data = vec![1, 2, 3, 4];
/// page.set_data(data);
/// // page.write_to(&mut mmap, 0)?;
/// ```
#[derive(Debug, Clone)]
pub struct Page {
    /// Page header with metadata and checksum.
    pub header: PageHeader,

    /// Data section (4064 bytes).
    ///
    /// Content interpretation depends on node type:
    /// - Internal: Serialised keys and child PageIds
    /// - Leaf: Serialised keys and values
    pub data: Vec<u8>,
}

impl Page {
    /// Create new internal node page.
    ///
    /// ## Input
    /// - `page_id`: Page identifier (used for error context, not stored in page)
    ///
    /// ## Output
    /// - Empty internal node page with zeroed data section
    ///
    /// ## Performance
    /// - O(1) constant time
    /// - Allocates 4064 bytes for data section
    ///
    /// ## Example
    /// ```rust
    /// let page = Page::new_internal(0);
    /// assert_eq!(page.header.page_type, NodeType::Internal as u8);
    /// assert_eq!(page.header.num_keys, 0);
    /// ```
    pub fn new_internal(_page_id: PageId) -> Self {
        let data = vec![0u8; DATA_SIZE];
        let checksum = crc32fast::hash(&data);

        Self {
            header: PageHeader {
                magic: BTREE_MAGIC,
                page_type: NodeType::Internal as u8,
                num_keys: 0,
                next_page: 0,
                checksum,
                _padding: [0u8; 17],
            },
            data,
        }
    }

    /// Create new leaf node page.
    ///
    /// ## Input
    /// - `page_id`: Page identifier (used for error context, not stored in page)
    ///
    /// ## Output
    /// - Empty leaf node page with zeroed data section
    ///
    /// ## Performance
    /// - O(1) constant time
    /// - Allocates 4064 bytes for data section
    ///
    /// ## Example
    /// ```rust
    /// let page = Page::new_leaf(0);
    /// assert_eq!(page.header.page_type, NodeType::Leaf as u8);
    /// assert_eq!(page.header.next_page, 0); // No next leaf yet
    /// ```
    pub fn new_leaf(_page_id: PageId) -> Self {
        let data = vec![0u8; DATA_SIZE];
        let checksum = crc32fast::hash(&data);

        Self {
            header: PageHeader {
                magic: BTREE_MAGIC,
                page_type: NodeType::Leaf as u8,
                num_keys: 0,
                next_page: 0,
                checksum,
                _padding: [0u8; 17],
            },
            data,
        }
    }

    /// Read page from memory-mapped file with validation.
    ///
    /// ## Input
    /// - `mmap`: Memory-mapped file (read-only)
    /// - `page_id`: Page identifier to read
    ///
    /// ## Output
    /// - `Ok(Page)`: Successfully read and validated page
    /// - `Err(ReedError::CorruptedIndex)`: CRC32 mismatch or invalid magic
    /// - `Err(ReedError::InvalidPageFormat)`: Malformed header or truncated data
    ///
    /// ## Performance
    /// - O(1) zero-copy read via memory mapping
    /// - CRC32 validation: ~2μs on modern CPUs
    ///
    /// ## Error Conditions
    /// - Page offset beyond mmap bounds
    /// - Invalid magic bytes in header
    /// - CRC32 checksum mismatch (data corruption)
    /// - Truncated page (< 4096 bytes available)
    ///
    /// ## Example
    /// ```rust
    /// use memmap2::Mmap;
    /// use std::fs::File;
    ///
    /// let file = File::open("index.btree")?;
    /// let mmap = unsafe { Mmap::map(&file)? };
    ///
    /// let page = Page::read_from(&mmap, 0)?;
    /// assert_eq!(page.header.magic, BTREE_MAGIC);
    /// ```
    pub fn read_from(mmap: &Mmap, page_id: PageId) -> ReedResult<Self> {
        Self::read_from_bytes(mmap.as_ref(), page_id)
    }

    /// Read page from byte slice (works with both Mmap and MmapMut).
    pub fn read_from_bytes(bytes: &[u8], page_id: PageId) -> ReedResult<Self> {
        let offset = (page_id as usize) * PAGE_SIZE;

        // Check bounds
        if offset + PAGE_SIZE > bytes.len() {
            return Err(ReedError::ParseError {
                reason: format!(
                    "Page {} at offset {} exceeds bounds ({})",
                    page_id,
                    offset,
                    bytes.len()
                ),
            });
        }

        // Read header (32 bytes)
        let header_bytes = &bytes[offset..offset + HEADER_SIZE];
        let header = PageHeader::from_bytes(header_bytes)?;

        // Read data (4064 bytes)
        let data_offset = offset + HEADER_SIZE;
        let data = bytes[data_offset..data_offset + DATA_SIZE].to_vec();

        // Validate checksum
        let computed_checksum = crc32fast::hash(&data);
        if computed_checksum != header.checksum {
            return Err(ReedError::ParseError {
                reason: format!(
                    "CRC32 mismatch on page {}: expected 0x{:X}, computed 0x{:X}",
                    page_id, header.checksum, computed_checksum
                ),
            });
        }

        Ok(Self { header, data })
    }

    /// Write page to memory-mapped file with checksum calculation.
    ///
    /// ## Input
    /// - `mmap`: Memory-mapped file (writable)
    /// - `page_id`: Page identifier to write
    ///
    /// ## Output
    /// - `Ok(())`: Successfully written and flushed
    /// - `Err(ReedError::IoError)`: Write failed or bounds exceeded
    ///
    /// ## Performance
    /// - O(1) zero-copy write via memory mapping
    /// - CRC32 calculation: ~2μs on modern CPUs
    /// - Automatic flush for durability
    ///
    /// ## Error Conditions
    /// - Page offset beyond mmap bounds
    /// - Data section size != 4064 bytes
    /// - I/O error during flush
    ///
    /// ## Example
    /// ```rust
    /// use memmap2::MmapMut;
    /// use std::fs::OpenOptions;
    ///
    /// let file = OpenOptions::new().read(true).write(true).open("index.btree")?;
    /// let mut mmap = unsafe { MmapMut::map_mut(&file)? };
    ///
    /// let mut page = Page::new_leaf(0);
    /// page.set_data(vec![1, 2, 3, 4]);
    /// page.write_to(&mut mmap, 0)?;
    /// ```
    pub fn write_to(&self, mmap: &mut MmapMut, page_id: PageId) -> ReedResult<()> {
        let offset = (page_id as usize) * PAGE_SIZE;

        // Check bounds
        if offset + PAGE_SIZE > mmap.len() {
            return Err(ReedError::IoError {
                operation: "write_page".to_string(),
                reason: format!(
                    "Page {} at offset {} exceeds mmap bounds ({})",
                    page_id,
                    offset,
                    mmap.len()
                ),
            });
        }

        // Validate data size
        if self.data.len() != DATA_SIZE {
            return Err(ReedError::IoError {
                operation: "write_page".to_string(),
                reason: format!(
                    "Data section must be {} bytes, got {}",
                    DATA_SIZE,
                    self.data.len()
                ),
            });
        }

        // Recalculate checksum
        let checksum = crc32fast::hash(&self.data);
        let mut header = self.header.clone();
        header.checksum = checksum;

        // Write header (32 bytes)
        let header_bytes = header.to_bytes();
        mmap[offset..offset + HEADER_SIZE].copy_from_slice(&header_bytes);

        // Write data (4064 bytes)
        let data_offset = offset + HEADER_SIZE;
        mmap[data_offset..data_offset + DATA_SIZE].copy_from_slice(&self.data);

        // Flush to ensure durability
        mmap.flush().map_err(|e| ReedError::IoError {
            operation: "flush_page".to_string(),
            reason: e.to_string(),
        })?;

        Ok(())
    }

    /// Validate page integrity (magic + CRC32).
    ///
    /// ## Output
    /// - `Ok(())`: Page is valid
    /// - `Err(ReedError::CorruptedIndex)`: Invalid magic or checksum mismatch
    ///
    /// ## Performance
    /// - O(n) where n = data size (4064 bytes)
    /// - CRC32 validation: ~2μs on modern CPUs
    ///
    /// ## Error Conditions
    /// - Magic bytes != BTREE_MAGIC
    /// - CRC32 checksum mismatch
    ///
    /// ## Example
    /// ```rust
    /// let page = Page::read_from(&mmap, 0)?;
    /// page.validate()?; // Ensures page wasn't corrupted
    /// ```
    pub fn validate(&self) -> ReedResult<()> {
        // Check magic
        if self.header.magic != BTREE_MAGIC {
            return Err(ReedError::ParseError {
                reason: format!(
                    "Invalid magic bytes: expected 0x{:X}, got 0x{:X}",
                    BTREE_MAGIC, self.header.magic
                ),
            });
        }

        // Validate checksum
        let computed_checksum = crc32fast::hash(&self.data);
        if computed_checksum != self.header.checksum {
            return Err(ReedError::ParseError {
                reason: format!(
                    "CRC32 mismatch: expected 0x{:X}, computed 0x{:X}",
                    self.header.checksum, computed_checksum
                ),
            });
        }

        Ok(())
    }

    /// Set data section and recalculate checksum.
    ///
    /// ## Input
    /// - `data`: New data section content (must be exactly 4064 bytes)
    ///
    /// ## Performance
    /// - O(n) where n = data size (4064 bytes)
    /// - CRC32 calculation: ~2μs on modern CPUs
    ///
    /// ## Panics
    /// - If data.len() != 4064 bytes (caller must pad/truncate)
    ///
    /// ## Example
    /// ```rust
    /// let mut page = Page::new_leaf(0);
    /// let data = vec![0u8; 4064]; // Must be exact size
    /// page.set_data(data);
    /// assert_eq!(page.get_data().len(), 4064);
    /// ```
    pub fn set_data(&mut self, data: Vec<u8>) {
        assert_eq!(
            data.len(),
            DATA_SIZE,
            "Data must be exactly {} bytes",
            DATA_SIZE
        );
        self.data = data;
        self.header.checksum = crc32fast::hash(&self.data);
    }

    /// Get immutable reference to data section.
    ///
    /// ## Output
    /// - Slice reference to 4064-byte data section
    ///
    /// ## Performance
    /// - O(1) constant time (no copy)
    ///
    /// ## Example
    /// ```rust
    /// let page = Page::read_from(&mmap, 0)?;
    /// let data = page.get_data();
    /// assert_eq!(data.len(), 4064);
    /// ```
    pub fn get_data(&self) -> &[u8] {
        &self.data
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_page_size_constants() {
        assert_eq!(PAGE_SIZE, 4096);
        assert_eq!(HEADER_SIZE, 32);
        assert_eq!(DATA_SIZE, 4064);
        assert_eq!(HEADER_SIZE + DATA_SIZE, PAGE_SIZE);
    }

    #[test]
    fn test_header_serialization() {
        let header = PageHeader {
            magic: BTREE_MAGIC,
            page_type: NodeType::Leaf as u8,
            num_keys: 42,
            next_page: 123,
            checksum: 0xDEADBEEF,
            _padding: [0u8; 17],
        };

        let bytes = header.to_bytes();
        assert_eq!(bytes.len(), HEADER_SIZE);

        let decoded = PageHeader::from_bytes(&bytes).unwrap();
        assert_eq!(decoded.magic, BTREE_MAGIC);
        assert_eq!(decoded.page_type, NodeType::Leaf as u8);
        assert_eq!(decoded.num_keys, 42);
        assert_eq!(decoded.next_page, 123);
        assert_eq!(decoded.checksum, 0xDEADBEEF);
    }

    #[test]
    fn test_header_invalid_magic() {
        let mut bytes = [0u8; HEADER_SIZE];
        bytes[0..4].copy_from_slice(&0xBADC0DE_u32.to_be_bytes());

        let result = PageHeader::from_bytes(&bytes);
        assert!(result.is_err());
    }

    #[test]
    fn test_page_new_internal() {
        let page = Page::new_internal(0);
        assert_eq!(page.header.magic, BTREE_MAGIC);
        assert_eq!(page.header.page_type, NodeType::Internal as u8);
        assert_eq!(page.header.num_keys, 0);
        assert_eq!(page.header.next_page, 0);
        assert_eq!(page.data.len(), DATA_SIZE);

        // Checksum should be valid for zeroed data
        let expected_checksum = crc32fast::hash(&page.data);
        assert_eq!(page.header.checksum, expected_checksum);
    }

    #[test]
    fn test_page_new_leaf() {
        let page = Page::new_leaf(0);
        assert_eq!(page.header.magic, BTREE_MAGIC);
        assert_eq!(page.header.page_type, NodeType::Leaf as u8);
        assert_eq!(page.header.num_keys, 0);
        assert_eq!(page.header.next_page, 0);
        assert_eq!(page.data.len(), DATA_SIZE);
    }

    #[test]
    fn test_page_set_data() {
        let mut page = Page::new_leaf(0);
        let original_checksum = page.header.checksum;

        let new_data = vec![1u8; DATA_SIZE];
        page.set_data(new_data);

        // Checksum should be updated
        assert_ne!(page.header.checksum, original_checksum);
        assert_eq!(page.header.checksum, crc32fast::hash(&page.data));
    }

    #[test]
    fn test_page_get_data() {
        let page = Page::new_leaf(0);
        let data = page.get_data();
        assert_eq!(data.len(), DATA_SIZE);
        assert!(data.iter().all(|&b| b == 0)); // All zeros initially
    }

    #[test]
    fn test_page_validate_success() {
        let page = Page::new_internal(0);
        assert!(page.validate().is_ok());
    }

    #[test]
    fn test_page_validate_invalid_magic() {
        let mut page = Page::new_leaf(0);
        page.header.magic = 0xBADC0DE;
        assert!(page.validate().is_err());
    }

    #[test]
    fn test_page_validate_invalid_checksum() {
        let mut page = Page::new_leaf(0);
        page.header.checksum = 0xDEADBEEF; // Wrong checksum
        assert!(page.validate().is_err());
    }

    #[test]
    #[should_panic(expected = "Data must be exactly 4064 bytes")]
    fn test_page_set_data_wrong_size() {
        let mut page = Page::new_leaf(0);
        let wrong_size_data = vec![1u8; 100]; // Wrong size
        page.set_data(wrong_size_data);
    }
}
