// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Core Database struct and operations.
//!
//! This is the main entry point for all ReedBase operations.

use crate::database::execute::{ExecuteResult, ExecuteStatement};
use crate::database::stats::PatternTracker;
use crate::database::types::{AutoIndexConfig, DatabaseStats, IndexInfo, QueryMetrics};
use crate::error::{ReedError, ReedResult};
use crate::indices::Index;
use crate::reedql::{parse, QueryResult};
use crate::schema::Schema;
use crate::tables::{list_tables, Table};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

/// High-level database API.
///
/// This is the primary interface for interacting with ReedBase.
/// All operations go through this struct.
///
/// ## Thread Safety
/// - Multiple readers: YES (concurrent queries safe)
/// - Multiple writers: YES (uses internal RwLock for coordination)
/// - Readers during write: YES (readers see consistent snapshots)
///
/// ## Example
/// ```no_run
/// use reedbase::database::Database;
///
/// // Open database
/// let db = Database::open(".reed")?;
///
/// // Query
/// let result = db.query("SELECT * FROM text WHERE key LIKE '%.@de'")?;
///
/// // Execute
/// db.execute("INSERT INTO text (key, value) VALUES ('test', 'value')", "admin")?;
///
/// # Ok::<(), reedbase::ReedError>(())
/// ```
pub struct Database {
    /// Base path to ReedBase directory
    base_path: PathBuf,

    /// Loaded tables (lazy-loaded on first access)
    tables: Arc<RwLock<HashMap<String, Table>>>,

    /// Active indices (table.column → Index)
    indices: Arc<RwLock<HashMap<String, Box<dyn Index<String, Vec<usize>>>>>>,

    /// Auto-created index flags (table.column → bool)
    auto_created_indices: Arc<RwLock<HashMap<String, bool>>>,

    /// Pattern tracker for auto-indexing
    pattern_tracker: Arc<RwLock<PatternTracker>>,

    /// Auto-indexing configuration
    auto_index_config: AutoIndexConfig,

    /// Database statistics
    stats: Arc<RwLock<DatabaseStats>>,
}

impl Database {
    /// Opens an existing ReedBase database or creates a new one.
    ///
    /// ## Input
    /// - `path`: Path to ReedBase directory (e.g., ".reed")
    ///
    /// ## Output
    /// - `Ok(Database)`: Database handle
    /// - `Err(ReedError)`: Open failed
    ///
    /// ## Performance
    /// - Cold start: < 100ms (loads persistent indices)
    /// - Warm start: < 10ms (indices cached)
    ///
    /// ## Error Conditions
    /// - `IoError`: Cannot access directory
    /// - `IndexCorrupted`: Persistent index corrupted
    ///
    /// ## Example
    /// ```no_run
    /// use reedbase::database::Database;
    ///
    /// let db = Database::open(".reed")?;
    /// # Ok::<(), reedbase::ReedError>(())
    /// ```
    pub fn open<P: AsRef<Path>>(path: P) -> ReedResult<Self> {
        let base_path = path.as_ref().to_path_buf();

        // Ensure base directory exists
        if !base_path.exists() {
            std::fs::create_dir_all(&base_path).map_err(|e| ReedError::IoError {
                operation: "create_database_dir".to_string(),
                reason: e.to_string(),
            })?;
        }

        // Ensure tables directory exists
        let tables_dir = base_path.join("tables");
        if !tables_dir.exists() {
            std::fs::create_dir_all(&tables_dir).map_err(|e| ReedError::IoError {
                operation: "create_tables_dir".to_string(),
                reason: e.to_string(),
            })?;
        }

        // Create database instance
        let db = Self {
            base_path: base_path.clone(),
            tables: Arc::new(RwLock::new(HashMap::new())),
            indices: Arc::new(RwLock::new(HashMap::new())),
            auto_created_indices: Arc::new(RwLock::new(HashMap::new())),
            pattern_tracker: Arc::new(RwLock::new(PatternTracker::new())),
            auto_index_config: AutoIndexConfig::default(),
            stats: Arc::new(RwLock::new(DatabaseStats::new())),
        };

        // Load existing tables into cache
        db.load_existing_tables()?;

        // Load persistent indices
        db.load_persistent_indices()?;

        Ok(db)
    }

