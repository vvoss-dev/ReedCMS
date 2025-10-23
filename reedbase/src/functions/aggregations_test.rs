// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Tests for aggregation functions.

#[cfg(test)]
mod tests {
    use crate::functions::aggregations::*;
    use crate::functions::cache::get_cache;
    use crate::tables::Table;
    use std::fs;
    use std::path::Path;

    // Helper: Create test CSV file
    fn create_test_table(name: &str, content: &str) {
        let base_path = Path::new(".reed");
        let table = Table::new(base_path, name);

        fs::create_dir_all(table.current_path().parent().unwrap()).ok();
        fs::write(table.current_path(), content).ok();
    }

    // Helper: Clean up test table
    fn cleanup_test_table(name: &str) {
        let base_path = Path::new(".reed");
        let table = Table::new(base_path, name);
        fs::remove_dir_all(table.current_path().parent().unwrap()).ok();
    }

    #[test]
    fn test_count_basic() {
        get_cache().clear();

        let table_name = "test_count_basic";
        create_test_table(
            table_name,
            "key|name|age\nuser1|Alice|30\nuser2|Bob|25\nuser3|Charlie|35\n",
        );

        let result = count(table_name).unwrap();
        assert_eq!(result, "3"); // Excludes header

        cleanup_test_table(table_name);
    }

    #[test]
    fn test_count_empty_table() {
        get_cache().clear();

        let table_name = "test_count_empty";
        create_test_table(table_name, "key|name\n"); // Only header

        let result = count(table_name).unwrap();
        assert_eq!(result, "0");

        cleanup_test_table(table_name);
    }

    #[test]
    fn test_count_cached() {
        get_cache().clear();

        let table_name = "test_count_cached";
        create_test_table(table_name, "key|name\nuser1|Alice\nuser2|Bob\n");

        let count1 = count(table_name).unwrap();
        let count2 = count(table_name).unwrap();

        assert_eq!(count1, count2);

        let stats = get_cache().stats();
        assert!(stats.hits >= 1);

        cleanup_test_table(table_name);
    }

    #[test]
    fn test_sum_basic() {
        get_cache().clear();

        let table_name = "test_sum_basic";
        create_test_table(
            table_name,
            "key|name|amount\norder1|Item1|10\norder2|Item2|20\norder3|Item3|30\n",
        );

        let result = sum(table_name, "amount").unwrap();
        assert_eq!(result, "60.00");

        cleanup_test_table(table_name);
    }

    #[test]
    fn test_sum_floats() {
        get_cache().clear();

        let table_name = "test_sum_floats";
        create_test_table(
            table_name,
            "key|name|price\nprod1|A|10.50\nprod2|B|20.75\nprod3|C|15.25\n",
        );

        let result = sum(table_name, "price").unwrap();
        assert_eq!(result, "46.50");

        cleanup_test_table(table_name);
    }

    #[test]
    fn test_sum_with_non_numeric() {
        get_cache().clear();

        let table_name = "test_sum_mixed";
        create_test_table(
            table_name,
            "key|name|value\nrow1|A|10\nrow2|B|invalid\nrow3|C|20\n",
        );

        // Should skip "invalid" and sum valid numbers
        let result = sum(table_name, "value").unwrap();
        assert_eq!(result, "30.00");

        cleanup_test_table(table_name);
    }

    #[test]
    fn test_sum_nonexistent_column() {
        get_cache().clear();

        let table_name = "test_sum_no_col";
        create_test_table(table_name, "key|name\nrow1|Alice\n");

        let result = sum(table_name, "nonexistent");
        assert!(result.is_err());

        cleanup_test_table(table_name);
    }

    #[test]
    fn test_avg_basic() {
        get_cache().clear();

        let table_name = "test_avg_basic";
        create_test_table(
            table_name,
            "key|name|age\nuser1|Alice|20\nuser2|Bob|30\nuser3|Charlie|40\n",
        );

        let result = avg(table_name, "age").unwrap();
        assert_eq!(result, "30.00");

        cleanup_test_table(table_name);
    }

    #[test]
    fn test_avg_with_decimals() {
        get_cache().clear();

        let table_name = "test_avg_decimals";
        create_test_table(
            table_name,
            "key|name|score\ntest1|A|85.5\ntest2|B|92.3\ntest3|C|78.2\n",
        );

        let result = avg(table_name, "score").unwrap();
        assert_eq!(result, "85.33");

        cleanup_test_table(table_name);
    }

    #[test]
    fn test_min_basic() {
        get_cache().clear();

        let table_name = "test_min_basic";
        create_test_table(
            table_name,
            "key|name|age\nuser1|Alice|30\nuser2|Bob|18\nuser3|Charlie|45\n",
        );

        let result = min(table_name, "age").unwrap();
        assert_eq!(result, "18.00");

        cleanup_test_table(table_name);
    }

    #[test]
    fn test_min_negative_numbers() {
        get_cache().clear();

        let table_name = "test_min_negative";
        create_test_table(
            table_name,
            "key|name|temp\nday1|Mon|5\nday2|Tue|-10\nday3|Wed|15\n",
        );

        let result = min(table_name, "temp").unwrap();
        assert_eq!(result, "-10.00");

        cleanup_test_table(table_name);
    }

    #[test]
    fn test_max_basic() {
        get_cache().clear();

        let table_name = "test_max_basic";
        create_test_table(
            table_name,
            "key|name|age\nuser1|Alice|30\nuser2|Bob|18\nuser3|Charlie|45\n",
        );

        let result = max(table_name, "age").unwrap();
        assert_eq!(result, "45.00");

        cleanup_test_table(table_name);
    }

    #[test]
    fn test_max_floats() {
        get_cache().clear();

        let table_name = "test_max_floats";
        create_test_table(
            table_name,
            "key|name|price\nprod1|A|99.99\nprod2|B|150.50\nprod3|C|75.25\n",
        );

        let result = max(table_name, "price").unwrap();
        assert_eq!(result, "150.50");

        cleanup_test_table(table_name);
    }

    #[test]
    fn test_group_by_basic() {
        get_cache().clear();

        let table_name = "test_group_basic";
        create_test_table(
            table_name,
            "key|name|status\nuser1|Alice|active\nuser2|Bob|active\nuser3|Charlie|inactive\n",
        );

        let result = group_by(table_name, "status").unwrap();

        // Parse JSON result
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        assert_eq!(parsed["active"], 2);
        assert_eq!(parsed["inactive"], 1);

        cleanup_test_table(table_name);
    }

    #[test]
    fn test_group_by_multiple_values() {
        get_cache().clear();

        let table_name = "test_group_multi";
        create_test_table(
            table_name,
            "key|name|lang\ntext1|Hello|en\ntext2|World|en\ntext3|Hallo|de\ntext4|Welt|de\ntext5|Bonjour|fr\n",
        );

        let result = group_by(table_name, "lang").unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        assert_eq!(parsed["en"], 2);
        assert_eq!(parsed["de"], 2);
        assert_eq!(parsed["fr"], 1);

        cleanup_test_table(table_name);
    }

    #[test]
    fn test_aggregation_table_not_found() {
        get_cache().clear();

        let result = count("nonexistent_table_xyz");
        assert!(result.is_err());
    }
}
