// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Tests for index builder factory.

#[cfg(test)]
mod tests {
    use crate::indices::builder::{IndexBackend, IndexBuilder, IndexConfig};
    use crate::indices::Index;
    use tempfile::TempDir;

    #[test]
    fn test_default_config() {
        let config = IndexConfig::default();
        assert_eq!(config.backend, IndexBackend::HashMap);
        assert_eq!(config.btree_order, None);
        assert_eq!(config.persist_path, None);
    }

    #[test]
    fn test_hashmap_backend() {
        let config = IndexConfig {
            backend: IndexBackend::HashMap,
            btree_order: None,
            persist_path: None,
        };

        let builder = IndexBuilder::new(config);
        assert_eq!(builder.config().backend, IndexBackend::HashMap);
    }

    #[test]
    fn test_btree_backend_config() {
        let config = IndexConfig {
            backend: IndexBackend::BTree,
            btree_order: Some(100),
            persist_path: Some("/tmp/test".to_string()),
        };

        let builder = IndexBuilder::new(config);
        assert_eq!(builder.config().backend, IndexBackend::BTree);
        assert_eq!(builder.config().btree_order, Some(100));
    }

    #[test]
    fn test_from_toml_hashmap() {
        let toml = r#"
            backend = "hashmap"
        "#;

        let builder = IndexBuilder::from_toml(toml).expect("Valid TOML");
        assert_eq!(builder.config().backend, IndexBackend::HashMap);
    }

    #[test]
    fn test_from_toml_btree() {
        let toml = r#"
            backend = "btree"
            btree_order = 150
            persist_path = "/tmp/indices"
        "#;

        let builder = IndexBuilder::from_toml(toml).expect("Valid TOML");
        assert_eq!(builder.config().backend, IndexBackend::BTree);
        assert_eq!(builder.config().btree_order, Some(150));
        assert_eq!(
            builder.config().persist_path.as_deref(),
            Some("/tmp/indices")
        );
    }

    #[test]
    fn test_from_toml_invalid() {
        let toml = "invalid { toml";
        let result = IndexBuilder::from_toml(toml);
        assert!(result.is_err());
    }

    #[test]
    fn test_build_hashmap_namespace_index() {
        let config = IndexConfig::default();
        let builder = IndexBuilder::new(config);

        let index = builder
            .build_namespace_index()
            .expect("Build HashMap index");
        assert_eq!(index.backend_type(), "hashmap");
        assert_eq!(index.disk_usage(), 0); // HashMap has no disk usage
    }

    #[test]
    fn test_build_hashmap_language_index() {
        let config = IndexConfig::default();
        let builder = IndexBuilder::new(config);

        let index = builder.build_language_index().expect("Build HashMap index");
        assert_eq!(index.backend_type(), "hashmap");
    }

    #[test]
    fn test_build_hashmap_environment_index() {
        let config = IndexConfig::default();
        let builder = IndexBuilder::new(config);

        let index = builder
            .build_environment_index()
            .expect("Build HashMap index");
        assert_eq!(index.backend_type(), "hashmap");
    }

    #[test]
    fn test_build_hashmap_season_index() {
        let config = IndexConfig::default();
        let builder = IndexBuilder::new(config);

        let index = builder.build_season_index().expect("Build HashMap index");
        assert_eq!(index.backend_type(), "hashmap");
    }

    #[test]
    fn test_build_hashmap_variant_index() {
        let config = IndexConfig::default();
        let builder = IndexBuilder::new(config);

        let index = builder.build_variant_index().expect("Build HashMap index");
        assert_eq!(index.backend_type(), "hashmap");
    }

    #[test]
    fn test_build_hashmap_hierarchy_index() {
        let config = IndexConfig::default();
        let builder = IndexBuilder::new(config);

        let index = builder
            .build_hierarchy_index()
            .expect("Build HashMap index");
        assert_eq!(index.backend_type(), "hashmap");
    }

    #[test]
    fn test_build_btree_namespace_index() {
        let temp_dir = TempDir::new().expect("Create temp dir");
        let persist_path = temp_dir.path().to_str().unwrap().to_string();

        let config = IndexConfig {
            backend: IndexBackend::BTree,
            btree_order: Some(100),
            persist_path: Some(persist_path),
        };

        let builder = IndexBuilder::new(config);
        let index = builder.build_namespace_index().expect("Build BTree index");

        assert_eq!(index.backend_type(), "btree");
        assert!(index.disk_usage() > 0); // BTree has disk usage
    }

    #[test]
    fn test_build_btree_language_index() {
        let temp_dir = TempDir::new().expect("Create temp dir");
        let persist_path = temp_dir.path().to_str().unwrap().to_string();

        let config = IndexConfig {
            backend: IndexBackend::BTree,
            btree_order: Some(100),
            persist_path: Some(persist_path),
        };

        let builder = IndexBuilder::new(config);
        let index = builder.build_language_index().expect("Build BTree index");

        assert_eq!(index.backend_type(), "btree");
    }

