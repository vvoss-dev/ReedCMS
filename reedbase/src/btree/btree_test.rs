// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Comprehensive tests for B+-Tree implementation.
//!
//! Tests cover all major functionality:
//! - Basic operations (insert, get, delete)
//! - Range queries and iteration
//! - Page splits and merges
//! - WAL recovery and crash safety
//! - Edge cases and error conditions
//! - Performance benchmarks
//!
//! Uses tempfile for test isolation (no interference between tests).

#[cfg(test)]
mod tests {
    use crate::btree::node::{InternalNode, LeafNode};
    use crate::btree::page::{Page, PAGE_SIZE};
    use crate::btree::tree::BPlusTree;
    use crate::btree::types::{Index, NodeType, Order, BTREE_MAGIC};
    use crate::btree::wal::{WalEntry, WriteAheadLog};
    use crate::error::ReedResult;
    use tempfile::{tempdir, NamedTempFile};

    // ============================================================================
    // Node Tests
    // ============================================================================

    #[test]
    fn test_internal_node_new() {
        let node = InternalNode::<String>::new();
        assert_eq!(node.keys.len(), 0);
        assert_eq!(node.children.len(), 0);
    }

    #[test]
    fn test_internal_node_find_child() {
        let mut node = InternalNode::new();
        node.keys = vec![10, 20, 30, 40];
        node.children = vec![1, 2, 3, 4, 5];

        // Test boundary cases
        assert_eq!(node.find_child(&5), 0); // < first key
        assert_eq!(node.find_child(&10), 1); // = first key
        assert_eq!(node.find_child(&15), 1); // between keys
        assert_eq!(node.find_child(&20), 2); // = second key
        assert_eq!(node.find_child(&35), 3); // between keys
        assert_eq!(node.find_child(&40), 4); // = last key
        assert_eq!(node.find_child(&50), 4); // > last key
    }

    #[test]
    fn test_internal_node_insert_key() -> ReedResult<()> {
        let mut node = InternalNode::new();
        node.children.push(1); // Initial leftmost child

        node.insert_key(20, 2)?;
        assert_eq!(node.keys, vec![20]);
        assert_eq!(node.children, vec![1, 2]);

        node.insert_key(10, 3)?;
        assert_eq!(node.keys, vec![10, 20]);
        assert_eq!(node.children, vec![1, 3, 2]);

        node.insert_key(30, 4)?;
        assert_eq!(node.keys, vec![10, 20, 30]);
        assert_eq!(node.children, vec![1, 3, 2, 4]);

        Ok(())
    }

    #[test]
    fn test_internal_node_split() -> ReedResult<()> {
        let mut node = InternalNode::new();
        node.keys = vec![10, 20, 30, 40, 50];
        node.children = vec![1, 2, 3, 4, 5, 6];

        let (middle_key, new_node) = node.split()?;

        // Original node keeps left half
        assert_eq!(node.keys, vec![10, 20]);
        assert_eq!(node.children, vec![1, 2, 3]);

        // Middle key promoted to parent
        assert_eq!(middle_key, 30);

        // New node gets right half
        assert_eq!(new_node.keys, vec![40, 50]);
        assert_eq!(new_node.children, vec![4, 5, 6]);

        Ok(())
    }

    #[test]
    fn test_internal_node_overflow() {
        let order = Order::new(4).unwrap();
        let mut node = InternalNode::<i32>::new();

        assert!(!node.is_overflow(order));

        node.keys = vec![1, 2, 3];
        assert!(!node.is_overflow(order));

        node.keys = vec![1, 2, 3, 4];
        assert!(node.is_overflow(order));
    }

    #[test]
    fn test_leaf_node_new() {
        let node = LeafNode::<String, Vec<u8>>::new();
        assert_eq!(node.keys.len(), 0);
        assert_eq!(node.values.len(), 0);
        assert_eq!(node.next, None);
    }

