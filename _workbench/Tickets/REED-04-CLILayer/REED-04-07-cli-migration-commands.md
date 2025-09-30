# REED-04-07: CLI Migration Commands

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
- **ID**: REED-04-07
- **Title**: CLI Migration & Validation Commands
- **Layer**: CLI Layer (REED-04)
- **Priority**: Medium
- **Status**: Open
- **Complexity**: Medium
- **Dependencies**: REED-04-01, REED-02-01

## Summary Reference
- **Section**: CLI Migration & Validation
- **Lines**: 1056-1059 in project_summary.md
- **Key Concepts**: Bulk data migration, consistency validation, data integrity checks

## Objective
Implement migration commands for bulk text/route import and validation commands for data consistency checking across the ReedCMS system.

## Requirements

### Commands to Implement

```bash
# Migration commands
reed migrate:text path/to/text.csv
reed migrate:text path/ --recursive
reed migrate:routes path/to/routes.csv
reed migrate:text --from-json path/data.json

# Validation commands
reed validate:routes
reed validate:consistency
reed validate:text --language de
reed validate:references
```

### Implementation (`src/reedcms/cli/migration_commands.rs`)

```rust
/// Migrates text content from CSV files.
///
/// ## Arguments
/// - args[0]: Path to CSV file or directory
///
/// ## Flags
/// - --recursive: Process directories recursively
/// - --dry-run: Preview changes without applying
/// - --backup: Create backup before migration
///
/// ## CSV Format Expected
/// ```csv
/// key;language;value;description
/// knowledge.title;en;Knowledge Base;Main page title
/// knowledge.title;de;Wissensdatenbank;Hauptseitentitel
/// ```
///
/// ## Process
/// 1. Validate CSV structure
/// 2. Check for duplicates
/// 3. Create backup (if --backup)
/// 4. Import entries
/// 5. Update cache
///
/// ## Output
/// 📦 Migrating text from: path/to/text.csv
/// ✓ Validated 150 entries
/// ✓ Backup created: .reed/backups/text.1704067200.csv.xz
/// ✓ Imported 148 entries
/// ⚠ Skipped 2 duplicates
///
/// Summary:
/// - Total entries: 150
/// - Imported: 148
/// - Skipped: 2
/// - Duration: 1.2s
pub fn migrate_text(args: &[String], flags: &HashMap<String, String>) -> ReedResult<ReedResponse<String>>

/// Migrates route definitions from CSV.
///
/// ## Arguments
/// - args[0]: Path to routes CSV file
///
/// ## Flags
/// - --dry-run: Preview changes without applying
/// - --backup: Create backup before migration
/// - --validate: Validate route conflicts
///
/// ## CSV Format Expected
/// ```csv
/// route;layout;language;description
/// wissen;knowledge;de;German route for knowledge
/// knowledge;knowledge;en;English route for knowledge
/// ```
///
/// ## Output
/// 📦 Migrating routes from: path/to/routes.csv
/// ✓ Validated 50 routes
/// ⚠ Found 2 route conflicts:
///   - /blog already mapped to layout 'news'
///   - /about already mapped to layout 'company'
/// ? Continue with migration? (y/N): _
pub fn migrate_routes(args: &[String], flags: &HashMap<String, String>) -> ReedResult<ReedResponse<String>>
```

### Validation Commands (`src/reedcms/cli/validation_commands.rs`)

```rust
/// Validates route consistency.
///
/// ## Checks
/// 1. Route uniqueness per language
/// 2. Layout existence in registry
/// 3. Template file existence
/// 4. Route format validation
/// 5. Orphaned routes (layout deleted)
///
/// ## Output
/// 🔍 Validating routes...
/// ✓ Route uniqueness: OK (no duplicates)
/// ✓ Layout references: OK (all layouts exist)
/// ⚠ Template files: 2 issues found
///   - knowledge.mouse.jinja: Missing
///   - blog.reader.jinja: Missing
/// ✓ Route format: OK
/// ⚠ Orphaned routes: 1 found
///   - /old-page → deleted-layout (de)
///
/// Summary: 3 issues found
/// Run 'reed validate:routes --fix' to attempt automatic fixes
pub fn validate_routes(flags: &HashMap<String, String>) -> ReedResult<ReedResponse<String>>

/// Validates complete data consistency.
///
/// ## Checks
/// 1. CSV file integrity
/// 2. Foreign key relationships
/// 3. User-role assignments
/// 4. Template-layout mappings
/// 5. Text-route consistency
/// 6. Meta data completeness
/// 7. Taxonomy assignments
/// 8. Backup integrity
///
/// ## Output
/// 🔍 Running comprehensive consistency check...
///
/// CSV Files:
/// ✓ text.csv: 1,250 entries, valid structure
/// ✓ routes.csv: 89 entries, valid structure
/// ✓ users.matrix.csv: 15 entries, valid structure
/// ⚠ meta.csv: 3 entries with invalid format
///
/// Relationships:
/// ✓ User-role assignments: OK
/// ⚠ Template-layout mappings: 2 missing templates
/// ✓ Text-route consistency: OK
/// ⚠ Orphaned taxonomy assignments: 5 found
///
/// Summary: 10 issues found across 3 categories
/// Details saved to: .reed/flow/validation-report.txt
pub fn validate_consistency(flags: &HashMap<String, String>) -> ReedResult<ReedResponse<String>>