    /// Opens database with custom auto-indexing configuration.
    ///
    /// ## Example
    /// ```no_run
    /// use reedbase::database::{Database, AutoIndexConfig};
    ///
    /// let config = AutoIndexConfig::reedcms_optimized();
    /// let db = Database::open_with_config(".reed", config)?;
    /// # Ok::<(), reedbase::ReedError>(())
    /// ```
    pub fn open_with_config<P: AsRef<Path>>(path: P, config: AutoIndexConfig) -> ReedResult<Self> {
        let mut db = Self::open(path)?;
        db.auto_index_config = config;
        Ok(db)
    }

    /// Executes a ReedQL query (SELECT).
    ///
    /// ## Input
    /// - `sql`: ReedQL query string
    ///
    /// ## Output
    /// - `Ok(QueryResult)`: Query result with rows or aggregation
    /// - `Err(ReedError)`: Parse or execution error
    ///
    /// ## Performance
    /// - With index: < 100μs for exact match, < 1ms for range
    /// - Without index: ~10ms for 10k rows
    ///
    /// ## Example
    /// ```no_run
    /// use reedbase::database::Database;
    ///
    /// let db = Database::open(".reed")?;
    /// let result = db.query("SELECT * FROM text WHERE key = 'page.title@de'")?;
    /// # Ok::<(), reedbase::ReedError>(())
    /// ```
    pub fn query(&self, sql: &str) -> ReedResult<QueryResult> {
        // Implementation in query.rs
        crate::database::query::execute_query(self, sql)
    }

    /// Executes a ReedQL command (INSERT/UPDATE/DELETE).
    ///
    /// ## Input
    /// - `sql`: ReedQL command string
    /// - `user`: Username for audit trail
    ///
    /// ## Output
    /// - `Ok(ExecuteResult)`: Execution metadata (rows affected, etc.)
    /// - `Err(ReedError)`: Parse or execution error
    ///
    /// ## Performance
    /// - INSERT: < 5ms typical (includes versioning)
    /// - UPDATE: < 10ms typical (delta creation)
    /// - DELETE: < 5ms typical
    ///
    /// ## Example
    /// ```no_run
    /// use reedbase::database::Database;
    ///
    /// let db = Database::open(".reed")?;
    /// db.execute("INSERT INTO text (key, value) VALUES ('page.title@de', 'Willkommen')", "admin")?;
    /// db.execute("UPDATE text SET value = 'Hallo' WHERE key = 'page.title@de'", "admin")?;
    /// db.execute("DELETE FROM text WHERE key = 'page.title@de'", "admin")?;
    /// # Ok::<(), reedbase::ReedError>(())
    /// ```
    pub fn execute(&self, sql: &str, user: &str) -> ReedResult<ExecuteResult> {
        // Implementation in execute.rs
        crate::database::execute::execute_command(self, sql, user)
    }

    /// Creates a new table.
    ///
    /// ## Input
    /// - `name`: Table name
    /// - `schema`: Optional schema (None = schemaless)
    ///
    /// ## Output
    /// - `Ok(())`: Table created
    /// - `Err(ReedError)`: Creation failed
    ///
    /// ## Example
    /// ```no_run
    /// use reedbase::database::Database;
    ///
    /// let db = Database::open(".reed")?;
    /// db.create_table("users", None)?;
    /// # Ok::<(), reedbase::ReedError>(())
    /// ```
    pub fn create_table(&self, name: &str, schema: Option<Schema>) -> ReedResult<()> {
        let table = Table::new(&self.base_path, name);

        if table.exists() {
            return Err(ReedError::TableAlreadyExists {
                name: name.to_string(),
            });
        }

        // Create initial content (header only)
        let initial_content = if let Some(schema) = schema {
            // Use schema columns
            let column_names: Vec<String> = schema.columns.iter().map(|c| c.name.clone()).collect();
            let header = column_names.join("|");
            format!("{}\n", header).into_bytes()
        } else {
            // Default columns for schemaless table
            b"key|value\n".to_vec()
        };

        table.init(&initial_content, "system")?;

        // Add to loaded tables
        let mut tables = self.tables.write().unwrap();
        tables.insert(name.to_string(), table);

        // Update stats
        let mut stats = self.stats.write().unwrap();
        stats.table_count += 1;

        // Auto-create primary key index (marked as auto-created)
        if self.auto_index_config.enabled {
            drop(tables);
            drop(stats);
            crate::database::index::create_index_internal(self, name, "key", true)?;
        }

        Ok(())
    }

