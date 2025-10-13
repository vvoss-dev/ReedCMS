// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Comprehensive tests for ReedCLI registry loader.
//!
//! Tests cover all registry functions with 100% code coverage:
//! - load_registry(): Load from file
//! - parse_registry(): Parse TOML string
//! - Registry methods: find_command(), list_tools(), list_commands()
//! - Validation: circular dependencies, missing dependencies
//!
//! All tests follow Arrange-Act-Assert pattern.

#[cfg(test)]
mod tests {
    use crate::registry::*;
    use crate::types::*;

    // ========================================================================
    // parse_registry() tests - valid registries
    // ========================================================================

    #[test]
    fn test_parse_minimal_registry() {
        let toml = r#"
            [registry]
            version = "1.0"

            [cli]
            name = "reedcli"

            [tools.reedbase]
            name = "reedbase"

            [tools.reedbase.commands]
            query = { handler = "execute_query", help = "Execute query" }
        "#;

        let registry = parse_registry(toml).unwrap();
        assert_eq!(registry.version, "1.0");
        assert_eq!(registry.cli.name, "reedcli");
        assert!(registry.tools.contains_key("reedbase"));
    }

    #[test]
    fn test_parse_full_registry() {
        let toml = r#"
            [registry]
            version = "1.0"

            [cli]
            name = "reedcli"
            binary = "reed"
            shell_prompt = "reed> "
            history_file = ".reed_history"

            [tools.reedbase]
            name = "reedbase"
            description = "CSV-based versioned database"
            binary = "reedbase"

            [tools.reedbase.commands]
            query = { handler = "execute_query", help = "Execute SQL query" }
            tables = { handler = "list_tables", help = "List all tables" }

            [tools.reedcms]
            name = "reedcms"
            description = "Content management system"
            dependencies = ["reedbase"]

            [tools.reedcms.commands]
            "server:start" = { handler = "server_start", help = "Start server" }
        "#;

        let registry = parse_registry(toml).unwrap();

        // Check version
        assert_eq!(registry.version, "1.0");

        // Check CLI config
        assert_eq!(registry.cli.name, "reedcli");
        assert_eq!(registry.cli.binary, Some("reed".to_string()));
        assert_eq!(registry.cli.shell_prompt, "reed> ");
        assert_eq!(registry.cli.history_file, ".reed_history");

        // Check tools
        assert_eq!(registry.tools.len(), 2);

        // Check reedbase
        let reedbase = registry.tools.get("reedbase").unwrap();
        assert_eq!(reedbase.name, "reedbase");
        assert_eq!(
            reedbase.description,
            Some("CSV-based versioned database".to_string())
        );
        assert_eq!(reedbase.binary, Some("reedbase".to_string()));
        assert_eq!(reedbase.commands.len(), 2);

        // Check reedcms
        let reedcms = registry.tools.get("reedcms").unwrap();
        assert_eq!(reedcms.name, "reedcms");
        assert_eq!(reedcms.dependencies, vec!["reedbase"]);
        assert_eq!(reedcms.commands.len(), 1);
    }

    #[test]
    fn test_default_shell_prompt() {
        let toml = r#"
            [registry]
            version = "1.0"
            [cli]
            name = "reedcli"
            [tools.reedbase]
            name = "reedbase"
            [tools.reedbase.commands]
            query = { handler = "execute_query", help = "" }
        "#;

        let registry = parse_registry(toml).unwrap();
        assert_eq!(registry.cli.shell_prompt, "reed> ");
    }

    #[test]
    fn test_default_history_file() {
        let toml = r#"
            [registry]
            version = "1.0"
            [cli]
            name = "reedcli"
            [tools.reedbase]
            name = "reedbase"
            [tools.reedbase.commands]
            query = { handler = "execute_query", help = "" }
        "#;

        let registry = parse_registry(toml).unwrap();
        assert_eq!(registry.cli.history_file, ".reed_history");
    }

    #[test]
    fn test_custom_shell_prompt() {
        let toml = r#"
            [registry]
            version = "1.0"
            [cli]
            name = "reedcli"
            shell_prompt = "custom> "
            [tools.reedbase]
            name = "reedbase"
            [tools.reedbase.commands]
            query = { handler = "execute_query", help = "" }
        "#;

        let registry = parse_registry(toml).unwrap();
        assert_eq!(registry.cli.shell_prompt, "custom> ");
    }

    #[test]
    fn test_tool_without_optional_fields() {
        let toml = r#"
            [registry]
            version = "1.0"
            [cli]
            name = "reedcli"
            [tools.reedbase]
            name = "reedbase"
            [tools.reedbase.commands]
            query = { handler = "execute_query", help = "" }
        "#;

        let registry = parse_registry(toml).unwrap();
        let tool = registry.tools.get("reedbase").unwrap();
        assert_eq!(tool.description, None);
        assert_eq!(tool.binary, None);
        assert_eq!(tool.dependencies, Vec::<String>::new());
    }

