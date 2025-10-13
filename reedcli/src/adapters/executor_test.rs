// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Tests for adapter executor.

#[cfg(test)]
mod tests {
    use crate::adapters::executor::*;
    use crate::types::*;

    #[test]
    fn test_build_adapter_args() {
        let resolved = ResolvedCommand {
            adapter: "test".to_string(),
            command: "query".to_string(),
            args: vec!["arg1".to_string(), "arg2".to_string()],
        };

        let args = build_adapter_args(&resolved);
        assert_eq!(args, vec!["query", "arg1", "arg2"]);
    }

    #[test]
    fn test_handle_adapter_result_success() {
        let result = AdapterResult {
            exit_code: 0,
            stdout: "success output".to_string(),
            stderr: "".to_string(),
            duration_ms: 100,
        };

        let output = handle_adapter_result(result, "test").unwrap();
        assert_eq!(output, "success output");
    }

    #[test]
    fn test_handle_adapter_result_error() {
        let result = AdapterResult {
            exit_code: 1,
            stdout: "".to_string(),
            stderr: "error message".to_string(),
            duration_ms: 50,
        };

        let output = handle_adapter_result(result, "test");
        assert!(matches!(output, Err(CliError::AdapterError { .. })));
    }
}
