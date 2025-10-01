# REED-02-06: Taxonomy System

## MANDATORY Development Standards

**CRITICAL**: Every implementation MUST follow these standards:

- **Language**: All documentation and code comments in BBC English
- **Principle**: KISS (Keep It Simple, Stupid) - minimal code and professional inline documentation
- **File Naming**: File name = Unique theme - crystal clear what single topic this file handles
- **Files**: One file = One responsibility
- **Functions**: One function = One distinctive job
- **Shared Functions**: Same patterns = One shared function
- **Testing**: Separate test files as `{name}_test.rs` - never inline `#[cfg(test)]` modules
- **Avoid**: Avoid Swiss Army knife functions
- **Avoid**: Generic file names like `handler.rs`, `utils.rs`
- **Templates**: See `_workbench/Tickets/templates/service-template.md` for complete implementation guide
- **Testing**: See `_workbench/Tickets/templates/service-template.test.md` for test structure

## Ticket Information
- **ID**: REED-02-06
- **Title**: Taxonomy System
- **Layer**: Data Layer (REED-02)
- **Priority**: Medium
- **Status**: Open
- **Complexity**: High
- **Dependencies**: REED-02-01, REED-02-02, REED-02-05

## Summary Reference
- **Section**: Universal Taxonomy System
- **Lines**: 399-450 in project_summary.md
- **Key Concepts**: Hierarchical terms, universal entity tagging, usage analytics

## Objective
Implement comprehensive hierarchical taxonomy system with unlimited depth parent-child relationships, universal entity tagging, cycle detection, and usage analytics.

## Requirements

### Taxonomy Term CSV Structure
```csv
# .reed/taxonomie.matrix.csv
term_id|term|parent_id|category|description|color|icon|status|created_by|usage_count
1|technology||category|Technology related content|#2563eb|tech|active|admin|42
2|rust|1|tag|Rust programming language|#ce422b|rust|active|admin|15
3|cms|1|tag|Content Management System|#059669|cms|active|admin|8
```

### Entity-Term Assignment CSV Structure
```csv
# .reed/entity_taxonomy.matrix.csv
entity_type|entity_id|term_ids|assigned_by|assigned_at|context|inherited_from
user|admin|1,4,6|system|2025-01-15T10:00:00Z|auto_assigned|
content|blog.post.001|1,2,3,5|editor|2025-01-15T12:00:00Z|content_creation|
template|blog.jinja|1,3,5|admin|2025-01-15T10:30:00Z|template_creation|
```

### Implementation Files

#### Term Management (`src/reedcms/taxonomy/terms.rs`)

```rust
/// Creates new taxonomy term with validation.
///
/// ## Input
/// - `req.key`: Term name (unique within same parent)
/// - `req.value`: Term category (category, tag, system, custom)
/// - `req.context`: JSON with parent_id, description, color, icon
///
/// ## Validation
/// - Term name uniqueness within same parent level
/// - Parent term existence check
/// - Circular hierarchy detection
/// - Status validation
///
/// ## Output
/// - `ReedResult<ReedResponse<TermInfo>>`: Created term info
pub fn create_term(req: &ReedRequest) -> ReedResult<ReedResponse<TermInfo>>

/// Retrieves term by ID.
pub fn get_term(term_id: u64) -> ReedResult<ReedResponse<TermInfo>>

/// Lists all terms with optional filtering.
pub fn list_terms(filter: Option<TermFilter>) -> ReedResult<ReedResponse<Vec<TermInfo>>>

/// Searches terms by name or description.
pub fn search_terms(query: &str) -> ReedResult<ReedResponse<Vec<TermInfo>>>

/// Updates term properties.
pub fn update_term(term_id: u64, updates: TermUpdate) -> ReedResult<ReedResponse<TermInfo>>

/// Deletes term (requires no children and no entity assignments).
pub fn delete_term(term_id: u64, confirm: bool) -> ReedResult<ReedResponse<()>>

/// Term information structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TermInfo {
    pub term_id: u64,
    pub term: String,
    pub parent_id: Option<u64>,
    pub category: String,  // category, tag, system, custom
    pub description: String,
    pub color: Option<String>,  // Hex color code
    pub icon: Option<String>,
    pub status: String,  // active, deprecated, hidden, pending
    pub created_by: String,
    pub usage_count: u64,
    pub created_at: u64,
    pub updated_at: u64,
}
```

#### Entity-Term Assignment (`src/reedcms/taxonomy/entities.rs`)

