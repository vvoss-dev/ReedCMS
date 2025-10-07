# MiniJinja Integration

> Rust-native Jinja2-compatible template engine

---

## Overview

ReedCMS uses MiniJinja, a Rust implementation of the Jinja2 template language, providing fast, memory-safe template rendering with zero Python dependencies.

---

## Why MiniJinja?

### Comparison

| Feature | MiniJinja | Tera | Handlebars |
|---------|-----------|------|------------|
| Syntax | Jinja2 | Jinja2-like | Mustache |
| Performance | ⚡ Fast | ⚡ Fast | ⚡⚡ Fastest |
| Familiarity | ✅ High | ⚠️ Medium | ⚠️ Low |
| Rust-native | ✅ Yes | ✅ Yes | ✅ Yes |
| Custom filters | ✅ Easy | ✅ Easy | ⚠️ Limited |
| Auto-reload | ✅ Yes | ❌ No | ❌ No |

**ReedCMS choice:** MiniJinja (familiar syntax, custom filters, hot-reload)

---

## Environment Setup

### Per-Request Pattern (Current Implementation)

**ReedCMS creates a new Environment for EACH request** with request-specific filters and functions.

**Why?**
- Filters need request language: `text("en")` vs `text("de")`
- Functions need variant: `organism()` with "mouse" vs "touch"
- Template functions required **before parsing** for `{% extends layout("page") %}`

```rust
use minijinja::Environment;

pub fn create_environment(lang: &str, variant: &str) -> Environment<'static> {
    let mut env = Environment::new();
    
    // Set template directory
    env.set_loader(path_loader("templates"));
    
    // Configure auto-escape (security)
    env.set_auto_escape_callback(|name| {
        name.ends_with(".html") || name.ends_with(".jinja")
    });
    
    // Add filters with request language
    env.add_filter("text", make_text_filter(lang.to_string()));
    env.add_filter("route", make_route_filter(lang.to_string()));
    env.add_filter("meta", make_meta_filter());
    env.add_filter("config", make_config_filter());
    
    // Add functions with request variant
    env.add_function("organism", make_organism_function(variant.to_string()));
    env.add_function("molecule", make_molecule_function(variant.to_string()));
    env.add_function("atom", make_atom_function(variant.to_string()));
    env.add_function("layout", make_layout_function(variant.to_string()));
    
    env
}
```

**Performance:**
- Environment creation: < 50μs
- MiniJinja internally caches parsed templates
- Total overhead: < 2ms per request
- Still fast due to O(1) ReedBase cache lookups

---

## Template Loading

### Path Loader

```rust
use minijinja::path_loader;

// Load from filesystem
env.set_loader(path_loader("templates"));

// Templates resolved relative to path:
// "layouts/knowledge/knowledge.mouse.jinja"
// → templates/layouts/knowledge/knowledge.mouse.jinja
```

### Custom Loader (Future)

```rust
use minijinja::Environment;

// Load from database or embedded resources
env.set_loader(|name| {
    // Custom logic
    load_template_from_db(name)
});
```

---

## Template Rendering

### Basic Rendering

```rust
use minijinja::context;

let env = create_environment("en", "mouse");

// Get template
let tmpl = env.get_template("layouts/knowledge/knowledge.mouse.jinja")?;

// Build context
let ctx = context! {
    client => context! {
        lang => "en",
        interaction_mode => "mouse",
        device => "desktop"
    },
    pagekey => "knowledge",  // NOT "layout" - conflicts with layout() function!
    page => context! {
        latest_update => "2025-10-07"
    },
    title => "Knowledge Base",
    items => vec![/* ... */],
};

// Render
let html = tmpl.render(ctx)?;
```

**Critical:** Use `pagekey` NOT `layout` - context variable `layout` conflicts with `layout()` template function!

### With Error Handling

```rust
pub fn render_template(
    layout: &str,
    variant: &str,
    lang: &str,
    context: serde_json::Value
) -> ReedResult<String> {
    let env = create_environment(lang, get_env_mode());
    
    // Build template path
    let template_path = format!(
        "layouts/{}/{}.{}.jinja",
        layout, layout, variant
    );
    
    // Get template
    let tmpl = env
        .get_template(&template_path)
        .map_err(|e| ReedError::TemplateError {
            template: template_path.clone(),
            reason: format!("Template not found: {}", e),
        })?;
    
    // Convert JSON context to MiniJinja context
    let ctx = minijinja::Value::from_serializable(&context);
    
    // Render
    let html = tmpl
        .render(ctx)
        .map_err(|e| ReedError::TemplateError {
            template: template_path,
            reason: format!("Render failed: {}", e),
        })?;
    
    Ok(html)
}
```

---

## Template Syntax

### Variables

```jinja
{{ variable }}
{{ object.property }}
{{ array[0] }}
```

**Example:**
```jinja
<h1>{{ title }}</h1>
<p>{{ user.name }} ({{ user.email }})</p>
```

### Control Flow

**If statements:**
```jinja
{% if condition %}
    Content when true
{% elif other_condition %}
    Content when other true
{% else %}
    Content when false
{% endif %}
```

**For loops:**
```jinja
{% for item in items %}
    <li>{{ item.name }}</li>
{% endfor %}

{# With index #}
{% for item in items %}
    {{ loop.index }}: {{ item.name }}
{% endfor %}
```

### Filters

```jinja
{{ variable | filter }}
{{ variable | filter(arg1, arg2) }}
{{ variable | filter1 | filter2 }}
```

**Built-in filters:**
```jinja
{{ text | upper }}              {# UPPERCASE #}
{{ text | lower }}              {# lowercase #}
{{ text | capitalize }}         {# Capitalize First #}
{{ number | abs }}              {# Absolute value #}
{{ array | length }}            {# Array length #}
{{ text | default("fallback") }}{# Default value #}
```

