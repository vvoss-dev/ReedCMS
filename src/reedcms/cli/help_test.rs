// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

#[cfg(test)]
mod tests {
    use super::super::help::*;

    #[test]
    fn test_print_general_help() {
        let result = print_general_help().unwrap();

        assert!(result.data.contains("ReedCMS Command-Line Interface"));
        assert!(result
            .data
            .contains("Usage: reed <command:action> [args] [flags]"));
        assert!(result.data.contains("Available Commands:"));
        assert!(result.data.contains("data:"));
        assert!(result.data.contains("layout:"));
        assert!(result.data.contains("user:"));
        assert!(result.data.contains("role:"));
        assert!(result.data.contains("taxonomy:"));
        assert!(result.source == "cli_help");
        assert!(!result.cached);
    }

    #[test]
    fn test_print_general_help_contains_all_categories() {
        let result = print_general_help().unwrap();

        // Verify all command categories are present
        let categories = vec![
            "data:",
            "layout:",
            "user:",
            "role:",
            "taxonomy:",
            "server:",
            "build:",
            "monitor:",
        ];

        for category in categories {
            assert!(
                result.data.contains(category),
                "Missing category: {}",
                category
            );
        }
    }

    #[test]
    fn test_print_general_help_contains_flags() {
        let result = print_general_help().unwrap();

        assert!(result.data.contains("Global Flags:"));
        assert!(result.data.contains("--help"));
        assert!(result.data.contains("--version"));
        assert!(result.data.contains("--verbose"));
        assert!(result.data.contains("--json"));
    }

    #[test]
    fn test_print_command_help_data_get() {
        let result = print_command_help("data", "get").unwrap();

        assert!(result.data.contains("data:get"));
        assert!(result.data.contains("Retrieve a value from ReedBase"));
        assert!(result.data.contains("Usage:"));
        assert!(result.data.contains("reed data:get <key>"));
        assert!(result.source == "cli_help");
    }

    #[test]
    fn test_print_command_help_data_set() {
        let result = print_command_help("data", "set").unwrap();

        assert!(result.data.contains("data:set"));
        assert!(result.data.contains("Store a value in ReedBase"));
        assert!(result.data.contains("reed data:set <key> <value>"));
    }

    #[test]
    fn test_print_command_help_layout_create() {
        let result = print_command_help("layout", "create").unwrap();

        assert!(result.data.contains("layout:create"));
        assert!(result.data.contains("Create a new layout"));
        assert!(result.data.contains("reed layout:create <name> <variant>"));
    }

    #[test]
    fn test_print_command_help_user_create() {
        let result = print_command_help("user", "create").unwrap();

        assert!(result.data.contains("user:create"));
        assert!(result.data.contains("Create a new user"));
        assert!(result.data.contains("reed user:create <username>"));
    }

    #[test]
    fn test_print_command_help_role_create() {
        let result = print_command_help("role", "create").unwrap();

        assert!(result.data.contains("role:create"));
        assert!(result.data.contains("Create a new role"));
        assert!(result.data.contains("reed role:create <rolename>"));
    }

    #[test]
    fn test_print_command_help_taxonomy_create() {
        let result = print_command_help("taxonomy", "create").unwrap();

        assert!(result.data.contains("taxonomy:create"));
        assert!(result.data.contains("Create a new taxonomy term"));
        assert!(result.data.contains("reed taxonomy:create <term>"));
    }

    #[test]
    fn test_print_command_help_server_start() {
        let result = print_command_help("server", "start").unwrap();

        assert!(result.data.contains("server:start"));
        assert!(result.data.contains("Start the ReedCMS server"));
        assert!(result.data.contains("reed server:start"));
    }

    #[test]
    fn test_print_command_help_unknown_command() {
        let result = print_command_help("unknown", "action").unwrap();

        assert!(result.data.contains("unknown:action"));
        assert!(result.data.contains("No detailed help available"));
        assert!(result.data.contains("reed --help"));
    }

    #[test]
    fn test_print_version() {
        let result = print_version().unwrap();

        assert!(result.data.contains("ReedCMS"));
        assert!(result.data.contains("Version:"));
        assert!(result.source == "cli_help");
        assert!(!result.cached);
    }

    #[test]
    fn test_print_version_contains_license() {
        let result = print_version().unwrap();

        assert!(result.data.contains("Licensed under the Apache License"));
    }

    #[test]
    fn test_print_command_help_contains_flags() {
        let result = print_command_help("data", "get").unwrap();

        assert!(result.data.contains("Flags:"));
        assert!(result.data.contains("--help"));
    }

    #[test]
    fn test_help_response_structure() {
        let result = print_general_help().unwrap();

        // Verify ReedResponse structure
        assert!(!result.data.is_empty());
        assert_eq!(result.source, "cli_help");
        assert!(!result.cached);
        assert!(result.timestamp > 0);
        assert!(result.metrics.is_none());
    }

    #[test]
    fn test_command_help_response_structure() {
        let result = print_command_help("data", "get").unwrap();

        assert!(!result.data.is_empty());
        assert_eq!(result.source, "cli_help");
        assert!(!result.cached);
        assert!(result.timestamp > 0);
        assert!(result.metrics.is_none());
    }

    #[test]
    fn test_version_response_structure() {
        let result = print_version().unwrap();

        assert!(!result.data.is_empty());
        assert_eq!(result.source, "cli_help");
        assert!(!result.cached);
        assert!(result.timestamp > 0);
        assert!(result.metrics.is_none());
    }
}
