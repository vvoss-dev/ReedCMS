# Data Commands

> Manage text, routes, and metadata in ReedBase

**Implementation:** REED-04-02  
**Status:** ✅ Complete  
**File:** `src/reedcms/cli/data_commands.rs`

---

## Overview

Data commands provide access to ReedBase, the CSV-based key-value store. All content (text, routes, metadata) is managed through these commands.

**Global patterns:** See [Common Patterns](common-patterns.md) for flags (`--help`, `--verbose`, `--json`, `--dry-run`, `--force`), output formats, error codes, and validation rules.

---

## Text Operations

### `reed text:set`

Store text content with language suffix.

```bash
reed text:set <key@lang> <value> [--desc <description>]
```

**Example:**
```bash
reed text:set page.title@en "Welcome" --desc "Homepage title"
reed text:set page.title@de "Willkommen"
```

**File:** `.reed/text.csv`  
**Performance:** < 50ms (includes backup)

---

### `reed text:get`

Retrieve text content.

```bash
reed text:get <key@lang>
```

**Example:**
```bash
reed text:get page.title@en
# Output: Welcome
```

**Fallback chain:** `key@lang@env` → `key@lang` → `key@env` → `key`  
**Performance:** < 100μs (cached), < 10ms (uncached)

---

### `reed text:list`

List all text keys matching a pattern.

```bash
reed text:list [pattern]
```

**Examples:**
```bash
reed text:list              # All keys
reed text:list "*@en"       # English only
reed text:list "page.*"     # Page-related
```

**Performance:** < 10ms for 1000 keys

---

## Route Operations

### `reed route:set`

Store URL route for a layout.

```bash
reed route:set <layout@lang> <path> [--desc <description>]
```

**Example:**
```bash
reed route:set knowledge@en "knowledge" --desc "English knowledge page"
reed route:set knowledge@de "wissen"     --desc "German knowledge page"
```

**Mapping:** `knowledge@en` → `/knowledge`, `knowledge@de` → `/wissen`  
**File:** `.reed/routes.csv`

---

### `reed route:get`

Retrieve route for a layout.

```bash
reed route:get <layout@lang>
```

**Example:**
```bash
reed route:get knowledge@en
# Output: knowledge
```

---

## Metadata Operations

### `reed meta:set`

Store metadata (SEO, technical configuration).

```bash
reed meta:set <key> <value> [--desc <description>]
```

**Examples:**
```bash
reed meta:set site.title "ReedCMS"
reed meta:set cache.ttl "3600"
reed meta:set og.image "/images/og.jpg"
```

**File:** `.reed/meta.csv`  
**Note:** No language suffix (metadata is global)

---

### `reed meta:get`

Retrieve metadata value.

```bash
reed meta:get <key>
```

---

## Configuration Operations

### `reed server:set`

Set server configuration value.

```bash
reed server:set <key> <value> [--desc <description>]
```

**Examples:**
```bash
reed server:set workers "8"
reed server:set timeout "30"
reed server:set enable_cors "true"
```

**File:** `.reed/server.csv`  
**Note:** Server binding (port/socket) uses `ENVIRONMENT` from `.env`, not CSV

---

### `reed project:set`

Set project configuration value.

```bash
reed project:set <key> <value> [--desc <description>]
```

**Examples:**
```bash
reed project:set name "My Website"
reed project:set default_language "en"
reed project:set available_languages "en,de,fr"
```

**File:** `.reed/project.csv`

---

## CSV File Formats

### text.csv
```
key|value|description
page.title@en|Welcome|Homepage title
page.title@de|Willkommen|German title
```

### routes.csv
```
layout@language|path|description
knowledge@en|knowledge|English knowledge page
knowledge@de|wissen|German knowledge page
home@en||Root URL (empty path)
```

### meta.csv
```
key|value|description
site.title|ReedCMS|Site title
cache.ttl|3600|Cache TTL seconds
```

**Format details:** See [Common Patterns → CSV File Locations](common-patterns.md#csv-file-locations)

---

## Template Integration

Data commands store values accessed via MiniJinja filters:

```jinja
<h1>{{ "page.title" | text("en") }}</h1>
<a href="{{ "knowledge" | route("de") }}">/wissen</a>
<meta property="og:image" content="{{ "og.image" | meta }}">
```

---

## Backup System

**Location:** `.reed/backups/`  
**Format:** XZ-compressed CSV  
**Retention:** Last 32 backups  
**Naming:** `{filename}.{timestamp}.csv.xz`

All write operations automatically create backups before modification.

---

## Common Workflows

### Multi-Language Content

```bash
# English
reed text:set page.title@en "Welcome"
reed route:set home@en ""

# German
reed text:set page.title@de "Willkommen"
reed route:set home@de ""
```

### Content Audit

```bash
# List all English keys
reed text:list "*@en" > english_keys.txt

# Find missing German translations
comm -23 <(reed text:list "*@en" | sed 's/@en/@de/') \
         <(reed text:list "*@de")
```

### Bulk Import

```bash
while IFS='|' read -r key value desc; do
    reed text:set "$key" "$value" --desc "$desc"
done < content.csv
```

---

## Performance

| Operation | Cached | Uncached | Writes |
|-----------|--------|----------|--------|
| `text:get` | < 100μs | < 10ms | - |
| `text:set` | - | - | < 50ms |
| `text:list` | < 10ms | - | - |

**Cache:** O(1) HashMap lookups  
**Writes:** Include CSV write + XZ backup creation

---

## Best Practices

**Consistent naming:**
```bash
# ✅ Good
reed text:set page.header.navigation.home@en "Home"

# ❌ Bad
reed text:set nav1@en "Home"
```

**Always describe:**
```bash
# ✅ Good
reed text:set page.cta@en "Get Started" --desc "Main CTA button"
```

**Environment suffixes:**
```bash
reed text:set debug.info@dev "Debug active"
reed text:set analytics.id@prod "UA-12345-1"
```

---

**See also:**
- [Common Patterns](common-patterns.md) - Global flags, errors, validation
- [Layout Commands](layout-commands.md) - Create layouts using this content
- [Config Commands](config-commands.md) - System configuration
