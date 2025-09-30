# REED-04-06: CLI Taxonomy Commands

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
- **ID**: REED-04-06
- **Title**: CLI Taxonomy Management Commands
- **Layer**: CLI Layer (REED-04)
- **Priority**: Medium
- **Status**: Open
- **Complexity**: Medium
- **Dependencies**: REED-04-01, REED-03-03

## Summary Reference
- **Section**: CLI Taxonomy Management
- **Lines**: 1099-1109 in project_summary.md
- **Key Concepts**: Hierarchical term management, entity tagging, usage analytics

## Objective
Implement complete CLI interface for taxonomy management including term creation, hierarchical organization, entity assignments, search capabilities, and usage statistics.

## Requirements

### Commands to Implement

```bash
# Term creation
reed taxonomy:create "rust" --category tag --parent 1
reed taxonomy:create "technology" --category category --color "#2563eb" --icon "tech"

# Term listing
reed taxonomy:list
reed taxonomy:list --tree
reed taxonomy:list --category tag

# Term details
reed taxonomy:show 2

# Term search
reed taxonomy:search "technology"
reed taxonomy:search "tech" --fuzzy

# Entity assignments
reed taxonomy:assign user:admin --terms "1,4,6"
reed taxonomy:assign content:blog.post.001 --terms "2,3,5"
reed taxonomy:unassign user:admin --terms "4"

# Entity search by taxonomy
reed taxonomy:entities 1
reed taxonomy:entities 1,2,3 --logic and

# Usage statistics
reed taxonomy:usage
reed taxonomy:usage --stats

# Term updates
reed taxonomy:update 2 --status "deprecated"
reed taxonomy:update 1 --description "Updated description"

# Term deletion
reed taxonomy:delete 5
reed taxonomy:delete 5 --confirm
```

### Implementation (`src/reedcms/cli/taxonomy_commands.rs`)

```rust
/// Creates new taxonomy term.
///
/// ## Arguments
/// - args[0]: Term name
///
/// ## Flags
/// - --category: Term category (category, tag, system, custom)
/// - --parent: Parent term ID (optional)
/// - --description: Term description
/// - --color: Hex color code for UI (#RRGGBB)
/// - --icon: Icon identifier
///
/// ## Output
/// ✓ Term 'rust' created successfully (ID: 2)
///   Category: tag
///   Parent: technology (ID: 1)
///   Color: #ce422b
pub fn create_term(args: &[String], flags: &HashMap<String, String>) -> ReedResult<ReedResponse<String>>

/// Lists taxonomy terms.
///
/// ## Flags
/// - --tree: Show hierarchical tree structure
/// - --category: Filter by category
/// - --format: Output format (tree, flat, json)
///
/// ## Output (tree format)
/// ```
/// technology (1)
/// ├── rust (2)
/// ├── cms (3)
/// └── web (4)
///     ├── html (5)
///     └── css (6)
/// ```
pub fn list_terms(flags: &HashMap<String, String>) -> ReedResult<ReedResponse<String>>

/// Shows detailed term information.
///
/// ## Arguments
/// - args[0]: Term ID
///
/// ## Output
/// Term ID: 2
/// Term: rust
/// Category: tag
/// Parent: technology (ID: 1)
/// Hierarchy: technology → rust
/// Description: Rust programming language
/// Color: #ce422b
/// Icon: rust
/// Status: active
/// Usage count: 15
/// Created by: admin
/// Entities using this term: 15
pub fn show_term(args: &[String]) -> ReedResult<ReedResponse<String>>

/// Searches terms by name or description.
///
/// ## Arguments
/// - args[0]: Search query
///
/// ## Flags
/// - --fuzzy: Enable fuzzy matching
/// - --category: Filter by category
///
/// ## Output
/// Found 3 terms matching 'tech':
/// 1. technology (category) - Technology related content
/// 2. tech-stack (tag) - Technical stack information
/// 5. fintech (tag) - Financial technology
pub fn search_terms(args: &[String], flags: &HashMap<String, String>) -> ReedResult<ReedResponse<String>>

/// Assigns terms to entity.
///
/// ## Arguments
/// - args[0]: Entity reference (type:id)
///
/// ## Flags
/// - --terms: Comma-separated term IDs
///
/// ## Supported Entity Types
/// - user:username
/// - content:id
/// - template:name
/// - route:key
/// - site:name
/// - project:name
/// - role:name
/// - asset:id
///
/// ## Output
/// ✓ Assigned 3 terms to user:admin
///   - technology (1)
///   - rust (2)
///   - cms (3)
pub fn assign_terms(args: &[String], flags: &HashMap<String, String>) -> ReedResult<ReedResponse<String>>