```rust
/// Assigns terms to entity.
///
/// ## Input
/// - `entity_type`: Type (user, content, template, route, site, project, asset, role)
/// - `entity_id`: Entity identifier
/// - `term_ids`: Vector of term IDs to assign
/// - `assigned_by`: User who assigns
/// - `context`: Assignment context (manual, automatic, inherited, templated, migrated)
///
/// ## Validation
/// - Entity type validation
/// - Term existence check
/// - Duplicate prevention
///
/// ## Output
/// - `ReedResult<ReedResponse<EntityTerms>>`: Updated entity-term assignments
pub fn assign_terms(
    entity_type: &str,
    entity_id: &str,
    term_ids: Vec<u64>,
    assigned_by: &str,
    context: &str,
) -> ReedResult<ReedResponse<EntityTerms>>

/// Unassigns terms from entity.
pub fn unassign_terms(
    entity_type: &str,
    entity_id: &str,
    term_ids: Vec<u64>,
) -> ReedResult<ReedResponse<EntityTerms>>

/// Gets all terms assigned to entity.
pub fn get_entity_terms(
    entity_type: &str,
    entity_id: &str,
) -> ReedResult<ReedResponse<EntityTerms>>

/// Lists all entities tagged with specific term.
pub fn list_entities_by_term(term_id: u64) -> ReedResult<ReedResponse<Vec<EntityTerms>>>

/// Entity-terms assignment structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityTerms {
    pub entity_type: String,
    pub entity_id: String,
    pub term_ids: Vec<u64>,
    pub assigned_by: String,
    pub assigned_at: u64,
    pub context: String,
    pub inherited_from: Option<String>,
}
```

#### Hierarchy Management (`src/reedcms/taxonomy/hierarchy.rs`)

```rust
/// Resolves term hierarchy path.
///
/// ## Input
/// - `term_id`: Term ID to resolve
///
/// ## Output
/// - `ReedResult<Vec<TermInfo>>`: Path from root to term
///
/// ## Example
/// - Input: term_id=3 (cms)
/// - Output: [technology, cms]
pub fn resolve_hierarchy(term_id: u64) -> ReedResult<Vec<TermInfo>>

/// Checks for circular hierarchy.
pub fn has_circular_hierarchy(term_id: u64, parent_id: u64) -> ReedResult<bool>

/// Gets all child terms recursively.
pub fn get_children(term_id: u64, recursive: bool) -> ReedResult<Vec<TermInfo>>

/// Gets term depth in hierarchy.
pub fn get_term_depth(term_id: u64) -> ReedResult<u64>
```

## Implementation Files

### Primary Implementation
- `src/reedcms/taxonomy/mod.rs` - Module organisation
- `src/reedcms/taxonomy/terms.rs` - Term CRUD operations
- `src/reedcms/taxonomy/entities.rs` - Entity-term assignments
- `src/reedcms/taxonomy/hierarchy.rs` - Hierarchy utilities

### Test Files
- `src/reedcms/taxonomy/terms_test.rs`
- `src/reedcms/taxonomy/entities_test.rs`
- `src/reedcms/taxonomy/hierarchy_test.rs`

## Testing Requirements

### Unit Tests
- [ ] Test term creation with validation
- [ ] Test parent-child relationships
- [ ] Test circular hierarchy detection
- [ ] Test term search functionality
- [ ] Test usage count tracking
- [ ] Test entity-term assignments

### Integration Tests
- [ ] Test complete taxonomy lifecycle
- [ ] Test hierarchical term retrieval
- [ ] Test cross-entity term search
- [ ] Test bulk term assignments
- [ ] Test term deletion with dependencies

### Performance Tests
- [ ] Term creation: < 10ms
- [ ] Term search: < 50ms for 10k+ terms
- [ ] Entity lookup: < 20ms
- [ ] Hierarchy resolution: < 30ms

## Acceptance Criteria
- [ ] Hierarchical term management with unlimited depth
- [ ] Circular hierarchy prevention working
- [ ] Universal entity support (8 types)
- [ ] Usage count tracking automatic
- [ ] Full-text search across terms
- [ ] Duplicate prevention at same level
- [ ] All tests pass with 100% coverage
- [ ] Documentation complete
- [ ] BBC English throughout

## Dependencies
- **Requires**: REED-02-01 (ReedBase), REED-02-02 (CSV Handler), REED-02-05 (Matrix CSV)

## Blocks
- REED-04-06 (CLI taxonomy commands)
- Template system (taxonomy-based navigation)

## References
- Service Template: `_workbench/Tickets/templates/service-template.md`
- Service Test Template: `_workbench/Tickets/templates/service-template.test.md`
- Summary: Lines 399-450 in `project_summary.md`

## Notes
The taxonomy system provides universal content organisation. Terms support hierarchical organisation (unlimited depth), visual identification (colors, icons), and lifecycle management (active, deprecated, hidden, pending). Usage count tracking enables analytics and popular term identification. Entity-term assignments support multiple contexts (manual, automatic, inherited, templated, migrated) for flexible tagging strategies.

Performance is critical for large taxonomies (10k+ terms, 100k+ entity assignments). Implement caching where appropriate and optimize hierarchy resolution algorithms.
