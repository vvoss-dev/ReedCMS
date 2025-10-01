# REED-03-03: Universal Taxonomy System

## MANDATORY Development Standards

**CRITICAL**: Every implementation MUST follow these standards:

- **Language**: All documentation and code comments in BBC English
- **Principle**: KISS (Keep It Simple, Stupid) - minimal code and professional inline documentation
- **File Naming**: File name = Unique theme - crystal clear what single topic this file handles
- **Files**: One file = One responsibility
- **Functions**: One function = One distinctive job
- **Shared Functions**: Same patterns = One shared function (CONFIG and SYSTEM use identical logic)
- **Testing**: Separate test files as `{name}.test.rs` - never inline `#[cfg(test)]` modules
- **Avoid**: Avoid Swiss Army knife functions
- **Avoid**: Generic file names like `handler.rs`, `middleware.rs`, `utils.rs` - every topic (auth, login,...) has its own scoped rs service file
- **Templates**: See `_workbench/Tickets/templates/service-template.md` for complete implementation guide
- **Testing**: See `_workbench/Tickets/templates/service-template.test.md` for test structure

## Ticket Information
- **ID**: REED-03-03
- **Title**: Universal Taxonomy System
- **Layer**: Security Layer (REED-03)
- **Priority**: Medium
- **Status**: Open
- **Complexity**: High
- **Dependencies**: REED-02-01, REED-02-02

## Summary Reference
- **Section**: Universal Taxonomy System
- **Lines**: 399-450 in project_summary.md
- **Key Concepts**: Hierarchical terms, universal entity tagging, usage analytics

## Objective
Implement hierarchical taxonomy system for universal entity tagging with usage analytics, unlimited depth parent-child relationships, and cross-entity search capabilities.

## Requirements

### Taxonomy Matrix CSV
```csv
term_id|term|parent_id|category|description|color|icon|status|created_by|usage_count
1|technology||category|Technology related content|#2563eb|tech|active|admin|42
2|rust|1|tag|Rust programming language|#ce422b|rust|active|admin|15
3|cms|1|tag|Content Management System|#059669|cms|active|admin|8
4|content-type||category|Content type classification|#8b5cf6|type|active|admin|128
5|blog|4|tag|Blog posts and articles|#06b6d4|blog|active|admin|89
```

### Entity Assignment CSV
```csv
entity_type|entity_id|term_ids|assigned_by|assigned_at|context|inherited_from
user|admin|1,4,6|system|2025-01-15T10:00:00Z|auto_assigned|
content|blog.post.001|1,2,3,5|editor|2025-01-15T12:00:00Z|content_creation|
template|blog.jinja|1,3,5|admin|2025-01-15T10:30:00Z|template_creation|
route|blog@de|1,3,5|admin|2025-01-15T10:31:00Z|route_creation|
site|main|1,4|admin|2025-01-15T09:00:00Z|site_creation|
content|blog.post.002|1,2,5|editor|2025-01-15T14:00:00Z|inherited|site:main
```

### Implementation Files

#### Term Management (`src/reedcms/taxonomy/terms.rs`)

```rust
/// Creates new taxonomy term.
///
/// ## Input
/// - `req.term`: Term name
/// - `req.parent_id`: Optional parent term ID
/// - `req.category`: Term category (category, tag, system, custom)
/// - `req.description`: Term description
/// - `req.color`: Hex color code for UI
/// - `req.icon`: Icon identifier
///
/// ## Validation
/// - Term uniqueness within same parent
/// - Parent term existence check
/// - Circular hierarchy detection
/// - Color format validation (hex)
pub fn create_term(req: &ReedRequest) -> ReedResult<ReedResponse<TaxonomyTerm>>

/// Retrieves term with full hierarchy path.
pub fn get_term(term_id: u32) -> ReedResult<ReedResponse<TaxonomyTerm>>

/// Lists all terms (optionally filtered).
pub fn list_terms(filter: Option<TermFilter>) -> ReedResult<ReedResponse<Vec<TaxonomyTerm>>>

/// Updates term properties.
pub fn update_term(term_id: u32, updates: TermUpdate) -> ReedResult<ReedResponse<TaxonomyTerm>>

/// Deletes term (requires no entity assignments).
pub fn delete_term(term_id: u32, confirm: bool) -> ReedResult<ReedResponse<()>>

/// Taxonomy term structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaxonomyTerm {
    pub term_id: u32,
    pub term: String,
    pub parent_id: Option<u32>,
    pub category: TermCategory,
    pub description: String,
    pub color: String,
    pub icon: String,
    pub status: TermStatus,
    pub created_by: String,
    pub usage_count: u32,
    pub hierarchy_path: Vec<String>,  // Full path from root
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TermCategory {
    Category,
    Tag,
    System,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TermStatus {
    Active,
    Deprecated,
    Hidden,
    Pending,
}
```

