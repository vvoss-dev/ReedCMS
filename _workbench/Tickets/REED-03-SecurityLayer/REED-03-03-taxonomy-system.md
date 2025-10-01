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

### Taxonomy Matrix CSV (Matrix Type 4)

ReedCMS uses **Matrix Type 4 syntax** for taxonomy: `key[modifier],key[modifier]` format for properties.

```csv
# .reed/taxonomie.matrix.csv
term_id|vocabulary|properties|desc
navigation|menu|weight[0],parent[],enabled[true]|Main Navigation Container
slider-rosengarten|media|weight[0],parent[],enabled[true]|Rosengarten Slider Collection
footer-legal|menu|weight[0],parent[],enabled[true]|Footer Legal Links
technology|category|weight[0],parent[],enabled[true],color[#2563eb],icon[tech]|Technology Category
rust|category|weight[10],parent[technology],enabled[true],color[#ce422b],icon[rust]|Rust Language
cms|category|weight[20],parent[technology],enabled[true],color[#059669],icon[cms]|CMS Technology
```

**Property Modifiers** (Type 4 List):
- `weight[int]` - Sort order (0-999)
- `parent[term_id]` - Parent term for hierarchy (empty = root)
- `enabled[bool]` - Visibility flag
- `color[hex]` - UI color code (optional)
- `icon[name]` - Icon identifier (optional)

### Entity Assignment Matrix CSV (Matrix Type 4)

```csv
# .reed/entity_taxonomy.matrix.csv
entity_id|term_id|properties|desc
knowledge|navigation|weight[10],enabled[true]|Main navigation entry
portfolio|navigation|weight[20],enabled[true]|Main navigation entry
blog|navigation|weight[30],enabled[true]|Main navigation entry
impressum|footer-legal|weight[10],enabled[true]|Footer link
image-rose-01|slider-rosengarten|weight[1],enabled[true],assigned_by[admin],assigned_at[1640995200]|Slider image
image-rose-02|slider-rosengarten|weight[2],enabled[true],assigned_by[admin],assigned_at[1640995200]|Slider image
blog.post.001|technology|weight[0],enabled[true],assigned_by[editor],context[manual]|Blog post tagging
blog.post.001|rust|weight[0],enabled[true],assigned_by[editor],context[manual]|Blog post tagging
```

**Assignment Property Modifiers**:
- `weight[int]` - Sort order within term
- `enabled[bool]` - Assignment active flag
- `assigned_by[username]` - Who created assignment (optional)
- `assigned_at[timestamp]` - Unix timestamp (optional)
- `context[type]` - Assignment context: manual, auto, inherited (optional)

### Implementation Files

#### Term Management (`src/reedcms/taxonomy/terms.rs`)

```rust
/// Creates new taxonomy term with Matrix Type 4 properties.
///
/// ## Input
/// - `req.term_id`: Term identifier (string, e.g., "navigation", "slider-rosengarten")
/// - `req.vocabulary`: Vocabulary name (e.g., "menu", "media", "category")
/// - `req.properties`: Type 4 properties string (e.g., "weight[10],parent[],enabled[true]")
/// - `req.desc`: Term description
///
/// ## Validation
/// - Term ID uniqueness
/// - Parent term existence check
/// - Circular hierarchy detection
/// - Properties syntax validation
pub fn create_term(req: &ReedRequest) -> ReedResult<ReedResponse<TaxonomyTerm>>

/// Retrieves term with parsed properties.
pub fn get_term(term_id: &str) -> ReedResult<ReedResponse<TaxonomyTerm>>

/// Lists all terms (optionally filtered by vocabulary).
pub fn list_terms(vocabulary: Option<&str>) -> ReedResult<ReedResponse<Vec<TaxonomyTerm>>>

/// Updates term properties (merges with existing).
pub fn update_term(term_id: &str, updates: TermUpdate) -> ReedResult<ReedResponse<TaxonomyTerm>>

/// Deletes term (requires no entity assignments).
pub fn delete_term(term_id: &str, confirm: bool) -> ReedResult<ReedResponse<()>>

/// Taxonomy term structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaxonomyTerm {
    pub term_id: String,
    pub vocabulary: String,
    pub properties: TermProperties,  // Parsed from Matrix Type 4
    pub desc: String,
    pub hierarchy_path: Vec<String>,  // Full path from root
}

/// Parsed term properties from Matrix Type 4 string
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TermProperties {
    pub weight: i32,
    pub parent: Option<String>,
    pub enabled: bool,
    pub color: Option<String>,
    pub icon: Option<String>,
    pub custom: HashMap<String, String>,  // Additional properties
}

/// Parses Matrix Type 4 properties string.
///
/// ## Input Example
/// "weight[10],parent[navigation],enabled[true],color[#ff0000]"
///
/// ## Output
/// TermProperties { weight: 10, parent: Some("navigation"), enabled: true, ... }
pub fn parse_properties(properties_str: &str) -> ReedResult<TermProperties>

/// Serializes properties back to Matrix Type 4 string.
pub fn serialize_properties(props: &TermProperties) -> String
```

#### Hierarchy Management (`src/reedcms/taxonomy/hierarchy.rs`)

