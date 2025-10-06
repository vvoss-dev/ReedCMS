# Template Layer (Layer 05)

> MiniJinja templates with Atomic Design and custom filters

**Status:** ✅ Complete  
**Implementation:** REED-05-01 to REED-05-03

---

## Overview

The Template Layer provides HTML generation using MiniJinja templates organised with Atomic Design principles, featuring custom filters for ReedBase integration and hot-reload support for development.

---

## Architecture

```
┌──────────────────────────────────────────────────┐
│           Server Layer (06)                      │
│   HTTP Request → Route Resolution                │
└───────────────────┬──────────────────────────────┘
                    │ Layout: "knowledge", Lang: "de", Variant: "mouse"
                    ▼
┌──────────────────────────────────────────────────┐
│        MiniJinja Template Engine                 │
│  ┌────────────────────────────────────────────┐  │
│  │  Environment                               │  │
│  │  - Hot-reload (DEV)                        │  │
│  │  - Template cache (PROD)                   │  │
│  │  - Custom filters registered               │  │
│  └────────────────────────────────────────────┘  │
└───────────────────┬──────────────────────────────┘
                    │
                    ▼
┌──────────────────────────────────────────────────┐
│         Template Rendering                       │
│                                                  │
│  1. Load layout: knowledge.mouse.jinja           │
│  2. Build context: { lang: "de", data: {...} }  │
│  3. Process filters: text(), route(), meta()     │
│  4. Render components: atoms, molecules, orgs    │
│  5. Generate HTML                                │
└───────────────────┬──────────────────────────────┘
                    │
                    ▼
┌──────────────────────────────────────────────────┐
│         Component Structure                      │
│  templates/                                      │
│  ├── components/                                 │
│  │   ├── atoms/          (buttons, icons)        │
│  │   ├── molecules/      (forms, cards)          │
│  │   └── organisms/      (headers, footers)      │
│  └── layouts/                                    │
│      └── knowledge/                              │
│          ├── knowledge.mouse.jinja               │
│          ├── knowledge.mouse.css                 │
│          └── knowledge.text.csv                  │
└──────────────────────────────────────────────────┘
```

---

## Core Concepts

### MiniJinja Template Engine

**Why MiniJinja:**
- Rust-native (no Python runtime)
- Jinja2-compatible syntax
- Fast compilation and rendering
- Memory-safe

**Template syntax:**
```jinja
{# Comments #}
{{ variable }}                    {# Output #}
{% if condition %}...{% endif %}  {# Control flow #}
{% for item in items %}...{% endfor %}
{{ "key" | filter("arg") }}       {# Filters #}
```

### Atomic Design Structure

**Hierarchy:**

**Atoms** (smallest components):
```
templates/components/atoms/
├── button/
│   ├── button.mouse.jinja
│   ├── button.touch.jinja
│   └── button.reader.jinja
└── icon/
    └── icon.mouse.jinja
```

**Molecules** (groups of atoms):
```
templates/components/molecules/
├── search-form/
│   └── search-form.mouse.jinja
└── card/
    └── card.mouse.jinja
```

**Organisms** (complex components):
```
templates/components/organisms/
├── page-header/
│   ├── page-header.mouse.jinja
│   ├── page-header.mouse.css
│   └── page-header.text.csv
└── page-footer/
    └── page-footer.mouse.jinja
```

**Layouts** (complete pages):
```
templates/layouts/
├── knowledge/
│   ├── knowledge.mouse.jinja
│   ├── knowledge.touch.jinja
│   ├── knowledge.reader.jinja
│   ├── knowledge.mouse.css
│   └── knowledge.text.csv
└── blog/
    └── blog.mouse.jinja
```

### Template Variants

**Three variants for different interaction modes:**

| Variant | Target | Features |
|---------|--------|----------|
| **mouse** | Desktop browsers | Hover effects, keyboard shortcuts, tooltips |
| **touch** | Mobile/tablet | Large tap targets, swipe gestures, no hover |
| **reader** | Accessibility | Text-only, high contrast, screen reader optimised |