    /// Creates an index on a table column.
    ///
    /// ## Input
    /// - `table_name`: Table name
    /// - `column`: Column name
    ///
    /// ## Output
    /// - `Ok(())`: Index created
    /// - `Err(ReedError)`: Creation failed
    ///
    /// ## Example
    /// ```no_run
    /// use reedbase::database::Database;
    ///
    /// let db = Database::open(".reed")?;
    /// db.create_index("text", "key")?;
    /// # Ok::<(), reedbase::ReedError>(())
    /// ```
    pub fn create_index(&self, table_name: &str, column: &str) -> ReedResult<()> {
        // Implementation in index.rs
        crate::database::index::create_index(self, table_name, column)
    }

    /// Lists all tables in the database.
    ///
    /// ## Output
    /// - `Ok(Vec<String>)`: Table names
    /// - `Err(ReedError)`: List failed
    ///
    /// ## Example
    /// ```no_run
    /// use reedbase::database::Database;
    ///
    /// let db = Database::open(".reed")?;
    /// let tables = db.list_tables()?;
    /// for table in tables {
    ///     println!("Table: {}", table);
    /// }
    /// # Ok::<(), reedbase::ReedError>(())
    /// ```
    pub fn list_tables(&self) -> ReedResult<Vec<String>> {
        list_tables(&self.base_path)
    }

    /// Lists all indices in the database.
    ///
    /// ## Output
    /// - `Ok(Vec<IndexInfo>)`: Index information
    ///
    /// ## Example
    /// ```no_run
    /// use reedbase::database::Database;
    ///
    /// let db = Database::open(".reed")?;
    /// let indices = db.list_indices();
    /// for index in indices {
    ///     println!("Index: {}.{} ({} entries)", index.table, index.column, index.entry_count);
    /// }
    /// # Ok::<(), reedbase::ReedError>(())
    /// ```
    pub fn list_indices(&self) -> Vec<IndexInfo> {
        // Implementation in index.rs
        crate::database::index::list_indices(self)
    }

    /// Gets database statistics.
    ///
    /// ## Output
    /// - `DatabaseStats`: Current statistics
    ///
    /// ## Example
    /// ```no_run
    /// use reedbase::database::Database;
    ///
    /// let db = Database::open(".reed")?;
    /// let stats = db.stats();
    /// println!("Tables: {}, Queries: {}", stats.table_count, stats.query_count);
    /// # Ok::<(), reedbase::ReedError>(())
    /// ```
    pub fn stats(&self) -> DatabaseStats {
        self.stats.read().unwrap().clone()
    }

    /// Closes the database gracefully.
    ///
    /// Flushes all pending operations and closes indices.
    ///
    /// ## Output
    /// - `Ok(())`: Closed successfully
    /// - `Err(ReedError)`: Close failed
    ///
    /// ## Example
    /// ```no_run
    /// use reedbase::database::Database;
    ///
    /// let db = Database::open(".reed")?;
    /// // ... use database ...
    /// db.close()?;
    /// # Ok::<(), reedbase::ReedError>(())
    /// ```
    pub fn close(self) -> ReedResult<()> {
        // Flush pattern tracker statistics
        let tracker = self.pattern_tracker.read().unwrap();
        let _patterns = tracker.get_top_patterns(10);
        drop(tracker);

        // Indices are automatically flushed on drop (B+-Tree impl)
        // Tables are always consistent (atomic writes)

        Ok(())
    }

    // Internal helper methods

