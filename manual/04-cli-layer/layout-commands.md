# Layout Commands

> Create and manage MiniJinja template layouts

**Implementation:** REED-04-06  
**Status:** ✅ Complete  
**File:** `src/reedcms/cli/layout_commands.rs`

---

## Overview

Layout commands create MiniJinja template structures with automatic file generation, registry updates, and default content.

**Global patterns:** See [Common Patterns](common-patterns.md) for flags, output formats, error codes.

---

## Commands

### `reed layout:init`

Create new layout(s) with variants and language support.

```bash
reed layout:init <name...> [options]
```

**Required:**
- `name` - One or more layout names

**Flags:**
- `--languages` - Comma-separated language codes (default: `de,en`)
- `--variants` - Comma-separated variants (default: `mouse,touch,reader`)
- `--routes` - Language:route pairs (default: layout name)
- `--parent` - Parent layout name (inheritance)

**Examples:**
```bash
# Basic layout (default: de,en + all variants)
reed layout:init knowledge

# Specific languages
reed layout:init blog --languages en,fr,es

# Specific variants
reed layout:init simple --variants mouse,touch

# With routes
reed layout:init knowledge --routes de:wissen,en:knowledge

# Multiple layouts at once
reed layout:init blog portfolio projects

# With parent (inheritance)
reed layout:init article --parent content
```

**Generated Files:**
```
templates/layouts/{name}/
├── {name}.mouse.jinja      # Desktop variant
├── {name}.mouse.css
├── {name}.touch.jinja      # Mobile variant
├── {name}.touch.css
├── {name}.reader.jinja     # Reader mode
├── {name}.reader.css
└── {name}.text.csv         # Layout-local text
```

**Registry Update:** Adds entry to `.reed/registry.csv`

**Route Creation:** Creates entries in `.reed/routes.csv` for each language

**Performance:**
- Single layout: < 500ms
- Multiple layouts: < 1s for 5 layouts

---

## Variants

### Mouse (Desktop)
Browser-based navigation, hover effects, keyboard shortcuts.

### Touch (Mobile/Tablet)
Touch-friendly, larger tap targets, swipe gestures.

### Reader (Accessibility)
Text-only, high contrast, screen reader optimised.

**Server automatically selects variant** based on `User-Agent` header.

---

## Layout Structure

### Template File (.jinja)

**Generated boilerplate:**
```jinja
{% extends "base.jinja" %}

{% block title %}{{ "layout-name.title" | text(lang) }}{% endblock %}

{% block content %}
<main class="layout-name">
  <h1>{{ "layout-name.heading" | text(lang) }}</h1>
  <!-- Add content here -->
</main>
{% endblock %}
```

### Stylesheet File (.css)

**Generated boilerplate:**
```css
/* Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0. */
/* SPDX-License-Identifier: Apache-2.0 */

.layout-name {
  /* Add styles here */
}
```

### Text File (.text.csv)

**Generated with default keys:**
```
key|value|description
layout-name.title@de|Titel|Seitentitel
layout-name.title@en|Title|Page title
layout-name.heading@de|Überschrift|Hauptüberschrift
layout-name.heading@en|Heading|Main heading
```

**Format:** See [Common Patterns → CSV File Locations](common-patterns.md#csv-file-locations)

---

## Registry System

### registry.csv

Tracks all layouts for server routing and template engine.

**Structure:**
```
layout|variants|languages|parent|created_at
```

**Example:**
```
knowledge|mouse,touch,reader|de,en||2025-01-01T00:00:00Z
article|mouse,touch,reader|de,en,fr|content|2025-01-02T00:00:00Z
```

**Automatic:** Updated by `layout:init`, read by server on startup.

---

## Layout Inheritance

**Parent-child relationships** allow layout reuse:

```bash
# Create base content layout
reed layout:init content --variants mouse,touch

# Create article that inherits from content
reed layout:init article --parent content
```

**Template inheritance (MiniJinja):**
```jinja
{% extends "layouts/content/content.mouse.jinja" %}

{% block article_content %}
  <!-- Article-specific content -->
{% endblock %}
```

---

## Common Workflows

### New Website Layout

```bash
# Create main layouts
reed layout:init home about contact

# Set routes
reed route:set home@de "" --desc "Homepage"
reed route:set home@en "" --desc "Homepage"
reed route:set about@de "ueber" --desc "About page"
reed route:set about@en "about" --desc "About page"

# Add content
reed text:set home.title@de "Willkommen"
reed text:set home.title@en "Welcome"
```

### Blog System

```bash
# Create blog layouts
reed layout:init blog --variants mouse,touch
reed layout:init article --parent blog

# Set routes
reed route:set blog@en "blog"
reed route:set article@en "blog/:slug"
```

### Multi-Language Site

```bash
# Create with all languages
reed layout:init product --languages de,en,fr,es,it

# Set language-specific routes
reed route:set product@de "produkt"
reed route:set product@en "product"
reed route:set product@fr "produit"
reed route:set product@es "producto"
reed route:set product@it "prodotto"
```

---

## Validation Rules

**Layout name:**
- 3-32 characters
- Lowercase letters, hyphens only
- Must be unique
- Examples: `knowledge`, `blog-post`, `user-profile`

**Language codes:**
- 2 characters (ISO 639-1)
- Lowercase
- Examples: `de`, `en`, `fr`, `es`, `it`

**Variants:**
- Must be: `mouse`, `touch`, or `reader`
- At least one required

**Validation:** See [Common Patterns → Common Validation Rules](common-patterns.md#common-validation-rules)

---

## Integration

### Server Layer

Server reads `.reed/registry.csv` on startup, maps routes to layouts, selects variant based on `User-Agent`.

### Template Layer

MiniJinja renders layouts with:
- Text filter: `{{ "key" | text(lang) }}`
- Route filter: `{{ "layout" | route(lang) }}`
- Meta filter: `{{ "key" | meta }}`

**See:** [Template Layer](../05-template-layer/) for filter details

---

## Performance

| Operation | Time | Notes |
|-----------|------|-------|
| `init` single | < 500ms | Creates 7 files + registry update |
| `init` batch (5) | < 1s | Parallel creation |

**File operations:** Creates directories, writes templates, updates registry atomically

---

## Best Practices

**Descriptive names:**
```bash
# ✅ Good - clear purpose
reed layout:init user-profile blog-article product-detail

# ❌ Bad - generic names
reed layout:init page1 page2 page3
```

**Consistent routing:**
```bash
# ✅ Good - logical hierarchy
reed route:set blog@en "blog"
reed route:set article@en "blog/:slug"

# ❌ Bad - inconsistent
reed route:set blog@en "posts"
reed route:set article@en "content/:id"
```

**Use inheritance:**
```bash
# ✅ Good - DRY principle
reed layout:init base --variants mouse,touch
reed layout:init article --parent base
reed layout:init product --parent base
```

---

**See also:**
- [Common Patterns](common-patterns.md) - Global flags, validation
- [Data Commands](data-commands.md) - Manage layout content
- [Template Layer](../05-template-layer/) - MiniJinja rendering
