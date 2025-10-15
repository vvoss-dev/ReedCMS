# REED-19-12: CLI SQL-Like Query Interface (ReedQL)

**Status**: Not Started  
**Priority**: High  
**Estimated Effort**: 1.5 weeks  
**Layer**: ReedBase (Data Layer)  
**Dependencies**: REED-19-02, REED-19-08, REED-19-10, REED-19-11  

---

## Overview

This ticket implements ReedQL, a SQL-like query language for ReedBase accessible exclusively via CLI (no API exposure for security). Uses a **custom hand-written parser** optimised for ReedBase's key-value structure and CSV storage.

**Purpose**: Enable users to query ReedBase tables using familiar SQL-like syntax whilst maintaining security by restricting queries to CLI-only execution.

**Why Custom Parser?**
- **10x faster parsing**: < 10μs vs ~50-100μs for generic SQL parsers
- **ReedBase-optimised**: Direct HashMap access, key-pattern fast paths
- **Smaller binary**: +5KB vs +50KB for sqlparser-rs
- **KISS**: Only features ReedCMS needs, no bloat
- **Performance**: Optimisations for `key LIKE '%.@de'` patterns (90% of queries)

**Scope**:
- Custom hand-written ReedQL parser (SELECT, WHERE, ORDER BY, LIMIT, IN subqueries)
- Query execution engine with ReedBase-specific optimisations
- Aggregation functions (COUNT, SUM, AVG, MIN, MAX, DISTINCT)
- Filtering with operators (=, !=, <, >, <=, >=, LIKE, IN)
- Subquery support (recursive execution)
- Sorting and pagination
- CLI-only execution (no API exposure)
- Query validation and error reporting

---

## MANDATORY Development Standards

1. **Language**: All code comments and documentation in BBC English
2. **Principle**: KISS (Keep It Simple, Stupid)
3. **File Naming**: Each file has unique theme and clear responsibility
4. **Files**: One file = One responsibility (no multi-purpose files)
5. **Functions**: One function = One distinctive job (no Swiss Army knives)
6. **Testing**: Separate test files as `{name}.test.rs` (never inline `#[cfg(test)]`)
7. **Avoid**: Generic names like `handler.rs`, `middleware.rs`, `utils.rs`
8. **Templates**: Reference `service-template.md` and `service-template.test.md`

---

## Implementation Files

### 1. `src/reedbase/reedql/parser.rs`

**Purpose**: Parse ReedQL query strings into abstract syntax tree (AST).

**Functions**:

```rust
/// Parse ReedQL query string into AST.
///
/// ## Arguments
/// - query: ReedQL query string
///
/// ## Returns
/// - ParsedQuery with AST representation
///
/// ## Performance
/// - O(n) where n = query length
/// - < 1ms for typical queries
///
/// ## Error Conditions
/// - ReedError::QuerySyntaxError: Invalid syntax
/// - ReedError::UnsupportedOperation: Unsupported SQL feature
///
/// ## Example Usage
/// ```rust
/// let query = parse_query("SELECT * FROM users WHERE age > 18 ORDER BY name")?;
/// assert_eq!(query.table, "users");
/// assert_eq!(query.columns, vec!["*"]);
/// ```
pub fn parse_query(query: &str) -> ReedResult<ParsedQuery>

/// Parse SELECT clause.
///
/// ## Arguments
/// - input: SELECT clause string (e.g., "name, email" or "*")
///
/// ## Returns
/// - Vec of column names (empty vec for "*")
///
/// ## Performance
/// - O(n) where n = number of columns
/// - < 100μs
///
/// ## Error Conditions
/// - ReedError::QuerySyntaxError: Invalid column syntax
///
/// ## Example Usage
/// ```rust
/// let columns = parse_select_clause("name, email, age")?;
/// assert_eq!(columns, vec!["name", "email", "age"]);
/// ```
pub fn parse_select_clause(input: &str) -> ReedResult<Vec<String>>

/// Parse WHERE clause into filter conditions.
///
/// ## Arguments
/// - input: WHERE clause string (e.g., "age > 18 AND role='admin'")
///
/// ## Returns
/// - FilterCondition tree
///
/// ## Performance
/// - O(n) where n = condition complexity
/// - < 500μs for typical conditions
///
/// ## Error Conditions
/// - ReedError::QuerySyntaxError: Invalid condition syntax
/// - ReedError::UnsupportedOperation: Unsupported operator
///
/// ## Example Usage
/// ```rust
/// let condition = parse_where_clause("age > 18 AND role='admin'")?;
/// // Returns: And(Greater(age, 18), Equal(role, "admin"))
/// ```
pub fn parse_where_clause(input: &str) -> ReedResult<FilterCondition>

/// Parse ORDER BY clause.
///
/// ## Arguments
/// - input: ORDER BY clause string (e.g., "name ASC, age DESC")
///
/// ## Returns
/// - Vec of OrderBy specifications
///
/// ## Performance
/// - O(n) where n = number of sort columns
/// - < 100μs
///
/// ## Error Conditions
/// - ReedError::QuerySyntaxError: Invalid ORDER BY syntax
///
/// ## Example Usage
/// ```rust
/// let order = parse_order_by_clause("name ASC, age DESC")?;
/// assert_eq!(order[0].column, "name");
/// assert_eq!(order[0].direction, SortDirection::Ascending);
/// ```
pub fn parse_order_by_clause(input: &str) -> ReedResult<Vec<OrderBy>>

/// Parse LIMIT clause.
///
/// ## Arguments
/// - input: LIMIT clause string (e.g., "10" or "10 OFFSET 20")
///
/// ## Returns
/// - LimitOffset with limit and optional offset
///
/// ## Performance
/// - O(1) - simple parsing
/// - < 10μs
///
/// ## Error Conditions
/// - ReedError::QuerySyntaxError: Invalid number format
///
/// ## Example Usage
/// ```rust
/// let limit = parse_limit_clause("10 OFFSET 20")?;
/// assert_eq!(limit.limit, 10);
/// assert_eq!(limit.offset, Some(20));
/// ```
pub fn parse_limit_clause(input: &str) -> ReedResult<LimitOffset>