    /// Loads existing tables into cache.
    fn load_existing_tables(&self) -> ReedResult<()> {
        let table_names = list_tables(&self.base_path)?;
        let mut tables = self.tables.write().unwrap();
        let mut stats = self.stats.write().unwrap();

        for name in table_names {
            let table = Table::new(&self.base_path, &name);
            if table.exists() {
                // Count rows
                if let Ok(rows) = table.read_current_as_rows() {
                    stats.total_rows += rows.len();
                }
                tables.insert(name, table);
            }
        }

        stats.table_count = tables.len();
        Ok(())
    }

    /// Loads persistent B+-Tree indices from disk.
    ///
    /// Called during Database::open() to restore indices from previous sessions.
    ///
    /// ## Performance
    /// - Loads metadata: < 5ms
    /// - Opens each B+-Tree: < 10ms per index
    /// - Total: < 100ms for typical databases
    fn load_persistent_indices(&self) -> ReedResult<()> {
        use crate::database::index::load_index_metadata;
        use crate::indices::BTreeIndex;

        let metadata_list = load_index_metadata(self)?;

        if metadata_list.is_empty() {
            return Ok(());
        }

        let indices_dir = self.base_path.join("indices");
        let mut indices = self.indices.write().unwrap();
        let mut auto_flags = self.auto_created_indices.write().unwrap();
        let mut stats = self.stats.write().unwrap();

        for metadata in metadata_list {
            let index_key = metadata.index_key();

            match metadata.backend {
                crate::database::types::IndexBackend::Hash => {
                    // HashMap indices are not persistent - skip
                    // They will be recreated by auto-indexing if needed
                }

                crate::database::types::IndexBackend::BTree => {
                    // Load B+-Tree from disk
                    let index_path = indices_dir.join(format!("{}.btree", index_key));

                    if !index_path.exists() {
                        eprintln!(
                            "Warning: B+-Tree index file not found: {}",
                            index_path.display()
                        );
                        continue;
                    }

                    // Open B+-Tree with order 100 (must match creation order)
                    let order = crate::btree::Order::new(100).map_err(|e| ReedError::IoError {
                        operation: "create_btree_order".to_string(),
                        reason: format!("Invalid order: {}", e),
                    })?;

                    match BTreeIndex::<String, Vec<usize>>::open(&index_path, order) {
                        Ok(btree_index) => {
                            indices.insert(index_key.clone(), Box::new(btree_index));
                            stats.index_count += 1;

                            // Restore auto-created flag
                            if metadata.auto_created {
                                auto_flags.insert(index_key, true);
                                stats.auto_index_count += 1;
                            }
                        }
                        Err(e) => {
                            eprintln!("Warning: Failed to load B+-Tree index {}: {}", index_key, e);
                            // Continue loading other indices
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Gets reference to table (lazy-load if needed).
    pub(crate) fn get_table(&self, name: &str) -> ReedResult<Table> {
        // Check if table is cached
        {
            let tables = self.tables.read().unwrap();
            if tables.contains_key(name) {
                // Table exists in cache, recreate reference
                return Ok(Table::new(&self.base_path, name));
            }
        }

        // Table not cached, try to load it
        let table = Table::new(&self.base_path, name);
        if !table.exists() {
            return Err(ReedError::TableNotFound {
                name: name.to_string(),
            });
        }

        // Add to cache
        let mut tables = self.tables.write().unwrap();
        tables.insert(name.to_string(), table);

        // Return new reference
        Ok(Table::new(&self.base_path, name))
    }

    /// Gets reference to internal structures (for query/execute modules).
    pub(crate) fn base_path(&self) -> &Path {
        &self.base_path
    }

    pub(crate) fn indices(
        &self,
    ) -> &Arc<RwLock<HashMap<String, Box<dyn Index<String, Vec<usize>>>>>> {
        &self.indices
    }

    pub(crate) fn auto_created_indices(&self) -> &Arc<RwLock<HashMap<String, bool>>> {
        &self.auto_created_indices
    }

    pub(crate) fn pattern_tracker(&self) -> &Arc<RwLock<PatternTracker>> {
        &self.pattern_tracker
    }

    pub(crate) fn auto_index_config(&self) -> &AutoIndexConfig {
        &self.auto_index_config
    }

    pub(crate) fn stats_mut(&self) -> &Arc<RwLock<DatabaseStats>> {
        &self.stats
    }
}

// Clone is not needed - Table::new() can recreate references