    #[test]
    fn test_build_btree_missing_persist_path() {
        let config = IndexConfig {
            backend: IndexBackend::BTree,
            btree_order: Some(100),
            persist_path: None, // Missing!
        };

        let builder = IndexBuilder::new(config);
        let result = builder.build_namespace_index();

        assert!(result.is_err());
        // Should error with message about missing persist_path
    }

    #[test]
    fn test_build_btree_invalid_order() {
        let temp_dir = TempDir::new().expect("Create temp dir");
        let persist_path = temp_dir.path().to_str().unwrap().to_string();

        let config = IndexConfig {
            backend: IndexBackend::BTree,
            btree_order: Some(2), // Invalid (< 3)
            persist_path: Some(persist_path),
        };

        let builder = IndexBuilder::new(config);
        let result = builder.build_namespace_index();

        assert!(result.is_err());
        // Should error with invalid order
    }

    #[test]
    fn test_hashmap_index_operations() {
        let config = IndexConfig::default();
        let builder = IndexBuilder::new(config);

        let mut index = builder.build_namespace_index().expect("Build index");

        // Test insert
        index
            .insert("page".to_string(), vec![1, 2, 3])
            .expect("Insert");

        // Test get
        let value = index.get(&"page".to_string()).expect("Get");
        assert_eq!(value, Some(vec![1, 2, 3]));

        // Test delete
        index.delete(&"page".to_string()).expect("Delete");
        let value = index.get(&"page".to_string()).expect("Get after delete");
        assert_eq!(value, None);
    }

    #[test]
    fn test_hashmap_range_unsupported() {
        let config = IndexConfig::default();
        let builder = IndexBuilder::new(config);

        let index = builder.build_namespace_index().expect("Build index");

        // HashMap should not support range queries
        let result = index.range(&"a".to_string(), &"z".to_string());
        assert!(result.is_err());
        // Should be IndexOperationUnsupported error
    }

    #[test]
    fn test_btree_index_operations() {
        let temp_dir = TempDir::new().expect("Create temp dir");
        let persist_path = temp_dir.path().to_str().unwrap().to_string();

        let config = IndexConfig {
            backend: IndexBackend::BTree,
            btree_order: Some(100),
            persist_path: Some(persist_path),
        };

        let builder = IndexBuilder::new(config);
        let mut index = builder.build_namespace_index().expect("Build index");

        // Test insert
        index
            .insert("page".to_string(), vec![1, 2, 3])
            .expect("Insert");

        // Test get
        let value = index.get(&"page".to_string()).expect("Get");
        assert_eq!(value, Some(vec![1, 2, 3]));

        // Test delete
        index.delete(&"page".to_string()).expect("Delete");
        let value = index.get(&"page".to_string()).expect("Get after delete");
        assert_eq!(value, None);
    }

    #[test]
    fn test_btree_range_supported() {
        let temp_dir = TempDir::new().expect("Create temp dir");
        let persist_path = temp_dir.path().to_str().unwrap().to_string();

        let config = IndexConfig {
            backend: IndexBackend::BTree,
            btree_order: Some(100),
            persist_path: Some(persist_path),
        };

        let builder = IndexBuilder::new(config);
        let mut index = builder.build_namespace_index().expect("Build index");

        // Insert multiple entries
        index
            .insert("page.title".to_string(), vec![1])
            .expect("Insert");
        index
            .insert("page.description".to_string(), vec![2])
            .expect("Insert");
        index
            .insert("api.endpoint".to_string(), vec![3])
            .expect("Insert");

        // Range query should work
        let results = index
            .range(&"page.a".to_string(), &"page.z".to_string())
            .expect("Range query");

        assert_eq!(results.len(), 2); // page.title and page.description
    }

    #[test]
    fn test_index_backend_serialization() {
        // Test that IndexBackend can be serialized/deserialized
        let hashmap = IndexBackend::HashMap;
        let btree = IndexBackend::BTree;

        assert_eq!(hashmap, IndexBackend::HashMap);
        assert_eq!(btree, IndexBackend::BTree);
        assert_ne!(hashmap, btree);
    }

    #[test]
    fn test_config_serialization() {
        let config = IndexConfig {
            backend: IndexBackend::BTree,
            btree_order: Some(150),
            persist_path: Some("/tmp/test".to_string()),
        };

        // Serialize to TOML
        let toml = toml::to_string(&config).expect("Serialize to TOML");
        assert!(toml.contains("btree"));
        assert!(toml.contains("150"));

        // Deserialize back
        let parsed: IndexConfig = toml::from_str(&toml).expect("Deserialize from TOML");
        assert_eq!(parsed.backend, IndexBackend::BTree);
        assert_eq!(parsed.btree_order, Some(150));
    }
}