/// Parse aggregation function.
///
/// ## Arguments
/// - input: Function call string (e.g., "COUNT(*)" or "SUM(amount)")
///
/// ## Returns
/// - AggregationFunction with function type and column
///
/// ## Performance
/// - O(1) - simple parsing
/// - < 10μs
///
/// ## Error Conditions
/// - ReedError::QuerySyntaxError: Invalid function syntax
/// - ReedError::UnsupportedOperation: Unsupported function
///
/// ## Example Usage
/// ```rust
/// let agg = parse_aggregation("COUNT(*)")?;
/// assert_eq!(agg.function, AggregationType::Count);
/// assert_eq!(agg.column, None); // COUNT(*) has no specific column
/// ```
pub fn parse_aggregation(input: &str) -> ReedResult<AggregationFunction>
```

**Key Types**:

```rust
pub struct ParsedQuery {
    pub table: String,
    pub columns: Vec<String>,
    pub condition: Option<FilterCondition>,
    pub order_by: Vec<OrderBy>,
    pub limit: Option<LimitOffset>,
    pub aggregation: Option<AggregationFunction>,
}

pub enum FilterCondition {
    Equal(String, String),              // column = value
    NotEqual(String, String),           // column != value
    Greater(String, String),            // column > value
    GreaterEqual(String, String),       // column >= value
    Less(String, String),               // column < value
    LessEqual(String, String),          // column <= value
    Like(String, String),               // column LIKE pattern
    In(String, Vec<String>),            // column IN (value1, value2, ...)
    InSubquery(String, Box<ParsedQuery>), // column IN (SELECT ...)
    And(Box<FilterCondition>, Box<FilterCondition>),
    Or(Box<FilterCondition>, Box<FilterCondition>),
    Not(Box<FilterCondition>),
}

pub struct OrderBy {
    pub column: String,
    pub direction: SortDirection,
}

pub enum SortDirection {
    Ascending,
    Descending,
}

pub struct LimitOffset {
    pub limit: usize,
    pub offset: Option<usize>,
}

pub struct AggregationFunction {
    pub function: AggregationType,
    pub column: Option<String>,
}

pub enum AggregationType {
    Count,      // COUNT(*)
    Sum,        // SUM(column)
    Avg,        // AVG(column)
    Min,        // MIN(column)
    Max,        // MAX(column)
    GroupBy,    // GROUP BY column
}
```

---

### 2. `src/reedbase/reedql/executor.rs`

**Purpose**: Execute parsed queries against ReedBase tables.

**Functions**:

```rust
/// Execute parsed query and return results.
///
/// ## Arguments
/// - query: ParsedQuery from parser
/// - table: Table to query
///
/// ## Returns
/// - QueryResult with matching rows
///
/// ## Performance
/// - O(n) where n = total rows (with optimisations for indexed columns)
/// - < 10ms for 10,000 rows with simple filter
/// - < 50ms for 10,000 rows with complex conditions
///
/// ## Error Conditions
/// - ReedError::ColumnNotFound: Referenced column does not exist
/// - ReedError::InvalidDataType: Type mismatch in comparison
///
/// ## Example Usage
/// ```rust
/// let query = parse_query("SELECT name, email WHERE age > 18")?;
/// let table = Table::new(".reed/tables/users")?;
/// let results = execute_query(&query, &table)?;
/// for row in results.rows {
///     println!("{:?}", row);
/// }
/// ```
pub fn execute_query(query: &ParsedQuery, table: &Table) -> ReedResult<QueryResult>

/// Apply filter condition to row.
///
/// ## Arguments
/// - condition: FilterCondition to evaluate
/// - row: Row to test against condition
///
/// ## Returns
/// - true if row matches condition, false otherwise
///
/// ## Performance
/// - O(1) for simple conditions (=, !=, <, >)
/// - O(m) for LIKE (m = pattern length)
/// - O(k) for IN (k = number of values)
///
/// ## Error Conditions
/// - ReedError::ColumnNotFound: Referenced column missing from row
/// - ReedError::InvalidDataType: Cannot compare values
///
/// ## Example Usage
/// ```rust
/// let condition = FilterCondition::Greater("age".into(), "18".into());
/// let mut row = Row::new();
/// row.set("age", "25");
/// assert!(apply_filter(&condition, &row)?);
/// ```
pub fn apply_filter(condition: &FilterCondition, row: &Row) -> ReedResult<bool>

/// Sort rows according to ORDER BY specification.
///
/// ## Arguments
/// - rows: Rows to sort (modified in place)
/// - order_by: Sort specifications
///
/// ## Returns
/// - () (rows sorted in place)
///
/// ## Performance
/// - O(n log n) where n = number of rows
/// - < 5ms for 1,000 rows
/// - < 50ms for 10,000 rows
///
/// ## Error Conditions
/// - ReedError::ColumnNotFound: Sort column missing
///
/// ## Example Usage
/// ```rust
/// let mut rows = vec![/* ... */];
/// let order = vec![OrderBy { column: "name".into(), direction: Ascending }];
/// sort_rows(&mut rows, &order)?;
/// ```
pub fn sort_rows(rows: &mut Vec<Row>, order_by: &[OrderBy]) -> ReedResult<()>

/// Apply LIMIT and OFFSET to result set.
///
/// ## Arguments
/// - rows: Rows to paginate
/// - limit_offset: Pagination specification
///
/// ## Returns
/// - Vec of rows within limit/offset window
///
/// ## Performance
/// - O(n) where n = limit (copies subset)
/// - < 1ms for typical limits (< 1000)
///
/// ## Error Conditions
/// - None (offsets beyond row count return empty vec)
///
/// ## Example Usage
/// ```rust
/// let rows = vec![/* 100 rows */];
/// let limit = LimitOffset { limit: 10, offset: Some(20) };
/// let page = apply_limit_offset(rows, &limit);
/// assert_eq!(page.len(), 10); // Rows 20-29
/// ```
pub fn apply_limit_offset(rows: Vec<Row>, limit_offset: &LimitOffset) -> Vec<Row>

