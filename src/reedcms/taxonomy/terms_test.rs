// Copyright 2025 Vivian Voss. Licensed under the Apache Licence, Version 2.0.
// SPDX-Licence-Identifier: Apache-2.0

#[cfg(test)]
mod tests {
    use super::super::terms::*;
    use std::fs;

    fn setup_test_env() {
        let _ = fs::create_dir_all(".reed");
        let _ = fs::create_dir_all(".reed/backups");
        cleanup_test_terms();
    }

    fn cleanup_test_terms() {
        let _ = fs::remove_file(".reed/taxonomie.matrix.csv");
    }

    #[test]
    fn test_create_term_basic() {
        setup_test_env();

        let result = create_term(
            "Rust",
            None,
            "Programming",
            Some("Systems programming language".to_string()),
            Some("#FF6600".to_string()),
            Some("rust-logo".to_string()),
            "admin",
        );

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.data.term, "Rust");
        assert_eq!(response.data.category, "Programming");
        assert_eq!(response.data.status, "active");
        assert_eq!(response.data.usage_count, 0);
    }

    #[test]
    fn test_create_term_with_parent() {
        setup_test_env();

        // Create parent
        create_term("Programming", None, "Topics", None, None, None, "admin").unwrap();

        // Create child
        let result = create_term(
            "Rust",
            Some("Topics:Programming".to_string()),
            "Topics",
            None,
            None,
            None,
            "admin",
        );

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(
            response.data.parent_id,
            Some("Topics:Programming".to_string())
        );
    }

    #[test]
    fn test_create_term_invalid_name() {
        setup_test_env();

        // Too short
        let result = create_term("R", None, "Programming", None, None, None, "admin");
        assert!(result.is_err());

        // Invalid characters
        let result = create_term("Rust@#$", None, "Programming", None, None, None, "admin");
        assert!(result.is_err());
    }

    #[test]
    fn test_create_term_invalid_color() {
        setup_test_env();

        let result = create_term(
            "Rust",
            None,
            "Programming",
            None,
            Some("FF6600".to_string()), // Missing #
            None,
            "admin",
        );
        assert!(result.is_err());

        let result = create_term(
            "Rust",
            None,
            "Programming",
            None,
            Some("#GGGGGG".to_string()), // Invalid hex
            None,
            "admin",
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_create_term_duplicate() {
        setup_test_env();

        create_term("Rust", None, "Programming", None, None, None, "admin").unwrap();

        let result = create_term("Rust", None, "Programming", None, None, None, "admin");
        assert!(result.is_err());
    }

    #[test]
    fn test_create_term_parent_not_exists() {
        setup_test_env();

        let result = create_term(
            "Rust",
            Some("NonExistent:Parent".to_string()),
            "Programming",
            None,
            None,
            None,
            "admin",
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_get_term() {
        setup_test_env();

        create_term(
            "Rust",
            None,
            "Programming",
            Some("Desc".to_string()),
            None,
            None,
            "admin",
        )
        .unwrap();

        let result = get_term("Programming:Rust");
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.data.term, "Rust");
        assert_eq!(response.data.description, Some("Desc".to_string()));
    }

    #[test]
    fn test_get_term_not_found() {
        setup_test_env();

        let result = get_term("NonExistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_list_terms_all() {
        setup_test_env();

        create_term("Rust", None, "Programming", None, None, None, "admin").unwrap();
        create_term("Python", None, "Programming", None, None, None, "admin").unwrap();
        create_term("Docker", None, "DevOps", None, None, None, "admin").unwrap();

        let result = list_terms(None, None, None);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().data.len(), 3);
    }

    #[test]
    fn test_list_terms_by_category() {
        setup_test_env();

        create_term("Rust", None, "Programming", None, None, None, "admin").unwrap();
        create_term("Python", None, "Programming", None, None, None, "admin").unwrap();
        create_term("Docker", None, "DevOps", None, None, None, "admin").unwrap();

        let result = list_terms(Some("Programming"), None, None);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().data.len(), 2);
    }

    #[test]
    fn test_list_terms_by_parent() {
        setup_test_env();

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

        let result = list_terms(None, Some("Topics:Programming"), None);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().data.len(), 2);
    }

    #[test]
    fn test_list_terms_root_only() {
        setup_test_env();

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

        let result = list_terms(None, Some("root"), None);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().data.len(), 1);
    }

    #[test]
    fn test_search_terms() {
        setup_test_env();

        create_term(
            "Rust",
            None,
            "Programming",
            Some("Systems programming".to_string()),
            None,
            None,
            "admin",
        )
        .unwrap();
        create_term(
            "Python",
            None,
            "Programming",
            Some("General purpose".to_string()),
            None,
            None,
            "admin",
        )
        .unwrap();

        let result = search_terms("rust", None);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().data.len(), 1);

        let result = search_terms("programming", None);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().data.len(), 2); // Matches description
    }

    #[test]
    fn test_update_term() {
        setup_test_env();

        create_term("Rust", None, "Programming", None, None, None, "admin").unwrap();

        let update = TermUpdate {
            description: Some(Some("Updated description".to_string())),
            color: Some(Some("#FF6600".to_string())),
            ..Default::default()
        };

        let result = update_term("Programming:Rust", update);
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(
            response.data.description,
            Some("Updated description".to_string())
        );
        assert_eq!(response.data.color, Some("#FF6600".to_string()));
    }

    #[test]
    fn test_update_term_change_parent() {
        setup_test_env();

        create_term("Languages", None, "Topics", None, None, None, "admin").unwrap();
        create_term("Programming", None, "Topics", None, None, None, "admin").unwrap();
        create_term(
            "Rust",
            Some("Topics:Languages".to_string()),
            "Topics",
            None,
            None,
            None,
            "admin",
        )
        .unwrap();

        let update = TermUpdate {
            parent_id: Some(Some("Topics:Programming".to_string())),
            ..Default::default()
        };

        let result = update_term("Topics:Rust", update);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap().data.parent_id,
            Some("Topics:Programming".to_string())
        );
    }

    #[test]
    fn test_update_term_remove_parent() {
        setup_test_env();

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

        let update = TermUpdate {
            parent_id: Some(None),
            ..Default::default()
        };

        let result = update_term("Topics:Rust", update);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().data.parent_id, None);
    }

    #[test]
    fn test_update_term_circular_parent() {
        setup_test_env();

        create_term("Rust", None, "Programming", None, None, None, "admin").unwrap();

        let update = TermUpdate {
            parent_id: Some(Some("Programming:Rust".to_string())),
            ..Default::default()
        };

        let result = update_term("Programming:Rust", update);
        assert!(result.is_err());
    }

    #[test]
    fn test_delete_term() {
        setup_test_env();

        create_term("Rust", None, "Programming", None, None, None, "admin").unwrap();

        let result = delete_term("Programming:Rust", false);
        assert!(result.is_ok());

        let get_result = get_term("Programming:Rust");
        assert!(get_result.is_err());
    }

    #[test]
    fn test_delete_term_with_children_no_force() {
        setup_test_env();

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

        let result = delete_term("Topics:Programming", false);
        assert!(result.is_err());
    }

    #[test]
    fn test_delete_term_with_children_force() {
        setup_test_env();

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

        let result = delete_term("Topics:Programming", true);
        assert!(result.is_ok());

        // Both parent and child should be deleted
        assert!(get_term("Topics:Programming").is_err());
        assert!(get_term("Topics:Rust").is_err());
    }

    #[test]
    fn test_performance_create_100_terms() {
        setup_test_env();

        let start = std::time::Instant::now();

        for i in 0..100 {
            create_term(
                &format!("Term{}", i),
                None,
                "Performance",
                None,
                None,
                None,
                "admin",
            )
            .unwrap();
        }

        let duration = start.elapsed();
        println!("Creating 100 terms took: {}ms", duration.as_millis());
        assert!(duration.as_secs() < 5); // Should complete in <5s
    }

    #[test]
    fn test_performance_search_1000_terms() {
        setup_test_env();

        // Create 1000 terms
        for i in 0..1000 {
            create_term(
                &format!("Term{}", i),
                None,
                "Performance",
                Some(format!("Description {}", i)),
                None,
                None,
                "admin",
            )
            .unwrap();
        }

        let start = std::time::Instant::now();
        let result = search_terms("Term5", None);
        let duration = start.elapsed();

        println!("Searching 1000 terms took: {}μs", duration.as_micros());
        assert!(result.is_ok());
        assert!(duration.as_millis() < 50); // Target: <50ms
    }
}
