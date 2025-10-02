// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

#[cfg(test)]
mod tests {
    use super::super::parser::*;
    use crate::reedcms::reedstream::ReedError;

    #[test]
    fn test_parse_simple_command() {
        let args = vec!["data:get".to_string(), "key".to_string()];
        let result = parse_command(args).unwrap();

        assert_eq!(result.namespace, "data");
        assert_eq!(result.action, "get");
        assert_eq!(result.args, vec!["key"]);
        assert!(result.flags.is_empty());
    }

    #[test]
    fn test_parse_command_with_multiple_args() {
        let args = vec![
            "layout:create".to_string(),
            "homepage".to_string(),
            "mouse".to_string(),
        ];
        let result = parse_command(args).unwrap();

        assert_eq!(result.namespace, "layout");
        assert_eq!(result.action, "create");
        assert_eq!(result.args, vec!["homepage", "mouse"]);
        assert!(result.flags.is_empty());
    }

    #[test]
    fn test_parse_command_with_boolean_flags() {
        let args = vec![
            "data:list".to_string(),
            "--verbose".to_string(),
            "--json".to_string(),
        ];
        let result = parse_command(args).unwrap();

        assert_eq!(result.namespace, "data");
        assert_eq!(result.action, "list");
        assert!(result.args.is_empty());
        assert_eq!(result.flags.get("verbose"), Some(&"true".to_string()));
        assert_eq!(result.flags.get("json"), Some(&"true".to_string()));
    }

    #[test]
    fn test_parse_command_with_value_flags() {
        let args = vec![
            "user:create".to_string(),
            "alice".to_string(),
            "--email".to_string(),
            "alice@example.com".to_string(),
            "--role".to_string(),
            "admin".to_string(),
        ];
        let result = parse_command(args).unwrap();

        assert_eq!(result.namespace, "user");
        assert_eq!(result.action, "create");
        assert_eq!(result.args, vec!["alice"]);
        assert_eq!(
            result.flags.get("email"),
            Some(&"alice@example.com".to_string())
        );
        assert_eq!(result.flags.get("role"), Some(&"admin".to_string()));
    }

    #[test]
    fn test_parse_command_mixed_args_and_flags() {
        let args = vec![
            "role:create".to_string(),
            "editor".to_string(),
            "--permissions".to_string(),
            "text[rwx],route[rw-]".to_string(),
            "--desc".to_string(),
            "Content editor role".to_string(),
            "--verbose".to_string(),
        ];
        let result = parse_command(args).unwrap();

        assert_eq!(result.namespace, "role");
        assert_eq!(result.action, "create");
        assert_eq!(result.args, vec!["editor"]);
        assert_eq!(
            result.flags.get("permissions"),
            Some(&"text[rwx],route[rw-]".to_string())
        );
        assert_eq!(
            result.flags.get("desc"),
            Some(&"Content editor role".to_string())
        );
        assert_eq!(result.flags.get("verbose"), Some(&"true".to_string()));
    }

    #[test]
    fn test_parse_command_help_flag() {
        let args = vec!["data:get".to_string(), "--help".to_string()];
        let result = parse_command(args).unwrap();

        assert_eq!(result.namespace, "data");
        assert_eq!(result.action, "get");
        assert!(result.args.is_empty());
        assert_eq!(result.flags.get("help"), Some(&"true".to_string()));
    }

    #[test]
    fn test_parse_command_no_colon() {
        let args = vec!["invalidcommand".to_string()];
        let result = parse_command(args);

        assert!(result.is_err());
        match result.unwrap_err() {
            ReedError::InvalidCommand { .. } => {}
            _ => panic!("Expected InvalidCommand"),
        }
    }

    #[test]
    fn test_parse_command_empty_namespace() {
        let args = vec![":action".to_string()];
        let result = parse_command(args);

        assert!(result.is_err());
        match result.unwrap_err() {
            ReedError::InvalidCommand { .. } => {}
            _ => panic!("Expected InvalidCommand"),
        }
    }

    #[test]
    fn test_parse_command_empty_action() {
        let args = vec!["namespace:".to_string()];
        let result = parse_command(args);

        assert!(result.is_err());
        match result.unwrap_err() {
            ReedError::InvalidCommand { .. } => {}
            _ => panic!("Expected InvalidCommand"),
        }
    }

    #[test]
    fn test_parse_command_invalid_characters_namespace() {
        let args = vec!["data-test:get".to_string()];
        let result = parse_command(args).unwrap();

        // Hyphens are allowed in namespace
        assert_eq!(result.namespace, "data-test");
        assert_eq!(result.action, "get");
    }

    #[test]
    fn test_parse_command_invalid_characters_action() {
        let args = vec!["data:get-key".to_string()];
        let result = parse_command(args).unwrap();

        // Hyphens are allowed in action
        assert_eq!(result.namespace, "data");
        assert_eq!(result.action, "get-key");
    }

    #[test]
    fn test_parse_command_flag_missing_value() {
        let args = vec![
            "user:create".to_string(),
            "alice".to_string(),
            "--email".to_string(),
        ];
        let result = parse_command(args);

        assert!(result.is_err());
        match result.unwrap_err() {
            ReedError::InvalidCommand { .. } => {}
            _ => panic!("Expected InvalidCommand"),
        }
    }

    #[test]
    fn test_parse_command_no_args() {
        let args = vec![];
        let result = parse_command(args);

        assert!(result.is_err());
        match result.unwrap_err() {
            ReedError::InvalidCommand { .. } => {}
            _ => panic!("Expected InvalidCommand"),
        }
    }

    #[test]
    fn test_parse_command_multiple_colons() {
        let args = vec!["data:get:key".to_string()];
        let result = parse_command(args);

        assert!(result.is_err());
        match result.unwrap_err() {
            ReedError::InvalidCommand { .. } => {}
            _ => panic!("Expected InvalidCommand"),
        }
    }

    #[test]
    fn test_parse_command_underscores_allowed() {
        let args = vec!["data_store:get_key".to_string(), "mykey".to_string()];
        let result = parse_command(args).unwrap();

        assert_eq!(result.namespace, "data_store");
        assert_eq!(result.action, "get_key");
        assert_eq!(result.args, vec!["mykey"]);
    }

    #[test]
    fn test_parse_command_short_flags() {
        let args = vec!["data:list".to_string(), "-h".to_string(), "-v".to_string()];
        let result = parse_command(args).unwrap();

        assert_eq!(result.namespace, "data");
        assert_eq!(result.action, "list");
        assert_eq!(result.flags.get("h"), Some(&"true".to_string()));
        assert_eq!(result.flags.get("v"), Some(&"true".to_string()));
    }

    #[test]
    fn test_parse_command_all_boolean_flags() {
        let args = vec![
            "server:start".to_string(),
            "--verbose".to_string(),
            "--dry-run".to_string(),
            "--help".to_string(),
            "--confirm".to_string(),
            "--recursive".to_string(),
        ];
        let result = parse_command(args).unwrap();

        assert_eq!(result.namespace, "server");
        assert_eq!(result.action, "start");
        assert_eq!(result.flags.get("verbose"), Some(&"true".to_string()));
        assert_eq!(result.flags.get("dry-run"), Some(&"true".to_string()));
        assert_eq!(result.flags.get("help"), Some(&"true".to_string()));
        assert_eq!(result.flags.get("confirm"), Some(&"true".to_string()));
        assert_eq!(result.flags.get("recursive"), Some(&"true".to_string()));
    }
}