```rust
/// Builds complete term hierarchy tree for vocabulary.
///
/// ## Output
/// - Tree structure with parent-child relationships
/// - Unlimited depth support
/// - Sorted by weight property
pub fn build_hierarchy_tree(vocabulary: &str) -> ReedResult<Vec<TermNode>>

/// Resolves full hierarchy path for term.
///
/// ## Example
/// - term_id: "rust"
/// - Output: ["technology", "rust"]
pub fn get_hierarchy_path(term_id: &str) -> ReedResult<Vec<String>>

/// Checks for circular hierarchy.
pub fn has_circular_hierarchy(term_id: &str, parent_id: &str) -> ReedResult<bool>

/// Gets all child terms (recursive).
pub fn get_all_children(term_id: &str) -> ReedResult<Vec<String>>

/// Gets all entities assigned to term (with hierarchy).
///
/// ## Example
/// If term "technology" has children "rust" and "cms",
/// returns entities tagged with technology, rust, or cms.
pub fn get_entities_in_hierarchy(term_id: &str, include_children: bool) -> ReedResult<Vec<EntityAssignment>>

/// Term tree node structure
#[derive(Debug, Clone)]
pub struct TermNode {
    pub term: TaxonomyTerm,
    pub children: Vec<TermNode>,
    pub entity_count: usize,  // Number of entities tagged with this term
}
```

#### Entity Assignments (`src/reedcms/taxonomy/assignments.rs`)

```rust
/// Assigns term to entity with Matrix Type 4 properties.
///
/// ## Input
/// - `entity_id`: Entity identifier (e.g., "knowledge", "blog.post.001", "image-rose-01")
/// - `term_id`: Term identifier (e.g., "navigation", "slider-rosengarten")
/// - `properties`: Type 4 properties string (e.g., "weight[10],enabled[true]")
///
/// ## Example
/// assign_term("knowledge", "navigation", "weight[10],enabled[true]")
pub fn assign_term(entity_id: &str, term_id: &str, properties: &str) -> ReedResult<()>

/// Removes term assignment from entity.
pub fn unassign_term(entity_id: &str, term_id: &str) -> ReedResult<()>

/// Gets all terms assigned to entity (sorted by weight).
pub fn get_entity_terms(entity_id: &str) -> ReedResult<Vec<EntityAssignment>>

/// Gets all entities assigned to term (sorted by weight).
pub fn get_term_entities(term_id: &str) -> ReedResult<Vec<EntityAssignment>>

/// Bulk assigns terms to multiple entities.
pub fn bulk_assign_terms(assignments: Vec<BulkAssignment>) -> ReedResult<BulkResult>

/// Entity assignment structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityAssignment {
    pub entity_id: String,
    pub term_id: String,
    pub properties: AssignmentProperties,
    pub desc: String,
}

/// Parsed assignment properties from Matrix Type 4 string
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssignmentProperties {
    pub weight: i32,
    pub enabled: bool,
    pub assigned_by: Option<String>,
    pub assigned_at: Option<u64>,
    pub context: Option<String>,  // manual, auto, inherited
    pub custom: HashMap<String, String>,
}
```

#### Template Integration (`src/reedcms/filters/taxonomy.rs`)

```rust
/// MiniJinja filter for taxonomy queries.
///
/// ## Usage in Templates
/// ```jinja
/// {# Get all entities in navigation term #}
/// {% for item in taxonomy('navigation') %}
///   <a href="/{{ client.lang }}/{{ item.entity_id | route('auto') }}/">
///     {{ item.entity_id | text('auto') }}
///   </a>
/// {% endfor %}
///
/// {# Get slider images #}
/// {% for image in taxonomy('slider-rosengarten') %}
///   <img src="{{ image.entity_id | asset }}">
/// {% endfor %}
/// ```
///
/// ## Implementation
/// - Queries entity_taxonomy.matrix.csv for term_id
/// - Returns entities sorted by weight property
/// - Filters by enabled[true]
/// - Zero allocations for cached data
pub fn make_taxonomy_filter() -> impl minijinja::filters::Filter + Send + Sync + 'static {
    |term_id: &str| -> Result<Vec<EntityAssignment>, minijinja::Error> {
        let assignments = get_term_entities(term_id)
            .map_err(|e| minijinja::Error::new(
                minijinja::ErrorKind::InvalidOperation,
                format!("Taxonomy query failed: {}", e)
            ))?;
        
        // Filter enabled, sort by weight
        let mut enabled: Vec<_> = assignments.into_iter()
            .filter(|a| a.properties.enabled)
            .collect();
        enabled.sort_by_key(|a| a.properties.weight);
        
        Ok(enabled)
    }
}
```

**Context Builder Integration** (REED-05-03):
```rust
/// Adds taxonomy filter to MiniJinja environment.
pub fn add_taxonomy_filter(env: &mut minijinja::Environment) {
    env.add_filter("taxonomy", make_taxonomy_filter());
}
```

**Template Examples**:
```jinja
{# Navigation menu with Drupal-style taxonomy #}
<nav>
  <ul>
    {% for item in taxonomy('navigation') %}
      <li>
        <a href="/{{ client.lang }}/{{ item.entity_id | route('auto') }}/">
          {{ item.entity_id | text('auto') }}
        </a>
      </li>
    {% endfor %}
  </ul>
</nav>

{# Footer navigation (separate taxonomy term) #}
<footer>
  {% for link in taxonomy('footer-legal') %}
    <a href="/{{ client.lang }}/{{ link.entity_id | route('auto') }}/">
      {{ link.entity_id | text('auto') }}
    </a>
  {% endfor %}
</footer>

{# Image slider with weighted ordering #}
<div class="slider">
  {% for slide in taxonomy('slider-rosengarten') %}
    <img src="/assets/{{ slide.entity_id }}.jpg" alt="{{ slide.desc }}">
  {% endfor %}
</div>
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