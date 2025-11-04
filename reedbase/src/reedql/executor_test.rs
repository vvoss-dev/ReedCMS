// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

#[cfg(test)]
mod tests {
    use crate::indices::{HashMapIndex, Index};
    use crate::reedql::{parse, OptimizedExecutor, QueryResult};
    use std::collections::HashMap;

    fn create_test_table() -> Vec<HashMap<String, String>> {
        vec![
            HashMap::from([
                ("key".to_string(), "page.header.title@de".to_string()),
                ("value".to_string(), "Willkommen".to_string()),
                ("namespace".to_string(), "page".to_string()),
            ]),
            HashMap::from([
                ("key".to_string(), "page.header.subtitle@de".to_string()),
                ("value".to_string(), "Untertitel".to_string()),
                ("namespace".to_string(), "page".to_string()),
            ]),
            HashMap::from([
                ("key".to_string(), "page.footer.copyright@de".to_string()),
                ("value".to_string(), "© 2025".to_string()),
                ("namespace".to_string(), "page".to_string()),
            ]),
            HashMap::from([
                ("key".to_string(), "global.header.logo@de".to_string()),
                ("value".to_string(), "Logo".to_string()),
                ("namespace".to_string(), "global".to_string()),
            ]),
            HashMap::from([
                ("key".to_string(), "global.footer.imprint@de".to_string()),
                ("value".to_string(), "Impressum".to_string()),
                ("namespace".to_string(), "global".to_string()),
            ]),
        ]
    }

    fn create_hierarchy_index() -> Box<dyn Index<String, Vec<usize>>> {
        let mut index = HashMapIndex::new();

        // Build index: key → [row_id]
        index
            .insert("page.header.title@de".to_string(), vec![0])
            .unwrap();
        index
            .insert("page.header.subtitle@de".to_string(), vec![1])
            .unwrap();
        index
            .insert("page.footer.copyright@de".to_string(), vec![2])
            .unwrap();
        index
            .insert("global.header.logo@de".to_string(), vec![3])
            .unwrap();
        index
            .insert("global.footer.imprint@de".to_string(), vec![4])
            .unwrap();

        Box::new(index)
    }

    #[test]
    fn test_optimized_executor_full_scan_no_index() {
        let table = create_test_table();
        let executor = OptimizedExecutor::new(vec![]);

        let query = parse("SELECT * FROM text WHERE namespace = 'page'").unwrap();
        let result = executor.execute_optimized(&query, &table).unwrap();

        assert_eq!(result.row_count(), 3);
    }

    #[test]
    fn test_optimized_executor_point_lookup() {
        let table = create_test_table();
        let hierarchy_index = create_hierarchy_index();
        let executor =
            OptimizedExecutor::new(vec![("hierarchy_index".to_string(), hierarchy_index)]);

        let query = parse("SELECT * FROM text WHERE key = 'page.header.title@de'").unwrap();
        let result = executor.execute_optimized(&query, &table).unwrap();

        match result {
            QueryResult::Rows(rows) => {
                assert_eq!(rows.len(), 1);
                assert_eq!(rows[0].get("value").unwrap(), "Willkommen");
            }
            _ => panic!("Expected rows result"),
        }
    }

    #[test]
    fn test_optimized_executor_point_lookup_not_found() {
        let table = create_test_table();
        let hierarchy_index = create_hierarchy_index();
        let executor =
            OptimizedExecutor::new(vec![("hierarchy_index".to_string(), hierarchy_index)]);

        let query = parse("SELECT * FROM text WHERE key = 'nonexistent.key'").unwrap();
        let result = executor.execute_optimized(&query, &table).unwrap();

        assert_eq!(result.row_count(), 0);
    }

    #[test]
    fn test_optimized_executor_prefix_scan() {
        let table = create_test_table();
        let hierarchy_index = create_hierarchy_index();
        let executor =
            OptimizedExecutor::new(vec![("hierarchy_index".to_string(), hierarchy_index)]);

        let query = parse("SELECT * FROM text WHERE key LIKE 'page.header.%'").unwrap();
        let result = executor.execute_optimized(&query, &table).unwrap();

        match result {
            QueryResult::Rows(rows) => {
                assert_eq!(rows.len(), 2);
                // Should contain title and subtitle
                let keys: Vec<&String> = rows.iter().map(|r| r.get("key").unwrap()).collect();
                assert!(keys.contains(&&"page.header.title@de".to_string()));
                assert!(keys.contains(&&"page.header.subtitle@de".to_string()));
            }
            _ => panic!("Expected rows result"),
        }
    }