    #[test]
    fn test_leaf_node_find_value() {
        let mut node = LeafNode::new();
        node.keys = vec![10, 20, 30, 40];
        node.values = vec![vec![1], vec![2], vec![3], vec![4]];

        assert_eq!(node.find_value(&10), Some(vec![1]));
        assert_eq!(node.find_value(&20), Some(vec![2]));
        assert_eq!(node.find_value(&30), Some(vec![3]));
        assert_eq!(node.find_value(&40), Some(vec![4]));
        assert_eq!(node.find_value(&15), None);
        assert_eq!(node.find_value(&50), None);
    }

    #[test]
    fn test_leaf_node_insert() -> ReedResult<()> {
        let mut node = LeafNode::new();

        // Insert in order
        node.insert(20, vec![2])?;
        node.insert(10, vec![1])?;
        node.insert(30, vec![3])?;

        assert_eq!(node.keys, vec![10, 20, 30]);
        assert_eq!(node.values, vec![vec![1], vec![2], vec![3]]);

        // Update existing key
        node.insert(20, vec![99])?;
        assert_eq!(node.values[1], vec![99]);

        Ok(())
    }

    #[test]
    fn test_leaf_node_split() -> ReedResult<()> {
        let mut node = LeafNode::new();
        node.keys = vec![10, 20, 30, 40];
        node.values = vec![vec![1], vec![2], vec![3], vec![4]];
        node.next = Some(100);

        let (split_key, new_node) = node.split()?;

        // Original node keeps left half
        assert_eq!(node.keys, vec![10, 20]);
        assert_eq!(node.values, vec![vec![1], vec![2]]);

        // Split key is first key in new node
        assert_eq!(split_key, 30);

        // New node gets right half
        assert_eq!(new_node.keys, vec![30, 40]);
        assert_eq!(new_node.values, vec![vec![3], vec![4]]);
        assert_eq!(new_node.next, Some(100));

        Ok(())
    }

    #[test]
    fn test_leaf_node_overflow() {
        let order = Order::new(4).unwrap();
        let mut node = LeafNode::<i32, i32>::new();

        assert!(!node.is_overflow(order));

        node.keys = vec![1, 2, 3];
        node.values = vec![1, 2, 3];
        assert!(!node.is_overflow(order));

        node.keys = vec![1, 2, 3, 4];
        node.values = vec![1, 2, 3, 4];
        assert!(node.is_overflow(order));
    }

    // ============================================================================
    // Page Tests
    // ============================================================================

    #[test]
    fn test_page_size_constants() {
        assert_eq!(PAGE_SIZE, 4096);
    }

    #[test]
    fn test_page_new_internal() {
        let page = Page::new_internal(0);
        assert_eq!(page.header.magic, BTREE_MAGIC);
        assert_eq!(page.header.page_type, NodeType::Internal as u8);
        assert_eq!(page.header.num_keys, 0);
        assert_eq!(page.data.len(), 4064);
    }

    #[test]
    fn test_page_new_leaf() {
        let page = Page::new_leaf(0);
        assert_eq!(page.header.magic, BTREE_MAGIC);
        assert_eq!(page.header.page_type, NodeType::Leaf as u8);
        assert_eq!(page.header.num_keys, 0);
        assert_eq!(page.data.len(), 4064);
    }

    #[test]
    fn test_page_validate() {
        let page = Page::new_leaf(0);
        assert!(page.validate().is_ok());
    }

    #[test]
    fn test_page_set_data() {
        let mut page = Page::new_leaf(0);
        let original_checksum = page.header.checksum;

        let new_data = vec![1u8; 4064];
        page.set_data(new_data);

        assert_ne!(page.header.checksum, original_checksum);
    }

    // ============================================================================
    // WAL Tests
    // ============================================================================

    #[test]
    fn test_wal_open() -> ReedResult<()> {
        let tmp = NamedTempFile::new().unwrap();
        let wal = WriteAheadLog::open(tmp.path())?;
        assert!(tmp.path().exists());
        Ok(())
    }