#### Hierarchy Management (`src/reedcms/taxonomy/hierarchy.rs`)

```rust
/// Builds complete term hierarchy tree.
///
/// ## Output
/// - Tree structure with parent-child relationships
/// - Unlimited depth support
pub fn build_hierarchy_tree() -> ReedResult<Vec<TermNode>>

/// Resolves full hierarchy path for term.
///
/// ## Example
/// - term_id: 2 (rust)
/// - Output: ["technology", "rust"]
pub fn get_hierarchy_path(term_id: u32) -> ReedResult<Vec<String>>

/// Checks for circular hierarchy.
pub fn has_circular_hierarchy(term_id: u32, parent_id: u32) -> ReedResult<bool>

/// Gets all child terms (recursive).
pub fn get_all_children(term_id: u32) -> ReedResult<Vec<u32>>

/// Term tree node structure
#[derive(Debug, Clone)]
pub struct TermNode {
    pub term: TaxonomyTerm,
    pub children: Vec<TermNode>,
}
```

#### Entity Assignments (`src/reedcms/taxonomy/assignments.rs`)

```rust
/// Assigns terms to entity.
///
/// ## Supported Entity Types
/// - content: Blog posts, pages, media files
/// - users: Staff, authors, administrators
/// - roles: Permission groups
/// - templates: Layout files and components
/// - routes: URL mappings
/// - sites: Multi-site configurations
/// - projects: Project-level organisation
/// - assets: Media and file categorisation
///
/// ## Input
/// - `entity_type`: Entity type
/// - `entity_id`: Entity identifier
/// - `term_ids`: Vector of term IDs to assign
/// - `context`: Assignment context (manual, auto, inherited, etc.)
pub fn assign_terms(entity_type: &str, entity_id: &str, term_ids: Vec<u32>, context: &str) -> ReedResult<()>

/// Removes term assignments from entity.
pub fn unassign_terms(entity_type: &str, entity_id: &str, term_ids: Vec<u32>) -> ReedResult<()>

/// Gets all terms assigned to entity.
pub fn get_entity_terms(entity_type: &str, entity_id: &str) -> ReedResult<Vec<TaxonomyTerm>>

/// Bulk assigns terms to multiple entities.
pub fn bulk_assign_terms(assignments: Vec<BulkAssignment>) -> ReedResult<BulkResult>

/// Assignment context types
#[derive(Debug, Clone)]
pub enum AssignmentContext {
    Manual,        // Manually assigned by user
    Automatic,     // Auto-assigned by system
    Inherited,     // Inherited from parent entity
    Templated,     // Applied via template
    Migrated,      // Migrated from old system
}
```

#### Term Search (`src/reedcms/taxonomy/search.rs`)

