# CSV File Formats

Complete specification for all `.reed/` CSV files in ReedCMS.

## General Format

**Delimiter**: Pipe (`|`)  
**Encoding**: UTF-8  
**Line Endings**: LF (`\n`)  
**Quoting**: Only when value contains `|` or `\n`

## Common Structure

```
key|value|comment
```

- **key**: Dot-notation lowercase identifier with optional `@lang` suffix
- **value**: Content or configuration value
- **comment**: Human-readable description

## text.csv

**Purpose**: All content text (UI labels, article content, etc.)

**Format**:
```
key|value|comment
```

**Key Format**: `component.element.property@lang`

**Examples**:
```
knowledge.title@en|Knowledge Base|English page title
knowledge.title@de|Wissensdatenbank|German page title
blog.intro@en|Latest articles|English introduction
actix.web.pros.1@en|Extremely fast|First advantage point
```

**Key Patterns**:
- Global text: `section.element@lang`
- Component text: `component.element@lang`
- Nested content: Max 8 levels recommended
- Language suffix: Always lowercase `@en`, `@de`

## routes.csv

**Purpose**: URL routing definitions

**Format**:
```
key|value|comment
```

**Key Format**: `layout-name@lang`

**Examples**:
```
landing@en||English homepage (root path)
knowledge@en|knowledge|English knowledge section
knowledge@de|wissen|German knowledge section
actix-web@en|knowledge/actix-web-framework|Article route
```

**Rules**:
- Empty value = root path for language
- No leading `/` in routes
- Language-specific paths supported

## meta.csv

**Purpose**: SEO and technical metadata

**Format**:
```
key|value|comment
```

**Key Format**: `layout.meta-type@lang`

**Examples**:
```
knowledge.title@en|Knowledge Base - ReedCMS|SEO title
knowledge.description@en|Learn about web technologies|Meta description
knowledge.keywords@en|knowledge,documentation,web|Meta keywords
knowledge.cache@en|3600|Cache duration in seconds
```

**Meta Types**:
- `title`: Page title (SEO)
- `description`: Meta description
- `keywords`: Meta keywords (comma-separated)
- `cache`: Cache duration in seconds
- `robots`: Robots directives
- `canonical`: Canonical URL

## server.csv

**Purpose**: Server configuration

**Format**:
```
key|value|comment
```

**Examples**:
```
server.port|3000|HTTP port
server.host|127.0.0.1|Bind address
server.workers|4|Number of worker threads
server.max_connections|10000|Maximum concurrent connections
server.timeout|30|Request timeout in seconds
```

**Key Patterns**:
- `server.*`: Server settings
- `security.*`: Security settings
- `cors.*`: CORS configuration

## project.csv

**Purpose**: Project-wide settings

**Format**:
```
key|value|comment
```

**Examples**:
```
project.name|ReedCMS|Project name
project.version|0.1.0|Project version
project.languages|en,de|Supported languages
project.default_language|en|Default language
project.session_hash|a7f3b2c8|Current asset session hash
```

**Key Patterns**:
- `project.*`: Project metadata
- `build.*`: Build configuration
- `deployment.*`: Deployment settings

## registry.csv

**Purpose**: Layout registry and metadata

**Format**:
```
key|value|comment
```

**Examples**:
```
layouts.available|landing,knowledge,blog,portfolio|Available layouts
layouts.landing.variants|mouse,touch,reader|Supported variants
layouts.knowledge.template|knowledge.mouse.jinja|Main template path
```

## Environment Suffixes

All CSV files support environment-specific overrides:

```
# Base value
server.port|3000|Default port

# Development override
server.port@dev|8333|Development port

# Production override
server.port@prod|80|Production port
```

**Resolution Order** (4-step fallback):
1. `key@environment@language` (e.g., `title@prod@de`)
2. `key@environment` (e.g., `title@prod`)
3. `key@language` (e.g., `title@de`)
4. `key` (e.g., `title`)

## Validation Rules

### Key Format

```rust
// Valid keys
"knowledge.title@en"         // ✓ Component + language
"server.port"                 // ✓ Config key
"actix.web.pros.1@de"        // ✓ Nested content

// Invalid keys
"Knowledge.Title@EN"          // ✗ Uppercase
"knowledge_title"             // ✗ Underscores
"knowledge.title.@en"         // ✗ Empty segment
```

### Value Format

```
# Plain values
knowledge.title@en|Knowledge Base|Comment

# Values with pipes (quoted)
csv.delimiter|"|"|Pipe character

# Values with newlines (quoted)
long.text@en|"Line 1
Line 2
Line 3"|Multi-line content
```

### Comment Format

Comments are optional but recommended for documentation:

```
# Good comments
knowledge.title@en|Knowledge Base|English page title
server.port|3000|HTTP listening port (1024-65535)

# Minimal comments
cache.ttl|3600|Cache duration

# No comment
temp.value|42|
```

## File Operations

### Atomic Writes

All CSV writes use atomic pattern:

```rust
// 1. Write to temporary file
write_csv_temp(".reed/text.csv.tmp", &data)?;

// 2. Rename atomically
fs::rename(".reed/text.csv.tmp", ".reed/text.csv")?;
```

### Backup Before Write

```rust
// Automatic backup
create_backup(".reed/text.csv")?;  // → .reed/backups/text.csv.{timestamp}.xz

// Then write
write_csv(".reed/text.csv", &data)?;
```

### Lock-Free Reads

```rust
// Multiple concurrent readers supported
let data1 = read_csv(".reed/text.csv")?;
let data2 = read_csv(".reed/text.csv")?;  // No blocking
```

## Migration

### CSV Format Migration

When changing CSV structure:

```bash
# Backup first
reed data:backup --all

# Run migration
reed migrate:csv-format --from=v1 --to=v2 --dry-run
reed migrate:csv-format --from=v1 --to=v2

# Verify
reed data:verify --all
```

### Key Renaming

```bash
# Rename keys (preserves values and comments)
reed data:rename \
  --from="knowledge.title" \
  --to="kb.title" \
  --dry-run
```

## Performance

| Operation | Timing | Note |
|-----------|--------|------|
| Read CSV | < 5ms | 1000 rows |
| Write CSV | < 10ms | 1000 rows |
| Parse line | < 1μs | Single row |
| Validate key | < 0.1μs | Regex match |

## See Also

- [Data Operations](../02-data-layer/data-operations.md) - CSV CRUD operations
- [ReedBase Cache](../02-data-layer/reedbase-cache.md) - Cache system
- [Backup System](../02-data-layer/backup-system.md) - Backup/restore