/// Execute aggregation function.
///
/// ## Arguments
/// - agg: AggregationFunction specification
/// - rows: Rows to aggregate
///
/// ## Returns
/// - AggregationResult with computed value
///
/// ## Performance
/// - O(n) where n = number of rows
/// - < 10ms for 10,000 rows
///
/// ## Error Conditions
/// - ReedError::ColumnNotFound: Aggregation column missing
/// - ReedError::InvalidDataType: Cannot aggregate non-numeric column
///
/// ## Example Usage
/// ```rust
/// let agg = AggregationFunction {
///     function: AggregationType::Count,
///     column: None,
/// };
/// let result = execute_aggregation(&agg, &rows)?;
/// println!("Count: {}", result.value);
/// ```
pub fn execute_aggregation(
    agg: &AggregationFunction,
    rows: &[Row],
) -> ReedResult<AggregationResult>

/// Select specific columns from rows.
///
/// ## Arguments
/// - rows: Input rows
/// - columns: Column names to select (empty for all)
///
/// ## Returns
/// - Vec of rows with only selected columns
///
/// ## Performance
/// - O(n × m) where n = rows, m = selected columns
/// - < 5ms for 1,000 rows with 10 columns
///
/// ## Error Conditions
/// - ReedError::ColumnNotFound: Selected column does not exist
///
/// ## Example Usage
/// ```rust
/// let rows = vec![/* rows with many columns */];
/// let selected = select_columns(rows, &["name", "email"])?;
/// // Result rows contain only name and email
/// ```
pub fn select_columns(rows: Vec<Row>, columns: &[String]) -> ReedResult<Vec<Row>>
```

**Key Types**:

```rust
pub struct QueryResult {
    pub rows: Vec<Row>,
    pub total_count: usize,      // Before LIMIT/OFFSET
    pub execution_time_us: u64,  // Query execution time
}

pub struct AggregationResult {
    pub function: AggregationType,
    pub column: Option<String>,
    pub value: String,           // Result as string
}
```

---

### 3. `src/reedbase/reedql/validator.rs`

**Purpose**: Validate queries before execution.

**Functions**:

```rust
/// Validate parsed query against table schema.
///
/// ## Arguments
/// - query: ParsedQuery to validate
/// - table: Table with optional schema
///
/// ## Returns
/// - ValidationResult with warnings and errors
///
/// ## Performance
/// - O(n) where n = number of referenced columns
/// - < 1ms
///
/// ## Error Conditions
/// - None (returns validation errors in result)
///
/// ## Example Usage
/// ```rust
/// let query = parse_query("SELECT * WHERE unknown_column = 'value'")?;
/// let validation = validate_query(&query, &table)?;
/// if !validation.is_valid() {
///     for error in &validation.errors {
///         eprintln!("Error: {}", error);
///     }
/// }
/// ```
pub fn validate_query(query: &ParsedQuery, table: &Table) -> ReedResult<ValidationResult>

/// Validate column references in query.
///
/// ## Arguments
/// - columns: Column names referenced in query
/// - table: Table to check against
///
/// ## Returns
/// - Vec of missing column names
///
/// ## Performance
/// - O(n × m) where n = query columns, m = table columns
/// - < 100μs for typical queries
///
/// ## Error Conditions
/// - None (returns missing columns)
///
/// ## Example Usage
/// ```rust
/// let missing = validate_columns(&["name", "age", "unknown"], &table)?;
/// if !missing.is_empty() {
///     eprintln!("Unknown columns: {:?}", missing);
/// }
/// ```
pub fn validate_columns(columns: &[String], table: &Table) -> ReedResult<Vec<String>>

/// Validate data types in comparisons.
///
/// ## Arguments
/// - condition: FilterCondition to validate
/// - table: Table with optional schema
///
/// ## Returns
/// - Vec of type mismatch warnings
///
/// ## Performance
/// - O(n) where n = condition complexity
/// - < 100μs
///
/// ## Error Conditions
/// - None (returns warnings)
///
/// ## Example Usage
/// ```rust
/// let condition = FilterCondition::Greater("age".into(), "not_a_number".into());
/// let warnings = validate_data_types(&condition, &table)?;
/// if !warnings.is_empty() {
///     for warning in warnings {
///         eprintln!("Warning: {}", warning);
///     }
/// }
/// ```
pub fn validate_data_types(
    condition: &FilterCondition,
    table: &Table,
) -> ReedResult<Vec<String>>

/// Validate aggregation function usage.
///
/// ## Arguments
/// - agg: AggregationFunction to validate
/// - table: Table with optional schema
///
/// ## Returns
/// - ValidationResult for aggregation
///
/// ## Performance
/// - O(1) - simple checks
/// - < 10μs
///
/// ## Error Conditions
/// - None (returns validation errors)
///
/// ## Example Usage
/// ```rust
/// let agg = AggregationFunction {
///     function: AggregationType::Sum,
///     column: Some("email".into()), // Invalid: email is not numeric
/// };
/// let result = validate_aggregation(&agg, &table)?;
/// assert!(!result.is_valid());
/// ```
pub fn validate_aggregation(
    agg: &AggregationFunction,
    table: &Table,
) -> ReedResult<ValidationResult>
```

**Key Types**:

```rust
pub struct ValidationResult {
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl ValidationResult {
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }
}
```

---

### 4. `src/reedbase/reedql/optimiser.rs`

**Purpose**: Optimise query execution plans.

**Functions**:

```rust
/// Optimise query execution plan.
///
/// ## Arguments
/// - query: ParsedQuery to optimise
/// - table: Table with statistics
///
/// ## Returns
/// - OptimisedQuery with execution plan
///
/// ## Performance
/// - O(1) - simple heuristics
/// - < 100μs
///
/// ## Error Conditions
/// - None
///
/// ## Example Usage
/// ```rust
/// let query = parse_query("SELECT * WHERE status='active' AND age > 18")?;
/// let optimised = optimise_query(&query, &table)?;
/// // May reorder conditions for faster filtering
/// ```
pub fn optimise_query(query: &ParsedQuery, table: &Table) -> ReedResult<OptimisedQuery>

