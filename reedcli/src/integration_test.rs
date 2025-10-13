// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

#[cfg(test)]
mod tests {
    use crate::integration::*;
    use crate::types::*;
    use std::collections::HashMap;

    #[test]
    fn test_determine_output_format_json() {
        let mut flags = HashMap::new();
        flags.insert("format".to_string(), "json".to_string());
        assert_eq!(determine_output_format(&flags), OutputFormat::Json);
    }

    #[test]
    fn test_determine_output_format_csv() {
        let mut flags = HashMap::new();
        flags.insert("format".to_string(), "csv".to_string());
        assert_eq!(determine_output_format(&flags), OutputFormat::Csv);
    }

    #[test]
    fn test_determine_output_format_plain() {
        let mut flags = HashMap::new();
        flags.insert("format".to_string(), "plain".to_string());
        assert_eq!(determine_output_format(&flags), OutputFormat::Plain);
    }

    #[test]
    fn test_determine_output_format_table() {
        let mut flags = HashMap::new();
        flags.insert("format".to_string(), "table".to_string());
        assert_eq!(determine_output_format(&flags), OutputFormat::Table);
    }

    #[test]
    fn test_determine_output_format_default() {
        let flags = HashMap::new();
        assert_eq!(determine_output_format(&flags), OutputFormat::Table);
    }

    #[test]
    fn test_determine_output_format_invalid() {
        let mut flags = HashMap::new();
        flags.insert("format".to_string(), "invalid".to_string());
        assert_eq!(determine_output_format(&flags), OutputFormat::Table);
    }

    #[test]
    fn test_get_exit_code() {
        let output = CommandOutput {
            data: serde_json::json!({}),
            format: OutputFormat::Table,
            exit_code: 42,
        };
        assert_eq!(get_exit_code(&output), 42);
    }

    #[test]
    fn test_get_exit_code_success() {
        let output = CommandOutput {
            data: serde_json::json!({}),
            format: OutputFormat::Json,
            exit_code: 0,
        };
        assert_eq!(get_exit_code(&output), 0);
    }

    #[test]
    fn test_error_to_exit_code_empty_command() {
        assert_eq!(error_to_exit_code(&CliError::EmptyCommand), 1);
    }

    #[test]
    fn test_error_to_exit_code_unmatched_quote() {
        assert_eq!(error_to_exit_code(&CliError::UnmatchedQuote), 1);
    }

    #[test]
    fn test_error_to_exit_code_invalid_flag() {
        assert_eq!(
            error_to_exit_code(&CliError::InvalidFlag {
                flag: "test".to_string(),
                reason: "test".to_string(),
            }),
            1
        );
    }

    #[test]
    fn test_error_to_exit_code_tool_not_found() {
        assert_eq!(
            error_to_exit_code(&CliError::ToolNotFound {
                tool: "test".to_string(),
            }),
            1
        );
    }

    #[test]
    fn test_error_to_exit_code_command_not_found() {
        assert_eq!(
            error_to_exit_code(&CliError::CommandNotFound {
                tool: "test".to_string(),
                command: "test".to_string(),
            }),
            1
        );
    }

    #[test]
    fn test_error_to_exit_code_handler_not_found() {
        assert_eq!(
            error_to_exit_code(&CliError::HandlerNotFound {
                handler: "test".to_string(),
            }),
            1
        );
    }

    #[test]
    fn test_error_to_exit_code_invalid_args() {
        assert_eq!(
            error_to_exit_code(&CliError::InvalidArgs {
                reason: "test".to_string(),
            }),
            1
        );
    }

    #[test]
    fn test_error_to_exit_code_unknown_command() {
        assert_eq!(
            error_to_exit_code(&CliError::UnknownCommand {
                command: "test".to_string(),
            }),
            1
        );
    }

    #[test]
    fn test_error_to_exit_code_adapter_not_found() {
        assert_eq!(
            error_to_exit_code(&CliError::AdapterNotFound {
                adapter: "test".to_string(),
            }),
            1
        );
    }

    #[test]
    fn test_error_to_exit_code_ambiguous_command() {
        assert_eq!(
            error_to_exit_code(&CliError::AmbiguousCommand {
                command: "test".to_string(),
                adapters: vec!["a".to_string(), "b".to_string()],
            }),
            1
        );
    }

    #[test]
    fn test_error_to_exit_code_invalid_registry() {
        assert_eq!(
            error_to_exit_code(&CliError::InvalidRegistry {
                reason: "test".to_string(),
            }),
            2
        );
    }

    #[test]
    fn test_error_to_exit_code_circular_dependency() {
        assert_eq!(
            error_to_exit_code(&CliError::CircularDependency {
                tool: "test".to_string(),
            }),
            2
        );
    }

    #[test]
    fn test_error_to_exit_code_missing_dependency() {
        assert_eq!(
            error_to_exit_code(&CliError::MissingDependency {
                tool: "test".to_string(),
                dependency: "dep".to_string(),
            }),
            2
        );
    }

    #[test]
    fn test_error_to_exit_code_format_error() {
        assert_eq!(
            error_to_exit_code(&CliError::FormatError {
                reason: "test".to_string(),
            }),
            2
        );
    }

    #[test]
    fn test_error_to_exit_code_shell_error() {
        assert_eq!(
            error_to_exit_code(&CliError::ShellError {
                reason: "test".to_string(),
            }),
            2
        );
    }

    #[test]
    fn test_error_to_exit_code_tool_error() {
        assert_eq!(
            error_to_exit_code(&CliError::ToolError {
                reason: "test".to_string(),
            }),
            2
        );
    }

