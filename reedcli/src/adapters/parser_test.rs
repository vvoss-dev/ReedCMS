// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Tests for adapter command parser.

#[cfg(test)]
mod tests {
    use crate::adapters::parser::*;
    use crate::types::*;
    use std::collections::HashMap;

    #[test]
    fn test_parse_adapter_command_with_namespace() {
        let parsed = parse_adapter_command("reedbase:query");
        assert_eq!(parsed.adapter, Some("reedbase".to_string()));
        assert_eq!(parsed.command, "query");
    }

    #[test]
    fn test_parse_adapter_command_without_namespace() {
        let parsed = parse_adapter_command("query");
        assert_eq!(parsed.adapter, None);
        assert_eq!(parsed.command, "query");
    }

    #[test]
    fn test_parse_adapter_command_multiple_colons() {
        let parsed = parse_adapter_command("reedcms:page:list");
        assert_eq!(parsed.adapter, Some("reedcms".to_string()));
        assert_eq!(parsed.command, "page:list");
    }

    #[test]
    fn test_expand_alias() {
        let mut aliases = HashMap::new();
        aliases.insert("q".to_string(), "query".to_string());

        let adapter = Adapter {
            name: "reedbase".to_string(),
            binary: std::path::PathBuf::from("reedbase"),
            description: "Test".to_string(),
            version_requirement: None,
            required: false,
            aliases,
            commands: vec![],
            validated: false,
        };

        assert_eq!(expand_alias("q", &adapter), "query");
        assert_eq!(expand_alias("query", &adapter), "query");
    }

    #[test]
    fn test_resolve_adapter_explicit() {
        let mut adapters = HashMap::new();
        adapters.insert(
            "reedbase".to_string(),
            Adapter {
                name: "reedbase".to_string(),
                binary: std::path::PathBuf::from("reedbase"),
                description: "".to_string(),
                version_requirement: None,
                required: false,
                aliases: HashMap::new(),
                commands: vec!["query".to_string()],
                validated: true,
            },
        );

        let registry = AdapterRegistry {
            adapters,
            cli_config: AdapterCliConfig {
                adapters: vec!["reedbase".to_string()],
                namespace_omission: true,
            },
            command_index: CommandIndex::new(),
        };

        let parsed = ParsedCommand {
            adapter: Some("reedbase".to_string()),
            command: "query".to_string(),
        };

        let result = resolve_adapter(&parsed, &registry).unwrap();
        assert_eq!(result, "reedbase");
    }

    #[test]
    fn test_resolve_adapter_not_found() {
        let registry = AdapterRegistry {
            adapters: HashMap::new(),
            cli_config: AdapterCliConfig {
                adapters: vec![],
                namespace_omission: true,
            },
            command_index: CommandIndex::new(),
        };

        let parsed = ParsedCommand {
            adapter: Some("nonexistent".to_string()),
            command: "query".to_string(),
        };

        let result = resolve_adapter(&parsed, &registry);
        assert!(matches!(result, Err(CliError::AdapterNotFound { .. })));
    }

    #[test]
    fn test_build_resolved_command() {
        let resolved = build_resolved_command(
            "reedbase".to_string(),
            "query".to_string(),
            vec!["arg1".to_string()],
        );

        assert_eq!(resolved.adapter, "reedbase");
        assert_eq!(resolved.command, "query");
        assert_eq!(resolved.args, vec!["arg1"]);
    }
}
