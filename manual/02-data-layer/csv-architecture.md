# CSV Architecture

> Design philosophy and format specifications for CSV-based storage

---

## Philosophy

### Why CSV Instead of Database?

**Git-Friendly:**
```bash
# Easy diffs
$ git diff .reed/text.csv
- page.title@en|Welcome|Homepage title
+ page.title@en|Welcome to ReedCMS|Updated homepage title

# Merge conflicts are visible
$ git merge feature-branch
CONFLICT in .reed/text.csv
```

**Human-Readable:**
```csv
page.title@en|Welcome|Homepage title
page.header.logo.alt@en|ReedCMS Logo|Logo image alt text
footer.copyright@en|© 2025 Company|Copyright notice
```

Anyone can read, edit, validate CSV files without special tools.

**Zero Dependencies:**
- No database server to install
- No connection pooling
- No ORM complexity
- Just file I/O

**Performance for Small Datasets:**
- < 10,000 records: CSV is faster than SQLite
- Sequential read: < 10ms for 3,000 lines
- Hash

map cache: O(1) lookups after load

**When to Migrate:**
- \> 10,000 records per file
- Complex queries (JOINs, aggregations)
- Multi-user concurrent writes

---

## CSV Format Specification

### Delimiter

**Pipe character:** `|`

**Why pipe?**
- Rarely used in natural language content
- More reliable than comma (used in text)
- More reliable than tab (invisible, formatting issues)
- Easy to escape if needed: `\|`

### File Structure

```csv
key|value|description
page.title@en|Welcome|Homepage title
page.subtitle@en|High-performance CMS|Subtitle text
```

**Header row:** Required, defines column names  
**Data rows:** One record per line  
**Empty lines:** Ignored  
**Comments:** Lines starting with `#` ignored

### Encoding

**UTF-8 without BOM**

**Line endings:** Unix-style (`\n`)

---

## CSV Types

### Type 1: Simple Key-Value

**Format:** `key|value|description`

**Three columns:**
1. `key` - Unique identifier with optional suffixes
2. `value` - Content (any UTF-8 string)
3. `description` - Human-readable documentation

**Files:**
- `.reed/text.csv` - All text content
- `.reed/routes.csv` - URL routing definitions
- `.reed/meta.csv` - SEO and technical metadata
- `.reed/server.csv` - Server configuration
- `.reed/project.csv` - Project settings

**Example (.reed/text.csv):**
```csv
key|value|description
page.title@en|Welcome|Homepage title in English
page.title@de|Willkommen|Homepage title in German
page.header.logo.alt@en|ReedCMS Logo|Logo alt text for accessibility
page.hero.cta@en|Get Started|Call-to-action button text
```

**Example (.reed/routes.csv):**
```csv
layout@language|path|description
knowledge@en|knowledge|English knowledge base URL
knowledge@de|wissen|German knowledge base URL
home@en||Homepage (empty path = root)
```

### Type 2: Matrix (Lists)

**Format:** Multiple columns with comma-separated lists in specific fields

**Files:**
- `.reed/users.matrix.csv` - User accounts
- `.reed/roles.matrix.csv` - RBAC roles

**Example (.reed/users.matrix.csv):**
```csv
username|password|roles|firstname|lastname|email
admin|$argon2id$...|admin,editor,viewer|John|Doe|admin@example.com
jdoe|$argon2id$...|editor|Jane|Doe|jane@example.com
```

**List columns:**
- `roles` - Comma-separated role list
- No spaces after commas: `admin,editor,viewer` ✅
- With spaces: `admin, editor, viewer` ❌

**Example (.reed/roles.matrix.csv):**
```csv
rolename|permissions|description
admin|text[rwx],user[rwx],role[rwx],system[rwx]|Full system administrator
editor|text[rwx],content[rwx],route[rw-]|Content editor
viewer|text[r--],content[r--]|Read-only access
```