**Custom filters:**
```jinja
{{ "page.title" | text("en") }}        {# ReedBase text #}
{{ "knowledge" | route("de") }}        {# URL routing #}
{{ "site.title" | meta }}              {# Metadata #}
```

### Template Inheritance

**Base template:**
```jinja
{# base.jinja #}
<!DOCTYPE html>
<html>
<head>
    <title>{% block title %}Default Title{% endblock %}</title>
</head>
<body>
    {% block content %}{% endblock %}
</body>
</html>
```

**Child template:**
```jinja
{# page.jinja #}
{% extends "base.jinja" %}

{% block title %}Custom Title{% endblock %}

{% block content %}
    <h1>Page Content</h1>
{% endblock %}
```

### Includes

```jinja
{# Include component #}
{% include "components/organisms/page-header/page-header.mouse.jinja" %}

{# Include with context #}
{% include "components/molecules/card/card.mouse.jinja" with context %}
```

### Comments

```jinja
{# Single line comment #}

{#
   Multi-line
   comment
#}
```

---

## Auto-Escape

### Security Feature

**Automatic HTML escaping:**
```jinja
{{ user_input }}
{# Automatically escapes: <script> → &lt;script&gt; #}
```

**Disable for trusted HTML:**
```jinja
{{ trusted_html | safe }}
{# No escaping - USE WITH CAUTION #}
```

**Configuration:**
```rust
env.set_auto_escape_callback(|name| {
    // Auto-escape .html and .jinja files
    name.ends_with(".html") || name.ends_with(".jinja")
});
```

---

## Error Handling

### Template Errors

**Common errors:**

**Template not found:**
```rust
ReedError::TemplateError {
    template: "layouts/missing/missing.mouse.jinja".to_string(),
    reason: "Template not found".to_string(),
}
```

**Syntax error:**
```jinja
{% if condition  {# Missing %} #}
```
```rust
ReedError::TemplateError {
    template: "page.jinja".to_string(),
    reason: "Unexpected end of template at line 5".to_string(),
}
```

**Missing key (graceful fallback):**
```jinja
{{ "invalid.key" | text("en") }}
<!-- Output: invalid.key (returns key itself, no error) -->
```

**Filter behaviour:**
- `text` filter returns key itself if not found
- `route` filter returns key itself if not found
- No template errors from missing keys
- Missing keys visible in rendered output for debugging

### Error Display

**Development mode:**
```html
<h1>Template Error</h1>
<pre>
Template: layouts/knowledge/knowledge.mouse.jinja
Line: 42
Error: Variable 'missing_var' not found
</pre>
```

**Production mode:**
```html
<h1>500 Internal Server Error</h1>
<p>An error occurred whilst rendering the page.</p>
```

---

## Performance Optimisation

### Template Caching

**Production (cached):**
```rust
env.set_auto_reload(false);

// First render: Compile + render (< 10ms)
let html1 = tmpl.render(ctx)?;

// Subsequent renders: Use cached compilation (< 1ms)
let html2 = tmpl.render(ctx)?;
```

**Development (no cache):**
```rust
env.set_auto_reload(true);

// Every render: Check file modified time + recompile if changed
let html = tmpl.render(ctx)?;  // < 10ms
```

### Filter Performance

**O(1) ReedBase lookups:**
```jinja
{{ "page.title" | text("en") }}  {# < 100μs via HashMap #}
```

**Avoid repeated filter calls:**
```jinja
{# ❌ BAD - Calls filter 100 times #}
{% for i in range(100) %}
    {{ "page.title" | text("en") }}
{% endfor %}

{# ✅ GOOD - Call once, store in variable #}
{% set title = "page.title" | text("en") %}
{% for i in range(100) %}
    {{ title }}
{% endfor %}
```

---

## Best Practices

**Use template inheritance:**
```jinja
{# ✅ Good - DRY #}
{% extends "base.jinja" %}

{# ❌ Bad - Duplicate HTML #}
<!DOCTYPE html>...
```

**Store filtered values in variables:**
```jinja
{# ✅ Good #}
{% set title = "page.title" | text(lang) %}
<title>{{ title }}</title>
<h1>{{ title }}</h1>

{# ❌ Bad - Duplicate filter calls #}
<title>{{ "page.title" | text(lang) }}</title>
<h1>{{ "page.title" | text(lang) }}</h1>
```

**Always escape user input:**
```jinja
{# ✅ Good - Auto-escaped #}
{{ user.comment }}

{# ❌ DANGEROUS - Unescaped #}
{{ user.comment | safe }}
```

**Descriptive block names:**
```jinja
{# ✅ Good #}
{% block page_title %}...{% endblock %}
{% block main_content %}...{% endblock %}

{# ❌ Bad #}
{% block b1 %}...{% endblock %}
{% block b2 %}...{% endblock %}
```

---

## Troubleshooting

**Template not found:**
```
Solution: Check path relative to templates/ directory
reed layout:init knowledge  # Creates correct structure
```

**Filter not found:**
```
Solution: Verify filter registered in environment
env.add_filter("text", make_text_filter(...));
```

**Variable undefined:**
```
Solution: Pass variable in context
let ctx = context! { missing_var => "value" };
```

**Hot-reload not working:**
```
Solution: Check ENVIRONMENT=dev in .env
env.set_auto_reload(true) only in dev mode
```

---

**See also:**
- [Template Filters](filters.md) - Complete filter reference
- [Atomic Design](atomic-design.md) - Component organisation
- [Hot-Reload System](hot-reload.md) - Development workflow
