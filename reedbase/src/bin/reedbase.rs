// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! ReedBase CLI Tool (ReedQCommand)
//!
//! Command-line interface for ReedBase operations.

use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod commands;
mod formatters;

use commands::{exec, explain, indices, query, shell, stats, tables};

#[derive(Parser)]
#[command(name = "reedbase")]
#[command(author = "Vivian Voss <ask@vvoss.dev>")]
#[command(version = "0.1.0")]
#[command(about = "ReedBase CLI - SQL-like database operations", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Execute a SELECT query
    Query {
        /// ReedQL SELECT query (quoted)
        sql: String,

        /// Path to ReedBase directory (e.g., .reed)
        path: PathBuf,

        /// Output format: table|json|csv
        #[arg(short, long, default_value = "table")]
        format: String,

        /// Write output to file
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Omit header row (CSV only)
        #[arg(long)]
        no_header: bool,
    },

    /// Execute INSERT/UPDATE/DELETE command
    Exec {
        /// ReedQL command (quoted)
        sql: String,

        /// Path to ReedBase directory
        path: PathBuf,

        /// Username for audit trail
        #[arg(short, long)]
        user: Option<String>,

        /// Don't print affected rows
        #[arg(short, long)]
        quiet: bool,
    },

    /// Open interactive shell
    Shell {
        /// Path to ReedBase directory
        path: PathBuf,

        /// Default username for exec commands
        #[arg(short, long)]
        user: Option<String>,
    },

    /// List or manage tables
    Tables {
        /// Path to ReedBase directory
        path: PathBuf,

        /// Create new table
        #[arg(short, long)]
        create: Option<String>,

        /// Drop table (requires --confirm)
        #[arg(short, long)]
        drop: Option<String>,

        /// Confirm destructive operation
        #[arg(long)]
        confirm: bool,

        /// Show table statistics
        #[arg(short, long)]
        verbose: bool,
    },

    /// List or manage indices
    Indices {
        /// Path to ReedBase directory
        path: PathBuf,

        /// Create index on table.column
        #[arg(short, long)]
        create: Option<String>,

        /// Drop index
        #[arg(short, long)]
        drop: Option<String>,

        /// Rebuild index
        #[arg(short, long)]
        rebuild: Option<String>,

        /// Show index statistics
        #[arg(short, long)]
        verbose: bool,
    },

    /// Show database statistics
    Stats {
        /// Path to ReedBase directory
        path: PathBuf,

        /// Output format: table|json
        #[arg(short, long, default_value = "table")]
        format: String,
    },

    /// Explain query execution plan
    Explain {
        /// ReedQL query (quoted)
        sql: String,

        /// Path to ReedBase directory
        path: PathBuf,

        /// Show detailed plan
        #[arg(short, long)]
        verbose: bool,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Query {
            sql,
            path,
            format,
            output,
            no_header,
        } => query::execute(&sql, &path, &format, output.as_deref(), no_header)?,

        Commands::Exec {
            sql,
            path,
            user,
            quiet,
        } => {
            let username = user
                .unwrap_or_else(|| std::env::var("USER").unwrap_or_else(|_| "unknown".to_string()));
            exec::execute(&sql, &path, &username, quiet)?;
        }

        Commands::Shell { path, user } => {
            let username = user
                .unwrap_or_else(|| std::env::var("USER").unwrap_or_else(|_| "unknown".to_string()));
            shell::run(&path, &username)?;
        }

        Commands::Tables {
            path,
            create,
            drop,
            confirm,
            verbose,
        } => tables::execute(&path, create.as_deref(), drop.as_deref(), confirm, verbose)?,

        Commands::Indices {
            path,
            create,
            drop,
            rebuild,
            verbose,
        } => indices::execute(
            &path,
            create.as_deref(),
            drop.as_deref(),
            rebuild.as_deref(),
            verbose,
        )?,

        Commands::Stats { path, format } => stats::execute(&path, &format)?,

        Commands::Explain { sql, path, verbose } => explain::execute(&sql, &path, verbose)?,
    }

    Ok(())
}