/// Estimate query cost.
///
/// ## Arguments
/// - query: ParsedQuery to estimate
/// - table: Table with row count
///
/// ## Returns
/// - EstimatedCost with time and memory estimates
///
/// ## Performance
/// - O(1) - heuristic calculation
/// - < 10μs
///
/// ## Error Conditions
/// - None
///
/// ## Example Usage
/// ```rust
/// let cost = estimate_query_cost(&query, &table)?;
/// if cost.estimated_time_ms > 1000 {
///     eprintln!("Warning: Query may take over 1 second");
/// }
/// ```
pub fn estimate_query_cost(query: &ParsedQuery, table: &Table) -> ReedResult<EstimatedCost>

/// Suggest query improvements.
///
/// ## Arguments
/// - query: ParsedQuery to analyse
/// - table: Table with schema and statistics
///
/// ## Returns
/// - Vec of QuerySuggestion with optimisation hints
///
/// ## Performance
/// - O(1) - simple heuristics
/// - < 100μs
///
/// ## Error Conditions
/// - None
///
/// ## Example Usage
/// ```rust
/// let suggestions = suggest_query_improvements(&query, &table)?;
/// for suggestion in suggestions {
///     println!("Tip: {}", suggestion.message);
/// }
/// ```
pub fn suggest_query_improvements(
    query: &ParsedQuery,
    table: &Table,
) -> ReedResult<Vec<QuerySuggestion>>
```

**Key Types**:

```rust
pub struct OptimisedQuery {
    pub original: ParsedQuery,
    pub execution_plan: ExecutionPlan,
}

pub struct ExecutionPlan {
    pub steps: Vec<ExecutionStep>,
    pub estimated_cost: EstimatedCost,
}

pub enum ExecutionStep {
    ScanTable,
    ApplyFilter(FilterCondition),
    Sort(Vec<OrderBy>),
    SelectColumns(Vec<String>),
    Aggregate(AggregationFunction),
    LimitOffset(LimitOffset),
}

pub struct EstimatedCost {
    pub estimated_time_ms: u64,
    pub estimated_memory_mb: u64,
    pub rows_scanned: usize,
}

pub struct QuerySuggestion {
    pub severity: SuggestionSeverity,
    pub message: String,
}

pub enum SuggestionSeverity {
    Info,
    Warning,
    Error,
}
```

---

### 5. `src/reedbase/reedql/formatter.rs`

**Purpose**: Format query results for CLI output.

**Functions**:

```rust
/// Format query results as ASCII table.
///
/// ## Arguments
/// - results: QueryResult to format
///
/// ## Returns
/// - Formatted string for CLI display
///
/// ## Performance
/// - O(n × m) where n = rows, m = columns
/// - < 10ms for 1,000 rows
///
/// ## Error Conditions
/// - None
///
/// ## Example Usage
/// ```rust
/// let results = execute_query(&query, &table)?;
/// let output = format_table(&results)?;
/// println!("{}", output);
/// ```
pub fn format_table(results: &QueryResult) -> ReedResult<String>

/// Format query results as JSON.
///
/// ## Arguments
/// - results: QueryResult to format
///
/// ## Returns
/// - JSON string
///
/// ## Performance
/// - O(n × m) where n = rows, m = columns
/// - < 20ms for 1,000 rows
///
/// ## Error Conditions
/// - ReedError::SerializationError: Cannot serialise to JSON
///
/// ## Example Usage
/// ```rust
/// let json = format_json(&results)?;
/// println!("{}", json);
/// ```
pub fn format_json(results: &QueryResult) -> ReedResult<String>

/// Format query results as CSV.
///
/// ## Arguments
/// - results: QueryResult to format
///
/// ## Returns
/// - CSV string with pipe delimiter
///
/// ## Performance
/// - O(n × m) where n = rows, m = columns
/// - < 5ms for 1,000 rows
///
/// ## Error Conditions
/// - None
///
/// ## Example Usage
/// ```rust
/// let csv = format_csv(&results)?;
/// println!("{}", csv);
/// ```
pub fn format_csv(results: &QueryResult) -> ReedResult<String>

