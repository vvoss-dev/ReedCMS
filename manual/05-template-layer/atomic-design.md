# Atomic Design

> Component organisation from atoms to layouts

---

## Overview

ReedCMS organises templates using Atomic Design methodology, breaking interfaces into fundamental building blocks that compose into complete pages.

---

## Hierarchy

```
Atoms (smallest)
    ↓ Compose into
Molecules (groups of atoms)
    ↓ Compose into
Organisms (complex components)
    ↓ Compose into
Layouts (complete pages)
```

---

## Directory Structure

```
templates/
├── base.jinja                          # Base template
├── components/
│   ├── atoms/                          # Fundamental elements
│   │   ├── button/
│   │   │   ├── button.mouse.jinja
│   │   │   ├── button.touch.jinja
│   │   │   ├── button.reader.jinja
│   │   │   └── button.text.csv
│   │   ├── icon/
│   │   │   └── icon.mouse.jinja
│   │   └── input/
│   │       └── input.mouse.jinja
│   │
│   ├── molecules/                      # Component groups
│   │   ├── search-form/
│   │   │   ├── search-form.mouse.jinja
│   │   │   └── search-form.text.csv
│   │   ├── card/
│   │   │   ├── card.mouse.jinja
│   │   │   └── card.mouse.css
│   │   └── navigation-item/
│   │       └── navigation-item.mouse.jinja
│   │
│   └── organisms/                      # Complex components
│       ├── page-header/
│       │   ├── page-header.mouse.jinja
│       │   ├── page-header.mouse.css
│       │   └── page-header.text.csv
│       ├── page-footer/
│       │   └── page-footer.mouse.jinja
│       └── knowledge-grid/
│           └── knowledge-grid.mouse.jinja
│
└── layouts/                            # Complete pages
    ├── knowledge/
    │   ├── knowledge.mouse.jinja
    │   ├── knowledge.touch.jinja
    │   ├── knowledge.reader.jinja
    │   ├── knowledge.mouse.css
    │   └── knowledge.text.csv
    └── blog/
        ├── blog.mouse.jinja
        └── blog.mouse.css
```

---

## Atoms

### Definition

**Smallest functional components** that can't be broken down further without losing meaning.

### Examples

**Button:**
```jinja
{# components/atoms/button/button.mouse.jinja #}
<button class="btn btn-{{ type }}">
    {{ label }}
</button>
```

**Icon:**
```jinja
{# components/atoms/icon/icon.mouse.jinja #}
<svg class="icon icon-{{ name }}" width="{{ size }}" height="{{ size }}">
    <use href="/assets/icons.svg#{{ name }}"></use>
</svg>
```

**Input:**
```jinja
{# components/atoms/input/input.mouse.jinja #}
<input 
    type="{{ type }}" 
    name="{{ name }}" 
    placeholder="{{ placeholder }}"
    class="input"
>
```

### Usage

```jinja
{% include "components/atoms/button/button.mouse.jinja" with context %}
```

---

## Molecules

### Definition

**Groups of atoms** functioning together as a unit.

### Examples

**Search Form:**
```jinja
{# components/molecules/search-form/search-form.mouse.jinja #}
<form class="search-form" action="/search" method="get">
    {% include "components/atoms/input/input.mouse.jinja" 
       with { type: "search", name: "q", placeholder: "Search..." } %}
    
    {% include "components/atoms/button/button.mouse.jinja" 
       with { type: "primary", label: "Search" } %}
</form>
```

**Card:**
```jinja
{# components/molecules/card/card.mouse.jinja #}
<article class="card">
    <h3>{{ title }}</h3>
    <p>{{ description }}</p>
    
    {% include "components/atoms/button/button.mouse.jinja" 
       with { type: "secondary", label: "Read More" } %}
</article>
```

**Navigation Item:**
```jinja
{# components/molecules/navigation-item/navigation-item.mouse.jinja #}
<li class="nav-item">
    {% include "components/atoms/icon/icon.mouse.jinja" 
       with { name: icon, size: 20 } %}
    <a href="{{ url }}">{{ label }}</a>
</li>
```

---

## Organisms

### Definition

**Complex components** composed of atoms and molecules, forming distinct sections.

### Examples

**Page Header:**
```jinja
{# components/organisms/page-header/page-header.mouse.jinja #}
<header class="page-header">
    <div class="logo">
        {% include "components/atoms/icon/icon.mouse.jinja" 
           with { name: "logo", size: 40 } %}
        <span>{{ "page-header.logo.title" | text(lang) }}</span>
    </div>
    
    <nav class="main-nav">
        <ul>
        {% for item in navigation %}
            {% include "components/molecules/navigation-item/navigation-item.mouse.jinja" 
               with item %}
        {% endfor %}
        </ul>
    </nav>
    
    {% include "components/molecules/search-form/search-form.mouse.jinja" %}
</header>
```