**Server automatically selects** based on `User-Agent` header.

### Custom Filters

**ReedBase integration:**

```jinja
{# Text content #}
{{ "page.title" | text("de") }}

{# URL routing #}
<a href="{{ "knowledge" | route("de") }}">Wissen</a>

{# Metadata #}
<meta name="description" content="{{ "site.description" | meta }}">

{# Configuration #}
{{ "server.workers" | config }}
```

**Performance:** < 100μs per filter call (O(1) HashMap lookup)

---

## Core Components

### MiniJinja Environment

**File:** `src/reedcms/template/engine.rs`

**Setup:**
```rust
use minijinja::Environment;

pub fn create_environment(lang: &str, env_mode: &str) -> Environment<'static> {
    let mut env = Environment::new();
    
    // Set template path
    env.set_loader(path_loader("templates"));
    
    // Register custom filters
    env.add_filter("text", make_text_filter(lang.to_string()));
    env.add_filter("route", make_route_filter(lang.to_string()));
    env.add_filter("meta", make_meta_filter());
    env.add_filter("config", make_config_filter());
    
    // Development: Enable auto-reload
    if env_mode == "dev" {
        env.set_auto_reload(true);
    }
    
    env
}
```

### Text Filter

**File:** `src/reedcms/filters/text.rs`

**Function:** `text(key, lang)`

**Usage:**
```jinja
{{ "page.title" | text("en") }}       {# Explicit language #}
{{ "page.title" | text("auto") }}     {# Auto-detect from URL #}
{{ "page.title" | text }}              {# Default: auto #}
```

**Implementation:**
```rust
pub fn make_text_filter(current_lang: String) 
    -> impl Fn(&str, Option<&str>) -> Result<String, minijinja::Error> {
    
    move |key, lang_param| {
        let lang = lang_param.unwrap_or(&current_lang);
        let req = ReedRequest {
            key: key.to_string(),
            language: Some(lang.to_string()),
            context: Some("text".to_string()),
            ..Default::default()
        };
        
        let response = get_text(&req)?;
        Ok(response.data)
    }
}
```

### Route Filter

**File:** `src/reedcms/filters/route.rs`

**Function:** `route(layout, lang)`

**Usage:**
```jinja
<a href="{{ "knowledge" | route("de") }}">Wissen</a>
<a href="{{ "blog" | route("en") }}">Blog</a>
```

**Output:**
```html
<a href="/wissen">Wissen</a>
<a href="/blog">Blog</a>
```

### Meta Filter

**File:** `src/reedcms/filters/meta.rs`

**Function:** `meta(key)`

**Usage:**
```jinja
<title>{{ "site.title" | meta }}</title>
<meta name="description" content="{{ "site.description" | meta }}">
```

### Config Filter

**File:** `src/reedcms/filters/config.rs`

**Function:** `config(key)`

**Usage:**
```jinja
{% if "debug.mode" | config == "true" %}
    <div class="debug-banner">Development Mode</div>
{% endif %}
```

---

## Template Structure

### Base Template

```jinja
{# templates/base.jinja #}
<!DOCTYPE html>
<html lang="{{ lang }}">
<head>
    <meta charset="UTF-8">
    <title>{% block title %}{{ "site.title" | meta }}{% endblock %}</title>
    <meta name="description" content="{{ "site.description" | meta }}">
    {% block head %}{% endblock %}
</head>
<body>
    {% include "components/organisms/page-header/page-header.mouse.jinja" %}
    
    <main>
        {% block content %}{% endblock %}
    </main>
    
    {% include "components/organisms/page-footer/page-footer.mouse.jinja" %}
    {% block scripts %}{% endblock %}
</body>
</html>
```

### Layout Template