/// Validates text content for specific language.
///
/// ## Arguments
/// - --language: Language code to validate
///
/// ## Checks
/// 1. Missing translations
/// 2. Empty values
/// 3. Description quality (length, content)
/// 4. Key format consistency
/// 5. Environment-specific completeness
///
/// ## Output
/// 🔍 Validating text content for language: de
/// ✓ Key format: OK (all valid)
/// ⚠ Missing translations: 15 found
///   - knowledge.subtitle (exists in en, missing in de)
///   - blog.author (exists in en, fr, missing in de)
/// ⚠ Empty values: 2 found
///   - news.title@de
/// ✓ Description quality: OK
///
/// Summary: 17 issues found
/// Completeness: 94.2% (1,235 of 1,310 expected entries)
pub fn validate_text(flags: &HashMap<String, String>) -> ReedResult<ReedResponse<String>>

/// Validates reference integrity.
///
/// ## Checks
/// 1. Layout → Template references
/// 2. Route → Layout references
/// 3. User → Role references
/// 4. Role → Permission references
/// 5. Taxonomy → Entity references
///
/// ## Output
/// 🔍 Validating reference integrity...
/// ✓ Layout references: OK
/// ⚠ Route references: 1 broken reference
///   - /deleted → unknown-layout
/// ✓ User references: OK
/// ⚠ Taxonomy references: 3 orphaned assignments
///   - content:deleted.post → term 5
///
/// Summary: 4 broken references found
pub fn validate_references(flags: &HashMap<String, String>) -> ReedResult<ReedResponse<String>>
```

### Migration Helpers (`src/reedcms/cli/migration_helpers.rs`)

```rust
/// Validates CSV structure before import.
pub fn validate_csv_structure(path: &str, expected_columns: &[&str]) -> ReedResult<()>

/// Detects duplicate entries in import data.
pub fn detect_duplicates(entries: &[CsvEntry]) -> Vec<String>

/// Merges import data with existing data.
pub fn merge_with_existing(new_entries: Vec<CsvEntry>, existing_entries: Vec<CsvEntry>) -> Vec<CsvEntry>

/// Generates migration report.
pub fn generate_migration_report(stats: &MigrationStats) -> String

#[derive(Debug, Clone)]
pub struct MigrationStats {
    pub total_entries: usize,
    pub imported: usize,
    pub skipped: usize,
    pub errors: usize,
    pub duration_ms: u64,
}
```

## Implementation Files

### Primary Implementation
- `src/reedcms/cli/migration_commands.rs` - Migration commands
- `src/reedcms/cli/validation_commands.rs` - Validation commands
- `src/reedcms/cli/migration_helpers.rs` - Helper functions

### Test Files
- `src/reedcms/cli/migration_commands.test.rs`
- `src/reedcms/cli/validation_commands.test.rs`
- `src/reedcms/cli/migration_helpers.test.rs`

## Testing Requirements

### Unit Tests
- [ ] Test CSV structure validation
- [ ] Test duplicate detection
- [ ] Test route conflict detection
- [ ] Test consistency checks
- [ ] Test reference validation

### Integration Tests
- [ ] Test complete migration workflow
- [ ] Test dry-run mode
- [ ] Test backup creation
- [ ] Test validation with real data
- [ ] Test automatic fixes

### Edge Case Tests
- [ ] Test malformed CSV
- [ ] Test missing columns
- [ ] Test circular references
- [ ] Test large dataset migration (10k+ entries)

### Performance Tests
- [ ] Text migration: < 5s for 1000 entries
- [ ] Route validation: < 2s for 100 routes
- [ ] Consistency check: < 10s for full system

## Acceptance Criteria
- [ ] Text migration from CSV working
- [ ] Route migration from CSV working
- [ ] Recursive directory processing
- [ ] Dry-run mode functional
- [ ] Route validation comprehensive
- [ ] Consistency checking thorough
- [ ] Reference integrity validation
- [ ] Automatic backup creation
- [ ] All tests pass with 100% coverage
- [ ] Documentation complete
- [ ] BBC English throughout

## Dependencies
- **Requires**: REED-04-01 (CLI Foundation), REED-02-01 (ReedBase)

## Blocks
- None

## References
- Service Template: `_workbench/Tickets/templates/service-template.md`
- Service Test Template: `_workbench/Tickets/templates/service-template.test.md`
- Summary: Lines 1056-1059 in `project_summary.md`

## Notes
Migration commands are critical for initial system setup and data imports. Dry-run mode prevents accidental data loss. Validation commands help maintain data integrity and catch configuration errors early. Automatic backup creation before migrations provides safety net for rollback if needed.