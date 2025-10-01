// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Matrix CSV Value Parsing
//!
//! Provides parsers for all 4 Matrix value types with automatic type detection.

use crate::reedcms::matrix::record::MatrixValue;

/// Parses modifiers from a value string.
///
/// ## Input
/// - `value`: String like "text[rwx]" or "file[dev,prod]"
///
/// ## Output
/// - `(value, modifiers)`: Tuple of base value and modifier list
///
/// ## Examples
/// ```
/// parse_modifiers("text[rwx]") → ("text", vec!["rwx"])
/// parse_modifiers("file[dev,prod]") → ("file", vec!["dev", "prod"])
/// parse_modifiers("simple") → ("simple", vec![])
/// ```
pub fn parse_modifiers(value: &str) -> (String, Vec<String>) {
    if let Some(bracket_start) = value.find('[') {
        if let Some(bracket_end) = value.rfind(']') {
            if bracket_end > bracket_start {
                let base_value = value[..bracket_start].trim();
                let modifiers_str = &value[bracket_start + 1..bracket_end];
                let modifiers = modifiers_str
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();

                return (base_value.to_string(), modifiers);
            }
        }
    }

    (value.trim().to_string(), vec![])
}

/// Parses a Matrix CSV value with automatic type detection.
///
/// ## Input
/// - `value`: String value from CSV field
///
/// ## Output
/// - `MatrixValue`: Appropriate type based on content
///
/// ## Type Detection Algorithm
/// 1. Check for comma AND brackets → Type 4 (ModifiedList)
/// 2. Check for brackets only → Type 3 (Modified)
/// 3. Check for comma only → Type 2 (List)
/// 4. Default → Type 1 (Single)
///
/// ## Performance
/// - O(n) where n is string length
/// - < 50μs typical
///
/// ## Examples
/// ```
/// parse_matrix_value("active") → Single("active")
/// parse_matrix_value("editor,author") → List(["editor", "author"])
/// parse_matrix_value("minify[prod]") → Modified("minify", ["prod"])
/// parse_matrix_value("text[rwx],route[rw-]") → ModifiedList([("text", ["rwx"]), ("route", ["rw-"])])
/// ```
pub fn parse_matrix_value(value: &str) -> MatrixValue {
    let trimmed = value.trim();

    // Type 4: List with modifiers - item1[mod],item2[mod]
    // Need to check if comma is OUTSIDE brackets (Type 4) vs INSIDE brackets (Type 3)
    if trimmed.contains(',') && trimmed.contains('[') {
        let mut bracket_depth: i32 = 0;
        let mut has_comma_outside_brackets = false;

        for ch in trimmed.chars() {
            match ch {
                '[' => bracket_depth += 1,
                ']' => bracket_depth = bracket_depth.saturating_sub(1),
                ',' if bracket_depth == 0 => {
                    has_comma_outside_brackets = true;
                    break;
                }
                _ => {}
            }
        }

        if has_comma_outside_brackets {
            // Smart split: only split at commas outside brackets
            let mut items = Vec::new();
            let mut current = String::new();
            let mut depth: i32 = 0;

            for ch in trimmed.chars() {
                match ch {
                    '[' => {
                        depth += 1;
                        current.push(ch);
                    }
                    ']' => {
                        depth = depth.saturating_sub(1);
                        current.push(ch);
                    }
                    ',' if depth == 0 => {
                        if !current.trim().is_empty() {
                            items.push(parse_modifiers(current.trim()));
                        }
                        current.clear();
                    }
                    _ => current.push(ch),
                }
            }

            // Don't forget the last item
            if !current.trim().is_empty() {
                items.push(parse_modifiers(current.trim()));
            }

            return MatrixValue::ModifiedList(items);
        }
    }

    // Type 3: Single with modifier - value[mod] or value[mod1,mod2]
    if trimmed.contains('[') && trimmed.contains(']') {
        let (val, mods) = parse_modifiers(trimmed);
        return MatrixValue::Modified(val, mods);
    }

    // Type 2: List - item1,item2,item3
    if trimmed.contains(',') {
        let items: Vec<String> = trimmed
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        return MatrixValue::List(items);
    }

    // Type 1: Simple value
    MatrixValue::Single(trimmed.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_modifiers_simple() {
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
    fn test_parse_matrix_value_type1() {
        let value = parse_matrix_value("active");
        assert!(value.is_single());
        assert_eq!(value, MatrixValue::Single("active".to_string()));
    }

    #[test]
    fn test_parse_matrix_value_type2() {
        let value = parse_matrix_value("editor,author,admin");
        assert!(value.is_list());
        if let MatrixValue::List(items) = value {
            assert_eq!(items, vec!["editor", "author", "admin"]);
        }
    }

    #[test]
    fn test_parse_matrix_value_type3() {
        let value = parse_matrix_value("minify[prod]");
        assert!(value.is_modified());
        if let MatrixValue::Modified(val, mods) = value {
            assert_eq!(val, "minify");
            assert_eq!(mods, vec!["prod"]);
        }
    }

    #[test]
    fn test_parse_matrix_value_type4() {
        let value = parse_matrix_value("text[rwx],route[rw-],content[r--]");
        assert!(value.is_modified_list());
        if let MatrixValue::ModifiedList(items) = value {
            assert_eq!(items.len(), 3);
            assert_eq!(items[0], ("text".to_string(), vec!["rwx".to_string()]));
            assert_eq!(items[1], ("route".to_string(), vec!["rw-".to_string()]));
            assert_eq!(items[2], ("content".to_string(), vec!["r--".to_string()]));
        }
    }
}