/// Removes term assignments from entity.
///
/// ## Arguments
/// - args[0]: Entity reference (type:id)
///
/// ## Flags
/// - --terms: Comma-separated term IDs to remove
///
/// ## Output
/// ✓ Removed 1 term from user:admin
///   - cms (3)
pub fn unassign_terms(args: &[String], flags: &HashMap<String, String>) -> ReedResult<ReedResponse<String>>

/// Lists entities tagged with term(s).
///
/// ## Arguments
/// - args[0]: Term ID or comma-separated IDs
///
/// ## Flags
/// - --logic: Search logic (and, or)
/// - --type: Filter by entity type
///
/// ## Output
/// Entities tagged with term 'technology' (1):
/// - user:admin
/// - content:blog.post.001
/// - template:knowledge.mouse.jinja
/// (3 entities found)
pub fn list_entities_by_term(args: &[String], flags: &HashMap<String, String>) -> ReedResult<ReedResponse<String>>

/// Shows usage statistics.
///
/// ## Flags
/// - --stats: Show detailed statistics
/// - --popular: Show most popular terms
/// - --unused: Show unused terms
///
/// ## Output
/// Taxonomy Usage Statistics:
/// Total terms: 42
/// Total assignments: 128
/// Average terms per entity: 3.2
/// Most used term: technology (42 assignments)
/// Least used term: deprecated-tag (0 assignments)
/// Unused terms: 3
pub fn usage_stats(flags: &HashMap<String, String>) -> ReedResult<ReedResponse<String>>

/// Updates term properties.
///
/// ## Arguments
/// - args[0]: Term ID
///
/// ## Flags
/// - --status: New status (active, deprecated, hidden, pending)
/// - --description: New description
/// - --color: New color
/// - --icon: New icon
///
/// ## Output
/// ✓ Term 2 updated successfully
pub fn update_term(args: &[String], flags: &HashMap<String, String>) -> ReedResult<ReedResponse<String>>

/// Deletes term with safety checks.
///
/// ## Arguments
/// - args[0]: Term ID
///
/// ## Flags
/// - --confirm: Skip confirmation prompt
///
/// ## Safety Checks
/// - Cannot delete if entities assigned
/// - Cannot delete if child terms exist
/// - Prompts for confirmation unless --confirm
///
/// ## Output
/// ⚠ Term 'technology' has 3 child terms: rust, cms, web
/// ⚠ Term 'technology' is assigned to 15 entities
/// ? Delete anyway? This will remove all assignments and child terms. (y/N): _
pub fn delete_term(args: &[String], flags: &HashMap<String, String>) -> ReedResult<ReedResponse<String>>
```

## Implementation Files

### Primary Implementation
- `src/reedcms/cli/taxonomy_commands.rs` - Taxonomy commands
- `src/reedcms/cli/taxonomy_output.rs` - Output formatting

### Test Files
- `src/reedcms/cli/taxonomy_commands.test.rs`
- `src/reedcms/cli/taxonomy_output.test.rs`

## Testing Requirements

### Unit Tests
- [ ] Test term creation
- [ ] Test hierarchical tree display
- [ ] Test term search
- [ ] Test entity assignment
- [ ] Test entity unassignment
- [ ] Test usage statistics

### Integration Tests
- [ ] Test complete taxonomy workflow
- [ ] Test cross-entity search
- [ ] Test circular hierarchy prevention
- [ ] Test term deletion with dependencies

### Performance Tests
- [ ] Term creation: < 20ms
- [ ] Tree rendering: < 100ms for 1000 terms
- [ ] Entity search: < 50ms for 10k entities

## Acceptance Criteria
- [ ] All taxonomy commands implemented
- [ ] Hierarchical tree visualization working
- [ ] Entity assignment to all 8 types supported
- [ ] Fuzzy search functional
- [ ] Usage analytics displayed
- [ ] Safety checks for term deletion
- [ ] All tests pass with 100% coverage
- [ ] Documentation complete
- [ ] BBC English throughout

## Dependencies
- **Requires**: REED-04-01 (CLI Foundation), REED-03-03 (Taxonomy System)

## Blocks
- None

## References
- Service Template: `_workbench/Tickets/templates/service-template.md`
- Service Test Template: `_workbench/Tickets/templates/service-template.test.md`
- Summary: Lines 1099-1109 in `project_summary.md`

## Notes
Taxonomy CLI provides powerful categorization tools. The tree visualization helps understand hierarchical relationships. Entity assignment supports all 8 entity types for universal tagging. Usage analytics help identify popular and unused terms for content strategy optimization.