    #[test]
    fn test_optimized_executor_range_scan() {
        let table = create_test_table();
        let hierarchy_index = create_hierarchy_index();
        let executor =
            OptimizedExecutor::new(vec![("hierarchy_index".to_string(), hierarchy_index)]);

        let query = parse("SELECT * FROM text WHERE key >= 'page.' AND key < 'page.~'").unwrap();
        let result = executor.execute_optimized(&query, &table).unwrap();

        match result {
            QueryResult::Rows(rows) => {
                assert_eq!(rows.len(), 3); // All page.* keys
                for row in rows {
                    assert!(row.get("key").unwrap().starts_with("page."));
                }
            }
            _ => panic!("Expected rows result"),
        }
    }

    #[test]
    fn test_optimized_executor_with_additional_filters() {
        let table = create_test_table();
        let hierarchy_index = create_hierarchy_index();
        let executor =
            OptimizedExecutor::new(vec![("hierarchy_index".to_string(), hierarchy_index)]);

        // Point lookup + namespace filter
        let query =
            parse("SELECT * FROM text WHERE key = 'page.header.title@de' AND namespace = 'page'")
                .unwrap();
        let result = executor.execute_optimized(&query, &table).unwrap();

        assert_eq!(result.row_count(), 1);
    }

    #[test]
    fn test_optimized_executor_with_order_by() {
        let table = create_test_table();
        let hierarchy_index = create_hierarchy_index();
        let executor =
            OptimizedExecutor::new(vec![("hierarchy_index".to_string(), hierarchy_index)]);

        let query = parse("SELECT * FROM text WHERE key LIKE 'page.%' ORDER BY key DESC").unwrap();
        let result = executor.execute_optimized(&query, &table).unwrap();

        match result {
            QueryResult::Rows(rows) => {
                assert_eq!(rows.len(), 3);
                // First row should be highest key alphabetically
                assert_eq!(rows[0].get("key").unwrap(), "page.header.title@de");
            }
            _ => panic!("Expected rows result"),
        }
    }

    #[test]
    fn test_optimized_executor_with_limit() {
        let table = create_test_table();
        let hierarchy_index = create_hierarchy_index();
        let executor =
            OptimizedExecutor::new(vec![("hierarchy_index".to_string(), hierarchy_index)]);

        let query = parse("SELECT * FROM text WHERE key LIKE 'page.%' LIMIT 2").unwrap();
        let result = executor.execute_optimized(&query, &table).unwrap();

        assert_eq!(result.row_count(), 2);
    }

    #[test]
    fn test_optimized_executor_with_projection() {
        let table = create_test_table();
        let hierarchy_index = create_hierarchy_index();
        let executor =
            OptimizedExecutor::new(vec![("hierarchy_index".to_string(), hierarchy_index)]);

        let query =
            parse("SELECT key, value FROM text WHERE key = 'page.header.title@de'").unwrap();
        let result = executor.execute_optimized(&query, &table).unwrap();

        match result {
            QueryResult::Rows(rows) => {
                assert_eq!(rows.len(), 1);
                assert!(rows[0].contains_key("key"));
                assert!(rows[0].contains_key("value"));
                assert!(!rows[0].contains_key("namespace")); // Not projected
            }
            _ => panic!("Expected rows result"),
        }
    }

    #[test]
    fn test_optimized_executor_with_aggregation() {
        let table = create_test_table();
        let hierarchy_index = create_hierarchy_index();
        let executor =
            OptimizedExecutor::new(vec![("hierarchy_index".to_string(), hierarchy_index)]);

        let query = parse("SELECT COUNT(*) FROM text WHERE key LIKE 'page.%'").unwrap();
        let result = executor.execute_optimized(&query, &table).unwrap();

        match result {
            QueryResult::Aggregation(value) => {
                assert_eq!(value, 3.0);
            }
            _ => panic!("Expected aggregation result"),
        }
    }

    #[test]
    fn test_optimized_executor_index_not_found_error() {
        let table = create_test_table();
        // No indices provided
        let executor = OptimizedExecutor::new(vec![]);

        // This query would want to use an index, but none available
        // Should fall back to full scan gracefully
        let query = parse("SELECT * FROM text WHERE key = 'page.header.title@de'").unwrap();
        let result = executor.execute_optimized(&query, &table).unwrap();

        // Falls back to full scan
        assert_eq!(result.row_count(), 1);
    }

    #[test]
    fn test_optimized_executor_empty_result() {
        let table = create_test_table();
        let hierarchy_index = create_hierarchy_index();
        let executor =
            OptimizedExecutor::new(vec![("hierarchy_index".to_string(), hierarchy_index)]);

        let query = parse("SELECT * FROM text WHERE key LIKE 'nonexistent.%'").unwrap();
        let result = executor.execute_optimized(&query, &table).unwrap();

        assert_eq!(result.row_count(), 0);
    }
}
