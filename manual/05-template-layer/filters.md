# Template Filters

> Custom MiniJinja filters for ReedBase integration

---

## Overview

ReedCMS provides four custom filters for accessing ReedBase data from templates: `text`, `route`, `meta`, and `config`.

**Performance:** < 100μs per filter call (O(1) HashMap lookup)

---

## Text Filter

### Syntax

```jinja
{{ key | text(language) }}
{{ key | text("auto") }}      {# Auto-detect from URL #}
{{ key | text }}               {# Default: auto #}
```

### Purpose

Retrieve text content from `.reed/text.csv` with language support.

### Examples

**Explicit language:**
```jinja
<h1>{{ "page.title" | text("en") }}</h1>
<!-- Output: Welcome -->

<h1>{{ "page.title" | text("de") }}</h1>
<!-- Output: Willkommen -->
```

**Auto-detect from URL:**
```jinja
{# URL: /de/wissen #}
<h1>{{ "page.title" | text("auto") }}</h1>
<!-- Output: Willkommen (German from URL) -->

{# URL: /en/knowledge #}
<h1>{{ "page.title" | text("auto") }}</h1>
<!-- Output: Welcome (English from URL) -->
```

**Default (auto):**
```jinja
<h1>{{ "page.title" | text }}</h1>
<!-- Uses language from current URL -->
```

### Environment Fallback

**Supports 4-step fallback chain:**
```
1. page.title@en@dev   (key + language + environment)
2. page.title@en       (key + language)
3. page.title@dev      (key + environment)
4. page.title          (base key)
```

**Example:**
```jinja
{# In development, shows dev-specific banner #}
{{ "banner.text" | text }}

{# CSV entries: #}
{# banner.text@dev|Debug Mode Active #}
{# banner.text|Welcome #}
```

### Error Handling

**Key not found:**
```jinja
{{ "invalid.key" | text("en") }}
<!-- Error: Key not found: invalid.key@en -->
```

**Template fails to render** - error displayed based on environment (dev/prod).

---

## Route Filter

### Syntax

```jinja
{{ layout | route(language) }}
{{ layout | route("auto") }}
{{ layout | route }}
```

### Purpose

Retrieve URL path for a layout from `.reed/routes.csv`.

### Examples

**Explicit language:**
```jinja
<a href="{{ "knowledge" | route("de") }}">Wissen</a>
<!-- Output: <a href="/wissen">Wissen</a> -->

<a href="{{ "knowledge" | route("en") }}">Knowledge</a>
<!-- Output: <a href="/knowledge">Knowledge</a> -->
```

**Auto-detect:**
```jinja
{# URL: /de/... #}
<a href="{{ "knowledge" | route }}">{{ "nav.knowledge" | text }}</a>
<!-- Output: <a href="/wissen">Wissen</a> -->
```

**Root URL (empty path):**
```jinja
<a href="{{ "home" | route("en") }}">Home</a>
<!-- Output: <a href="/">Home</a> -->
```

### Multi-Language Navigation

```jinja
<nav>
    <a href="{{ "home" | route(lang) }}">{{ "nav.home" | text(lang) }}</a>
    <a href="{{ "knowledge" | route(lang) }}">{{ "nav.knowledge" | text(lang) }}</a>
    <a href="{{ "blog" | route(lang) }}">{{ "nav.blog" | text(lang) }}</a>
</nav>
```

---

## Meta Filter

### Syntax

```jinja
{{ key | meta }}
```

### Purpose

Retrieve metadata from `.reed/meta.csv` (no language suffix).

### Examples

**SEO metadata:**
```jinja
<title>{{ "site.title" | meta }}</title>
<meta name="description" content="{{ "site.description" | meta }}">
<meta name="keywords" content="{{ "site.keywords" | meta }}">
```

**OpenGraph tags:**
```jinja
<meta property="og:title" content="{{ "og.title" | meta }}">
<meta property="og:description" content="{{ "og.description" | meta }}">
<meta property="og:image" content="{{ "og.image" | meta }}">
<meta property="og:type" content="{{ "og.type" | meta }}">
```

**Technical metadata:**
```jinja
{% if "cache.enabled" | meta == "true" %}
    <meta http-equiv="Cache-Control" content="max-age={{ 'cache.ttl' | meta }}">
{% endif %}
```

### Common Keys

| Key | Description |
|-----|-------------|
| `site.title` | Site name |
| `site.description` | Site description |
| `site.keywords` | SEO keywords |
| `og.title` | OpenGraph title |
| `og.description` | OpenGraph description |
| `og.image` | OpenGraph image URL |
| `og.type` | OpenGraph type (website, article) |
| `cache.ttl` | Cache time-to-live (seconds) |
| `robots` | Robots directives |

---

## Config Filter

### Syntax

```jinja
{{ key | config }}
```

### Purpose

Retrieve configuration values from `.reed/server.csv` and `.reed/project.csv`.

### Examples