    #[test]
    fn test_wal_log_insert() -> ReedResult<()> {
        let tmp = NamedTempFile::new().unwrap();
        let mut wal = WriteAheadLog::open(tmp.path())?;

        wal.log_insert("key1".to_string(), vec![1u8, 2u8, 3u8])?;
        wal.log_insert("key2".to_string(), vec![4u8, 5u8, 6u8])?;
        wal.sync()?;

        Ok(())
    }

    #[test]
    fn test_wal_log_delete() -> ReedResult<()> {
        let tmp = NamedTempFile::new().unwrap();
        let mut wal = WriteAheadLog::open(tmp.path())?;

        wal.log_delete("key1".to_string())?;
        wal.sync()?;

        Ok(())
    }

    #[test]
    fn test_wal_replay() -> ReedResult<()> {
        let tmp = NamedTempFile::new().unwrap();

        // Write entries
        {
            let mut wal = WriteAheadLog::open(tmp.path())?;
            wal.log_insert("key1".to_string(), vec![1u8, 2u8, 3u8])?;
            wal.log_insert("key2".to_string(), vec![4u8, 5u8, 6u8])?;
            wal.log_delete("key1".to_string())?;
            wal.sync()?;
        }

        // Replay entries
        {
            let wal = WriteAheadLog::open(tmp.path())?;
            let entries: Vec<WalEntry<String, Vec<u8>>> = wal.replay()?;

            assert_eq!(entries.len(), 3);

            match &entries[0] {
                WalEntry::Insert { key, value } => {
                    assert_eq!(key, "key1");
                    assert_eq!(value, &vec![1, 2, 3]);
                }
                _ => panic!("Expected Insert entry"),
            }

            match &entries[1] {
                WalEntry::Insert { key, value } => {
                    assert_eq!(key, "key2");
                    assert_eq!(value, &vec![4, 5, 6]);
                }
                _ => panic!("Expected Insert entry"),
            }

            match &entries[2] {
                WalEntry::Delete { key } => {
                    assert_eq!(key, "key1");
                }
                _ => panic!("Expected Delete entry"),
            }
        }

        Ok(())
    }

    #[test]
    fn test_wal_truncate() -> ReedResult<()> {
        let tmp = NamedTempFile::new().unwrap();
        let mut wal = WriteAheadLog::open(tmp.path())?;

        wal.log_insert("key1".to_string(), vec![1u8, 2u8, 3u8])?;
        wal.sync()?;
        wal.truncate()?;

        let entries: Vec<WalEntry<String, Vec<u8>>> = wal.replay()?;
        assert_eq!(entries.len(), 0);

        Ok(())
    }

    // ============================================================================
    // B+-Tree Basic Operations
    // ============================================================================

    #[test]
    fn test_btree_open_new() -> ReedResult<()> {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.btree");

        let order = Order::new(10)?;
        let tree = BPlusTree::<String, Vec<u8>>::open(&path, order)?;

        assert!(path.exists());
        assert_eq!(tree.backend_type(), "btree");

        Ok(())
    }

    #[test]
    fn test_btree_insert_and_get() -> ReedResult<()> {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.btree");
        let order = Order::new(10)?;

        let mut tree = BPlusTree::open(&path, order)?;

        // Insert key-value pairs
        tree.insert("key1".to_string(), vec![1u8, 2u8, 3u8])?;
        tree.insert("key2".to_string(), vec![4u8, 5u8, 6u8])?;
        tree.insert("key3".to_string(), vec![7u8, 8u8, 9u8])?;

        // Retrieve values
        assert_eq!(tree.get(&"key1".to_string())?, Some(vec![1u8, 2u8, 3u8]));
        assert_eq!(tree.get(&"key2".to_string())?, Some(vec![4u8, 5u8, 6u8]));
        assert_eq!(tree.get(&"key3".to_string())?, Some(vec![7u8, 8u8, 9u8]));
        assert_eq!(tree.get(&"key4".to_string())?, None);

        Ok(())
    }