```rust
/// Searches terms by name and description.
///
/// ## Features
/// - Full-text search
/// - Fuzzy matching
/// - Category filtering
/// - Status filtering
pub fn search_terms(query: &str, filters: Option<SearchFilters>) -> ReedResult<Vec<TaxonomyTerm>>

/// Finds all entities tagged with term.
pub fn find_entities_by_term(term_id: u32) -> ReedResult<Vec<EntityReference>>

/// Cross-entity search by multiple terms.
///
/// ## Logic
/// - AND: Entity must have all specified terms
/// - OR: Entity must have at least one term
pub fn search_entities_by_terms(term_ids: Vec<u32>, logic: SearchLogic) -> ReedResult<Vec<EntityReference>>

/// Entity reference structure
#[derive(Debug, Clone)]
pub struct EntityReference {
    pub entity_type: String,
    pub entity_id: String,
    pub assigned_at: u64,
    pub assigned_by: String,
}
```

#### Usage Analytics (`src/reedcms/taxonomy/analytics.rs`)

```rust
/// Updates usage count for term.
pub fn increment_usage(term_id: u32) -> ReedResult<()>

/// Gets usage statistics.
pub fn get_usage_stats() -> ReedResult<UsageStats>

/// Gets most popular terms.
pub fn get_popular_terms(limit: usize) -> ReedResult<Vec<TaxonomyTerm>>

/// Gets unused terms.
pub fn get_unused_terms() -> ReedResult<Vec<TaxonomyTerm>>

/// Usage statistics structure
#[derive(Debug, Clone)]
pub struct UsageStats {
    pub total_terms: u32,
    pub total_assignments: u32,
    pub avg_terms_per_entity: f32,
    pub most_used_term: Option<TaxonomyTerm>,
    pub least_used_term: Option<TaxonomyTerm>,
}
```

## Implementation Files

### Primary Implementation
- `src/reedcms/taxonomy/terms.rs` - Term CRUD operations
- `src/reedcms/taxonomy/hierarchy.rs` - Parent-child relationships
- `src/reedcms/taxonomy/assignments.rs` - Entity-term assignments
- `src/reedcms/taxonomy/search.rs` - Term and entity search
- `src/reedcms/taxonomy/analytics.rs` - Usage tracking

### Test Files
- `src/reedcms/taxonomy/terms.test.rs`
- `src/reedcms/taxonomy/hierarchy.test.rs`
- `src/reedcms/taxonomy/assignments.test.rs`
- `src/reedcms/taxonomy/search.test.rs`
- `src/reedcms/taxonomy/analytics.test.rs`

## Testing Requirements

### Unit Tests
- [ ] Test term creation with validation
- [ ] Test hierarchy path resolution
- [ ] Test circular hierarchy detection
- [ ] Test term assignment to entities
- [ ] Test bulk operations
- [ ] Test usage counting

### Integration Tests
- [ ] Test complete taxonomy lifecycle
- [ ] Test cross-entity term search
- [ ] Test inherited term assignments
- [ ] Test term deletion with entity checks
- [ ] Test usage analytics

### Performance Tests
- [ ] Term creation: < 10ms
- [ ] Hierarchy path resolution: < 5ms
- [ ] Entity search: < 50ms for 10k+ entities
- [ ] Bulk assignment: < 100ms for 100 entities

## Acceptance Criteria
- [ ] Hierarchical term management (unlimited depth)
- [ ] Circular dependency detection working
- [ ] Bulk term assignment/removal implemented
- [ ] Usage analytics tracking functional
- [ ] Cross-entity search operational
- [ ] Support for all 8 entity types
- [ ] All tests pass with 100% coverage
- [ ] Performance targets met
- [ ] Documentation complete
- [ ] BBC English throughout

## Dependencies
- **Requires**: REED-02-01 (ReedBase), REED-02-02 (CSV Handler)

## Blocks
- REED-04-06 (CLI taxonomy commands need this)

## References
- Service Template: `_workbench/Tickets/templates/service-template.md`
- Service Test Template: `_workbench/Tickets/templates/service-template.test.md`
- Summary: Lines 399-450 in `project_summary.md`

## Notes
The taxonomy system provides universal categorisation across all ReedCMS entities. The hierarchical structure with unlimited depth enables complex classification schemes. Usage analytics help identify popular and unused terms. The system must maintain performance even with 10k+ entities and 1k+ terms through efficient indexing and caching.