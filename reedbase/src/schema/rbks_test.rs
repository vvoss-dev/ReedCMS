// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Tests for RBKS v2 key validation.

#[cfg(test)]
mod tests {
    use crate::schema::rbks::{normalize_key, parse_key, validate_key, Modifiers, ParsedKey};

    // ============================================================================
    // Validation Tests
    // ============================================================================

    #[test]
    fn test_validate_key_valid_basic() {
        assert!(validate_key("page.title").is_ok());
        assert!(validate_key("page.header.title").is_ok());
        assert!(validate_key("blog.post.headline").is_ok());
    }

    #[test]
    fn test_validate_key_valid_with_modifiers() {
        assert!(validate_key("page.title<de>").is_ok());
        assert!(validate_key("page.title<prod>").is_ok());
        assert!(validate_key("page.title<de,prod>").is_ok());
        assert!(validate_key("page.title<de,prod,christmas>").is_ok());
        assert!(validate_key("page.title<prod,de>").is_ok()); // Order independent
    }

    #[test]
    fn test_validate_key_invalid_uppercase() {
        let result = validate_key("Page.Title");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("must be lowercase"));
    }

    #[test]
    fn test_validate_key_invalid_hyphens() {
        let result = validate_key("page-title");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Use dots (.) for hierarchy"));
    }

    #[test]
    fn test_validate_key_invalid_empty_segments() {
        let result = validate_key("page..title");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Empty segments not allowed"));
    }

    #[test]
    fn test_validate_key_invalid_leading_dot() {
        let result = validate_key(".page.title");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Leading/trailing dots"));
    }

    #[test]
    fn test_validate_key_invalid_trailing_dot() {
        let result = validate_key("page.title.");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Leading/trailing dots"));
    }

    #[test]
    fn test_validate_key_invalid_empty_modifiers() {
        let result = validate_key("page.title<>");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Empty modifiers"));
    }

    #[test]
    fn test_validate_key_invalid_spaces() {
        let result = validate_key("page title");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Spaces not allowed"));
    }

    #[test]
    fn test_validate_key_invalid_too_shallow() {
        let result = validate_key("page");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("at least 2 segments"));
    }

    #[test]
    fn test_validate_key_invalid_too_deep() {
        let result = validate_key("a.b.c.d.e.f.g.h.i");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("depth exceeds maximum"));
    }

    #[test]
    fn test_validate_key_depth_boundary() {
        // 2 segments (minimum)
        assert!(validate_key("page.title").is_ok());

        // 8 segments (maximum)
        assert!(validate_key("a.b.c.d.e.f.g.h").is_ok());

        // 9 segments (too deep)
        assert!(validate_key("a.b.c.d.e.f.g.h.i").is_err());
    }

    #[test]
    fn test_validate_key_multiple_languages() {
        let result = validate_key("page.title<de,en>");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Multiple languages"));
    }

    #[test]
    fn test_validate_key_multiple_environments() {
        let result = validate_key("page.title<dev,prod>");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Multiple environments"));
    }

    #[test]
    fn test_validate_key_multiple_seasons() {
        let result = validate_key("page.title<christmas,easter>");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Multiple seasons"));
    }

    #[test]
    fn test_validate_key_multiple_variants() {
        let result = validate_key("page.title<mobile,desktop>");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Multiple variants"));
    }

    // ============================================================================
    // Parsing Tests
    // ============================================================================

    #[test]
    fn test_parse_key_no_modifiers() {
        let parsed = parse_key("page.title").unwrap();
        assert_eq!(parsed.base, "page.title");
        assert!(parsed.modifiers.is_empty());
        assert_eq!(parsed.raw, "page.title");
    }

    #[test]
    fn test_parse_key_with_language() {
        let parsed = parse_key("page.title<de>").unwrap();
        assert_eq!(parsed.base, "page.title");
        assert_eq!(parsed.modifiers.language, Some("de".to_string()));
        assert!(parsed.modifiers.environment.is_none());
    }

    #[test]
    fn test_parse_key_with_environment() {
        let parsed = parse_key("page.title<prod>").unwrap();
        assert_eq!(parsed.base, "page.title");
        assert_eq!(parsed.modifiers.environment, Some("prod".to_string()));
        assert!(parsed.modifiers.language.is_none());
    }

    #[test]
    fn test_parse_key_with_multiple_modifiers() {
        let parsed = parse_key("page.title<de,prod,christmas>").unwrap();
        assert_eq!(parsed.base, "page.title");
        assert_eq!(parsed.modifiers.language, Some("de".to_string()));
        assert_eq!(parsed.modifiers.environment, Some("prod".to_string()));
        assert_eq!(parsed.modifiers.season, Some("christmas".to_string()));
    }

    #[test]
    fn test_parse_key_with_variant() {
        let parsed = parse_key("page.header<mobile>").unwrap();
        assert_eq!(parsed.base, "page.header");
        assert_eq!(parsed.modifiers.variant, Some("mobile".to_string()));
    }

    #[test]
    fn test_parse_key_with_custom_modifiers() {
        let parsed = parse_key("component.widget<custom1,custom2>").unwrap();
        assert_eq!(parsed.base, "component.widget");
        assert_eq!(parsed.modifiers.custom, vec!["custom1", "custom2"]);
    }

    #[test]
    fn test_parse_key_mixed_modifiers() {
        let parsed = parse_key("page.title<de,prod,custom1>").unwrap();
        assert_eq!(parsed.modifiers.language, Some("de".to_string()));
        assert_eq!(parsed.modifiers.environment, Some("prod".to_string()));
        assert_eq!(parsed.modifiers.custom, vec!["custom1"]);
    }

    #[test]
    fn test_parsed_key_depth() {
        let parsed = parse_key("page.header.title").unwrap();
        assert_eq!(parsed.depth(), 3);

        let parsed = parse_key("blog.post.headline.subtitle").unwrap();
        assert_eq!(parsed.depth(), 4);
    }

    #[test]
    fn test_parsed_key_namespace() {
        let parsed = parse_key("page.header.title").unwrap();
        assert_eq!(parsed.namespace(), "page");

        let parsed = parse_key("blog.post.headline").unwrap();
        assert_eq!(parsed.namespace(), "blog");
    }

    #[test]
    fn test_parsed_key_to_canonical() {
        let parsed = parse_key("page.title<prod,de>").unwrap();
        assert_eq!(parsed.to_canonical(), "page.title<de,prod>");

        let parsed = parse_key("page.title").unwrap();
        assert_eq!(parsed.to_canonical(), "page.title");
    }

    // ============================================================================
    // Modifier Tests
    // ============================================================================

    #[test]
    fn test_modifiers_is_empty() {
        let mods = Modifiers::default();
        assert!(mods.is_empty());

        let mods = Modifiers {
            language: Some("de".to_string()),
            ..Default::default()
        };
        assert!(!mods.is_empty());
    }

    #[test]
    fn test_modifiers_to_string() {
        let mods = Modifiers {
            language: Some("de".to_string()),
            environment: Some("prod".to_string()),
            season: Some("christmas".to_string()),
            ..Default::default()
        };
        assert_eq!(mods.to_string(), "christmas,de,prod");
    }

    #[test]
    fn test_modifiers_to_string_with_custom() {
        let mods = Modifiers {
            language: Some("de".to_string()),
            custom: vec!["custom1".to_string(), "custom2".to_string()],
            ..Default::default()
        };
        assert_eq!(mods.to_string(), "custom1,custom2,de");
    }

    #[test]
    fn test_modifiers_fallback_chain() {
        let mods = Modifiers {
            language: Some("de".to_string()),
            environment: Some("prod".to_string()),
            season: Some("christmas".to_string()),
            ..Default::default()
        };

        let chain = mods.fallback_chain();

        // Should contain all 8 combinations
        assert!(chain.contains(&"christmas,de,prod".to_string())); // Full
        assert!(chain.contains(&"de,prod".to_string())); // Without season
        assert!(chain.contains(&"christmas,de".to_string())); // Without env
        assert!(chain.contains(&"de".to_string())); // Language only
        assert!(chain.contains(&"christmas,prod".to_string())); // Env + season
        assert!(chain.contains(&"prod".to_string())); // Env only
        assert!(chain.contains(&"christmas".to_string())); // Season only
        assert!(chain.contains(&"".to_string())); // Base (empty)
    }

    #[test]
    fn test_modifiers_fallback_chain_language_only() {
        let mods = Modifiers {
            language: Some("de".to_string()),
            ..Default::default()
        };

        let chain = mods.fallback_chain();

        assert_eq!(chain.len(), 2);
        assert_eq!(chain[0], "de");
        assert_eq!(chain[1], "");
    }

    #[test]
    fn test_modifiers_fallback_chain_empty() {
        let mods = Modifiers::default();
        let chain = mods.fallback_chain();

        assert_eq!(chain.len(), 1);
        assert_eq!(chain[0], "");
    }

    #[test]
    fn test_modifiers_fallback_chain_deduplication() {
        let mods = Modifiers {
            environment: Some("prod".to_string()),
            ..Default::default()
        };

        let chain = mods.fallback_chain();

        // Should not contain duplicates
        let unique_count = chain.iter().collect::<std::collections::HashSet<_>>().len();
        assert_eq!(unique_count, chain.len());
    }

    // ============================================================================
    // Normalization Tests
    // ============================================================================

    #[test]
    fn test_normalize_key_lowercase() {
        let normalized = normalize_key("Page.Title").unwrap();
        assert_eq!(normalized, "page.title");
    }

    #[test]
    fn test_normalize_key_remove_duplicate_dots() {
        let normalized = normalize_key("page..title").unwrap();
        assert_eq!(normalized, "page.title");

        let normalized = normalize_key("page...title").unwrap();
        assert_eq!(normalized, "page.title");
    }

    #[test]
    fn test_normalize_key_trim_dots() {
        let normalized = normalize_key(".page.title").unwrap();
        assert_eq!(normalized, "page.title");

        let normalized = normalize_key("page.title.").unwrap();
        assert_eq!(normalized, "page.title");
    }

    #[test]
    fn test_normalize_key_sort_modifiers() {
        let normalized = normalize_key("page.title<prod,de>").unwrap();
        assert_eq!(normalized, "page.title<de,prod>");

        let normalized = normalize_key("page.title<christmas,de,prod>").unwrap();
        assert_eq!(normalized, "page.title<christmas,de,prod>");
    }

    #[test]
    fn test_normalize_key_deduplicate_modifiers() {
        let normalized = normalize_key("page.title<prod,de,prod>").unwrap();
        assert_eq!(normalized, "page.title<de,prod>");
    }

    #[test]
    fn test_normalize_key_remove_empty_modifiers() {
        let normalized = normalize_key("page.title<de,,prod>").unwrap();
        assert_eq!(normalized, "page.title<de,prod>");
    }

    #[test]
    fn test_normalize_key_complex() {
        let normalized = normalize_key("Page.Header..Title<PROD,DE,prod>").unwrap();
        assert_eq!(normalized, "page.header.title<de,prod>");
    }

    #[test]
    fn test_normalize_key_trim_whitespace() {
        let normalized = normalize_key("  page.title  ").unwrap();
        assert_eq!(normalized, "page.title");
    }

    #[test]
    fn test_normalize_key_validation_error() {
        // After normalization, key must still be valid
        let result = normalize_key("page"); // Too shallow
        assert!(result.is_err());
    }

    // ============================================================================
    // Edge Cases
    // ============================================================================

    #[test]
    fn test_all_known_languages() {
        for lang in &["de", "en", "fr", "es", "it", "pt", "nl", "pl", "ru"] {
            let key = format!("page.title<{}>", lang);
            assert!(validate_key(&key).is_ok(), "Failed for language: {}", lang);

            let parsed = parse_key(&key).unwrap();
            assert_eq!(parsed.modifiers.language, Some(lang.to_string()));
        }
    }

    #[test]
    fn test_all_known_environments() {
        for env in &["dev", "prod", "staging", "test"] {
            let key = format!("page.title<{}>", env);
            assert!(
                validate_key(&key).is_ok(),
                "Failed for environment: {}",
                env
            );

            let parsed = parse_key(&key).unwrap();
            assert_eq!(parsed.modifiers.environment, Some(env.to_string()));
        }
    }

    #[test]
    fn test_all_known_seasons() {
        for season in &["christmas", "easter", "summer", "winter"] {
            let key = format!("page.title<{}>", season);
            assert!(validate_key(&key).is_ok(), "Failed for season: {}", season);

            let parsed = parse_key(&key).unwrap();
            assert_eq!(parsed.modifiers.season, Some(season.to_string()));
        }
    }

    #[test]
    fn test_all_known_variants() {
        for variant in &["mobile", "desktop", "tablet"] {
            let key = format!("page.title<{}>", variant);
            assert!(
                validate_key(&key).is_ok(),
                "Failed for variant: {}",
                variant
            );

            let parsed = parse_key(&key).unwrap();
            assert_eq!(parsed.modifiers.variant, Some(variant.to_string()));
        }
    }

    #[test]
    fn test_complex_hierarchy() {
        let key = "blog.post.author.bio.headline.subtitle";
        assert!(validate_key(key).is_ok());

        let parsed = parse_key(key).unwrap();
        assert_eq!(parsed.depth(), 6);
        assert_eq!(parsed.namespace(), "blog");
    }

    #[test]
    fn test_alphanumeric_segments() {
        assert!(validate_key("page2.title3").is_ok());
        assert!(validate_key("blog1.post2.headline3").is_ok());
    }

    #[test]
    fn test_modifier_order_independence() {
        let key1 = "page.title<de,prod>";
        let key2 = "page.title<prod,de>";

        let parsed1 = parse_key(key1).unwrap();
        let parsed2 = parse_key(key2).unwrap();

        assert_eq!(parsed1.to_canonical(), parsed2.to_canonical());
    }

    #[test]
    fn test_custom_modifiers_multiple() {
        let key = "component.widget<custom1,custom2,custom3>";
        assert!(validate_key(key).is_ok());

        let parsed = parse_key(key).unwrap();
        assert_eq!(parsed.modifiers.custom.len(), 3);
        assert!(parsed.modifiers.custom.contains(&"custom1".to_string()));
        assert!(parsed.modifiers.custom.contains(&"custom2".to_string()));
        assert!(parsed.modifiers.custom.contains(&"custom3".to_string()));
    }

    #[test]
    fn test_mixed_known_and_custom_modifiers() {
        let key = "page.title<de,prod,custom1,christmas,custom2>";
        assert!(validate_key(key).is_ok());

        let parsed = parse_key(key).unwrap();
        assert_eq!(parsed.modifiers.language, Some("de".to_string()));
        assert_eq!(parsed.modifiers.environment, Some("prod".to_string()));
        assert_eq!(parsed.modifiers.season, Some("christmas".to_string()));
        assert_eq!(parsed.modifiers.custom.len(), 2);
    }

    // ============================================================================
    // Performance-Related Tests
    // ============================================================================

    #[test]
    fn test_large_key() {
        let key = "namespace.level2.level3.level4.level5.level6.level7.level8";
        assert!(validate_key(key).is_ok());

        let parsed = parse_key(key).unwrap();
        assert_eq!(parsed.depth(), 8);
    }

    #[test]
    fn test_many_custom_modifiers() {
        let modifiers = (1..=10).map(|i| format!("custom{}", i)).collect::<Vec<_>>();
        let key = format!("page.title<{}>", modifiers.join(","));

        assert!(validate_key(&key).is_ok());

        let parsed = parse_key(&key).unwrap();
        assert_eq!(parsed.modifiers.custom.len(), 10);
    }
}
