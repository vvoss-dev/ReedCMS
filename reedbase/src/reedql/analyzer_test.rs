// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

#[cfg(test)]
mod tests {
    use crate::reedql::analyzer::{QueryAnalyzer, QueryPattern};
    use crate::reedql::parse;

    #[test]
    fn test_analyze_full_scan_no_conditions() {
        let query = parse("SELECT * FROM text").unwrap();
        let pattern = QueryAnalyzer::analyze(&query).unwrap();
        assert_eq!(pattern, QueryPattern::FullScan);
    }

    #[test]
    fn test_analyze_full_scan_non_key_condition() {
        let query = parse("SELECT * FROM text WHERE namespace = 'page'").unwrap();
        let pattern = QueryAnalyzer::analyze(&query).unwrap();
        assert_eq!(pattern, QueryPattern::FullScan);
    }

    #[test]
    fn test_analyze_point_lookup_exact_match() {
        let query = parse("SELECT * FROM text WHERE key = 'page.header.title'").unwrap();
        let pattern = QueryAnalyzer::analyze(&query).unwrap();
        assert_eq!(
            pattern,
            QueryPattern::PointLookup {
                column: "key".to_string(),
                value: "page.header.title".to_string(),
            }
        );
    }

    #[test]
    fn test_analyze_point_lookup_with_language() {
        let query = parse("SELECT * FROM text WHERE key = 'page.header.title@de'").unwrap();
        let pattern = QueryAnalyzer::analyze(&query).unwrap();
        assert_eq!(
            pattern,
            QueryPattern::PointLookup {
                column: "key".to_string(),
                value: "page.header.title@de".to_string(),
            }
        );
    }

    #[test]
    fn test_analyze_prefix_scan_simple() {
        let query = parse("SELECT * FROM text WHERE key LIKE 'page.%'").unwrap();
        let pattern = QueryAnalyzer::analyze(&query).unwrap();
        assert_eq!(
            pattern,
            QueryPattern::PrefixScan {
                column: "key".to_string(),
                prefix: "page.".to_string(),
            }
        );
    }

    #[test]
    fn test_analyze_prefix_scan_nested() {
        let query = parse("SELECT * FROM text WHERE key LIKE 'page.header.logo.%'").unwrap();
        let pattern = QueryAnalyzer::analyze(&query).unwrap();
        assert_eq!(
            pattern,
            QueryPattern::PrefixScan {
                column: "key".to_string(),
                prefix: "page.header.logo.".to_string(),
            }
        );
    }

    #[test]
    fn test_analyze_prefix_scan_no_dot() {
        let query = parse("SELECT * FROM text WHERE key LIKE 'page%'").unwrap();
        let pattern = QueryAnalyzer::analyze(&query).unwrap();
        assert_eq!(
            pattern,
            QueryPattern::PrefixScan {
                column: "key".to_string(),
                prefix: "page".to_string(),
            }
        );
    }

    #[test]
    fn test_analyze_suffix_pattern_not_optimized() {
        let query = parse("SELECT * FROM text WHERE key LIKE '%@de'").unwrap();
        let pattern = QueryAnalyzer::analyze(&query).unwrap();
        assert_eq!(pattern, QueryPattern::FullScan);
    }

    #[test]
    fn test_analyze_range_scan_exclusive_bounds() {
        let query = parse("SELECT * FROM text WHERE key > 'page.a' AND key < 'page.z'").unwrap();
        let pattern = QueryAnalyzer::analyze(&query).unwrap();
        assert_eq!(
            pattern,
            QueryPattern::RangeScan {
                column: "key".to_string(),
                start: "page.a".to_string(),
                end: "page.z".to_string(),
                inclusive_start: false,
                inclusive_end: false,
            }
        );
    }

    #[test]
    fn test_analyze_range_scan_inclusive_bounds() {
        let query = parse("SELECT * FROM text WHERE key >= 'page.a' AND key <= 'page.z'").unwrap();
        let pattern = QueryAnalyzer::analyze(&query).unwrap();
        assert_eq!(
            pattern,
            QueryPattern::RangeScan {
                column: "key".to_string(),
                start: "page.a".to_string(),
                end: "page.z".to_string(),
                inclusive_start: true,
                inclusive_end: true,
            }
        );
    }

    #[test]
    fn test_analyze_range_scan_mixed_bounds() {
        let query = parse("SELECT * FROM text WHERE key >= 'page.a' AND key < 'page.z'").unwrap();
        let pattern = QueryAnalyzer::analyze(&query).unwrap();
        assert_eq!(
            pattern,
            QueryPattern::RangeScan {
                column: "key".to_string(),
                start: "page.a".to_string(),
                end: "page.z".to_string(),
                inclusive_start: true,
                inclusive_end: false,
            }
        );
    }

    #[test]
    fn test_analyze_range_scan_prefix_emulation() {
        // Common pattern for prefix scans using range
        let query = parse("SELECT * FROM text WHERE key >= 'page.' AND key < 'page.~'").unwrap();
        let pattern = QueryAnalyzer::analyze(&query).unwrap();
        assert_eq!(
            pattern,
            QueryPattern::RangeScan {
                column: "key".to_string(),
                start: "page.".to_string(),
                end: "page.~".to_string(),
                inclusive_start: true,
                inclusive_end: false,
            }
        );
    }

    #[test]
    fn test_analyze_multiple_conditions_on_key_no_pattern() {
        // Three conditions on key: > and < create range scan
        // Analyzer detects range from first two conditions (ignores != as it's not part of pattern)
        let query =
            parse("SELECT * FROM text WHERE key > 'a' AND key < 'z' AND key != 'middle'").unwrap();
        let pattern = QueryAnalyzer::analyze(&query).unwrap();
        // Range scan detected from > and < conditions
        assert_eq!(
            pattern,
            QueryPattern::RangeScan {
                column: "key".to_string(),
                start: "a".to_string(),
                end: "z".to_string(),
                inclusive_start: false,
                inclusive_end: false,
            }
        );
    }

    #[test]
    fn test_analyze_mixed_conditions_point_lookup_wins() {
        // Point lookup + non-key condition → still returns PointLookup
        let query =
            parse("SELECT * FROM text WHERE key = 'page.title' AND namespace = 'page'").unwrap();
        let pattern = QueryAnalyzer::analyze(&query).unwrap();
        // Analyzer only looks at key conditions, so should still detect point lookup
        assert_eq!(
            pattern,
            QueryPattern::PointLookup {
                column: "key".to_string(),
                value: "page.title".to_string(),
            }
        );
    }

    #[test]
    fn test_analyze_only_lower_bound_no_pattern() {
        let query = parse("SELECT * FROM text WHERE key >= 'page.a'").unwrap();
        let pattern = QueryAnalyzer::analyze(&query).unwrap();
        assert_eq!(pattern, QueryPattern::FullScan);
    }

    #[test]
    fn test_analyze_only_upper_bound_no_pattern() {
        let query = parse("SELECT * FROM text WHERE key < 'page.z'").unwrap();
        let pattern = QueryAnalyzer::analyze(&query).unwrap();
        assert_eq!(pattern, QueryPattern::FullScan);
    }
}
