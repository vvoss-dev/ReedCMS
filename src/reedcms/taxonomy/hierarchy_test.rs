// Copyright 2025 Vivian Voss. Licensed under the Apache Licence, Version 2.0.
// SPDX-Licence-Identifier: Apache-2.0

#[cfg(test)]
mod tests {
    use super::super::hierarchy::*;
    use super::super::terms::*;
    use std::fs;

    fn setup_test_env() {
        let _ = fs::create_dir_all(".reed");
        let _ = fs::create_dir_all(".reed/backups");
        cleanup_test_data();
    }

    fn cleanup_test_data() {
        let _ = fs::remove_file(".reed/taxonomie.matrix.csv");
    }

    fn create_test_hierarchy() {
        // Create a tree:
        // Programming
        //   ├─ Rust
        //   │   ├─ Async
        //   │   └─ Macros
        //   └─ Python
        //       └─ Django

        create_term("Programming", None, "Topics", None, None, None, "admin").unwrap();
        create_term(
            "Rust",
            Some("Topics:Programming".to_string()),
            "Topics",
            None,
            None,
            None,
            "admin",
        )
        .unwrap();
        create_term(
            "Python",
            Some("Topics:Programming".to_string()),
            "Topics",
            None,
            None,
            None,
            "admin",
        )
        .unwrap();
        create_term(
            "Async",
            Some("Topics:Rust".to_string()),
            "Topics",
            None,
            None,
            None,
            "admin",
        )
        .unwrap();
        create_term(
            "Macros",
            Some("Topics:Rust".to_string()),
            "Topics",
            None,
            None,
            None,
            "admin",
        )
        .unwrap();
        create_term(
            "Django",
            Some("Topics:Python".to_string()),
            "Topics",
            None,
            None,
            None,
            "admin",
        )
        .unwrap();
    }

    #[test]
    fn test_get_children_direct() {
        setup_test_env();
        create_test_hierarchy();

        let result = get_children("Topics:Programming", false);
        assert!(result.is_ok());
        let children = result.unwrap().data;
        assert_eq!(children.len(), 2); // Rust, Python
        assert!(children.iter().any(|c| c.term == "Rust"));
        assert!(children.iter().any(|c| c.term == "Python"));
    }

    #[test]
    fn test_get_children_recursive() {
        setup_test_env();
        create_test_hierarchy();

        let result = get_children("Topics:Programming", true);
        assert!(result.is_ok());
        let children = result.unwrap().data;
        assert_eq!(children.len(), 5); // Rust, Python, Async, Macros, Django
    }

