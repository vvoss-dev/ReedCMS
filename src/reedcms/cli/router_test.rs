// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

#[cfg(test)]
mod tests {
    use super::super::parser::Command;
    use super::super::router::*;
    use crate::reedcms::reedstream::{ReedError, ReedResponse, ReedResult};
    use std::collections::HashMap;

    // Test handler that returns success
    fn test_handler_success(
        _args: &[String],
        _flags: &HashMap<String, String>,
    ) -> ReedResult<ReedResponse<String>> {
        Ok(ReedResponse::new(
            "Test command executed".to_string(),
            "test_handler",
        ))
    }

    // Test handler that returns error
    fn test_handler_error(
        _args: &[String],
        _flags: &HashMap<String, String>,
    ) -> ReedResult<ReedResponse<String>> {
        Err(ReedError::ValidationError {
            field: "test".to_string(),
            value: "test_value".to_string(),
            constraint: "Test error".to_string(),
        })
    }

    #[test]
    fn test_router_register_and_route() {
        let mut router = Router::new();
        router.register("test", "action", test_handler_success);

        let cmd = Command {
            namespace: "test".to_string(),
            action: "action".to_string(),
            args: vec![],
            flags: HashMap::new(),
        };

        let result = router.route(cmd).unwrap();
        assert_eq!(result.data, "Test command executed");
        assert_eq!(result.source, "test_handler");
    }

    #[test]
    fn test_router_command_not_found() {
        let router = Router::new();

        let cmd = Command {
            namespace: "unknown".to_string(),
            action: "action".to_string(),
            args: vec![],
            flags: HashMap::new(),
        };

        let result = router.route(cmd);
        assert!(result.is_err());
        match result.unwrap_err() {
            ReedError::InvalidCommand { .. } => {}
            _ => panic!("Expected InvalidCommand"),
        }
    }

    #[test]
    fn test_router_help_interception() {
        let mut router = Router::new();
        router.register("test", "action", test_handler_success);

        let mut flags = HashMap::new();
        flags.insert("help".to_string(), "true".to_string());

        let cmd = Command {
            namespace: "test".to_string(),
            action: "action".to_string(),
            args: vec![],
            flags,
        };

        let result = router.route(cmd).unwrap();
        assert!(result.data.starts_with("test:action"));
        assert_eq!(result.source, "cli_help");
    }

    #[test]
    fn test_router_help_short_flag() {
        let mut router = Router::new();
        router.register("test", "action", test_handler_success);

        let mut flags = HashMap::new();
        flags.insert("h".to_string(), "true".to_string());

        let cmd = Command {
            namespace: "test".to_string(),
            action: "action".to_string(),
            args: vec![],
            flags,
        };

        let result = router.route(cmd).unwrap();
        assert!(result.data.starts_with("test:action"));
        assert_eq!(result.source, "cli_help");
    }

    #[test]
    fn test_router_multiple_handlers() {
        let mut router = Router::new();
        router.register("test", "action1", test_handler_success);
        router.register("test", "action2", test_handler_error);

        let cmd1 = Command {
            namespace: "test".to_string(),
            action: "action1".to_string(),
            args: vec![],
            flags: HashMap::new(),
        };

        let result1 = router.route(cmd1).unwrap();
        assert_eq!(result1.data, "Test command executed");

        let cmd2 = Command {
            namespace: "test".to_string(),
            action: "action2".to_string(),
            args: vec![],
            flags: HashMap::new(),
        };

        let result2 = router.route(cmd2);
        assert!(result2.is_err());
    }

    #[test]
    fn test_router_with_args() {
        let mut router = Router::new();
        router.register("test", "action", test_handler_success);

        let cmd = Command {
            namespace: "test".to_string(),
            action: "action".to_string(),
            args: vec!["arg1".to_string(), "arg2".to_string()],
            flags: HashMap::new(),
        };

        let result = router.route(cmd).unwrap();
        assert_eq!(result.data, "Test command executed");
    }

    #[test]
    fn test_router_with_flags() {
        let mut router = Router::new();
        router.register("test", "action", test_handler_success);

        let mut flags = HashMap::new();
        flags.insert("verbose".to_string(), "true".to_string());
        flags.insert("format".to_string(), "json".to_string());

        let cmd = Command {
            namespace: "test".to_string(),
            action: "action".to_string(),
            args: vec![],
            flags,
        };

        let result = router.route(cmd).unwrap();
        assert_eq!(result.data, "Test command executed");
    }

    #[test]
    fn test_create_router_structure() {
        let _router = create_router();

        // Router should be created without panic
        // All stub handlers are commented out, so no handlers registered yet
        // This test just verifies the router can be created
        assert!(true);
    }

    #[test]
    fn test_router_different_namespaces() {
        let mut router = Router::new();
        router.register("namespace1", "action", test_handler_success);
        router.register("namespace2", "action", test_handler_error);

        let cmd1 = Command {
            namespace: "namespace1".to_string(),
            action: "action".to_string(),
            args: vec![],
            flags: HashMap::new(),
        };

        let result1 = router.route(cmd1).unwrap();
        assert_eq!(result1.data, "Test command executed");

        let cmd2 = Command {
            namespace: "namespace2".to_string(),
            action: "action".to_string(),
            args: vec![],
            flags: HashMap::new(),
        };

        let result2 = router.route(cmd2);
        assert!(result2.is_err());
    }

    #[test]
    fn test_router_case_sensitive() {
        let mut router = Router::new();
        router.register("test", "action", test_handler_success);

        let cmd = Command {
            namespace: "Test".to_string(),
            action: "Action".to_string(),
            args: vec![],
            flags: HashMap::new(),
        };

        let result = router.route(cmd);
        assert!(result.is_err());
        match result.unwrap_err() {
            ReedError::InvalidCommand { .. } => {}
            _ => panic!("Expected InvalidCommand"),
        }
    }
}
