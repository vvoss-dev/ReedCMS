// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

#[cfg(test)]
mod tests {
    use crate::help::*;
    use crate::types::{CliConfig, CliError, CommandSpec, Registry, Tool};
    use std::collections::HashMap;

    fn create_test_registry() -> Registry {
        let mut tools = HashMap::new();

        let mut reedbase_commands = HashMap::new();
        reedbase_commands.insert(
            "query".to_string(),
            CommandSpec {
                handler: "execute_query".to_string(),
                help: "Execute SQL query".to_string(),
            },
        );
        reedbase_commands.insert(
            "tables".to_string(),
            CommandSpec {
                handler: "list_tables".to_string(),
                help: "List all tables".to_string(),
            },
        );

        tools.insert(
            "reedbase".to_string(),
            Tool {
                name: "reedbase".to_string(),
                description: Some("CSV-based versioned database".to_string()),
                binary: None,
                dependencies: vec![],
                commands: reedbase_commands,
            },
        );

        let mut reedcms_commands = HashMap::new();
        reedcms_commands.insert(
            "server:start".to_string(),
            CommandSpec {
                handler: "server_start".to_string(),
                help: "Start the server".to_string(),
            },
        );

        tools.insert(
            "reedcms".to_string(),
            Tool {
                name: "reedcms".to_string(),
                description: Some("Content management system".to_string()),
                binary: None,
                dependencies: vec![],
                commands: reedcms_commands,
            },
        );

        Registry {
            version: "1.0".to_string(),
            cli: CliConfig {
                name: "reedcli".to_string(),
                binary: Some("reed".to_string()),
                shell_prompt: "reed> ".to_string(),
                history_file: ".reed_history".to_string(),
            },
            tools,
        }
    }

    #[test]
    fn test_show_tools() {
        let registry = create_test_registry();
        let help = show_tools(&registry).unwrap();

        assert!(help.contains("Available tools:"));
        assert!(help.contains("reedbase"));
        assert!(help.contains("CSV-based versioned database"));
        assert!(help.contains("reedcms"));
        assert!(help.contains("Content management system"));
        assert!(help.contains("Use 'reed help <tool>'"));
    }

    #[test]
    fn test_show_tools_sorted() {
        let registry = create_test_registry();
        let help = show_tools(&registry).unwrap();

        // Check that reedbase comes before reedcms (alphabetically)
        let reedbase_pos = help.find("reedbase").unwrap();
        let reedcms_pos = help.find("reedcms").unwrap();
        assert!(reedbase_pos < reedcms_pos);
    }

    #[test]
    fn test_show_tool_commands() {
        let registry = create_test_registry();
        let help = show_tool_commands(&registry, "reedbase").unwrap();

        assert!(help.contains("Commands for reedbase:"));
        assert!(help.contains("CSV-based versioned database"));
        assert!(help.contains("query"));
        assert!(help.contains("Execute SQL query"));
        assert!(help.contains("tables"));
        assert!(help.contains("List all tables"));
        assert!(help.contains("Use 'reed help reedbase <command>'"));
    }

    #[test]
    fn test_show_tool_commands_sorted() {
        let registry = create_test_registry();
        let help = show_tool_commands(&registry, "reedbase").unwrap();

        // Check that query comes before tables (alphabetically)
        let query_pos = help.find("query").unwrap();
        let tables_pos = help.find("tables").unwrap();
        assert!(query_pos < tables_pos);
    }

    #[test]
    fn test_show_tool_commands_not_found() {
        let registry = create_test_registry();
        let result = show_tool_commands(&registry, "nonexistent");

        assert!(matches!(result, Err(CliError::ToolNotFound { .. })));
        if let Err(CliError::ToolNotFound { tool }) = result {
            assert_eq!(tool, "nonexistent");
        }
    }

    #[test]
    fn test_show_command_help() {
        let registry = create_test_registry();
        let help = show_command_help(&registry, "reedbase", "query").unwrap();

        assert!(help.contains("Command: reedbase query"));
        assert!(help.contains("Execute SQL query"));
        assert!(help.contains("Handler: execute_query"));
    }

    #[test]
    fn test_show_command_help_tool_not_found() {
        let registry = create_test_registry();
        let result = show_command_help(&registry, "nonexistent", "query");

        assert!(matches!(result, Err(CliError::ToolNotFound { .. })));
    }

    #[test]
    fn test_show_command_help_command_not_found() {
        let registry = create_test_registry();
        let result = show_command_help(&registry, "reedbase", "nonexistent");

        assert!(matches!(result, Err(CliError::CommandNotFound { .. })));
        if let Err(CliError::CommandNotFound { tool, command }) = result {
            assert_eq!(tool, "reedbase");
            assert_eq!(command, "nonexistent");
        }
    }

    #[test]
    fn test_show_help_no_args() {
        let registry = create_test_registry();
        let help = show_help(&registry, &[]).unwrap();

        assert!(help.contains("Available tools:"));
        assert!(help.contains("reedbase"));
        assert!(help.contains("reedcms"));
    }

    #[test]
    fn test_show_help_tool_arg() {
        let registry = create_test_registry();
        let args = vec!["reedbase".to_string()];
        let help = show_help(&registry, &args).unwrap();

        assert!(help.contains("Commands for reedbase:"));
        assert!(help.contains("query"));
    }

    #[test]
    fn test_show_help_tool_and_command_arg() {
        let registry = create_test_registry();
        let args = vec!["reedbase".to_string(), "query".to_string()];
        let help = show_help(&registry, &args).unwrap();

        assert!(help.contains("Command: reedbase query"));
        assert!(help.contains("Execute SQL query"));
    }

    #[test]
    fn test_show_help_too_many_args() {
        let registry = create_test_registry();
        let args = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let result = show_help(&registry, &args);

        assert!(matches!(result, Err(CliError::InvalidArgs { .. })));
        if let Err(CliError::InvalidArgs { reason }) = result {
            assert!(reason.contains("Too many arguments"));
        }
    }

    #[test]
    fn test_show_help_with_server_start_command() {
        let registry = create_test_registry();
        let help = show_command_help(&registry, "reedcms", "server:start").unwrap();

        assert!(help.contains("Command: reedcms server:start"));
        assert!(help.contains("Start the server"));
        assert!(help.contains("Handler: server_start"));
    }

    #[test]
    fn test_show_tool_commands_with_no_description() {
        let mut registry = create_test_registry();

        // Add a tool without description
        let mut test_commands = HashMap::new();
        test_commands.insert(
            "test".to_string(),
            CommandSpec {
                handler: "test_handler".to_string(),
                help: "Test command".to_string(),
            },
        );

        registry.tools.insert(
            "testtool".to_string(),
            Tool {
                name: "testtool".to_string(),
                description: None,
                binary: None,
                dependencies: vec![],
                commands: test_commands,
            },
        );

        let help = show_tools(&registry).unwrap();
        assert!(help.contains("testtool"));
        assert!(help.contains("(no description)"));
    }
}
