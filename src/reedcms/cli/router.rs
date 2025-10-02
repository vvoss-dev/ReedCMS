// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! CLI command router.
//!
//! Routes parsed commands to appropriate handler functions.

use crate::reedcms::cli::parser::Command;
use crate::reedcms::reedstream::{
    current_timestamp, ReedError, ReedResponse, ReedResult, ResponseMetrics,
};
use std::collections::HashMap;

/// Command handler function type.
pub type CommandHandler =
    fn(&[String], &HashMap<String, String>) -> ReedResult<ReedResponse<String>>;

/// Command router with registered handlers.
pub struct Router {
    handlers: HashMap<(String, String), CommandHandler>,
}

impl Router {
    /// Creates a new empty router.
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
        }
    }

    /// Registers a command handler.
    ///
    /// ## Input
    /// - namespace: Command namespace (e.g., "set")
    /// - action: Command action (e.g., "text")
    /// - handler: Handler function
    ///
    /// ## Example Usage
    /// ```rust
    /// router.register("set", "text", set_text_handler);
    /// ```
    pub fn register(&mut self, namespace: &str, action: &str, handler: CommandHandler) {
        self.handlers
            .insert((namespace.to_string(), action.to_string()), handler);
    }

    /// Routes command to appropriate handler.
    ///
    /// ## Input
    /// - cmd: Parsed command
    ///
    /// ## Output
    /// - ReedResponse<String> with command output
    ///
    /// ## Performance
    /// - Routing: O(1) HashMap lookup
    /// - Target: < 0.1ms for routing overhead
    ///
    /// ## Error Conditions
    /// - Unknown command (no handler registered)
    /// - Handler execution errors
    ///
    /// ## Example Usage
    /// ```rust
    /// let output = router.route(cmd)?;
    /// ```
    pub fn route(&self, cmd: Command) -> ReedResult<ReedResponse<String>> {
        let start = std::time::Instant::now();

        // Check for --help or -h flag (intercept before routing)
        if cmd.get_flag_bool("help") || cmd.get_flag_bool("h") {
            return self.route_help(&cmd);
        }

        // Lookup handler
        let key = (cmd.namespace.clone(), cmd.action.clone());
        let handler = self
            .handlers
            .get(&key)
            .ok_or_else(|| ReedError::InvalidCommand {
                command: format!("{}:{}", cmd.namespace, cmd.action),
                reason: format!(
                    "Unknown command '{}:{}'. Use 'reed --help' for available commands.",
                    cmd.namespace, cmd.action
                ),
            })?;

        // Execute handler
        let mut response = handler(&cmd.args, &cmd.flags)?;

        // Add routing metrics
        let duration = start.elapsed();
        if let Some(ref mut metrics) = response.metrics {
            metrics.processing_time_us += duration.as_micros() as u64;
        } else {
            response.metrics = Some(ResponseMetrics {
                processing_time_us: duration.as_micros() as u64,
                memory_allocated: None,
                csv_files_accessed: 0,
                cache_info: None,
            });
        }

        Ok(response)
    }

    /// Routes --help flag to help system.
    fn route_help(&self, cmd: &Command) -> ReedResult<ReedResponse<String>> {
        use super::help::print_command_help;
        print_command_help(&cmd.namespace, &cmd.action)
    }

    /// Lists all registered commands.
    pub fn list_commands(&self) -> Vec<(String, String)> {
        self.handlers.keys().cloned().collect()
    }

    /// Checks if a command is registered.
    pub fn has_command(&self, namespace: &str, action: &str) -> bool {
        self.handlers
            .contains_key(&(namespace.to_string(), action.to_string()))
    }
}

impl Default for Router {
    fn default() -> Self {
        Self::new()
    }
}

/// Creates router with all command handlers registered.
///
/// ## Output
/// - Router with all handlers from REED-04-02 through REED-04-09
///
/// ## Performance
/// - Initialization: O(n) where n = number of commands
/// - One-time cost at startup
pub fn create_router() -> Router {
    let mut router = Router::new();

    // REED-04-02: Data commands
    use super::data_commands;

    // Set commands
    router.register("set", "text", data_commands::set_text);
    router.register("set", "route", data_commands::set_route);
    router.register("set", "meta", data_commands::set_meta);

    // Get commands
    router.register("get", "text", |args, _flags| data_commands::get_text(args));
    router.register("get", "route", |args, _flags| {
        data_commands::get_route(args)
    });
    router.register("get", "meta", |args, _flags| data_commands::get_meta(args));

    // List commands
    router.register("list", "text", |args, _flags| {
        data_commands::list_text(args)
    });
    router.register("list", "route", |args, _flags| {
        data_commands::list_route(args)
    });
    router.register("list", "meta", |args, _flags| {
        data_commands::list_meta(args)
    });

    // REED-04-03: Layout commands
    use super::layout_commands;
    router.register("init", "layout", layout_commands::init_layout);

    // REED-04-04: User commands
    use super::user_commands;
    router.register("user", "create", user_commands::create_user);
    router.register("user", "list", |_args, flags| {
        user_commands::list_users(flags)
    });
    router.register("user", "show", |args, _flags| {
        user_commands::show_user(args)
    });
    router.register("user", "update", user_commands::update_user);
    router.register("user", "delete", user_commands::delete_user);
    router.register("user", "passwd", user_commands::change_password);
    router.register("user", "roles", user_commands::manage_roles);

    // REED-04-05: Role commands
    // router.register("role", "create", role_commands::create_role);
    // router.register("role", "list", role_commands::list_roles);

    // REED-04-06: Taxonomy commands
    // router.register("taxonomy", "create", taxonomy_commands::create_term);
    // router.register("taxonomy", "list", taxonomy_commands::list_terms);

    // REED-04-07: Migration commands
    // router.register("migrate", "text", migration_commands::migrate_text);
    // router.register("validate", "routes", validation_commands::validate_routes);

    // REED-04-08: Build commands
    // router.register("build", "kernel", build_commands::build_kernel);
    // router.register("build", "complete", build_commands::build_complete);

    // REED-04-09: Server commands
    // router.register("server", "io", server_commands::server_io);
    // router.register("server", "start", server_commands::server_start);

    router
}