/// Format aggregation result.
///
/// ## Arguments
/// - result: AggregationResult to format
///
/// ## Returns
/// - Formatted string
///
/// ## Performance
/// - O(1) - simple formatting
/// - < 10μs
///
/// ## Error Conditions
/// - None
///
/// ## Example Usage
/// ```rust
/// let agg_result = execute_aggregation(&agg, &rows)?;
/// let output = format_aggregation(&agg_result)?;
/// println!("{}", output); // "COUNT(*): 1,234"
/// ```
pub fn format_aggregation(result: &AggregationResult) -> ReedResult<String>
```

---

### 6. `src/reedbase/reedql/mod.rs`

**Purpose**: Public API for ReedQL system.

**Functions**:

```rust
/// Execute ReedQL query string and return formatted results.
///
/// ## Arguments
/// - query_string: ReedQL query
/// - table: Table to query
/// - format: Output format (table, json, csv)
///
/// ## Returns
/// - Formatted query results as string
///
/// ## Performance
/// - Depends on query complexity and data size
/// - < 10ms for simple queries on 10k rows
/// - < 100ms for complex queries with aggregations
///
/// ## Error Conditions
/// - ReedError::QuerySyntaxError: Invalid syntax
/// - ReedError::ColumnNotFound: Referenced column missing
/// - ReedError::InvalidDataType: Type mismatch
///
/// ## Example Usage
/// ```rust
/// let table = Table::new(".reed/tables/users")?;
/// let output = execute_reedql(
///     "SELECT name, email WHERE age > 18 ORDER BY name LIMIT 10",
///     &table,
///     OutputFormat::Table
/// )?;
/// println!("{}", output);
/// ```
pub fn execute_reedql(
    query_string: &str,
    table: &Table,
    format: OutputFormat,
) -> ReedResult<String>
```

**Key Types**:

```rust
pub enum OutputFormat {
    Table,   // ASCII table
    Json,    // JSON array
    Csv,     // Pipe-delimited CSV
}
```

---

## Subquery Support (Recursive Execution)

### Implementation

Subqueries are implemented as **recursive calls** to the same query executor:

```rust
/// Execute query with subquery support.
///
/// ## Implementation
/// - Subqueries are detected in WHERE clause (IN operator with SELECT)
/// - Inner query executed first (recursive call!)
/// - Result used as IN filter for outer query
/// - No depth limit (trust CLI user)
///
/// ## Performance
/// - Inner query: O(n) where n = inner table rows
/// - Outer query: O(m) where m = outer table rows
/// - Total: O(n + m) for simple subquery
///
/// ## Example
/// ```rust
/// // Parse: SELECT key FROM text WHERE key IN (SELECT key FROM routes)
/// let outer = ParsedQuery { ... };
/// let inner = ParsedQuery { ... };
///
/// // Execute inner first
/// let inner_results = execute_query(&inner, &routes_table)?; // Recursive!
/// let keys: Vec<String> = inner_results.rows.iter()
///     .map(|r| r.get("key"))
///     .collect();
///
/// // Execute outer with IN filter
/// let condition = FilterCondition::In("key".into(), keys);
/// let outer_results = execute_with_filter(&outer, condition)?;
/// ```
pub fn execute_query(query: &ParsedQuery, table: &Table) -> ReedResult<QueryResult> {
    // Check for subqueries in WHERE clause
    if let Some(FilterCondition::InSubquery(column, subquery)) = &query.condition {
        // Execute subquery first (RECURSIVE!)
        let inner_table = Table::new(&subquery.table)?;
        let inner_results = execute_query(subquery, &inner_table)?;
        
        // Extract values from inner result
        let values: Vec<String> = inner_results.rows.iter()
            .map(|r| r.get(column).unwrap_or_default())
            .collect();
        
        // Replace subquery with IN list
        let mut query_copy = query.clone();
        query_copy.condition = Some(FilterCondition::In(column.clone(), values));
        
        // Execute modified query
        return execute_simple(&query_copy, table);
    }
    
    execute_simple(query, table)
}
```

### Supported Subquery Patterns

```sql
-- IN with subquery (most common)
SELECT key FROM text 
WHERE key IN (SELECT key FROM routes WHERE path LIKE '/blog/%')

-- Multiple subqueries with AND/OR
SELECT * FROM text 
WHERE key IN (SELECT key FROM routes) 
  AND value IN (SELECT value FROM meta WHERE cache_ttl > 3600)

-- Nested subqueries (recursive)
SELECT * FROM text 
WHERE key IN (
    SELECT key FROM routes 
    WHERE target IN (
        SELECT target FROM meta WHERE cache_ttl > 3600
    )
)
```

---

## ReedBase-Specific Optimisations

### 1. Key-Pattern Fast Paths

**90% of ReedCMS queries use key patterns**:

```rust
/// Optimise key LIKE patterns for direct HashMap filtering.
///
/// ## Optimised Patterns
/// - `key LIKE '%.@de'` → Language suffix filter (direct)
/// - `key LIKE 'page.%'` → Namespace prefix filter (direct)
/// - `key LIKE 'page.%.title@de'` → Namespace + language filter (direct)
/// - `key LIKE '%'` → No filter (return all)
///
/// ## Performance
/// - Optimised: < 1ms for 10k rows
/// - Generic: ~10ms for 10k rows
/// - **10x faster for common patterns**
pub fn optimise_key_pattern(pattern: &str) -> OptimisedPattern {
    if pattern.ends_with("@de") || pattern.ends_with("@en") {
        // Language filter
        let lang = &pattern[pattern.len()-2..];
        return OptimisedPattern::LanguageFilter { lang: lang.to_string() };
    }
    
    if pattern.starts_with("page.") || pattern.starts_with("blog.") {
        // Namespace filter
        let ns = pattern.split('.').next().unwrap();
        return OptimisedPattern::NamespaceFilter { namespace: ns.to_string() };
    }
    
    // Generic pattern (fallback to regex)
    OptimisedPattern::Regex(pattern_to_regex(pattern))
}

pub enum OptimisedPattern {
    LanguageFilter { lang: String },        // Direct string.ends_with()
    NamespaceFilter { namespace: String },  // Direct string.starts_with()
    Combined { namespace: String, lang: String }, // Both checks
    Regex(Regex),                           // Fallback
}
```

### 2. Direct HashMap Access

```rust
/// Execute query with direct HashMap access (no AST conversion).
///
/// ## Fast Path
/// - WHERE key LIKE pattern → Direct HashMap iteration
/// - No intermediate allocations
/// - < 1ms for 10k rows
///
/// ## Slow Path
/// - Complex conditions → Generic filter
/// - ~10ms for 10k rows
pub fn execute_simple(query: &ParsedQuery, table: &Table) -> ReedResult<QueryResult> {
    let cache = REEDBASE_CACHE.read()?;
    let table_data = cache.get(&query.table)?;
    
    // Fast path: Simple key LIKE pattern
    if let Some(FilterCondition::Like(col, pattern)) = &query.condition {
        if col == "key" {
            let optimised = optimise_key_pattern(pattern);
            let rows: Vec<Row> = table_data.iter()
                .filter(|(key, _)| matches_optimised_pattern(key, &optimised))
                .map(|(key, value)| Row::from_kv(key, value))
                .collect();
            
            return Ok(QueryResult { rows, .. });
        }
    }
    
    // Slow path: Generic filter
    execute_generic(query, table_data)
}
```

### 3. ReedCMS-Specific Functions

```sql
-- Custom functions for common ReedCMS patterns
SELECT * FROM text WHERE key.language = 'de'
-- Equivalent to: key LIKE '%@de'