**List columns:**
- `permissions` - Comma-separated permission list
- Format: `resource[rwx]`

---

## Key Naming Convention

### Structure

```
component.element.property@language@environment
```

**Example:**
```
page.header.navigation.home@en@dev
│    │      │          │    │  │
│    │      │          │    │  └─ Environment (optional)
│    │      │          │    └──── Language (optional)
│    │      │          └───────── Property/element
│    │      └──────────────────── Sub-component
│    └─────────────────────────── Component
└──────────────────────────────── Namespace
```

### Rules

**Lowercase only:**
```
✅ page.title@en
❌ Page.Title@EN
❌ PAGE.TITLE@en
```

**Dots as separators:**
```
✅ page.header.logo.title
❌ page_header_logo_title
❌ page-header-logo-title
```

**Language suffix (2 chars, lowercase):**
```
✅ @en, @de, @fr, @es
❌ @EN, @english, @deu
```

**Environment suffix (lowercase):**
```
✅ @dev, @prod, @christmas
❌ @DEV, @PROD, @Christmas
```

**Nesting depth:**
- Optimal: 4 levels (`page.header.logo.title`)
- Maximum: 8 levels
- Avoid deep nesting (hard to maintain)

### Examples

**Global components:**
```
page-header.logo.title@en
page-header.logo.alt@en
page-header.navigation.home@en
page-header.navigation.about@en
footer.copyright.text@de
footer.social.twitter.label@en
```

**Layout-specific content:**
```
knowledge.hero.title@en
knowledge.hero.subtitle@en
knowledge.section.intro@en
blog.article.readmore@de
```

**Environment-specific:**
```
debug.mode@dev                    # Only in dev
analytics.id@prod                 # Only in prod
banner.text@christmas             # Seasonal variant
```

---

## File Locations

### Directory Structure

```
.reed/
├── text.csv              # All content text (Type 1)
├── routes.csv            # URL routing (Type 1)
├── meta.csv              # SEO metadata (Type 1)
├── server.csv            # Server config (Type 1)
├── project.csv           # Project settings (Type 1)
├── registry.csv          # Layout registry (Type 1)
├── users.matrix.csv      # User accounts (Type 2)
├── roles.matrix.csv      # RBAC roles (Type 2)
├── user_roles.csv        # User-role assignments (Type 1)
└── backups/              # XZ-compressed backups
    ├── text.csv.1704067200.csv.xz
    ├── routes.csv.1704067200.csv.xz
    └── ...
```

### Path Convention

**Absolute project root:** `.reed/` directory in project root

**CLI access:**
```bash
# Commands automatically use .reed/ prefix
reed text:get page.title@en         # Reads .reed/text.csv
reed route:set knowledge@de wissen  # Writes .reed/routes.csv
```

**Rust access:**
```rust
use std::env;

let project_root = env::current_dir()?;
let text_path = project_root.join(".reed/text.csv");
let routes_path = project_root.join(".reed/routes.csv");
```

---

## Atomic Write Operations

### Problem

**Non-atomic write:**
```rust
// ❌ BAD - Can leave corrupted file
fs::write(".reed/text.csv", new_content)?;
// If crash happens here, file may be incomplete
```

### Solution

**Temp file + rename (atomic on Unix):**
```rust
// ✅ GOOD - Atomic operation
let temp_path = ".reed/text.csv.tmp";
fs::write(temp_path, new_content)?;    // Write to temp
fs::rename(temp_path, ".reed/text.csv")?; // Atomic rename
```

**Benefits:**
- Original file untouched until write complete
- Rename is atomic on Unix/Linux/macOS
- Crash during write leaves original intact
- No partial/corrupted files

**Implementation:**
Every `set()` operation in ReedBase uses atomic write pattern.

---

## Validation Rules

### Key Validation

**Rules:**
- 3-255 characters
- Lowercase letters, dots, hyphens only
- Must not start/end with dot
- Language suffix: 2 lowercase letters
- Environment suffix: 3-12 lowercase letters

