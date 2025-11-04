// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

#[cfg(test)]
mod tests {
    use crate::reedql::analyzer::QueryPattern;
    use crate::reedql::planner::{ExecutionPlan, QueryPlanner};

    fn create_planner_with_key_index() -> QueryPlanner {
        QueryPlanner::new(vec![("hierarchy_index".to_string(), "key".to_string())])
    }

    fn create_planner_no_indices() -> QueryPlanner {
        QueryPlanner::new(vec![])
    }

    #[test]
    fn test_plan_full_scan_no_pattern() {
        let planner = create_planner_with_key_index();
        let pattern = QueryPattern::FullScan;
        let plan = planner.plan(&pattern, 1_000_000).unwrap();
        assert_eq!(plan, ExecutionPlan::FullScan);
    }

    #[test]
    fn test_plan_point_lookup_with_index() {
        let planner = create_planner_with_key_index();
        let pattern = QueryPattern::PointLookup {
            column: "key".to_string(),
            value: "page.header.title".to_string(),
        };
        let plan = planner.plan(&pattern, 1_000_000).unwrap();
        assert_eq!(
            plan,
            ExecutionPlan::IndexPointLookup {
                index_name: "hierarchy_index".to_string(),
                key: "page.header.title".to_string(),
            }
        );
    }

    #[test]
    fn test_plan_point_lookup_without_index() {
        let planner = create_planner_no_indices();
        let pattern = QueryPattern::PointLookup {
            column: "key".to_string(),
            value: "page.header.title".to_string(),
        };
        let plan = planner.plan(&pattern, 1_000_000).unwrap();
        assert_eq!(plan, ExecutionPlan::FullScan);
    }

    #[test]
    fn test_plan_prefix_scan_high_selectivity() {
        let planner = create_planner_with_key_index();
        let pattern = QueryPattern::PrefixScan {
            column: "key".to_string(),
            prefix: "page.header.logo.".to_string(), // Very specific (0.01% estimated)
        };
        let plan = planner.plan(&pattern, 1_000_000).unwrap();
        assert_eq!(
            plan,
            ExecutionPlan::IndexRangeScan {
                index_name: "hierarchy_index".to_string(),
                start: "page.header.logo.".to_string(),
                end: "page.header.logo.~".to_string(),
            }
        );
    }

    #[test]
    fn test_plan_prefix_scan_low_selectivity() {
        let planner = create_planner_with_key_index();
        let pattern = QueryPattern::PrefixScan {
            column: "key".to_string(),
            prefix: "p".to_string(), // Very broad (10% estimated)
        };
        let plan = planner.plan(&pattern, 1_000).unwrap();
        // For small tables, full scan is faster
        assert_eq!(plan, ExecutionPlan::FullScan);
    }

    #[test]
    fn test_plan_prefix_scan_medium_table_uses_index() {
        let planner = create_planner_with_key_index();
        let pattern = QueryPattern::PrefixScan {
            column: "key".to_string(),
            prefix: "page.header.".to_string(), // 1% estimated
        };
        let plan = planner.plan(&pattern, 100_000).unwrap();
        assert_eq!(
            plan,
            ExecutionPlan::IndexRangeScan {
                index_name: "hierarchy_index".to_string(),
                start: "page.header.".to_string(),
                end: "page.header.~".to_string(),
            }
        );
    }

    #[test]
    fn test_plan_range_scan_with_index() {
        let planner = create_planner_with_key_index();
        let pattern = QueryPattern::RangeScan {
            column: "key".to_string(),
            start: "page.a".to_string(),
            end: "page.z".to_string(),
            inclusive_start: true,
            inclusive_end: false,
        };
        let plan = planner.plan(&pattern, 1_000_000).unwrap();
        assert_eq!(
            plan,
            ExecutionPlan::IndexRangeScan {
                index_name: "hierarchy_index".to_string(),
                start: "page.a".to_string(),
                end: "page.z".to_string(),
            }
        );
    }

    #[test]
    fn test_plan_range_scan_without_index() {
        let planner = create_planner_no_indices();
        let pattern = QueryPattern::RangeScan {
            column: "key".to_string(),
            start: "page.a".to_string(),
            end: "page.z".to_string(),
            inclusive_start: true,
            inclusive_end: false,
        };
        let plan = planner.plan(&pattern, 1_000_000).unwrap();
        assert_eq!(plan, ExecutionPlan::FullScan);
    }

    #[test]
    fn test_plan_cost_model_small_table_still_uses_index() {
        let planner = create_planner_with_key_index();
        let pattern = QueryPattern::PrefixScan {
            column: "key".to_string(),
            prefix: "page.".to_string(), // 10% estimated (2-part prefix)
        };
        // Even for 100 rows, index is 10x faster: (log2(100)+10)*10 = 166 < 1000
        let plan = planner.plan(&pattern, 100).unwrap();
        // Index is still cost-effective
        match plan {
            ExecutionPlan::IndexRangeScan { .. } => {
                // Expected: index is used
            }
            _ => panic!("Expected index scan even for small table"),
        }
    }

    #[test]
    fn test_plan_cost_model_large_table_prefers_index() {
        let planner = create_planner_with_key_index();
        let pattern = QueryPattern::PrefixScan {
            column: "key".to_string(),
            prefix: "page.header.".to_string(), // 1% estimated
        };
        // For 1M rows, index is much faster
        let plan = planner.plan(&pattern, 1_000_000).unwrap();
        match plan {
            ExecutionPlan::IndexRangeScan { .. } => {
                // Expected
            }
            _ => panic!("Expected index scan for large table"),
        }
    }

    #[test]
    fn test_plan_wrong_column_no_index() {
        let planner = create_planner_with_key_index();
        let pattern = QueryPattern::PointLookup {
            column: "value".to_string(), // Index is on 'key', not 'value'
            value: "test".to_string(),
        };
        let plan = planner.plan(&pattern, 1_000_000).unwrap();
        assert_eq!(plan, ExecutionPlan::FullScan);
    }

    #[test]
    fn test_plan_prefix_tilde_suffix() {
        let planner = create_planner_with_key_index();
        let pattern = QueryPattern::PrefixScan {
            column: "key".to_string(),
            prefix: "page.header.".to_string(),
        };
        let plan = planner.plan(&pattern, 1_000_000).unwrap();
        match plan {
            ExecutionPlan::IndexRangeScan { start, end, .. } => {
                assert_eq!(start, "page.header.");
                assert_eq!(end, "page.header.~"); // Tilde ensures all prefixed keys included
            }
            _ => panic!("Expected index range scan"),
        }
    }
}