SELECT * FROM text WHERE key.namespace = 'page'
-- Equivalent to: key LIKE 'page.%'

SELECT * FROM text WHERE key.missing_translation('en', 'de')
-- Finds keys with @en but without @de
```

---

## ReedQL Syntax Specification

### Complete Syntax

```sql
SELECT <columns>
FROM <table>
[WHERE <condition>]
[ORDER BY <column> [ASC|DESC] [, ...]]
[LIMIT <n> [OFFSET <m>]]
```

**Note**: FROM clause is optional when table specified as CLI argument.

### Supported Features

**SELECT Clause**:
- `SELECT *` - All columns
- `SELECT column1, column2, ...` - Specific columns
- `SELECT COUNT(*)` - Count rows
- `SELECT SUM(column)` - Sum numeric column
- `SELECT AVG(column)` - Average numeric column
- `SELECT MIN(column)` - Minimum value
- `SELECT MAX(column)` - Maximum value

**WHERE Clause Operators**:
- `=` - Equal
- `!=` or `<>` - Not equal
- `<` - Less than
- `<=` - Less than or equal
- `>` - Greater than
- `>=` - Greater than or equal
- `LIKE` - Pattern matching (% and _ wildcards)
- `IN (value1, value2, ...)` - Value in list
- `AND` - Logical AND
- `OR` - Logical OR
- `NOT` - Logical NOT

**ORDER BY Clause**:
- `ORDER BY column` - Sort ascending (default)
- `ORDER BY column ASC` - Sort ascending
- `ORDER BY column DESC` - Sort descending
- `ORDER BY col1 ASC, col2 DESC` - Multiple columns

**LIMIT Clause**:
- `LIMIT n` - Return first n rows
- `LIMIT n OFFSET m` - Return n rows starting from row m

### Query Examples

```sql
-- Simple select
SELECT * FROM users

-- Specific columns
SELECT name, email FROM users

-- Filter rows
SELECT * FROM users WHERE age > 18

-- Complex conditions
SELECT * FROM users WHERE age > 18 AND role = 'admin'

-- Pattern matching
SELECT * FROM users WHERE email LIKE '%@example.com'

-- IN operator
SELECT * FROM posts WHERE status IN ('published', 'featured')

-- Sorting
SELECT * FROM users ORDER BY created_at DESC

-- Pagination
SELECT * FROM users ORDER BY name LIMIT 10 OFFSET 20

-- Aggregation
SELECT COUNT(*) FROM users

SELECT SUM(amount) FROM transactions WHERE user_id = '123'

SELECT AVG(rating) FROM reviews

-- Combined features
SELECT name, email 
FROM users 
WHERE active = true AND role != 'guest'
ORDER BY last_login DESC
LIMIT 50

-- Subquery with IN (recursive execution)
SELECT key, value FROM text 
WHERE key IN (SELECT key FROM routes WHERE path LIKE '/blog/%')

-- Multiple subqueries
SELECT * FROM text 
WHERE key IN (SELECT key FROM routes) 
  AND value IN (SELECT value FROM meta WHERE cache_ttl > 3600)

-- Nested subqueries (recursive)
SELECT * FROM text 
WHERE key IN (
    SELECT key FROM routes 
    WHERE target IN (
        SELECT target FROM meta WHERE cache_ttl > 3600
    )
)

-- ReedBase-specific optimisations (fast paths)
SELECT * FROM text WHERE key LIKE '%.title@de'     -- Language filter
SELECT * FROM text WHERE key LIKE 'page.%'         -- Namespace filter
SELECT * FROM text WHERE key LIKE 'page.%.@de'     -- Combined filter
```

---

## CLI Commands

### `reed query`
**Purpose**: Execute ReedQL query.

```bash
# Basic query
reed query users "SELECT * FROM users"

# With WHERE clause
reed query users "SELECT * WHERE age > 18"

# Note: FROM clause is optional (table inferred from first argument)
reed query users "SELECT name, email WHERE role='admin'"

# Output formats
reed query users "SELECT *" --format json
reed query users "SELECT *" --format csv
reed query users "SELECT *" --format table  # default

# Query with statistics
reed query users "SELECT *" --explain

# Output:
# Query: SELECT * FROM users
# Rows scanned: 1,234
# Rows returned: 876
# Execution time: 8.4ms
#
# name  | email             | age | role
# ------|-------------------|-----|-------
# Alice | alice@example.com | 25  | admin
# ...
```

### `reed query:explain`
**Purpose**: Show query execution plan without executing.

```bash
reed query:explain users "SELECT * WHERE age > 18 AND role='admin'"

# Output:
# Execution Plan
# ==============
# 1. Scan table: users (1,234 rows)
# 2. Apply filter: age > 18 (estimated: 456 rows)
# 3. Apply filter: role='admin' (estimated: 45 rows)
#
# Estimated cost:
#   Time: ~8ms
#   Memory: ~2MB
#   Rows scanned: 1,234
#   Rows returned: ~45
#
# Suggestions:
#   ✓ Good: Filters reduce result set significantly
```

### `reed query:validate`
**Purpose**: Validate query syntax without executing.

```bash
reed query:validate users "SELECT * WHERE unknown_column = 'value'"

# Output:
# Validation Failed
# =================
# Errors:
#   - Column 'unknown_column' does not exist in table 'users'
#
# Available columns: id, name, email, age, role, created_at
```

---

## Test Files

### `src/reedbase/reedql/parser.test.rs`

**Test Coverage**:

```rust
#[test]
fn test_parse_simple_select()
// Verify: "SELECT * FROM users" parses correctly

#[test]
fn test_parse_select_columns()
// Verify: "SELECT name, email" parses column list

#[test]
fn test_parse_where_clause()
// Verify: WHERE conditions parse correctly

#[test]
fn test_parse_complex_conditions()
// Verify: AND, OR, NOT combinations work

#[test]
fn test_parse_order_by()
// Verify: ORDER BY with ASC/DESC

#[test]
fn test_parse_limit_offset()
// Verify: LIMIT and OFFSET parsing

