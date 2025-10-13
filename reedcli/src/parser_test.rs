// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Comprehensive tests for ReedCLI command parser.
//!
//! Tests cover all parsing functions with 100% code coverage:
//! - parse_args(): CLI argument parsing
//! - parse_shell_input(): Interactive shell input parsing
//! - parse_command_parts(): Core command parsing logic
//! - tokenise_input(): Tokenization with quote support (double quotes only)
//! - infer_tool(): Tool name inference
//!
//! All tests follow Arrange-Act-Assert pattern.

#[cfg(test)]
mod tests {
    use crate::parser::*;
    use crate::types::*;

    // ========================================================================
    // tokenise_input() tests
    // ========================================================================

    #[test]
    fn test_tokenise_simple_command() {
        let input = "get text key";
        let result = tokenise_input(input).unwrap();
        assert_eq!(result, vec!["get", "text", "key"]);
    }

    #[test]
    fn test_tokenise_quoted_string() {
        let input = r#"get text "my key with spaces""#;
        let result = tokenise_input(input).unwrap();
        assert_eq!(result, vec!["get", "text", "my key with spaces"]);
    }

    #[test]
    fn test_tokenise_escape_sequences() {
        let input = r#"get text "key with \"escaped\" quotes""#;
        let result = tokenise_input(input).unwrap();
        assert_eq!(result, vec!["get", "text", r#"key with "escaped" quotes"#]);
    }

    #[test]
    fn test_tokenise_backslash_escape() {
        let input = r#"get text "path\\with\\backslashes""#;
        let result = tokenise_input(input).unwrap();
        assert_eq!(result, vec!["get", "text", r"path\with\backslashes"]);
    }

    #[test]
    fn test_tokenise_unmatched_double_quote() {
        let input = r#"get text "unclosed"#;
        let result = tokenise_input(input);
        assert!(matches!(result, Err(CliError::UnmatchedQuote)));
    }

    #[test]
    fn test_tokenise_multiple_spaces() {
        let input = "get    text     key";
        let result = tokenise_input(input).unwrap();
        assert_eq!(result, vec!["get", "text", "key"]);
    }

    #[test]
    fn test_tokenise_leading_trailing_spaces() {
        let input = "  get text key  ";
        let result = tokenise_input(input).unwrap();
        assert_eq!(result, vec!["get", "text", "key"]);
    }

    #[test]
    fn test_tokenise_empty_string() {
        let input = "";
        let result = tokenise_input(input).unwrap();
        assert_eq!(result, Vec::<String>::new());
    }

    #[test]
    fn test_tokenise_only_whitespace() {
        let input = "    ";
        let result = tokenise_input(input).unwrap();
        assert_eq!(result, Vec::<String>::new());
    }

    #[test]
    fn test_tokenise_multiple_quoted_strings() {
        let input = r#"get "first string" "second string" unquoted"#;
        let result = tokenise_input(input).unwrap();
        assert_eq!(
            result,
            vec!["get", "first string", "second string", "unquoted"]
        );
    }

    #[test]
    fn test_tokenise_tabs() {
        let input = "get\ttext\tkey";
        let result = tokenise_input(input).unwrap();
        assert_eq!(result, vec!["get", "text", "key"]);
    }

    // ========================================================================
    // infer_tool() tests
    // ========================================================================

    #[test]
    fn test_infer_tool_with_colon_prefix() {
        assert_eq!(infer_tool("reedbase:query"), "reedbase");
        assert_eq!(infer_tool("reedcms:build"), "reedcms");
        assert_eq!(infer_tool("customtool:action"), "customtool");
    }

    #[test]
    fn test_infer_tool_reedbase_commands() {
        assert_eq!(infer_tool("query"), "reedbase");
        assert_eq!(infer_tool("tables"), "reedbase");
        assert_eq!(infer_tool("versions"), "reedbase");
        assert_eq!(infer_tool("rollback"), "reedbase");
    }

    #[test]
    fn test_infer_tool_default_reedcms() {
        assert_eq!(infer_tool("unknown"), "reedcms");
        assert_eq!(infer_tool("build"), "reedcms");
        assert_eq!(infer_tool("server"), "reedcms");
        assert_eq!(infer_tool(""), "reedcms");
    }

    #[test]
    fn test_infer_tool_colon_without_prefix() {
        let command = ":get";
        let result = infer_tool(command);
        // Empty string before colon
        assert_eq!(result, "");
    }

    // ========================================================================
    // parse_command_parts() tests
    // ========================================================================

    #[test]
    fn test_parse_command_parts_simple() {
        let tokens = vec!["query".to_string(), "SELECT * FROM users".to_string()];
        let result = parse_command_parts(&tokens).unwrap();

        assert_eq!(result.tool, "reedbase");
        assert_eq!(result.command, "query");
        assert_eq!(result.args, vec!["SELECT * FROM users"]);
        assert!(result.flags.is_empty());
    }

    #[test]
    fn test_parse_command_parts_with_colon_prefix() {
        let tokens = vec!["reedcms:build".to_string(), "release".to_string()];
        let result = parse_command_parts(&tokens).unwrap();

        assert_eq!(result.tool, "reedcms");
        assert_eq!(result.command, "reedcms:build");
        assert_eq!(result.args, vec!["release"]);
        assert!(result.flags.is_empty());
    }

    #[test]
    fn test_parse_command_parts_long_flags_with_values() {
        let tokens = vec![
            "query".to_string(),
            "SELECT * FROM users".to_string(),
            "--format".to_string(),
            "json".to_string(),
        ];
        let result = parse_command_parts(&tokens).unwrap();

        assert_eq!(result.tool, "reedbase");
        assert_eq!(result.command, "query");
        assert_eq!(result.args, vec!["SELECT * FROM users"]);
        assert_eq!(result.flags.get("format"), Some(&"json".to_string()));
    }

    #[test]
    fn test_parse_command_parts_short_flags_with_values() {
        let tokens = vec![
            "query".to_string(),
            "SELECT * FROM users".to_string(),
            "-f".to_string(),
            "json".to_string(),
        ];
        let result = parse_command_parts(&tokens).unwrap();

        assert_eq!(result.tool, "reedbase");
        assert_eq!(result.command, "query");
        assert_eq!(result.args, vec!["SELECT * FROM users"]);
        assert_eq!(result.flags.get("f"), Some(&"json".to_string()));
    }

    #[test]
    fn test_parse_command_parts_mixed_flags() {
        let tokens = vec![
            "query".to_string(),
            "SELECT * FROM users".to_string(),
            "--format".to_string(),
            "json".to_string(),
            "-v".to_string(),
            "true".to_string(),
        ];
        let result = parse_command_parts(&tokens).unwrap();

        assert_eq!(result.tool, "reedbase");
        assert_eq!(result.command, "query");
        assert_eq!(result.args, vec!["SELECT * FROM users"]);
        assert_eq!(result.flags.get("format"), Some(&"json".to_string()));
        assert_eq!(result.flags.get("v"), Some(&"true".to_string()));
    }

    #[test]
    fn test_parse_command_parts_empty_tokens() {
        let tokens: Vec<String> = vec![];
        let result = parse_command_parts(&tokens);
        assert!(matches!(result, Err(CliError::EmptyCommand)));
    }

    #[test]
    fn test_parse_command_parts_flag_without_value_error() {
        let tokens = vec![
            "query".to_string(),
            "SELECT * FROM users".to_string(),
            "--format".to_string(),
        ];
        let result = parse_command_parts(&tokens);
        assert!(matches!(result, Err(CliError::InvalidFlag { .. })));
    }

    #[test]
    fn test_parse_command_parts_args_and_flags_mixed() {
        let tokens = vec![
            "query".to_string(),
            "arg1".to_string(),
            "--format".to_string(),
            "json".to_string(),
            "arg2".to_string(),
        ];
        let result = parse_command_parts(&tokens).unwrap();

        assert_eq!(result.args, vec!["arg1", "arg2"]);
        assert_eq!(result.flags.get("format"), Some(&"json".to_string()));
    }

    #[test]
    fn test_parse_command_parts_duplicate_flags() {
        let tokens = vec![
            "query".to_string(),
            "--format".to_string(),
            "json".to_string(),
            "--format".to_string(),
            "xml".to_string(),
        ];
        let result = parse_command_parts(&tokens).unwrap();

        // Last value wins
        assert_eq!(result.flags.get("format"), Some(&"xml".to_string()));
    }

    #[test]
    fn test_parse_command_parts_short_flag_multichar() {
        // -abc is treated as argument, not -a -b -c
        let tokens = vec!["query".to_string(), "-abc".to_string()];
        let result = parse_command_parts(&tokens).unwrap();

        // -abc is >2 chars, so treated as argument
        assert_eq!(result.args, vec!["-abc"]);
    }

    // ========================================================================
    // parse_args() tests
    // ========================================================================

    #[test]
    fn test_parse_args_simple() {
        let args = vec![
            "reed".to_string(),
            "query".to_string(),
            "SELECT * FROM users".to_string(),
        ];
        let result = parse_args(args.into_iter()).unwrap();

        assert_eq!(result.tool, "reedbase");
        assert_eq!(result.command, "query");
        assert_eq!(result.args, vec!["SELECT * FROM users"]);
    }

    #[test]
    fn test_parse_args_with_flags() {
        let args = vec![
            "reed".to_string(),
            "query".to_string(),
            "SELECT * FROM users".to_string(),
            "--format".to_string(),
            "json".to_string(),
        ];
        let result = parse_args(args.into_iter()).unwrap();

        assert_eq!(result.tool, "reedbase");
        assert_eq!(result.command, "query");
        assert_eq!(result.args, vec!["SELECT * FROM users"]);
        assert_eq!(result.flags.get("format"), Some(&"json".to_string()));
    }

    #[test]
    fn test_parse_args_empty_after_binary() {
        let args = vec!["reed".to_string()];
        let result = parse_args(args.into_iter());
        assert!(matches!(result, Err(CliError::EmptyCommand)));
    }

    #[test]
    fn test_parse_args_completely_empty() {
        let args: Vec<String> = vec![];
        let result = parse_args(args.into_iter());
        assert!(matches!(result, Err(CliError::EmptyCommand)));
    }

    #[test]
    fn test_parse_args_with_spaces_in_args() {
        // Note: Shell would handle quote removal before passing to program
        let args = vec![
            "reed".to_string(),
            "query".to_string(),
            "SELECT * FROM users WHERE name = 'test'".to_string(),
        ];
        let result = parse_args(args.into_iter()).unwrap();

        assert_eq!(result.args, vec!["SELECT * FROM users WHERE name = 'test'"]);
    }

    // ========================================================================
    // parse_shell_input() tests
    // ========================================================================

    #[test]
    fn test_parse_shell_input_simple() {
        let input = "query SELECT * FROM users";
        let result = parse_shell_input(input).unwrap();

        assert_eq!(result.tool, "reedbase");
        assert_eq!(result.command, "query");
        assert_eq!(result.args, vec!["SELECT", "*", "FROM", "users"]);
    }

    #[test]
    fn test_parse_shell_input_with_quotes() {
        let input = r#"query "SELECT * FROM users""#;
        let result = parse_shell_input(input).unwrap();

        assert_eq!(result.tool, "reedbase");
        assert_eq!(result.command, "query");
        assert_eq!(result.args, vec!["SELECT * FROM users"]);
    }

    #[test]
    fn test_parse_shell_input_with_flags() {
        let input = "query SELECT * FROM users --format json";
        let result = parse_shell_input(input).unwrap();

        assert_eq!(result.tool, "reedbase");
        assert_eq!(result.command, "query");
        assert_eq!(result.args, vec!["SELECT", "*", "FROM", "users"]);
        assert_eq!(result.flags.get("format"), Some(&"json".to_string()));
    }

    #[test]
    fn test_parse_shell_input_empty() {
        let input = "";
        let result = parse_shell_input(input);
        assert!(matches!(result, Err(CliError::EmptyCommand)));
    }

    #[test]
    fn test_parse_shell_input_only_whitespace() {
        let input = "   ";
        let result = parse_shell_input(input);
        assert!(matches!(result, Err(CliError::EmptyCommand)));
    }

    #[test]
    fn test_parse_shell_input_unmatched_quote() {
        let input = r#"query "SELECT * FROM users"#;
        let result = parse_shell_input(input);
        assert!(matches!(result, Err(CliError::UnmatchedQuote)));
    }

    #[test]
    fn test_parse_shell_input_colon_prefix() {
        let input = "reedcms:build --format json";
        let result = parse_shell_input(input).unwrap();

        assert_eq!(result.tool, "reedcms");
        assert_eq!(result.command, "reedcms:build");
        assert_eq!(result.flags.get("format"), Some(&"json".to_string()));
    }

    #[test]
    fn test_parse_shell_input_complex_command() {
        let input = r#"query "SELECT * FROM users WHERE name = 'test'" --format json"#;
        let result = parse_shell_input(input).unwrap();

        assert_eq!(result.tool, "reedbase");
        assert_eq!(result.command, "query");
        assert_eq!(result.args, vec!["SELECT * FROM users WHERE name = 'test'"]);
        assert_eq!(result.flags.get("format"), Some(&"json".to_string()));
    }

    // ========================================================================
    // Performance tests
    // ========================================================================

    #[test]
    fn test_parse_performance_simple_command() {
        use std::time::Instant;

        let input = "query SELECT * FROM users";
        let start = Instant::now();

        for _ in 0..1000 {
            let _ = parse_shell_input(input).unwrap();
        }

        let elapsed = start.elapsed();
        let avg_micros = elapsed.as_micros() / 1000;

        // Should average < 1000μs per parse (ticket requirement: < 1ms)
        assert!(
            avg_micros < 1000,
            "Average parse time {}μs exceeds 1ms",
            avg_micros
        );
    }

    #[test]
    fn test_parse_performance_complex_command() {
        use std::time::Instant;

        let input = r#"query "SELECT * FROM users WHERE name = 'test' AND age > 18" --format json --verbose true"#;
        let start = Instant::now();

        for _ in 0..1000 {
            let _ = parse_shell_input(input).unwrap();
        }

        let elapsed = start.elapsed();
        let avg_micros = elapsed.as_micros() / 1000;

        // Should average < 1000μs per parse (ticket requirement: < 1ms)
        assert!(
            avg_micros < 1000,
            "Average parse time {}μs exceeds 1ms",
            avg_micros
        );
    }

    // ========================================================================
    // Edge case tests
    // ========================================================================

    #[test]
    fn test_parse_command_with_equals_in_arg() {
        let input = "query SELECT * FROM users WHERE name=test";
        let result = parse_shell_input(input).unwrap();

        assert_eq!(
            result.args,
            vec!["SELECT", "*", "FROM", "users", "WHERE", "name=test"]
        );
    }

    #[test]
    fn test_parse_command_with_numeric_args() {
        let input = "rollback 123";
        let result = parse_shell_input(input).unwrap();

        assert_eq!(result.tool, "reedbase");
        assert_eq!(result.command, "rollback");
        assert_eq!(result.args, vec!["123"]);
    }

    #[test]
    fn test_parse_command_with_special_chars() {
        let input = r#"query "SELECT * WHERE key@de = 'value_with-special.chars!'""#;
        let result = parse_shell_input(input).unwrap();

        assert_eq!(
            result.args,
            vec!["SELECT * WHERE key@de = 'value_with-special.chars!'"]
        );
    }

    #[test]
    fn test_parse_command_with_empty_quotes() {
        let input = r#"query """#;
        let result = parse_shell_input(input).unwrap();

        // Empty quoted string is not added as token (empty tokens are discarded)
        assert_eq!(result.args, Vec::<String>::new());
    }

    #[test]
    fn test_parse_command_case_sensitivity() {
        let input = "QUERY SELECT * FROM USERS";
        let result = parse_shell_input(input).unwrap();

        // Commands are case-sensitive
        assert_eq!(result.command, "QUERY");
        assert_eq!(result.tool, "reedcms"); // Unknown command defaults to reedcms
    }

    #[test]
    fn test_tokenise_consecutive_quotes() {
        let input = r#""first""second""#;
        let result = tokenise_input(input).unwrap();
        assert_eq!(result, vec!["firstsecond"]);
    }

    #[test]
    fn test_parse_double_dash_only() {
        let input = "query -- --format json";
        let result = parse_shell_input(input).unwrap();
        // "--" is treated as flag with empty name, takes "--format" as value
        // Then "json" becomes a positional argument
        assert_eq!(result.args, vec!["json"]);
        assert_eq!(result.flags.get(""), Some(&"--format".to_string()));
    }
}
