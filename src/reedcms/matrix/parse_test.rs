// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

#[cfg(test)]
mod tests {
    use crate::reedcms::matrix::parse::{parse_matrix_value, parse_modifiers};
    use crate::reedcms::matrix::record::MatrixValue;

    // parse_modifiers tests
    #[test]
    fn test_parse_modifiers_single() {
        let (val, mods) = parse_modifiers("text[rwx]");
        assert_eq!(val, "text");
        assert_eq!(mods, vec!["rwx"]);
    }

    #[test]
    fn test_parse_modifiers_multiple() {
        let (val, mods) = parse_modifiers("file[dev,prod,test]");
        assert_eq!(val, "file");
        assert_eq!(mods, vec!["dev", "prod", "test"]);
    }

    #[test]
    fn test_parse_modifiers_no_brackets() {
        let (val, mods) = parse_modifiers("simple");
        assert_eq!(val, "simple");
        assert!(mods.is_empty());
    }

    #[test]
    fn test_parse_modifiers_empty_brackets() {
        let (val, mods) = parse_modifiers("value[]");
        assert_eq!(val, "value");
        assert!(mods.is_empty());
    }

    #[test]
    fn test_parse_modifiers_whitespace() {
        let (val, mods) = parse_modifiers(" text [ rwx , rw- ] ");
        assert_eq!(val, "text");
        assert_eq!(mods, vec!["rwx", "rw-"]);
    }

    // parse_matrix_value Type 1 tests
    #[test]
    fn test_parse_matrix_value_type1_simple() {
        let value = parse_matrix_value("active");
        assert!(value.is_single());
        match value {
            MatrixValue::Single(s) => assert_eq!(s, "active"),
            _ => panic!("Expected Single variant"),
        }
    }

    #[test]
    fn test_parse_matrix_value_type1_with_whitespace() {
        let value = parse_matrix_value("  active  ");
        assert!(value.is_single());
        match value {
            MatrixValue::Single(s) => assert_eq!(s, "active"),
            _ => panic!("Expected Single variant"),
        }
    }

    // parse_matrix_value Type 2 tests
    #[test]
    fn test_parse_matrix_value_type2_two_items() {
        let value = parse_matrix_value("editor,author");
        assert!(value.is_list());
        match value {
            MatrixValue::List(items) => {
                assert_eq!(items, vec!["editor", "author"]);
            }
            _ => panic!("Expected List variant"),
        }
    }

    #[test]
    fn test_parse_matrix_value_type2_multiple_items() {
        let value = parse_matrix_value("admin,editor,author,viewer");
        assert!(value.is_list());
        match value {
            MatrixValue::List(items) => {
                assert_eq!(items.len(), 4);
            }
            _ => panic!("Expected List variant"),
        }
    }

    #[test]
    fn test_parse_matrix_value_type2_with_whitespace() {
        let value = parse_matrix_value(" editor , author , admin ");
        assert!(value.is_list());
        match value {
            MatrixValue::List(items) => {
                assert_eq!(items, vec!["editor", "author", "admin"]);
            }
            _ => panic!("Expected List variant"),
        }
    }

    // parse_matrix_value Type 3 tests
    #[test]
    fn test_parse_matrix_value_type3_single_modifier() {
        let value = parse_matrix_value("minify[prod]");
        assert!(value.is_modified());
        match value {
            MatrixValue::Modified(val, mods) => {
                assert_eq!(val, "minify");
                assert_eq!(mods, vec!["prod"]);
            }
            _ => panic!("Expected Modified variant"),
        }
    }

    #[test]
    fn test_parse_matrix_value_type3_multiple_modifiers() {
        let value = parse_matrix_value("bundle[dev,test,prod]");
        assert!(value.is_modified());
        match value {
            MatrixValue::Modified(val, mods) => {
                assert_eq!(val, "bundle");
                assert_eq!(mods, vec!["dev", "test", "prod"]);
            }
            _ => panic!("Expected Modified variant"),
        }
    }

    #[test]
    fn test_parse_matrix_value_type3_unix_permissions() {
        let value = parse_matrix_value("text[rwx]");
        assert!(value.is_modified());
        match value {
            MatrixValue::Modified(val, mods) => {
                assert_eq!(val, "text");
                assert_eq!(mods, vec!["rwx"]);
            }
            _ => panic!("Expected Modified variant"),
        }
    }

    // parse_matrix_value Type 4 tests
    #[test]
    fn test_parse_matrix_value_type4_two_items() {
        let value = parse_matrix_value("text[rwx],route[rw-]");
        assert!(value.is_modified_list());
        match value {
            MatrixValue::ModifiedList(items) => {
                assert_eq!(items.len(), 2);
                assert_eq!(items[0], ("text".to_string(), vec!["rwx".to_string()]));
                assert_eq!(items[1], ("route".to_string(), vec!["rw-".to_string()]));
            }
            _ => panic!("Expected ModifiedList variant"),
        }
    }

    #[test]
    fn test_parse_matrix_value_type4_multiple_items() {
        let value = parse_matrix_value("text[rwx],route[rw-],content[r--],system[---]");
        assert!(value.is_modified_list());
        match value {
            MatrixValue::ModifiedList(items) => {
                assert_eq!(items.len(), 4);
                assert_eq!(items[0].0, "text");
                assert_eq!(items[1].0, "route");
                assert_eq!(items[2].0, "content");
                assert_eq!(items[3].0, "system");
            }
            _ => panic!("Expected ModifiedList variant"),
        }
    }

    #[test]
    fn test_parse_matrix_value_type4_wildcard() {
        let value = parse_matrix_value("*[rwx]");
        assert!(value.is_modified()); // Single item with modifier, not a list
    }

    #[test]
    fn test_parse_matrix_value_type4_multiple_modifiers_per_item() {
        let value = parse_matrix_value("file[dev,prod],asset[test,stage]");
        assert!(value.is_modified_list());
        match value {
            MatrixValue::ModifiedList(items) => {
                assert_eq!(items.len(), 2);
                assert_eq!(items[0].1, vec!["dev", "prod"]);
                assert_eq!(items[1].1, vec!["test", "stage"]);
            }
            _ => panic!("Expected ModifiedList variant"),
        }
    }

    // Performance test
    #[test]
    fn test_parse_matrix_value_performance() {
        let start = std::time::Instant::now();
        for _ in 0..10000 {
            parse_matrix_value("text[rwx],route[rw-],content[r--]");
        }
        let duration = start.elapsed();

        // Should complete in < 50ms (< 5μs per parse)
        assert!(
            duration.as_millis() < 50,
            "Parsing too slow: {:?}",
            duration
        );
    }
}