    #[test]
    fn test_btree_insert_update() -> ReedResult<()> {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.btree");
        let order = Order::new(10)?;

        let mut tree = BPlusTree::open(&path, order)?;

        tree.insert("key1".to_string(), vec![1u8, 2u8, 3u8])?;
        assert_eq!(tree.get(&"key1".to_string())?, Some(vec![1u8, 2u8, 3u8]));

        // Update existing key
        tree.insert("key1".to_string(), vec![99u8, 99u8, 99u8])?;
        assert_eq!(tree.get(&"key1".to_string())?, Some(vec![99u8, 99u8, 99u8]));

        Ok(())
    }

    #[test]
    fn test_btree_delete() -> ReedResult<()> {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.btree");
        let order = Order::new(10)?;

        let mut tree = BPlusTree::open(&path, order)?;

        tree.insert("key1".to_string(), vec![1u8, 2u8, 3u8])?;
        tree.insert("key2".to_string(), vec![4u8, 5u8, 6u8])?;
        tree.insert("key3".to_string(), vec![7u8, 8u8, 9u8])?;

        // Delete key
        tree.delete(&"key2".to_string())?;
        assert_eq!(tree.get(&"key2".to_string())?, None);

        // Other keys still exist
        assert_eq!(tree.get(&"key1".to_string())?, Some(vec![1u8, 2u8, 3u8]));
        assert_eq!(tree.get(&"key3".to_string())?, Some(vec![7u8, 8u8, 9u8]));

        Ok(())
    }

    #[test]
    fn test_btree_delete_nonexistent() -> ReedResult<()> {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.btree");
        let order = Order::new(10)?;

        let mut tree = BPlusTree::<String, Vec<u8>>::open(&path, order)?;

        // Delete non-existent key (should not error)
        tree.delete(&"nonexistent".to_string())?;

        Ok(())
    }

    // ============================================================================
    // Range Query Tests
    // ============================================================================

