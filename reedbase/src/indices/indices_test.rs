// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Tests for Smart Indices system.

#[cfg(test)]
mod tests {
    use crate::indices::*;

    // Helper: Create test KeyIndex
    fn make_key_index(
        row: usize,
        key: &str,
        namespace: &str,
        hierarchy: Vec<&str>,
        lang: Option<&str>,
        env: Option<&str>,
    ) -> KeyIndex {
        KeyIndex {
            row,
            key: key.to_string(),
            namespace: namespace.to_string(),
            hierarchy: hierarchy.into_iter().map(|s| s.to_string()).collect(),
            modifiers: Modifiers {
                language: lang.map(|s| s.to_string()),
                environment: env.map(|s| s.to_string()),
                season: None,
                variant: None,
            },
        }
    }

    // NamespaceIndex Tests
    #[test]
    fn test_namespace_index_basic() {
        let mut index = NamespaceIndex::new();

        let keys = vec![
            make_key_index(
                0,
                "page.header.title",
                "page",
                vec!["page", "header", "title"],
                None,
                None,
            ),
            make_key_index(
                5,
                "page.footer.links",
                "page",
                vec!["page", "footer", "links"],
                None,
                None,
            ),
            make_key_index(
                10,
                "api.auth.token",
                "api",
                vec!["api", "auth", "token"],
                None,
                None,
            ),
        ];

        index.build(&keys).unwrap();

        assert_eq!(index.query("page"), Some(&[0, 5][..]));
        assert_eq!(index.query("api"), Some(&[10][..]));
        assert_eq!(index.query("unknown"), None);
        assert_eq!(index.namespace_count(), 2);
        assert_eq!(index.key_count(), 3);
    }

    #[test]
    fn test_namespace_index_insert_remove() {
        let mut index = NamespaceIndex::new();

        let key = make_key_index(0, "page.title", "page", vec!["page", "title"], None, None);
        index.insert(&key).unwrap();

        assert_eq!(index.query("page"), Some(&[0][..]));

        index.remove(0).unwrap();
        assert_eq!(index.query("page"), Some(&[][..]));
    }

    // ModifierIndex Tests
    #[test]
    fn test_language_index() {
        let mut index = ModifierIndex::language();

        let keys = vec![
            make_key_index(
                0,
                "page.title<de>",
                "page",
                vec!["page", "title"],
                Some("de"),
                None,
            ),
            make_key_index(
                5,
                "page.subtitle<de>",
                "page",
                vec!["page", "subtitle"],
                Some("de"),
                None,
            ),
            make_key_index(
                10,
                "page.title<en>",
                "page",
                vec!["page", "title"],
                Some("en"),
                None,
            ),
        ];

        index.build(&keys).unwrap();

        assert_eq!(index.query("de"), Some(&[0, 5][..]));
        assert_eq!(index.query("en"), Some(&[10][..]));
        assert_eq!(index.query("fr"), None);
        assert_eq!(index.value_count(), 2);
    }

    #[test]
    fn test_environment_index() {
        let mut index = ModifierIndex::environment();

        let keys = vec![
            make_key_index(
                0,
                "api.url<,dev>",
                "api",
                vec!["api", "url"],
                None,
                Some("dev"),
            ),
            make_key_index(
                5,
                "api.key<,prod>",
                "api",
                vec!["api", "key"],
                None,
                Some("prod"),
            ),
        ];

        index.build(&keys).unwrap();

        assert_eq!(index.query("dev"), Some(&[0][..]));
        assert_eq!(index.query("prod"), Some(&[5][..]));
    }

    // HierarchyTrie Tests
    #[test]
    fn test_hierarchy_trie_exact() {
        let mut trie = HierarchyTrie::new();

        let keys = vec![
            make_key_index(
                0,
                "page.header.logo",
                "page",
                vec!["page", "header", "logo"],
                None,
                None,
            ),
            make_key_index(
                5,
                "page.header.title",
                "page",
                vec!["page", "header", "title"],
                None,
                None,
            ),
            make_key_index(
                10,
                "page.footer.links",
                "page",
                vec!["page", "footer", "links"],
                None,
                None,
            ),
        ];

        trie.build(&keys).unwrap();

        let pattern = vec!["page".into(), "header".into(), "logo".into()];
        let result = trie.query(&pattern);
        assert_eq!(result, vec![0]);
    }

    #[test]
    fn test_hierarchy_trie_wildcard() {
        let mut trie = HierarchyTrie::new();

        let keys = vec![
            make_key_index(
                0,
                "page.header.logo",
                "page",
                vec!["page", "header", "logo"],
                None,
                None,
            ),
            make_key_index(
                5,
                "page.header.title",
                "page",
                vec!["page", "header", "title"],
                None,
                None,
            ),
            make_key_index(
                10,
                "page.footer.links",
                "page",
                vec!["page", "footer", "links"],
                None,
                None,
            ),
        ];

        trie.build(&keys).unwrap();

        let pattern = vec!["page".into(), "header".into(), "*".into()];
        let mut result = trie.query(&pattern);
        result.sort();
        assert_eq!(result, vec![0, 5]);
    }

