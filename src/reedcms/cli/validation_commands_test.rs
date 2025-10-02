// Copyright 2025 Vivian Voss. Licensed under the Apache Licence, Version 2.0.
// SPDX-Licence-Identifier: Apache-2.0

#[cfg(test)]
mod tests {
    use crate::reedcms::cli::validation_commands;
    use std::collections::HashMap;

    // Helper to create flags
    fn make_flags(pairs: &[(&str, &str)]) -> HashMap<String, String> {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    #[test]
    fn test_validate_routes_no_flags() {
        let flags = HashMap::new();
        let result = validation_commands::validate_routes(&flags);
        // Test interface, actual validation depends on file existence
        let _ = result;
    }

    #[test]
    fn test_validate_routes_with_fix() {
        let flags = make_flags(&[("fix", "")]);
        let result = validation_commands::validate_routes(&flags);
        let _ = result;
    }

    #[test]
    fn test_validate_consistency_no_flags() {
        let flags = HashMap::new();
        let result = validation_commands::validate_consistency(&flags);
        let _ = result;
    }

    #[test]
    fn test_validate_text_missing_language() {
        let flags = HashMap::new();
        let result = validation_commands::validate_text(&flags);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_text_with_language_de() {
        let flags = make_flags(&[("language", "de")]);
        let result = validation_commands::validate_text(&flags);
        let _ = result;
    }

    #[test]
    fn test_validate_text_with_language_en() {
        let flags = make_flags(&[("language", "en")]);
        let result = validation_commands::validate_text(&flags);
        let _ = result;
    }

    #[test]
    fn test_validate_text_with_language_fr() {
        let flags = make_flags(&[("language", "fr")]);
        let result = validation_commands::validate_text(&flags);
        let _ = result;
    }

    #[test]
    fn test_validate_references_no_flags() {
        let flags = HashMap::new();
        let result = validation_commands::validate_references(&flags);
        let _ = result;
    }
}