**Page Footer:**
```jinja
{# components/organisms/page-footer/page-footer.mouse.jinja #}
<footer class="page-footer">
    <div class="footer-columns">
        <div class="footer-column">
            <h4>{{ "footer.about.title" | text(lang) }}</h4>
            <p>{{ "footer.about.text" | text(lang) }}</p>
        </div>
        
        <div class="footer-column">
            <h4>{{ "footer.links.title" | text(lang) }}</h4>
            <ul>
            {% for link in footer_links %}
                <li><a href="{{ link.url }}">{{ link.label }}</a></li>
            {% endfor %}
            </ul>
        </div>
    </div>
    
    <div class="copyright">
        {{ "footer.copyright" | text(lang) }}
    </div>
</footer>
```

---

## Layouts

### Definition

**Complete page templates** composed of organisms, molecules, and atoms.

### Example

**Knowledge Layout:**
```jinja
{# layouts/knowledge/knowledge.mouse.jinja #}
{% extends "base.jinja" %}

{% block title %}{{ "knowledge.title" | text(lang) }}{% endblock %}

{% block content %}
{% include "components/organisms/page-header/page-header.mouse.jinja" %}

<main class="knowledge-layout">
    <h1>{{ "knowledge.heading" | text(lang) }}</h1>
    <p class="subtitle">{{ "knowledge.subtitle" | text(lang) }}</p>
    
    <div class="knowledge-grid">
    {% for item in knowledge_items %}
        {% include "components/molecules/card/card.mouse.jinja" 
           with { title: item.title, description: item.description } %}
    {% endfor %}
    </div>
</main>

{% include "components/organisms/page-footer/page-footer.mouse.jinja" %}
{% endblock %}
```

---

## Component-Local Text

### Structure

Each component can have its own `text.csv`:

```
components/organisms/page-header/
├── page-header.mouse.jinja
├── page-header.mouse.css
└── page-header.text.csv
```

### Format

```csv
key|value|description
page-header.logo.title@en|ReedCMS|Logo title
page-header.logo.title@de|ReedCMS|Logo-Titel
page-header.nav.home@en|Home|Home link
page-header.nav.home@de|Startseite|Startseite-Link
```

### Benefits

- **Localised content** - Text next to component
- **Maintainability** - Easy to find component strings
- **Reusability** - Component is self-contained

---

## Variants

### Three Variants Per Component

**mouse (Desktop):**
```jinja
{# button.mouse.jinja #}
<button class="btn" onmouseover="showTooltip()">
    {{ label }}
</button>
```

**touch (Mobile/Tablet):**
```jinja
{# button.touch.jinja #}
<button class="btn btn-large" ontouchstart="handleTap()">
    {{ label }}
</button>
```

**reader (Accessibility):**
```jinja
{# button.reader.jinja #}
<button class="btn btn-accessible" aria-label="{{ label }}">
    {{ label }}
</button>
```

### Server Selection

Server automatically selects variant based on `User-Agent`:

```rust
fn detect_variant(req: &HttpRequest) -> &str {
    let ua = req.headers().get("User-Agent");
    
    if is_mobile(ua) || is_tablet(ua) {
        "touch"
    } else if is_reader_mode(ua) {
        "reader"
    } else {
        "mouse"
    }
}
```

---

## CLI Scaffolding

### Create Layout

```bash
reed layout:init knowledge --languages de,en --variants mouse,touch
```

**Generated:**
```
templates/layouts/knowledge/
├── knowledge.mouse.jinja    # Desktop template
├── knowledge.touch.jinja    # Mobile template
├── knowledge.mouse.css      # Desktop styles
├── knowledge.touch.css      # Mobile styles (optional)
└── knowledge.text.csv       # Component-local text
```

**See:** [CLI Layout Commands](../04-cli-layer/layout-commands.md)

---

## Best Practices

**Start small, compose up:**
```
✅ Good: Button → Search Form → Header → Layout
❌ Bad: Create entire header in one file
```

**Reusable components:**
```jinja
{# ✅ Good - Generic, reusable #}
{% include "components/atoms/button/button.mouse.jinja" 
   with { type: "primary", label: "Submit" } %}

{# ❌ Bad - Hardcoded #}
<button class="btn-primary">Submit</button>
```

**Component-local text:**
```
✅ Good: page-header/page-header.text.csv
❌ Bad: All text in global .reed/text.csv
```

**Variant-specific features:**
```jinja
{# mouse.jinja - Hover effects #}
<button onmouseover="...">

{# touch.jinja - Touch gestures #}
<button ontouchstart="...">
```

---

**See also:**
- [MiniJinja Integration](minijinja-integration.md) - Template engine
- [Template Filters](filters.md) - Custom filters
- [CLI Layout Commands](../04-cli-layer/layout-commands.md) - Scaffolding
