// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Interactive shell (REPL) implementation.

use anyhow::{Context, Result};
use reedbase::Database;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use std::path::Path;

use crate::formatters;

pub fn run(path: &Path, user: &str) -> Result<()> {
    // Open database
    let db = Database::open(path)
        .with_context(|| format!("Failed to open database at {}", path.display()))?;

    println!("ReedBase Shell v0.1.0");
    println!("Database: {}", path.display());
    println!("User: {}", user);
    println!("Type .help for help, .exit to quit\n");

    let mut rl = DefaultEditor::new()?;
    let mut format = "table".to_string();

    loop {
        let readline = rl.readline("reedbase> ");

        match readline {
            Ok(line) => {
                let trimmed = line.trim();

                if trimmed.is_empty() {
                    continue;
                }

                // Add to history
                let _ = rl.add_history_entry(trimmed);

                // Handle dot-commands
                if trimmed.starts_with('.') {
                    match handle_dot_command(trimmed, &db, &mut format) {
                        Ok(should_exit) => {
                            if should_exit {
                                break;
                            }
                        }
                        Err(e) => eprintln!("Error: {}", e),
                    }
                    continue;
                }

                // Execute SQL
                if is_query(trimmed) {
                    // SELECT query
                    match db.query(trimmed) {
                        Ok(result) => {
                            let output = match format.as_str() {
                                "json" => formatters::format_json(&result),
                                "csv" => formatters::format_csv(&result, true),
                                _ => formatters::format_table(&result),
                            };
                            print!("{}", output);
                        }
                        Err(e) => eprintln!("Error: {}", e),
                    }
                } else {
                    // INSERT/UPDATE/DELETE command
                    match db.execute(trimmed, user) {
                        Ok(result) => {
                            println!(
                                "{} row{} affected ({:.2}ms)",
                                result.rows_affected,
                                if result.rows_affected == 1 { "" } else { "s" },
                                result.execution_time_us as f64 / 1000.0
                            );
                        }
                        Err(e) => eprintln!("Error: {}", e),
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("^C");
                continue;
            }
            Err(ReadlineError::Eof) => {
                println!("Goodbye!");
                break;
            }
            Err(err) => {
                eprintln!("Error: {:?}", err);
                break;
            }
        }
    }

    Ok(())
}

fn is_query(sql: &str) -> bool {
    sql.trim().to_uppercase().starts_with("SELECT")
}

fn handle_dot_command(cmd: &str, db: &Database, format: &mut String) -> Result<bool> {
    let parts: Vec<&str> = cmd.split_whitespace().collect();
    let command = parts[0];

    match command {
        ".exit" | ".quit" => {
            println!("Goodbye!");
            return Ok(true);
        }

        ".help" => {
            println!("Special commands:");
            println!("  .tables          List all tables");
            println!("  .indices         List all indices");
            println!("  .stats           Show database statistics");
            println!("  .explain <SQL>   Explain query execution plan");
            println!("  .format <FORMAT> Set output format (table|json|csv)");
            println!("  .clear           Clear screen");
            println!("  .help            Show this help");
            println!("  .exit            Exit shell");
        }

        ".tables" => match db.list_tables() {
            Ok(tables) => {
                println!("Tables:");
                for table in tables {
                    println!("  - {}", table);
                }
            }
            Err(e) => eprintln!("Error: {}", e),
        },

        ".indices" => {
            let indices = db.list_indices();
            println!("Indices:");
            for index in indices {
                println!(
                    "  - {}.{} ({})",
                    index.table, index.column, index.index_type
                );
            }
        }

        ".stats" => {
            let stats = db.stats();
            println!("Database Statistics:");
            println!("  Tables:        {}", stats.table_count);
            println!("  Total Rows:    {}", stats.total_rows);
            println!("  Indices:       {}", stats.index_count);
            println!("  Total Queries: {}", stats.query_count);
            println!(
                "  Avg Query:     {:.2}ms",
                stats.avg_query_time_us as f64 / 1000.0
            );
        }

        ".format" => {
            if parts.len() < 2 {
                println!("Current format: {}", format);
                println!("Usage: .format <table|json|csv>");
            } else {
                *format = parts[1].to_string();
                println!("Output format set to: {}", format);
            }
        }

        ".clear" => {
            print!("\x1B[2J\x1B[1;1H");
        }

        ".explain" => {
            if parts.len() < 2 {
                println!("Usage: .explain <SQL>");
            } else {
                let sql = parts[1..].join(" ");
                println!("Query explanation not yet implemented for: {}", sql);
                // TODO: Implement query explanation
            }
        }

        _ => {
            println!("Unknown command: {}", command);
            println!("Type .help for available commands");
        }
    }

    Ok(false)
}