    #[test]
    fn test_error_to_exit_code_adapter_error() {
        assert_eq!(
            error_to_exit_code(&CliError::AdapterError {
                adapter: "test".to_string(),
                message: "test".to_string(),
            }),
            2
        );
    }

    #[test]
    fn test_error_to_exit_code_version_mismatch() {
        assert_eq!(
            error_to_exit_code(&CliError::VersionMismatch {
                adapter: "test".to_string(),
                required: "1.0".to_string(),
                actual: "0.9".to_string(),
            }),
            2
        );
    }

    #[test]
    fn test_error_to_exit_code_required_adapter_missing() {
        assert_eq!(
            error_to_exit_code(&CliError::RequiredAdapterMissing {
                adapter: "test".to_string(),
            }),
            2
        );
    }

    #[test]
    fn test_error_to_exit_code_invalid_version() {
        assert_eq!(
            error_to_exit_code(&CliError::InvalidVersion {
                version: "invalid".to_string(),
            }),
            2
        );
    }

    #[test]
    fn test_execute_command_reedbase_list_tables() {
        let registry = create_test_registry();
        let command = Command {
            tool: "reedbase".to_string(),
            command: "tables".to_string(),
            args: vec![],
            flags: HashMap::new(),
        };

        let result = execute_command(&command, &registry);
        assert!(result.is_ok());

        let output = result.unwrap();
        assert_eq!(output.exit_code, 0);
        assert_eq!(output.format, OutputFormat::Table);
        assert!(output.data.is_array());
    }

    #[test]
    fn test_execute_command_reedbase_query() {
        let registry = create_test_registry();
        let command = Command {
            tool: "reedbase".to_string(),
            command: "query".to_string(),
            args: vec!["SELECT * FROM users".to_string()],
            flags: HashMap::new(),
        };

        let result = execute_command(&command, &registry);
        assert!(result.is_ok());

        let output = result.unwrap();
        assert_eq!(output.exit_code, 0);
        assert!(output.data.is_array());
    }

    #[test]
    fn test_execute_command_with_json_format() {
        let registry = create_test_registry();
        let mut flags = HashMap::new();
        flags.insert("format".to_string(), "json".to_string());

        let command = Command {
            tool: "reedbase".to_string(),
            command: "tables".to_string(),
            args: vec![],
            flags,
        };

        let result = execute_command(&command, &registry);
        assert!(result.is_ok());

        let output = result.unwrap();
        assert_eq!(output.format, OutputFormat::Json);
    }

    #[test]
    fn test_execute_command_tool_not_found() {
        let registry = create_test_registry();
        let command = Command {
            tool: "nonexistent".to_string(),
            command: "test".to_string(),
            args: vec![],
            flags: HashMap::new(),
        };

        let result = execute_command(&command, &registry);
        assert!(result.is_err());

        match result.unwrap_err() {
            CliError::CommandNotFound { .. } => {}
            _ => panic!("Expected CommandNotFound error"),
        }
    }

    #[test]
    fn test_execute_command_command_not_found() {
        let registry = create_test_registry();
        let command = Command {
            tool: "reedbase".to_string(),
            command: "nonexistent".to_string(),
            args: vec![],
            flags: HashMap::new(),
        };

        let result = execute_command(&command, &registry);
        assert!(result.is_err());

        match result.unwrap_err() {
            CliError::CommandNotFound { tool, command } => {
                assert_eq!(tool, "reedbase");
                assert_eq!(command, "nonexistent");
            }
            _ => panic!("Expected CommandNotFound error"),
        }
    }

    #[test]
    fn test_execute_command_handler_not_found() {
        let registry = create_test_registry();
        let command = Command {
            tool: "reedbase".to_string(),
            command: "versions".to_string(),
            args: vec![],
            flags: HashMap::new(),
        };

        let result = execute_command(&command, &registry);
        assert!(result.is_err());

        match result.unwrap_err() {
            CliError::HandlerNotFound { handler } => {
                assert_eq!(handler, "list_versions");
            }
            _ => panic!("Expected HandlerNotFound error"),
        }
    }

    // Helper function to create test registry
    fn create_test_registry() -> Registry {
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
        reedbase_commands.insert(
            "versions".to_string(),
            CommandSpec {
                handler: "list_versions".to_string(),
                help: "List table versions".to_string(),
            },
        );

        let mut reedcms_commands = HashMap::new();
        reedcms_commands.insert(
            "server:start".to_string(),
            CommandSpec {
                handler: "server_start".to_string(),
                help: "Start server".to_string(),
            },
        );

        let mut tools = HashMap::new();
        tools.insert(
            "reedbase".to_string(),
            Tool {
                name: "reedbase".to_string(),
                description: Some("ReedBase database".to_string()),
                binary: Some("reedbase".to_string()),
                dependencies: vec![],
                commands: reedbase_commands,
            },
        );
        tools.insert(
            "reedcms".to_string(),
            Tool {
                name: "reedcms".to_string(),
                description: Some("ReedCMS server".to_string()),
                binary: Some("reedcms".to_string()),
                dependencies: vec![],
                commands: reedcms_commands,
            },
        );

        Registry {
            version: "1.0.0".to_string(),
            cli: CliConfig {
                name: "reed".to_string(),
                binary: Some("reed".to_string()),
                shell_prompt: "reed> ".to_string(),
                history_file: ".reed_history".to_string(),
            },
            tools,
        }
    }
}