    #[test]
    fn test_command_without_help() {
        let toml = r#"
            [registry]
            version = "1.0"
            [cli]
            name = "reedcli"
            [tools.reedbase]
            name = "reedbase"
            [tools.reedbase.commands]
            query = { handler = "execute_query" }
        "#;

        let registry = parse_registry(toml).unwrap();
        let cmd = registry.find_command("reedbase", "query").unwrap();
        assert_eq!(cmd.handler, "execute_query");
        assert_eq!(cmd.help, "");
    }

    // ========================================================================
    // parse_registry() tests - invalid registries
    // ========================================================================

    #[test]
    fn test_missing_registry_version() {
        let toml = r#"
            [cli]
            name = "reedcli"
        "#;

        let result = parse_registry(toml);
        assert!(matches!(result, Err(CliError::InvalidRegistry { .. })));
    }

    #[test]
    fn test_missing_cli_section() {
        let toml = r#"
            [registry]
            version = "1.0"
        "#;

        let result = parse_registry(toml);
        assert!(matches!(result, Err(CliError::InvalidRegistry { .. })));
    }

    #[test]
    fn test_missing_cli_name() {
        let toml = r#"
            [registry]
            version = "1.0"
            [cli]
            shell_prompt = "reed> "
        "#;

        let result = parse_registry(toml);
        assert!(matches!(result, Err(CliError::InvalidRegistry { .. })));
    }

    #[test]
    fn test_missing_tools_section() {
        let toml = r#"
            [registry]
            version = "1.0"
            [cli]
            name = "reedcli"
        "#;

        let result = parse_registry(toml);
        assert!(matches!(result, Err(CliError::InvalidRegistry { .. })));
    }

    #[test]
    fn test_tool_not_a_table() {
        let toml = r#"
            [registry]
            version = "1.0"
            [cli]
            name = "reedcli"
            [tools]
            reedbase = "invalid"
        "#;

        let result = parse_registry(toml);
        assert!(matches!(result, Err(CliError::InvalidRegistry { .. })));
    }

    #[test]
    fn test_missing_commands_section() {
        let toml = r#"
            [registry]
            version = "1.0"
            [cli]
            name = "reedcli"
            [tools.reedbase]
            name = "reedbase"
        "#;

        let result = parse_registry(toml);
        assert!(matches!(result, Err(CliError::InvalidRegistry { .. })));
    }

    #[test]
    fn test_command_not_a_table() {
        let toml = r#"
            [registry]
            version = "1.0"
            [cli]
            name = "reedcli"
            [tools.reedbase]
            name = "reedbase"
            [tools.reedbase.commands]
            query = "invalid"
        "#;

        let result = parse_registry(toml);
        assert!(matches!(result, Err(CliError::InvalidRegistry { .. })));
    }

    #[test]
    fn test_command_missing_handler() {
        let toml = r#"
            [registry]
            version = "1.0"
            [cli]
            name = "reedcli"
            [tools.reedbase]
            name = "reedbase"
            [tools.reedbase.commands]
            query = { help = "Execute query" }
        "#;

        let result = parse_registry(toml);
        assert!(matches!(result, Err(CliError::InvalidRegistry { .. })));
    }