    #[test]
    fn test_hierarchy_trie_wildcard_all_descendants() {
        let mut trie = HierarchyTrie::new();

        let keys = vec![
            make_key_index(
                0,
                "page.header.logo",
                "page",
                vec!["page", "header", "logo"],
                None,
                None,
            ),
            make_key_index(
                5,
                "page.footer.links",
                "page",
                vec!["page", "footer", "links"],
                None,
                None,
            ),
            make_key_index(
                10,
                "api.auth.token",
                "api",
                vec!["api", "auth", "token"],
                None,
                None,
            ),
        ];

        trie.build(&keys).unwrap();

        let pattern = vec!["page".into(), "*".into()];
        let mut result = trie.query(&pattern);
        result.sort();
        assert_eq!(result, vec![0, 5]);
    }

    // IndexManager Tests
    #[test]
    fn test_index_manager_single_filter() {
        let manager = IndexManager::new();

        let filter = QueryFilter::new().with_namespace("page");

        // Empty result for unbuilt index
        let result = manager.query(&filter).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_index_manager_combined_filters() {
        let mut manager = IndexManager::new();

        let keys = vec![
            make_key_index(
                0,
                "page.title<de,prod>",
                "page",
                vec!["page", "title"],
                Some("de"),
                Some("prod"),
            ),
            make_key_index(
                5,
                "page.subtitle<de,dev>",
                "page",
                vec!["page", "subtitle"],
                Some("de"),
                Some("dev"),
            ),
            make_key_index(
                10,
                "api.url<en,prod>",
                "api",
                vec!["api", "url"],
                Some("en"),
                Some("prod"),
            ),
        ];

        for key in &keys {
            manager.insert(key).unwrap();
        }

        // Test: namespace=page AND language=de AND environment=prod
        let filter = QueryFilter::new()
            .with_namespace("page")
            .with_language("de")
            .with_environment("prod");

        let result = manager.query(&filter).unwrap();
        assert_eq!(result, vec![0]);
    }

    #[test]
    fn test_index_manager_no_matches() {
        let mut manager = IndexManager::new();

        let key = make_key_index(
            0,
            "page.title<de>",
            "page",
            vec!["page", "title"],
            Some("de"),
            None,
        );
        manager.insert(&key).unwrap();

        // Query with non-existent language
        let filter = QueryFilter::new()
            .with_namespace("page")
            .with_language("fr");

        let result = manager.query(&filter).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_index_manager_stats() {
        let mut manager = IndexManager::new();

        let keys = vec![
            make_key_index(
                0,
                "page.title<de,prod>",
                "page",
                vec!["page", "title"],
                Some("de"),
                Some("prod"),
            ),
            make_key_index(
                5,
                "api.url<en,dev>",
                "api",
                vec!["api", "url"],
                Some("en"),
                Some("dev"),
            ),
        ];

        for key in &keys {
            manager.insert(key).unwrap();
        }

        let stats = manager.stats();
        assert_eq!(stats.total_keys, 2);
        assert_eq!(stats.namespaces, 2);
        assert_eq!(stats.languages, 2);
        assert_eq!(stats.environments, 2);
        assert!(stats.memory_bytes > 0);
    }

    #[test]
    fn test_query_filter_builder() {
        let filter = QueryFilter::new()
            .with_namespace("page")
            .with_language("de")
            .with_environment("prod");

        assert_eq!(filter.namespace, Some("page".to_string()));
        assert_eq!(filter.language, Some("de".to_string()));
        assert_eq!(filter.environment, Some("prod".to_string()));
        assert!(!filter.is_empty());
    }

    #[test]
    fn test_query_filter_empty() {
        let filter = QueryFilter::new();
        assert!(filter.is_empty());
    }

    #[test]
    fn test_index_manager_memory_usage() {
        let mut manager = IndexManager::new();

        let keys = vec![
            make_key_index(0, "page.title", "page", vec!["page", "title"], None, None),
            make_key_index(5, "api.url", "api", vec!["api", "url"], None, None),
        ];

        for key in &keys {
            manager.insert(key).unwrap();
        }

        let memory = manager.memory_usage();
        assert!(memory > 0);
        assert!(memory < 10000); // Should be small for 2 keys
    }

    #[test]
    fn test_index_manager_clear() {
        let mut manager = IndexManager::new();

        let key = make_key_index(0, "page.title", "page", vec!["page", "title"], None, None);
        manager.insert(&key).unwrap();

        let stats_before = manager.stats();
        assert_eq!(stats_before.total_keys, 1);

        manager.clear();

        let stats_after = manager.stats();
        assert_eq!(stats_after.total_keys, 0);
    }

    #[test]
    fn test_index_manager_insert() {
        let mut manager = IndexManager::new();

        let key = make_key_index(
            0,
            "page.title<de>",
            "page",
            vec!["page", "title"],
            Some("de"),
            None,
        );
        manager.insert(&key).unwrap();

        // Verify via query
        let filter = QueryFilter::new().with_namespace("page");
        let result = manager.query(&filter).unwrap();
        assert_eq!(result, vec![0]);

        let filter = QueryFilter::new().with_language("de");
        let result = manager.query(&filter).unwrap();
        assert_eq!(result, vec![0]);
    }

    #[test]
    fn test_index_manager_remove() {
        let mut manager = IndexManager::new();

        let key = make_key_index(0, "page.title", "page", vec!["page", "title"], None, None);
        manager.insert(&key).unwrap();

        let filter = QueryFilter::new().with_namespace("page");
        let result = manager.query(&filter).unwrap();
        assert_eq!(result, vec![0]);

        manager.remove(0).unwrap();

        let result = manager.query(&filter).unwrap();
        assert!(result.is_empty());
    }
}