#[test]
fn test_parse_aggregation()
// Verify: COUNT, SUM, AVG, MIN, MAX parse

#[test]
fn test_syntax_errors()
// Verify: Invalid syntax produces clear errors
```

### `src/reedbase/reedql/executor.test.rs`

**Test Coverage**:

```rust
#[test]
fn test_execute_simple_query()
// Verify: SELECT * returns all rows

#[test]
fn test_filter_equal()
// Verify: WHERE column=value works

#[test]
fn test_filter_comparison_operators()
// Verify: <, >, <=, >= work correctly

#[test]
fn test_filter_like_pattern()
// Verify: LIKE with % and _ wildcards

#[test]
fn test_filter_in_operator()
// Verify: IN (value1, value2) works

#[test]
fn test_logical_operators()
// Verify: AND, OR, NOT combinations

#[test]
fn test_sort_ascending_descending()
// Verify: ORDER BY ASC and DESC

#[test]
fn test_limit_offset()
// Verify: Pagination works correctly

#[test]
fn test_aggregation_count()
// Verify: COUNT(*) returns correct count

#[test]
fn test_aggregation_sum_avg()
// Verify: SUM and AVG on numeric columns

#[test]
fn test_performance_large_dataset()
// Verify: 10,000 rows < 10ms simple query
```

### `src/reedbase/reedql/validator.test.rs`

**Test Coverage**:

```rust
#[test]
fn test_validate_unknown_column()
// Verify: Detects missing columns

#[test]
fn test_validate_type_mismatch()
// Verify: Warns on type mismatches

#[test]
fn test_validate_aggregation_on_text()
// Verify: Prevents SUM/AVG on non-numeric

#[test]
fn test_valid_query_passes()
// Verify: Valid queries pass validation
```

---

## Performance Requirements

| Operation | Target | Measurement | Notes |
|-----------|--------|-------------|-------|
| Parse query (custom parser) | < 10μs | Wall time | 10x faster than sqlparser-rs |
| Execute simple filter (10k rows) | < 10ms | Wall time | Generic pattern matching |
| Execute key LIKE pattern (10k rows) | < 1ms | Wall time | **Optimised fast path** |
| Execute complex filter (10k rows) | < 50ms | Wall time | AND/OR/NOT combinations |
| Execute subquery (10k + 10k rows) | < 20ms | Wall time | Recursive execution |
| Sort 1k rows | < 5ms | Wall time | Single column |
| Sort 10k rows | < 50ms | Wall time | Single column |
| Aggregation (10k rows) | < 10ms | Wall time | COUNT/SUM/AVG |
| Format output (1k rows, table) | < 10ms | Wall time | ASCII table |
| Format output (1k rows, JSON) | < 20ms | Wall time | JSON serialisation |
| Query validation | < 100μs | Wall time | Column checks |
| Binary size overhead | < 10KB | Build artifact | Custom parser vs sqlparser-rs: +50KB |

---

## Error Conditions

### `ReedError::QuerySyntaxError`
**When**: Invalid ReedQL syntax.  
**Context**: Query string, position of error, expected syntax.  
**Recovery**: Fix query syntax according to error message.

### `ReedError::ColumnNotFound`
**When**: Referenced column does not exist.  
**Context**: Column name, table name, available columns.  
**Recovery**: Correct column name or check table schema.

### `ReedError::InvalidDataType`
**When**: Type mismatch in operation.  
**Context**: Operation, column type, provided value.  
**Recovery**: Ensure value types match column types.

### `ReedError::UnsupportedOperation`
**When**: Query uses unsupported SQL feature.  
**Context**: Attempted operation.  
**Recovery**: Use supported ReedQL subset.

---

## Metrics & Observability

### Performance Metrics

| Metric | Type | Unit | Target | P99 Alert | Collection Point |
|--------|------|------|--------|-----------|------------------|
| parse_time | Histogram | μs | <10 | >50 | parser.rs:parse() |
| query_execution_time | Histogram | ms | <50 | >500 | executor.rs:execute() |
| index_utilization_rate | Gauge | % | >80 | <50 | executor.rs:execute() |
| query_error_rate | Gauge | % | <5 | >20 | executor.rs:execute() |
| subquery_depth | Histogram | count | <3 | >5 | parser.rs:parse() |
| optimization_speedup | Histogram | ratio | >2 | <1.5 | optimiser.rs:optimize() |

### Alert Rules

**CRITICAL Alerts:**
- `query_execution_time p99 > 1000ms` for 5 minutes → "Queries critically slow - investigate"
- `query_error_rate > 20%` for 10 minutes → "High query failure rate - check syntax/data"

**WARNING Alerts:**
- `index_utilization_rate < 50%` for 10 minutes → "Low index usage - queries not optimized"
- `parse_time p99 > 50μs` for 5 minutes → "Parser slower than expected"

### Implementation

```rust
use crate::reedbase::metrics::global as metrics;
use std::time::Instant;

pub fn parse(query: &str) -> ReedResult<ParsedQuery> {
    let start = Instant::now();
    let parsed = parse_inner(query)?;
    
    metrics().record(Metric {
        name: "parse_time".to_string(),
        value: start.elapsed().as_nanos() as f64 / 1000.0, // Convert to μs
        unit: MetricUnit::Microseconds,
        tags: hashmap!{ "query_type" => parsed.query_type() },
    });
    
    Ok(parsed)
}

