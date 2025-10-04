// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Comprehensive tests for change detection and rebuild scope.

#[cfg(test)]
mod tests {
    use super::super::change_detect::{detect_rebuild_scope, extract_layout_variant, RebuildScope};

    /// Test: Core CSS change triggers full rebuild
    #[test]
    fn test_core_css_triggers_all_css() {
        let scope = detect_rebuild_scope("assets/css/core/reset.css");
        assert_eq!(scope, RebuildScope::AllCss);
    }

    /// Test: Component CSS change triggers full rebuild
    #[test]
    fn test_component_css_triggers_all_css() {
        let scope = detect_rebuild_scope("assets/css/components/atoms/button/button.css");
        assert_eq!(scope, RebuildScope::AllCss);
    }

    /// Test: Layout CSS change triggers specific rebuild
    #[test]
    fn test_layout_css_triggers_specific() {
        let scope = detect_rebuild_scope("assets/css/layouts/knowledge/knowledge.mouse.css");

        match scope {
            RebuildScope::SpecificCss { layout, variant } => {
                assert_eq!(layout, "knowledge");
                assert_eq!(variant, "mouse");
            }
            _ => panic!("Expected SpecificCss"),
        }
    }

    /// Test: Core JS change triggers full rebuild
    #[test]
    fn test_core_js_triggers_all_js() {
        let scope = detect_rebuild_scope("assets/js/core/utils.js");
        assert_eq!(scope, RebuildScope::AllJs);
    }

    /// Test: Component JS change triggers full rebuild
    #[test]
    fn test_component_js_triggers_all_js() {
        let scope = detect_rebuild_scope("assets/js/components/molecules/slider/slider.js");
        assert_eq!(scope, RebuildScope::AllJs);
    }

    /// Test: Layout JS change triggers specific rebuild
    #[test]
    fn test_layout_js_triggers_specific() {
        let scope = detect_rebuild_scope("assets/js/layouts/blog/blog.touch.js");

        match scope {
            RebuildScope::SpecificJs { layout, variant } => {
                assert_eq!(layout, "blog");
                assert_eq!(variant, "touch");
            }
            _ => panic!("Expected SpecificJs"),
        }
    }

    /// Test: Template change detected
    #[test]
    fn test_template_change() {
        let scope = detect_rebuild_scope("templates/layouts/knowledge/knowledge.mouse.jinja");

        match scope {
            RebuildScope::Template { path } => {
                assert!(path.starts_with("templates/"));
            }
            _ => panic!("Expected Template"),
        }
    }

    /// Test: Config change detected
    #[test]
    fn test_config_change() {
        let scope = detect_rebuild_scope(".reed/text.csv");

        match scope {
            RebuildScope::Config { path } => {
                assert!(path.starts_with(".reed/"));
            }
            _ => panic!("Expected Config"),
        }
    }

    /// Test: Unknown file triggers None
    #[test]
    fn test_unknown_file() {
        let scope = detect_rebuild_scope("some/random/file.txt");
        assert_eq!(scope, RebuildScope::None);
    }

    /// Test: Extract layout/variant from CSS path
    #[test]
    fn test_extract_layout_variant_css() {
        let result =
            extract_layout_variant("assets/css/layouts/knowledge/knowledge.mouse.css", "css");

        assert_eq!(result, Some(("knowledge".to_string(), "mouse".to_string())));
    }

    /// Test: Extract layout/variant from JS path
    #[test]
    fn test_extract_layout_variant_js() {
        let result = extract_layout_variant("assets/js/layouts/blog/blog.touch.js", "js");

        assert_eq!(result, Some(("blog".to_string(), "touch".to_string())));
    }

    /// Test: Extract with reader variant
    #[test]
    fn test_extract_layout_variant_reader() {
        let result = extract_layout_variant("assets/css/layouts/landing/landing.reader.css", "css");

        assert_eq!(result, Some(("landing".to_string(), "reader".to_string())));
    }

    /// Test: Extract fails on malformed path
    #[test]
    fn test_extract_layout_variant_malformed() {
        let result = extract_layout_variant("assets/css/layouts/invalid", "css");

        assert_eq!(result, None);
    }

    /// Test: Extract fails on core path
    #[test]
    fn test_extract_layout_variant_core() {
        let result = extract_layout_variant("assets/css/core/reset.css", "css");

        assert_eq!(result, None);
    }

    /// Test: RebuildScope Debug implementation
    #[test]
    fn test_rebuild_scope_debug() {
        let scope = RebuildScope::AllCss;
        let debug = format!("{:?}", scope);

        assert!(debug.contains("AllCss"));
    }

    /// Test: RebuildScope Clone implementation
    #[test]
    fn test_rebuild_scope_clone() {
        let scope = RebuildScope::AllCss;
        let cloned = scope.clone();

        assert_eq!(scope, cloned);
    }

    /// Test: RebuildScope PartialEq implementation
    #[test]
    fn test_rebuild_scope_equality() {
        let scope1 = RebuildScope::AllCss;
        let scope2 = RebuildScope::AllCss;
        let scope3 = RebuildScope::AllJs;

        assert_eq!(scope1, scope2);
        assert_ne!(scope1, scope3);
    }

    /// Test: Specific CSS scopes with same layout are equal
    #[test]
    fn test_specific_css_equality() {
        let scope1 = RebuildScope::SpecificCss {
            layout: "knowledge".to_string(),
            variant: "mouse".to_string(),
        };
        let scope2 = RebuildScope::SpecificCss {
            layout: "knowledge".to_string(),
            variant: "mouse".to_string(),
        };

        assert_eq!(scope1, scope2);
    }

    /// Test: Different variants are not equal
    #[test]
    fn test_different_variants_not_equal() {
        let scope1 = RebuildScope::SpecificCss {
            layout: "knowledge".to_string(),
            variant: "mouse".to_string(),
        };
        let scope2 = RebuildScope::SpecificCss {
            layout: "knowledge".to_string(),
            variant: "touch".to_string(),
        };

        assert_ne!(scope1, scope2);
    }
}