    #[test]
    fn test_get_children_leaf_node() {
        setup_test_env();
        create_test_hierarchy();

        let result = get_children("Topics:Async", false);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().data.len(), 0);
    }

    #[test]
    fn test_get_children_not_found() {
        setup_test_env();
        create_test_hierarchy();

        let result = get_children("NonExistent", false);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_ancestors() {
        setup_test_env();
        create_test_hierarchy();

        let result = get_ancestors("Topics:Async");
        assert!(result.is_ok());
        let ancestors = result.unwrap().data;
        assert_eq!(ancestors.len(), 2); // Programming, Rust
        assert_eq!(ancestors[0].term, "Programming");
        assert_eq!(ancestors[1].term, "Rust");
    }

    #[test]
    fn test_get_ancestors_root() {
        setup_test_env();
        create_test_hierarchy();

        let result = get_ancestors("Topics:Programming");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().data.len(), 0); // No ancestors
    }

    #[test]
    fn test_get_ancestors_not_found() {
        setup_test_env();
        create_test_hierarchy();

        let result = get_ancestors("NonExistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_get_path() {
        setup_test_env();
        create_test_hierarchy();

        let result = get_path("Topics:Async", " > ");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().data, "Programming > Rust > Async");
    }

    #[test]
    fn test_get_path_custom_separator() {
        setup_test_env();
        create_test_hierarchy();

        let result = get_path("Topics:Django", " / ");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().data, "Programming / Python / Django");
    }

    #[test]
    fn test_get_path_root() {
        setup_test_env();
        create_test_hierarchy();

        let result = get_path("Topics:Programming", " > ");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().data, "Programming");
    }

    #[test]
    fn test_get_depth() {
        setup_test_env();
        create_test_hierarchy();

        assert_eq!(get_depth("Topics:Programming").unwrap().data, 0);
        assert_eq!(get_depth("Topics:Rust").unwrap().data, 1);
        assert_eq!(get_depth("Topics:Async").unwrap().data, 2);
    }

    #[test]
    fn test_has_circular_reference_self() {
        setup_test_env();
        create_test_hierarchy();

        let result = has_circular_reference("Topics:Programming", "Topics:Programming");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().data, true);
    }

    #[test]
    fn test_has_circular_reference_descendant() {
        setup_test_env();
        create_test_hierarchy();

        // Try to make Programming a child of Rust (which is a descendant)
        let result = has_circular_reference("Topics:Programming", "Topics:Rust");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().data, true);

        // Try to make Programming a child of Async (deeper descendant)
        let result = has_circular_reference("Topics:Programming", "Topics:Async");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().data, true);
    }

    #[test]
    fn test_has_circular_reference_valid() {
        setup_test_env();
        create_test_hierarchy();

        // Making Rust a child of Python is valid (siblings)
        let result = has_circular_reference("Topics:Rust", "Topics:Python");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().data, false);

        // Making Django a child of Programming is valid (ancestor)
        let result = has_circular_reference("Topics:Django", "Topics:Programming");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().data, false);
    }

    #[test]
    fn test_get_tree() {
        setup_test_env();
        create_test_hierarchy();

        let result = get_tree(Some("Topics"));
        assert!(result.is_ok());
        let tree = result.unwrap().data;

        assert_eq!(tree.len(), 1); // One root: Programming
        assert_eq!(tree[0].term.term, "Programming");
        assert_eq!(tree[0].children.len(), 2); // Rust, Python

        // Check Rust branch
        let rust = tree[0]
            .children
            .iter()
            .find(|c| c.term.term == "Rust")
            .unwrap();
        assert_eq!(rust.children.len(), 2); // Async, Macros

        // Check Python branch
        let python = tree[0]
            .children
            .iter()
            .find(|c| c.term.term == "Python")
            .unwrap();
        assert_eq!(python.children.len(), 1); // Django
    }

    #[test]
    fn test_get_tree_multiple_roots() {
        setup_test_env();

        // Create two separate trees
        create_term("Programming", None, "Topics", None, None, None, "admin").unwrap();
        create_term("DevOps", None, "Topics", None, None, None, "admin").unwrap();
        create_term(
            "Rust",
            Some("Topics:Programming".to_string()),
            "Topics",
            None,
            None,
            None,
            "admin",
        )
        .unwrap();
        create_term(
            "Docker",
            Some("Topics:DevOps".to_string()),
            "Topics",
            None,
            None,
            None,
            "admin",
        )
        .unwrap();

        let result = get_tree(Some("Topics"));
        assert!(result.is_ok());
        let tree = result.unwrap().data;

        assert_eq!(tree.len(), 2); // Two roots
    }

    #[test]
    fn test_get_tree_empty() {
        setup_test_env();

        let result = get_tree(Some("NonExistent"));
        assert!(result.is_ok());
        assert_eq!(result.unwrap().data.len(), 0);
    }

    #[test]
    fn test_deep_hierarchy() {
        setup_test_env();

        // Create a deep chain
        create_term("L0", None, "Deep", None, None, None, "admin").unwrap();
        create_term(
            "L1",
            Some("Deep:L0".to_string()),
            "Deep",
            None,
            None,
            None,
            "admin",
        )
        .unwrap();
        create_term(
            "L2",
            Some("Deep:L1".to_string()),
            "Deep",
            None,
            None,
            None,
            "admin",
        )
        .unwrap();
        create_term(
            "L3",
            Some("Deep:L2".to_string()),
            "Deep",
            None,
            None,
            None,
            "admin",
        )
        .unwrap();
        create_term(
            "L4",
            Some("Deep:L3".to_string()),
            "Deep",
            None,
            None,
            None,
            "admin",
        )
        .unwrap();

        // Test depth
        assert_eq!(get_depth("Deep:L4").unwrap().data, 4);

        // Test ancestors
        let ancestors = get_ancestors("Deep:L4").unwrap().data;
        assert_eq!(ancestors.len(), 4);

        // Test path
        let path = get_path("Deep:L4", " → ").unwrap().data;
        assert_eq!(path, "L0 → L1 → L2 → L3 → L4");
    }

    #[test]
    fn test_performance_get_children_1000_terms() {
        setup_test_env();

        // Create root
        create_term("Root", None, "Performance", None, None, None, "admin").unwrap();

        // Create 1000 children
        for i in 0..1000 {
            create_term(
                &format!("Child{}", i),
                Some("Performance:Root".to_string()),
                "Performance",
                None,
                None,
                None,
                "admin",
            )
            .unwrap();
        }

        let start = std::time::Instant::now();
        let result = get_children("Performance:Root", false);
        let duration = start.elapsed();

        println!("Getting 1000 children took: {}μs", duration.as_micros());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().data.len(), 1000);
        assert!(duration.as_millis() < 50);
    }

    #[test]
    fn test_performance_get_tree_deep() {
        setup_test_env();

        // Create a tree with depth 10
        let mut parent = None;
        for i in 0..10 {
            create_term(
                &format!("Level{}", i),
                parent.clone(),
                "Performance",
                None,
                None,
                None,
                "admin",
            )
            .unwrap();
            parent = Some(format!("Performance:Level{}", i));
        }

        let start = std::time::Instant::now();
        let result = get_tree(Some("Performance"));
        let duration = start.elapsed();

        println!("Building tree (depth 10) took: {}μs", duration.as_micros());
        assert!(result.is_ok());
        assert!(duration.as_millis() < 100);
    }

    #[test]
    fn test_performance_get_ancestors_depth_10() {
        setup_test_env();

        // Create chain with depth 10
        let mut parent = None;
        for i in 0..10 {
            create_term(
                &format!("Level{}", i),
                parent.clone(),
                "Performance",
                None,
                None,
                None,
                "admin",
            )
            .unwrap();
            parent = Some(format!("Performance:Level{}", i));
        }

        let start = std::time::Instant::now();
        let result = get_ancestors("Performance:Level9");
        let duration = start.elapsed();

        println!(
            "Getting ancestors (depth 10) took: {}μs",
            duration.as_micros()
        );
        assert!(result.is_ok());
        assert_eq!(result.unwrap().data.len(), 9);
        assert!(duration.as_millis() < 5);
    }
}