pub fn execute(query: &ParsedQuery) -> ReedResult<QueryResult> {
    let start = Instant::now();
    let result = execute_inner(query)?;
    
    metrics().record(Metric {
        name: "query_execution_time".to_string(),
        value: start.elapsed().as_millis() as f64,
        unit: MetricUnit::Milliseconds,
        tags: hashmap!{ "query_type" => query.query_type(), "used_index" => result.used_index.to_string() },
    });
    
    let index_used = if result.used_index { 100.0 } else { 0.0 };
    metrics().record(Metric {
        name: "index_utilization_rate".to_string(),
        value: index_used,
        unit: MetricUnit::Percent,
        tags: hashmap!{},
    });
    
    Ok(result)
}
```

### Collection Strategy

- **Sampling**: All operations
- **Aggregation**: 1-minute rolling window
- **Storage**: `.reedbase/metrics/reedql.csv`
- **Retention**: 7 days raw, 90 days aggregated

### Why These Metrics Matter

**parse_time**: Parser performance
- Custom parser should be 10x faster than generic SQL parsers
- Target <10μs validates design decision
- Slow parsing affects interactive CLI experience

**query_execution_time**: Query performance
- Core user-facing metric
- Slow queries impact productivity
- Helps identify optimization opportunities

**index_utilization_rate**: Optimization effectiveness
- High utilization (>80%) = optimizer working well
- Low rates = queries doing full table scans
- Directly correlates with query performance

**optimization_speedup**: Optimizer value
- Shows actual improvement from query optimization
- Target >2x validates optimizer implementation
- Low speedup indicates optimizer ineffectiveness

## Acceptance Criteria

### Core Implementation
- [ ] `parser.rs` implements custom hand-written ReedQL parser
- [ ] Parse time < 10μs (10x faster than generic SQL parsers)
- [ ] `executor.rs` executes all query types correctly
- [ ] `validator.rs` validates queries before execution
- [ ] `optimiser.rs` provides query optimisation hints
- [ ] `formatter.rs` formats output in table/json/csv

### Query Features
- [ ] SELECT with * and column list works
- [ ] WHERE with all operators (=, !=, <, >, <=, >=, LIKE, IN) works
- [ ] IN with subquery support (recursive execution)
- [ ] Nested subqueries work correctly (no depth limit)
- [ ] Logical operators (AND, OR, NOT) work correctly
- [ ] ORDER BY with ASC/DESC and multiple columns works
- [ ] LIMIT and OFFSET pagination works
- [ ] Aggregations (COUNT, SUM, AVG, MIN, MAX, DISTINCT) work

### ReedBase Optimisations
- [ ] Key pattern optimisation: `key LIKE '%.@de'` < 1ms for 10k rows
- [ ] Language filter fast path (ends_with check)
- [ ] Namespace filter fast path (starts_with check)
- [ ] Direct HashMap access (no AST conversion overhead)
- [ ] Optimised patterns 10x faster than generic regex

### Performance
- [ ] Parse query < 10μs
- [ ] Execute key LIKE pattern < 1ms for 10k rows
- [ ] Execute simple filter < 10ms for 10k rows
- [ ] Execute subquery < 20ms for 10k + 10k rows
- [ ] All performance targets met
- [ ] Binary size overhead < 10KB

### Quality
- [ ] Query validation detects all error types
- [ ] CLI commands provide clear output
- [ ] Error messages clear and actionable
- [ ] Test coverage 100% for all modules
- [ ] All tests pass
- [ ] Documentation complete with syntax reference
- [ ] ReedQL is CLI-only (no API exposure)
- [ ] All code in BBC English
- [ ] KISS principle followed (no bloat)

---

## Security Considerations

### CLI-Only Execution

**Critical**: ReedQL must NEVER be exposed via API endpoints.

**Rationale**:
- SQL injection risks
- Unauthorised data access
- Resource exhaustion attacks
- Complex permission model required

**Implementation**:
- ReedQL parser and executor only called from CLI commands
- No HTTP/API handlers for arbitrary queries
- API provides only specific, validated operations
- CLI requires local file system access (inherent authentication)

### Query Limits

- Maximum query length: 10,000 characters
- Maximum result rows: 100,000 (configurable)
- Query timeout: 30 seconds (configurable)
- Memory limit: 100MB per query

### Input Validation

- All user input sanitised in parser
- Column names validated against schema
- No arbitrary code execution
- No file system access from queries

---

## Dependencies

- **REED-19-02**: Universal Table API for data access
- **REED-19-08**: Schema validation for column checking
- **REED-19-09**: Function system for aggregations

---

## Notes

### Design Decisions

**Why SQL-like syntax?**
- Familiar to most users
- Reduces learning curve
- Industry standard for data queries

**Why CLI-only?**
- Security: No SQL injection via API
- Simplicity: No complex permission system needed
- Performance: Local execution avoids network overhead

**Why custom parser instead of sqlparser-rs?**
- **10x faster parsing**: < 10μs vs ~50-100μs
- **Smaller binary**: +5KB vs +50KB
- **ReedBase-optimised**: Direct HashMap access, key-pattern fast paths
- **KISS**: Only features ReedCMS needs, no bloat
- **Control**: Can add ReedCMS-specific syntax (key.language, key.namespace)

**Why support subqueries?**
- **Simple implementation**: Just recursive function calls
- **Common use-case**: "Find text keys that exist in routes"
- **No complexity**: No depth limit needed (CLI user is trusted)
- **Performance**: O(n + m) for simple subquery

**Why subset of SQL?**
- No JOINs: Single table operations are sufficient for ReedCMS
- No transactions: Read-only operations
- No DDL: Schema managed by REED-19-08
- Focus on essential operations only

### Future Enhancements

Potential future additions (not in scope for this ticket):
- Query caching for repeated queries
- Query history and favourites
- Query templates/macros
- Query performance profiling
- Index support for faster lookups

### Performance Optimisations

1. **Early filtering**: Apply WHERE clauses before sorting
2. **Column selection**: Select columns before sorting if possible
3. **Limit pushdown**: Apply LIMIT early if no ORDER BY
4. **Type caching**: Cache column types from schema
5. **Compiled patterns**: Cache compiled LIKE patterns

### Query Complexity Limits

Maximum complexity to prevent abuse:
- Maximum 10 AND/OR operators
- Maximum 5 ORDER BY columns
- Maximum 100 values in IN clause
- Maximum 3 nested NOT operators

---

## References

- Service Template: `_workbench/Tickets/templates/service-template.md`
- Test Template: `_workbench/Tickets/templates/service-template.test.md`
- REED-19-02: Universal Table API
- REED-19-08: Schema Validation
- REED-19-09: Function System
