// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

#[cfg(test)]
mod tests {
    use crate::shell::*;

    #[test]
    fn test_is_exit_command_exit() {
        assert!(is_exit_command("exit"));
    }

    #[test]
    fn test_is_exit_command_quit() {
        assert!(is_exit_command("quit"));
    }

    #[test]
    fn test_is_exit_command_backslash_q() {
        assert!(is_exit_command("\\q"));
    }

    #[test]
    fn test_is_exit_command_not_exit() {
        assert!(!is_exit_command("query"));
    }

    #[test]
    fn test_is_exit_command_not_tables() {
        assert!(!is_exit_command("tables"));
    }

    #[test]
    fn test_is_exit_command_not_exiting() {
        assert!(!is_exit_command("exiting"));
    }

    #[test]
    fn test_is_exit_command_empty() {
        assert!(!is_exit_command(""));
    }

    #[test]
    fn test_build_editor_config() {
        let config = build_editor_config();
        // Config is opaque, just verify it builds without panic
        let _ = config;
    }

    #[test]
    fn test_is_exit_command_uppercase() {
        // Exit commands are case-sensitive (lowercase only)
        assert!(!is_exit_command("EXIT"));
        assert!(!is_exit_command("QUIT"));
    }

    #[test]
    fn test_is_exit_command_with_whitespace() {
        // Caller is responsible for trimming
        assert!(!is_exit_command(" exit"));
        assert!(!is_exit_command("exit "));
    }

    // Note: Full integration tests for run_shell() would require
    // mocking stdin/stdout, which is complex. Focus on unit tests
    // for individual functions. Integration testing can be done
    // manually or with expect-style tests.
}