**Regex:**
```regex
^[a-z][a-z0-9.-]*[a-z0-9](@[a-z]{2})?(@[a-z]{3,12})?$
```

**Examples:**
```
✅ page.title@en
✅ page.header.logo.alt@de@dev
✅ footer.copyright@christmas
❌ .page.title        (starts with dot)
❌ page.title.        (ends with dot)
❌ Page.Title         (uppercase)
❌ page_title         (underscore)
❌ page.title@EN      (language uppercase)
```

### Value Validation

**Text values:**
- Any UTF-8 string
- Pipe character must be escaped: `\|`
- Newlines must be escaped: `\n`
- No maximum length (practical limit: 10 KB)

**Route values:**
- Lowercase letters, numbers, hyphens, slashes
- No leading slash: `knowledge` ✅, `/knowledge` ❌
- No trailing slash: `blog/posts` ✅, `blog/posts/` ❌
- Empty string valid (= root URL)

### File Validation

**On read:**
```rust
// Check header
let first_line = lines.next()?;
if first_line != "key|value|description" {
    return Err(ReedError::ParseError { ... });
}

// Validate each row
for line in lines {
    let parts: Vec<&str> = line.split('|').collect();
    if parts.len() != 3 {
        return Err(ReedError::ParseError { ... });
    }
}
```

---

## Performance Characteristics

### Read Performance

| Records | Sequential Read | HashMap Init | Lookup (Cached) |
|---------|----------------|--------------|-----------------|
| 100     | < 1ms          | < 1ms        | < 100μs         |
| 1,000   | < 5ms          | < 5ms        | < 100μs         |
| 3,000   | < 10ms         | < 10ms       | < 100μs         |
| 10,000  | < 30ms         | < 30ms       | < 100μs         |

**Cache hit ratio:** > 95% in production (most keys accessed repeatedly)

### Write Performance

| Operation | Time | Notes |
|-----------|------|-------|
| Atomic write (1 KB) | < 10ms | Temp + rename |
| XZ compression (1 KB) | < 20ms | LZMA2 algorithm |
| Complete write + backup | < 50ms | Including both |

### Memory Usage

**Per entry:** ~100 bytes (key + value + HashMap overhead)

**3,000 entries:** ~300 KB memory

**Scaling:**
- Linear memory growth
- Consider SQLite beyond 10,000 entries

---

## Migration Path

### From CSV to Database

**When to migrate:**
- \> 10,000 records
- Complex queries needed
- Multi-user concurrent writes
- Need transactions

**Recommended:** SQLite (still single-file, Git-friendly)

**Migration strategy:**
```rust
// 1. Create SQLite schema
CREATE TABLE text (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    description TEXT
);

// 2. Import CSV
for line in csv_lines {
    let (key, value, desc) = parse_line(line);
    db.execute("INSERT INTO text VALUES (?, ?, ?)", 
               [key, value, desc])?;
}

// 3. Update ReedBase to use SQLite
```

**Maintain CSV for:**
- Small projects (< 5,000 entries)
- Configuration files
- Version-controlled content

---

## Best Practices

**Version control CSV files:**
```bash
# ✅ Track changes
git add .reed/*.csv
git commit -m "feat: add German translations"
```

**Don't commit backups:**
```gitignore
# .gitignore
.reed/backups/
```

**Regular exports:**
```bash
# Monthly backup to JSON
reed text:export backup-$(date +%Y%m).json
```

**Validate before deploy:**
```bash
# Check CSV syntax
reed config:validate
```

**Use descriptions:**
```csv
# ✅ Good - documented
page.title@en|Welcome|Homepage title for English visitors

# ❌ Bad - no context
page.title@en|Welcome|
```

---

**See also:**
- [ReedBase Cache](reedbase-cache.md) - Cache implementation
- [Backup System](backup-system.md) - XZ compression and retention
- [Data Operations](data-operations.md) - API reference
