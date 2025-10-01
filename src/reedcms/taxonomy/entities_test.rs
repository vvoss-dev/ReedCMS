// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

#[cfg(test)]
mod tests {
    use super::super::entities::*;
    use super::super::terms::*;
    use std::fs;

    fn setup_test_env() {
        let _ = fs::create_dir_all(".reed");
        let _ = fs::create_dir_all(".reed/backups");
        cleanup_test_data();
    }

    fn cleanup_test_data() {
        let _ = fs::remove_file(".reed/taxonomie.matrix.csv");
        let _ = fs::remove_file(".reed/entity_taxonomy.matrix.csv");
    }

    #[test]
    fn test_assign_terms() {
        setup_test_env();

        // Create terms
        create_term("Rust", None, "Programming", None, None, None, "admin").unwrap();
        create_term("Systems", None, "Topics", None, None, None, "admin").unwrap();

        // Assign terms
        let result = assign_terms(
            EntityType::Content,
            "post-123",
            vec!["Programming:Rust".to_string(), "Topics:Systems".to_string()],
            "admin",
        );

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.data.entity_type, EntityType::Content);
        assert_eq!(response.data.entity_id, "post-123");
        assert_eq!(response.data.term_ids.len(), 2);
    }

    #[test]
    fn test_assign_terms_nonexistent_term() {
        setup_test_env();

        let result = assign_terms(
            EntityType::Content,
            "post-123",
            vec!["NonExistent:Term".to_string()],
            "admin",
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_assign_terms_empty_entity_id() {
        setup_test_env();

        create_term("Rust", None, "Programming", None, None, None, "admin").unwrap();

        let result = assign_terms(
            EntityType::Content,
            "",
            vec!["Programming:Rust".to_string()],
            "admin",
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_assign_terms_update_existing() {
        setup_test_env();

        // Create terms
        create_term("Rust", None, "Programming", None, None, None, "admin").unwrap();
        create_term("Python", None, "Programming", None, None, None, "admin").unwrap();

        // Initial assignment
        assign_terms(
            EntityType::Content,
            "post-123",
            vec!["Programming:Rust".to_string()],
            "admin",
        )
        .unwrap();

        // Update assignment
        let result = assign_terms(
            EntityType::Content,
            "post-123",
            vec!["Programming:Python".to_string()],
            "admin",
        );

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.data.term_ids.len(), 1);
        assert_eq!(response.data.term_ids[0], "Programming:Python");
    }

    #[test]
    fn test_get_entity_terms() {
        setup_test_env();

        // Create and assign terms
        create_term("Rust", None, "Programming", None, None, None, "admin").unwrap();
        assign_terms(
            EntityType::Content,
            "post-123",
            vec!["Programming:Rust".to_string()],
            "admin",
        )
        .unwrap();

        // Get entity terms
        let result = get_entity_terms(EntityType::Content, "post-123");
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.data.entity_id, "post-123");
        assert_eq!(response.data.term_ids.len(), 1);
    }

    #[test]
    fn test_get_entity_terms_not_found() {
        setup_test_env();

        let result = get_entity_terms(EntityType::Content, "nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_list_entities_by_term() {
        setup_test_env();

        // Create term
        create_term("Rust", None, "Programming", None, None, None, "admin").unwrap();

        // Assign to multiple entities
        assign_terms(
            EntityType::Content,
            "post-1",
            vec!["Programming:Rust".to_string()],
            "admin",
        )
        .unwrap();

        assign_terms(
            EntityType::Content,
            "post-2",
            vec!["Programming:Rust".to_string()],
            "admin",
        )
        .unwrap();

        assign_terms(
            EntityType::Template,
            "tmpl-1",
            vec!["Programming:Rust".to_string()],
            "admin",
        )
        .unwrap();

        // List all entities with term
        let result = list_entities_by_term("Programming:Rust", None);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().data.len(), 3);

        // Filter by entity type
        let result = list_entities_by_term("Programming:Rust", Some(EntityType::Content));
        assert!(result.is_ok());
        assert_eq!(result.unwrap().data.len(), 2);
    }

    #[test]
    fn test_unassign_terms_specific() {
        setup_test_env();

        // Create and assign multiple terms
        create_term("Rust", None, "Programming", None, None, None, "admin").unwrap();
        create_term("Systems", None, "Topics", None, None, None, "admin").unwrap();

        assign_terms(
            EntityType::Content,
            "post-123",
            vec!["Programming:Rust".to_string(), "Topics:Systems".to_string()],
            "admin",
        )
        .unwrap();

        // Unassign specific term
        let result = unassign_terms(
            EntityType::Content,
            "post-123",
            Some(vec!["Programming:Rust".to_string()]),
        );

        assert!(result.is_ok());

        // Verify only one term remains
        let entity_terms = get_entity_terms(EntityType::Content, "post-123").unwrap();
        assert_eq!(entity_terms.data.term_ids.len(), 1);
        assert_eq!(entity_terms.data.term_ids[0], "Topics:Systems");
    }

    #[test]
    fn test_unassign_terms_all() {
        setup_test_env();

        // Create and assign terms
        create_term("Rust", None, "Programming", None, None, None, "admin").unwrap();
        assign_terms(
            EntityType::Content,
            "post-123",
            vec!["Programming:Rust".to_string()],
            "admin",
        )
        .unwrap();

        // Unassign all terms
        let result = unassign_terms(EntityType::Content, "post-123", None);
        assert!(result.is_ok());

        // Verify entity no longer exists in assignments
        let get_result = get_entity_terms(EntityType::Content, "post-123");
        assert!(get_result.is_err());
    }

    #[test]
    fn test_unassign_terms_last_term() {
        setup_test_env();

        // Create and assign single term
        create_term("Rust", None, "Programming", None, None, None, "admin").unwrap();
        assign_terms(
            EntityType::Content,
            "post-123",
            vec!["Programming:Rust".to_string()],
            "admin",
        )
        .unwrap();

        // Unassign the only term
        let result = unassign_terms(
            EntityType::Content,
            "post-123",
            Some(vec!["Programming:Rust".to_string()]),
        );

        assert!(result.is_ok());

        // Entity should be removed entirely
        let get_result = get_entity_terms(EntityType::Content, "post-123");
        assert!(get_result.is_err());
    }

    #[test]
    fn test_term_usage_count() {
        setup_test_env();

        // Create term
        create_term("Rust", None, "Programming", None, None, None, "admin").unwrap();

        // Initial usage count should be 0
        let term = get_term("Programming:Rust").unwrap();
        assert_eq!(term.data.usage_count, 0);

        // Assign to entity
        assign_terms(
            EntityType::Content,
            "post-1",
            vec!["Programming:Rust".to_string()],
            "admin",
        )
        .unwrap();

        // Usage count should increment
        let term = get_term("Programming:Rust").unwrap();
        assert_eq!(term.data.usage_count, 1);

        // Assign to another entity
        assign_terms(
            EntityType::Content,
            "post-2",
            vec!["Programming:Rust".to_string()],
            "admin",
        )
        .unwrap();

        let term = get_term("Programming:Rust").unwrap();
        assert_eq!(term.data.usage_count, 2);

        // Unassign from one entity
        unassign_terms(EntityType::Content, "post-1", None).unwrap();

        let term = get_term("Programming:Rust").unwrap();
        assert_eq!(term.data.usage_count, 1);
    }

    #[test]
    fn test_entity_type_conversion() {
        assert_eq!(EntityType::User.as_str(), "user");
        assert_eq!(EntityType::Content.as_str(), "content");
        assert_eq!(EntityType::Template.as_str(), "template");
        assert_eq!(EntityType::Route.as_str(), "route");
        assert_eq!(EntityType::Site.as_str(), "site");
        assert_eq!(EntityType::Project.as_str(), "project");
        assert_eq!(EntityType::Asset.as_str(), "asset");
        assert_eq!(EntityType::Role.as_str(), "role");

        assert_eq!(EntityType::from_str("user").unwrap(), EntityType::User);
        assert_eq!(
            EntityType::from_str("content").unwrap(),
            EntityType::Content
        );
        assert!(EntityType::from_str("invalid").is_err());
    }

    #[test]
    fn test_all_entity_types() {
        setup_test_env();

        create_term("Test", None, "Category", None, None, None, "admin").unwrap();

        let entity_types = vec![
            EntityType::User,
            EntityType::Content,
            EntityType::Template,
            EntityType::Route,
            EntityType::Site,
            EntityType::Project,
            EntityType::Asset,
            EntityType::Role,
        ];

        for (i, entity_type) in entity_types.iter().enumerate() {
            let result = assign_terms(
                *entity_type,
                &format!("entity-{}", i),
                vec!["Category:Test".to_string()],
                "admin",
            );
            assert!(result.is_ok());
        }

        let result = list_entities_by_term("Category:Test", None);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().data.len(), 8);
    }

    #[test]
    fn test_performance_assign_100_entities() {
        setup_test_env();

        create_term("Performance", None, "Testing", None, None, None, "admin").unwrap();

        let start = std::time::Instant::now();

        for i in 0..100 {
            assign_terms(
                EntityType::Content,
                &format!("post-{}", i),
                vec!["Testing:Performance".to_string()],
                "admin",
            )
            .unwrap();
        }

        let duration = start.elapsed();
        println!("Assigning 100 entities took: {}ms", duration.as_millis());
        assert!(duration.as_secs() < 5);
    }

    #[test]
    fn test_performance_list_by_term_100_entities() {
        setup_test_env();

        create_term("Performance", None, "Testing", None, None, None, "admin").unwrap();

        for i in 0..100 {
            assign_terms(
                EntityType::Content,
                &format!("post-{}", i),
                vec!["Testing:Performance".to_string()],
                "admin",
            )
            .unwrap();
        }

        let start = std::time::Instant::now();
        let result = list_entities_by_term("Testing:Performance", None);
        let duration = start.elapsed();

        println!("Listing 100 entities took: {}μs", duration.as_micros());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().data.len(), 100);
        assert!(duration.as_millis() < 20);
    }
}