    #[test]
    fn test_invalid_toml_syntax() {
        let toml = r#"
            [registry
            version = "1.0"
        "#;

        let result = parse_registry(toml);
        assert!(matches!(result, Err(CliError::InvalidRegistry { .. })));
    }

    // ========================================================================
    // Dependency validation tests
    // ========================================================================

    #[test]
    fn test_parse_dependencies() {
        let toml = r#"
            [registry]
            version = "1.0"
            [cli]
            name = "reedcli"
            [tools.reedbase]
            name = "reedbase"
            [tools.reedbase.commands]
            query = { handler = "execute_query", help = "" }
            [tools.reedcms]
            name = "reedcms"
            dependencies = ["reedbase"]
            [tools.reedcms.commands]
            "server:start" = { handler = "server_start", help = "" }
        "#;

        let registry = parse_registry(toml).unwrap();
        let tool = registry.tools.get("reedcms").unwrap();
        assert_eq!(tool.dependencies, vec!["reedbase"]);
    }

    #[test]
    fn test_circular_dependency_detection() {
        let toml = r#"
            [registry]
            version = "1.0"
            [cli]
            name = "reedcli"
            [tools.a]
            name = "a"
            dependencies = ["b"]
            [tools.a.commands]
            cmd = { handler = "handler", help = "" }
            [tools.b]
            name = "b"
            dependencies = ["a"]
            [tools.b.commands]
            cmd = { handler = "handler", help = "" }
        "#;

        let result = parse_registry(toml);
        assert!(matches!(result, Err(CliError::CircularDependency { .. })));
    }

    #[test]
    fn test_circular_dependency_three_tools() {
        let toml = r#"
            [registry]
            version = "1.0"
            [cli]
            name = "reedcli"
            [tools.a]
            name = "a"
            dependencies = ["b"]
            [tools.a.commands]
            cmd = { handler = "handler", help = "" }
            [tools.b]
            name = "b"
            dependencies = ["c"]
            [tools.b.commands]
            cmd = { handler = "handler", help = "" }
            [tools.c]
            name = "c"
            dependencies = ["a"]
            [tools.c.commands]
            cmd = { handler = "handler", help = "" }
        "#;

        let result = parse_registry(toml);
        assert!(matches!(result, Err(CliError::CircularDependency { .. })));
    }

    #[test]
    fn test_missing_dependency_detection() {
        let toml = r#"
            [registry]
            version = "1.0"
            [cli]
            name = "reedcli"
            [tools.reedcms]
            name = "reedcms"
            dependencies = ["nonexistent"]
            [tools.reedcms.commands]
            cmd = { handler = "handler", help = "" }
        "#;

        let result = parse_registry(toml);
        assert!(matches!(result, Err(CliError::MissingDependency { .. })));
    }

    #[test]
    fn test_valid_dependency_chain() {
        let toml = r#"
            [registry]
            version = "1.0"
            [cli]
            name = "reedcli"
            [tools.a]
            name = "a"
            [tools.a.commands]
            cmd = { handler = "handler", help = "" }
            [tools.b]
            name = "b"
            dependencies = ["a"]
            [tools.b.commands]
            cmd = { handler = "handler", help = "" }
            [tools.c]
            name = "c"
            dependencies = ["b"]
            [tools.c.commands]
            cmd = { handler = "handler", help = "" }
        "#;

        let registry = parse_registry(toml).unwrap();
        assert_eq!(registry.tools.len(), 3);
    }

    // ========================================================================
    // Registry::find_command() tests
    // ========================================================================

    #[test]
    fn test_find_command() {
        let toml = r#"
            [registry]
            version = "1.0"
            [cli]
            name = "reedcli"
            [tools.reedbase]
            name = "reedbase"
            [tools.reedbase.commands]
            query = { handler = "execute_query", help = "Execute query" }
        "#;

        let registry = parse_registry(toml).unwrap();
        let cmd = registry.find_command("reedbase", "query").unwrap();
        assert_eq!(cmd.handler, "execute_query");
        assert_eq!(cmd.help, "Execute query");
    }

    #[test]
    fn test_find_command_with_colon() {
        let toml = r#"
            [registry]
            version = "1.0"
            [cli]
            name = "reedcli"
            [tools.reedcms]
            name = "reedcms"
            [tools.reedcms.commands]
            "server:start" = { handler = "server_start", help = "Start server" }
        "#;

        let registry = parse_registry(toml).unwrap();
        let cmd = registry.find_command("reedcms", "server:start").unwrap();
        assert_eq!(cmd.handler, "server_start");
    }

    #[test]
    fn test_command_not_found() {
        let toml = r#"
            [registry]
            version = "1.0"
            [cli]
            name = "reedcli"
            [tools.reedbase]
            name = "reedbase"
            [tools.reedbase.commands]
            query = { handler = "execute_query", help = "" }
        "#;

        let registry = parse_registry(toml).unwrap();
        let result = registry.find_command("reedbase", "nonexistent");
        assert!(matches!(result, Err(CliError::CommandNotFound { .. })));
    }

    #[test]
    fn test_tool_not_found() {
        let toml = r#"
            [registry]
            version = "1.0"
            [cli]
            name = "reedcli"
            [tools.reedbase]
            name = "reedbase"
            [tools.reedbase.commands]
            query = { handler = "execute_query", help = "" }
        "#;

        let registry = parse_registry(toml).unwrap();
        let result = registry.find_command("nonexistent", "query");
        assert!(matches!(result, Err(CliError::ToolNotFound { .. })));
    }

    // ========================================================================
    // Registry::list_tools() tests
    // ========================================================================

    #[test]
    fn test_list_tools() {
        let toml = r#"
            [registry]
            version = "1.0"
            [cli]
            name = "reedcli"
            [tools.reedbase]
            name = "reedbase"
            [tools.reedbase.commands]
            query = { handler = "execute_query", help = "" }
            [tools.reedcms]
            name = "reedcms"
            [tools.reedcms.commands]
            cmd = { handler = "handler", help = "" }
        "#;

        let registry = parse_registry(toml).unwrap();
        let mut tools = registry.list_tools();
        tools.sort();
        assert_eq!(tools, vec!["reedbase", "reedcms"]);
    }

    #[test]
    fn test_list_tools_single() {
        let toml = r#"
            [registry]
            version = "1.0"
            [cli]
            name = "reedcli"
            [tools.reedbase]
            name = "reedbase"
            [tools.reedbase.commands]
            query = { handler = "execute_query", help = "" }
        "#;

        let registry = parse_registry(toml).unwrap();
        let tools = registry.list_tools();
        assert_eq!(tools, vec!["reedbase"]);
    }

    // ========================================================================
    // Registry::list_commands() tests
    // ========================================================================

    #[test]
    fn test_list_commands() {
        let toml = r#"
            [registry]
            version = "1.0"
            [cli]
            name = "reedcli"
            [tools.reedbase]
            name = "reedbase"
            [tools.reedbase.commands]
            query = { handler = "execute_query", help = "" }
            tables = { handler = "list_tables", help = "" }
        "#;

        let registry = parse_registry(toml).unwrap();
        let mut commands = registry.list_commands("reedbase").unwrap();
        commands.sort();
        assert_eq!(commands, vec!["query", "tables"]);
    }

    #[test]
    fn test_list_commands_single() {
        let toml = r#"
            [registry]
            version = "1.0"
            [cli]
            name = "reedcli"
            [tools.reedbase]
            name = "reedbase"
            [tools.reedbase.commands]
            query = { handler = "execute_query", help = "" }
        "#;

        let registry = parse_registry(toml).unwrap();
        let commands = registry.list_commands("reedbase").unwrap();
        assert_eq!(commands, vec!["query"]);
    }

    #[test]
    fn test_list_commands_tool_not_found() {
        let toml = r#"
            [registry]
            version = "1.0"
            [cli]
            name = "reedcli"
            [tools.reedbase]
            name = "reedbase"
            [tools.reedbase.commands]
            query = { handler = "execute_query", help = "" }
        "#;

        let registry = parse_registry(toml).unwrap();
        let result = registry.list_commands("nonexistent");
        assert!(matches!(result, Err(CliError::ToolNotFound { .. })));
    }

    // ========================================================================
    // load_registry() tests (file I/O)
    // ========================================================================

    #[test]
    fn test_load_registry_file_not_found() {
        let result = load_registry("/nonexistent/path/Reed.toml");
        assert!(matches!(result, Err(CliError::RegistryNotFound { .. })));
    }

    #[test]
    fn test_load_registry_from_file() {
        use std::io::Write;

        let toml = r#"
            [registry]
            version = "1.0"
            [cli]
            name = "reedcli"
            [tools.reedbase]
            name = "reedbase"
            [tools.reedbase.commands]
            query = { handler = "execute_query", help = "Execute query" }
        "#;

        // Create temp file
        let temp_dir = std::env::temp_dir();
        let temp_file = temp_dir.join("test_reed.toml");
        let mut file = std::fs::File::create(&temp_file).unwrap();
        file.write_all(toml.as_bytes()).unwrap();

        // Load registry
        let registry = load_registry(&temp_file).unwrap();
        assert_eq!(registry.version, "1.0");

        // Cleanup
        std::fs::remove_file(temp_file).ok();
    }

    // ========================================================================
    // Performance validation tests
    // ========================================================================

    #[test]
    fn test_parse_performance() {
        use std::time::Instant;

        let toml = r#"
            [registry]
            version = "1.0"
            [cli]
            name = "reedcli"
            [tools.reedbase]
            name = "reedbase"
            [tools.reedbase.commands]
            query = { handler = "execute_query", help = "" }
            tables = { handler = "list_tables", help = "" }
            versions = { handler = "list_versions", help = "" }
        "#;

        let start = Instant::now();
        for _ in 0..100 {
            let _ = parse_registry(toml).unwrap();
        }
        let elapsed = start.elapsed();
        let avg_ms = elapsed.as_millis() / 100;

        // Should average < 10ms per parse (ticket requirement)
        assert!(avg_ms < 10, "Average parse time {}ms exceeds 10ms", avg_ms);
    }

    #[test]
    fn test_find_command_performance() {
        use std::time::Instant;

        let toml = r#"
            [registry]
            version = "1.0"
            [cli]
            name = "reedcli"
            [tools.reedbase]
            name = "reedbase"
            [tools.reedbase.commands]
            query = { handler = "execute_query", help = "" }
        "#;

        let registry = parse_registry(toml).unwrap();

        let start = Instant::now();
        for _ in 0..10000 {
            let _ = registry.find_command("reedbase", "query").unwrap();
        }
        let elapsed = start.elapsed();
        let avg_micros = elapsed.as_micros() / 10000;

        // Should average < 10μs per lookup (ticket requirement)
        assert!(
            avg_micros < 10,
            "Average lookup time {}μs exceeds 10μs",
            avg_micros
        );
    }
}