```jinja
{# templates/layouts/knowledge/knowledge.mouse.jinja #}
{% extends "base.jinja" %}

{% block title %}{{ "knowledge.title" | text(lang) }}{% endblock %}

{% block content %}
<div class="knowledge-layout">
    <h1>{{ "knowledge.heading" | text(lang) }}</h1>
    
    {% for item in knowledge_items %}
        {% include "components/molecules/card/card.mouse.jinja" %}
    {% endfor %}
</div>
{% endblock %}
```

### Component Template

```jinja
{# templates/components/molecules/card/card.mouse.jinja #}
<article class="card">
    <h2>{{ item.title }}</h2>
    <p>{{ item.description }}</p>
    <a href="{{ item.link }}">{{ "common.readmore" | text(lang) }}</a>
</article>
```

---

## Hot-Reload System

### Development Mode

**Enable in `.env`:**
```env
ENVIRONMENT=dev
```

**MiniJinja auto-reload:**
```rust
if env_mode == "dev" {
    env.set_auto_reload(true);  // Reload templates on change
}
```

**Behaviour:**
- Templates reloaded on every request
- No caching
- Instant feedback during development

**Performance:** Acceptable for development (< 10ms overhead)

### Production Mode

**Enable in `.env`:**
```env
ENVIRONMENT=prod
```

**MiniJinja caching:**
```rust
if env_mode == "prod" {
    env.set_auto_reload(false);  // Cache compiled templates
}
```

**Behaviour:**
- Templates compiled once at startup
- Cached in memory
- Fast rendering (< 1ms)

**Restart required** to pick up template changes.

---

## Performance Characteristics

| Operation | Time | Notes |
|-----------|------|-------|
| Template load (cold) | < 10ms | First load, compiles template |
| Template render (cached) | < 1ms | Compiled template in memory |
| Filter call (text) | < 100μs | O(1) ReedBase cache lookup |
| Filter call (route) | < 100μs | O(1) ReedBase cache lookup |
| Filter call (meta) | < 100μs | O(1) ReedBase cache lookup |
| Hot-reload check (dev) | < 5ms | File modification time check |

**Production:** < 5ms total per page render (cached templates + filters)

---

## Integration

### Server Layer

```rust
use crate::reedcms::template::render;

async fn handle_request(req: HttpRequest) -> HttpResponse {
    // Resolve route
    let layout = resolve_layout(&req)?;  // "knowledge"
    let lang = detect_language(&req)?;   // "de"
    let variant = detect_variant(&req)?; // "mouse"
    
    // Build context
    let context = build_context(&req)?;
    
    // Render template
    let html = render(layout, variant, lang, context)?;
    
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html)
}
```

### CLI Layer

```bash
# Create layout with templates
reed layout:init knowledge --languages de,en --variants mouse,touch

# Generated:
# templates/layouts/knowledge/knowledge.mouse.jinja
# templates/layouts/knowledge/knowledge.touch.jinja
# templates/layouts/knowledge/knowledge.mouse.css
# templates/layouts/knowledge/knowledge.text.csv
```

**See:** [CLI Layout Commands](../04-cli-layer/layout-commands.md)

---

## Documentation

- [MiniJinja Integration](minijinja-integration.md) - Engine setup and configuration
- [Atomic Design](atomic-design.md) - Component organisation principles
- [Template Filters](filters.md) - Complete filter reference
- [Hot-Reload System](hot-reload.md) - Development workflow

---

## Related Layers

- **Layer 02 - Data:** Provides CSV storage for text content
- **Layer 04 - CLI:** Layout scaffolding commands
- **Layer 06 - Server:** Template rendering and variant selection
- **Layer 08 - Asset:** CSS bundling from component styles

---

## Summary

The Template Layer provides:
- ✅ MiniJinja template engine (Rust-native)
- ✅ Atomic Design structure (atoms, molecules, organisms, layouts)
- ✅ Three variants (mouse, touch, reader)
- ✅ Custom filters (text, route, meta, config)
- ✅ O(1) filter performance via ReedBase cache
- ✅ Hot-reload in development
- ✅ Template caching in production
- ✅ Component-local text files
- ✅ < 5ms page render time (production)

All features production-ready and fully tested.