**Debug mode:**
```jinja
{% if "debug.mode" | config == "true" %}
    <div class="debug-banner">
        Development Mode Active
    </div>
{% endif %}
```

**Feature flags:**
```jinja
{% if "features.comments" | config == "enabled" %}
    {% include "components/organisms/comments/comments.mouse.jinja" %}
{% endif %}
```

**Configuration display:**
```jinja
<footer>
    Version: {{ "project.version" | config }}
    <br>
    Workers: {{ "server.workers" | config }}
</footer>
```

### Common Keys

**Project configuration:**
- `project.name` - Project name
- `project.version` - Version number
- `project.url` - Site URL
- `project.default_language` - Default language

**Server configuration:**
- `server.workers` - Worker thread count
- `server.timeout` - Request timeout
- `debug.mode` - Debug flag

---

## Filter Chaining

### Combining Filters

```jinja
{# Text + default value #}
{{ "optional.key" | text | default("Fallback") }}

{# Text + uppercase #}
{{ "page.title" | text("en") | upper }}
<!-- Output: WELCOME -->

{# Route + absolute URL #}
<a href="{{ base_url }}{{ "knowledge" | route("de") }}">
```

### Built-in + Custom

**MiniJinja built-in filters:**
```jinja
{{ text | upper }}           {# UPPERCASE #}
{{ text | lower }}           {# lowercase #}
{{ text | capitalize }}      {# Capitalize First #}
{{ number | abs }}           {# Absolute value #}
{{ array | length }}         {# Length #}
{{ value | default("def") }} {# Default value #}
```

**Combining:**
```jinja
{{ "page.title" | text("en") | upper }}
{{ "page.items" | text("en") | split(",") | length }}
```

---

## Performance Optimization

### Cache Filter Results

**❌ Bad - Repeated calls:**
```jinja
<title>{{ "page.title" | text(lang) }}</title>
<h1>{{ "page.title" | text(lang) }}</h1>
<meta property="og:title" content="{{ "page.title" | text(lang) }}">
```

**✅ Good - Cache in variable:**
```jinja
{% set title = "page.title" | text(lang) %}
<title>{{ title }}</title>
<h1>{{ title }}</h1>
<meta property="og:title" content="{{ title }}">
```

### Batch Filter Calls

**❌ Bad - Loop with filter:**
```jinja
{% for item in items %}
    {{ "item.label" | text(lang) }}: {{ item.value }}
{% endfor %}
```

**✅ Good - Filter outside loop:**
```jinja
{% set label = "item.label" | text(lang) %}
{% for item in items %}
    {{ label }}: {{ item.value }}
{% endfor %}
```

---

## Error Handling

### Missing Keys

**Development mode:**
```
Filter Error: text
Key: invalid.key@en
Reason: Key not found in .reed/text.csv
Template: layouts/knowledge/knowledge.mouse.jinja
Line: 42
```

**Production mode:**
```
500 Internal Server Error
```

### Invalid Arguments

```jinja
{{ "page.title" | text(123) }}
<!-- Error: Language must be string, got integer -->
```

---

## Best Practices

**Use variables for repeated filters:**
```jinja
{# ✅ Good #}
{% set title = "page.title" | text(lang) %}
<title>{{ title }}</title>
<h1>{{ title }}</h1>

{# ❌ Bad #}
<title>{{ "page.title" | text(lang) }}</title>
<h1>{{ "page.title" | text(lang) }}</h1>
```

**Explicit language for fixed content:**
```jinja
{# ✅ Good - Footer always in site default language #}
<footer>{{ "footer.copyright" | text("en") }}</footer>

{# ⚠️ Questionable - Footer changes with URL language #}
<footer>{{ "footer.copyright" | text }}</footer>
```

**Provide defaults:**
```jinja
{# ✅ Good - Graceful degradation #}
{{ "optional.text" | text | default("Default text") }}
```

**Validate configuration values:**
```jinja
{% if "debug.mode" | config == "true" %}
    {# Debug features #}
{% endif %}

{# ❌ Bad - No validation #}
{% if "debug.mode" | config %}  {# Could be any string! #}
```

---

## Implementation Reference

**Files:**
- `src/reedcms/filters/text.rs` - Text filter
- `src/reedcms/filters/route.rs` - Route filter
- `src/reedcms/filters/meta.rs` - Meta filter
- `src/reedcms/filters/config.rs` - Config filter

**Registration:**
```rust
// src/reedcms/template/engine.rs
env.add_filter("text", make_text_filter(lang.to_string()));
env.add_filter("route", make_route_filter(lang.to_string()));
env.add_filter("meta", make_meta_filter());
env.add_filter("config", make_config_filter());
```

---

**See also:**
- [MiniJinja Integration](minijinja-integration.md) - Template engine setup
- [Data Layer - ReedBase Cache](../02-data-layer/reedbase-cache.md) - O(1) lookups
- [Data Commands](../04-cli-layer/data-commands.md) - Managing filter data