    #[test]
    fn test_btree_range_query() -> ReedResult<()> {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.btree");
        let order = Order::new(10)?;

        let mut tree = BPlusTree::open(&path, order)?;

        // Insert keys with namespace prefixes
        tree.insert("page.a".to_string(), vec![1u8])?;
        tree.insert("page.b".to_string(), vec![2u8])?;
        tree.insert("page.c".to_string(), vec![3u8])?;
        tree.insert("post.a".to_string(), vec![4u8])?;
        tree.insert("post.b".to_string(), vec![5u8])?;

        // Range query: page.* keys
        let results = tree.range(&"page.a".to_string(), &"page.z".to_string())?;
        assert_eq!(results.len(), 3);
        assert_eq!(results[0].0, "page.a");
        assert_eq!(results[1].0, "page.b");
        assert_eq!(results[2].0, "page.c");

        // Range query: post.* keys
        let results = tree.range(&"post.a".to_string(), &"post.z".to_string())?;
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].0, "post.a");
        assert_eq!(results[1].0, "post.b");

        Ok(())
    }

    #[test]
    fn test_btree_range_empty() -> ReedResult<()> {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.btree");
        let order = Order::new(10)?;

        let mut tree = BPlusTree::open(&path, order)?;

        tree.insert("page.a".to_string(), vec![1u8])?;
        tree.insert("page.b".to_string(), vec![2u8])?;

        // Range with no matches
        let results = tree.range(&"post.a".to_string(), &"post.z".to_string())?;
        assert_eq!(results.len(), 0);

        Ok(())
    }

    #[test]
    fn test_btree_range_single() -> ReedResult<()> {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.btree");
        let order = Order::new(10)?;

        let mut tree = BPlusTree::open(&path, order)?;

        tree.insert("page.a".to_string(), vec![1u8])?;
        tree.insert("page.b".to_string(), vec![2u8])?;
        tree.insert("page.c".to_string(), vec![3u8])?;

        // Range query for single key
        let results = tree.range(&"page.b".to_string(), &"page.c".to_string())?;
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0, "page.b");

        Ok(())
    }

    // ============================================================================
    // Iterator Tests
    // ============================================================================

    #[test]
    fn test_btree_iter() -> ReedResult<()> {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.btree");
        let order = Order::new(10)?;

        let mut tree = BPlusTree::open(&path, order)?;

        tree.insert("c".to_string(), vec![3u8])?;
        tree.insert("a".to_string(), vec![1u8])?;
        tree.insert("b".to_string(), vec![2u8])?;

        let results: Vec<(String, Vec<u8>)> = tree.iter().collect();

        assert_eq!(results.len(), 3);
        assert_eq!(results[0].0, "a");
        assert_eq!(results[1].0, "b");
        assert_eq!(results[2].0, "c");

        Ok(())
    }

    #[test]
    fn test_btree_iter_empty() -> ReedResult<()> {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.btree");
        let order = Order::new(10)?;

        let tree = BPlusTree::<String, Vec<u8>>::open(&path, order)?;

        let results: Vec<(String, Vec<u8>)> = tree.iter().collect();
        assert_eq!(results.len(), 0);

        Ok(())
    }

    // ============================================================================
    // Page Split Tests
    // ============================================================================

    #[test]
    fn test_btree_leaf_split() -> ReedResult<()> {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.btree");
        let order = Order::new(4)?; // Small order to trigger splits

        let mut tree = BPlusTree::open(&path, order)?;

        // Insert enough keys to cause split
        for i in 0..10 {
            let key = format!("key{:03}", i);
            tree.insert(key, vec![i as u8])?;
        }

        // Verify all keys are retrievable
        for i in 0..10 {
            let key = format!("key{:03}", i);
            let value = tree.get(&key)?;
            assert_eq!(value, Some(vec![i as u8]));
        }

        Ok(())
    }

    // ============================================================================
    // WAL Recovery Tests
    // ============================================================================

    #[test]
    fn test_btree_wal_recovery() -> ReedResult<()> {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.btree");
        let order = Order::new(10)?;

        // Create tree and insert data
        {
            let mut tree = BPlusTree::open(&path, order)?;
            tree.insert("key1".to_string(), vec![1u8, 2u8, 3u8])?;
            tree.insert("key2".to_string(), vec![4u8, 5u8, 6u8])?;
        }

        // Reopen tree (simulates restart with WAL replay)
        {
            let tree = BPlusTree::<String, Vec<u8>>::open(&path, order)?;
            assert_eq!(tree.get(&"key1".to_string())?, Some(vec![1u8, 2u8, 3u8]));
            assert_eq!(tree.get(&"key2".to_string())?, Some(vec![4u8, 5u8, 6u8]));
        }

        Ok(())
    }

    // ============================================================================
    // Edge Cases
    // ============================================================================

    #[test]
    fn test_btree_empty() -> ReedResult<()> {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.btree");
        let order = Order::new(10)?;

        let tree = BPlusTree::<String, Vec<u8>>::open(&path, order)?;

        assert_eq!(tree.get(&"key".to_string())?, None);
        let results = tree.range(&"a".to_string(), &"z".to_string())?;
        assert_eq!(results.len(), 0);

        Ok(())
    }

    #[test]
    fn test_btree_single_key() -> ReedResult<()> {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.btree");
        let order = Order::new(10)?;

        let mut tree = BPlusTree::open(&path, order)?;
        tree.insert("only".to_string(), vec![1u8])?;

        assert_eq!(tree.get(&"only".to_string())?, Some(vec![1u8]));
        assert_eq!(tree.get(&"other".to_string())?, None);

        Ok(())
    }

    #[test]
    fn test_btree_large_values() -> ReedResult<()> {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.btree");
        let order = Order::new(10)?;

        let mut tree = BPlusTree::open(&path, order)?;

        let large_value = vec![42u8; 1024]; // 1KB value
        tree.insert("key".to_string(), large_value.clone())?;

        assert_eq!(tree.get(&"key".to_string())?, Some(large_value));

        Ok(())
    }

    #[test]
    fn test_btree_special_characters() -> ReedResult<()> {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.btree");
        let order = Order::new(10)?;

        let mut tree = BPlusTree::open(&path, order)?;

        tree.insert("key.with.dots".to_string(), vec![1u8])?;
        tree.insert("key-with-dashes".to_string(), vec![2u8])?;
        tree.insert("key_with_underscores".to_string(), vec![3u8])?;

        assert_eq!(tree.get(&"key.with.dots".to_string())?, Some(vec![1u8]));
        assert_eq!(tree.get(&"key-with-dashes".to_string())?, Some(vec![2u8]));
        assert_eq!(
            tree.get(&"key_with_underscores".to_string())?,
            Some(vec![3u8])
        );

        Ok(())
    }

    // ============================================================================
    // Performance Tests (lightweight benchmarks)
    // ============================================================================

    #[test]
    fn test_btree_bulk_insert() -> ReedResult<()> {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.btree");
        let order = Order::new(100)?;

        let mut tree = BPlusTree::open(&path, order)?;

        // Insert 1000 keys
        for i in 0..1000 {
            let key = format!("key{:06}", i);
            tree.insert(key, vec![i as u8])?;
        }

        // Verify some keys
        assert_eq!(tree.get(&"key000000".to_string())?, Some(vec![0]));
        assert_eq!(tree.get(&"key000500".to_string())?, Some(vec![244])); // 500 % 256
        assert_eq!(tree.get(&"key000999".to_string())?, Some(vec![231])); // 999 % 256

        Ok(())
    }

    #[test]
    fn test_btree_sequential_scan() -> ReedResult<()> {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.btree");
        let order = Order::new(100)?;

        let mut tree = BPlusTree::open(&path, order)?;

        // Insert 100 keys
        for i in 0..100 {
            let key = format!("key{:03}", i);
            tree.insert(key, vec![i as u8])?;
        }

        // Scan all keys
        let results: Vec<(String, Vec<u8>)> = tree.iter().collect();
        assert_eq!(results.len(), 100);

        // Verify sorted order
        for i in 0..99 {
            assert!(results[i].0 < results[i + 1].0);
        }

        Ok(())
    }

    // ============================================================================
    // Memory and Disk Usage Tests
    // ============================================================================

    #[test]
    fn test_btree_memory_usage() -> ReedResult<()> {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.btree");
        let order = Order::new(100)?;

        let tree = BPlusTree::<String, Vec<u8>>::open(&path, order)?;

        let mem_usage = tree.memory_usage();
        assert!(mem_usage > 0);
        assert!(mem_usage < 10 * 1024 * 1024); // < 10MB for empty tree

        Ok(())
    }

    #[test]
    fn test_btree_disk_usage() -> ReedResult<()> {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.btree");
        let order = Order::new(100)?;

        let mut tree = BPlusTree::open(&path, order)?;

        let initial_disk = tree.disk_usage();
        assert!(initial_disk >= 1024 * 1024); // >= 1MB initial allocation

        // Insert data
        for i in 0..100 {
            tree.insert(format!("key{}", i), vec![i as u8])?;
        }

        let after_insert_disk = tree.disk_usage();
        assert!(after_insert_disk >= initial_disk);

        Ok(())
    }

    // ============================================================================
    // Order Configuration Tests
    // ============================================================================

    #[test]
    fn test_order_validation() {
        assert!(Order::new(3).is_ok());
        assert!(Order::new(100).is_ok());
        assert!(Order::new(1000).is_ok());

        assert!(Order::new(0).is_err());
        assert!(Order::new(1).is_err());
        assert!(Order::new(2).is_err());
    }

    #[test]
    fn test_order_properties() {
        let order = Order::new(100).unwrap();
        assert_eq!(order.value(), 100);
        assert_eq!(order.max_keys(), 100);
        assert_eq!(order.min_keys(), 50);
    }
}
