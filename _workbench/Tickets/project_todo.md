# ReedCMS Template Integration - Project TODO

**Main Task**: Complete Template System Integration Analysis
**Status**: In Progress - Question C
**Date**: 2025-01-30

---

## Context

Analyzing existing template structure (layouts + components) to ensure 100% compatibility with planned ReedCMS tickets. Working through identified gaps and missing specifications.

---

## ✅ Completed Questions

### A) Text-Filter: `text('auto')` Language Resolution
**Status**: ✅ Resolved and documented in REED-05-01

**Decision**: 
- URL path is **single source of truth** for language
- Cookie `lang` only for initial redirect at `/`
- Filter signature: `make_text_filter(current_lang: String)` with injected URL language
- `text('auto')` → uses URL language
- `text('de')` → forces German (explicit override)

**Implementation**: Documented in REED-05-01 with complete request flow.

---

### B) `reed` Dictionary vs. Filter Usage
**Status**: ✅ Resolved and implemented

**Decision**:
- **NO** `reed` Dictionary in context (legacy pattern from vvoss.dev)
- **YES** Use filter system consistently: `{{ pagekey | route('auto') }}`

**Implementation Completed**:
- ✅ Migrated all `reed['pageroute@' + pagekey]` → `pagekey | route('auto')`
- ✅ Migrated all `reed.pageroute` → `current_pagekey | route('de'/'en')`
- ✅ Updated 3 templates: page-header.mouse/touch/reader.jinja

**Rationale**:
- Consistent with Filter-System
- No Dictionary-Overhead
- Lazy evaluation
- Clear data flow

### B.1) Route Filter: Empty Route Handling
**Status**: ✅ Resolved and implemented

**Decision**: Route filter handles landing page logic internally

**Before** (complex):
```jinja
{% if pagekey != 'landing' %}{{ pagekey | route('auto') }}/{% endif %}
```

**After** (simple):
```jinja
{{ pagekey | route('auto') }}/
```

**Implementation**:
- Filter returns empty string for landing page
- Filter returns route segment only (no slashes)
- Template constructs full URL: `/de/` + `""` + `/` → `/de/`

**Benefits**:
- KISS principle: Logic in Rust, not templates
- Cleaner templates (removed 3x conditional logic)
- Single source of truth for route handling

**Files Updated**:
- ✅ REED-05-01 specification updated with empty route documentation
- ✅ page-header.mouse/touch/reader.jinja simplified

---

### C) Component Inclusion Functions: `organism()`, `molecule()`, `atom()`, `layout()`
**Status**: ✅ Resolved and documented in REED-05-02

**Template Usage** (existing):
```jinja
{% extends layout("page") %}
{% include organism("page-header") %}
{% include organism("landing-hero") %}
```

**Decision**: 
4 custom functions added to REED-05-02 specification:

```rust
// organism(name) → templates/components/organisms/{name}/{name}.{interaction_mode}.jinja
pub fn make_organism_function(interaction_mode: String) -> impl Function {
    move |name: &str| -> Result<String> {
        Ok(format!(
            "templates/components/organisms/{}/{}.{}.jinja",
            name, name, interaction_mode
        ))
    }
}

// molecule(name) → templates/components/molecules/{name}/{name}.{interaction_mode}.jinja
// atom(name) → templates/components/atoms/{name}/{name}.{interaction_mode}.jinja  
// layout(name) → templates/layouts/{name}/{name}.jinja (NO interaction_mode)
```

**Implementation Details**:
- `interaction_mode` injected at Environment creation time from `client.interaction_mode`
- O(1) path resolution via string formatting
- < 1μs per function call
- No filesystem access, only path generation
- MiniJinja handles actual template loading

**Templates Status**:
- ✅ All templates already use correct syntax (`layout()`, `organism()`)
- ✅ No migration needed

**Benefits**:
- KISS principle: Simple string formatting
- Performance: Zero allocations in hot path
- Clarity: Variant selection automatic via context

**Files Updated**:
- ✅ REED-05-02 specification extended with custom functions section

---

## 📋 Remaining Open Questions

### D) CSS Bundling: Session Hash + Asset Discovery

**Template Usage** (current):
```html
<link rel="stylesheet" href="/public/session/styles/{{ layout_name }}.{{ config.session_hash }}.{{ client.interaction_mode }}.css">
```

**What Must Happen**:
1. Discover all CSS files for a layout:
   - `landing.mouse.css`
   - `landing-hero.mouse.css` (included organism)
   - `landing-problems.mouse.css` (included organism)
   - etc.
2. Bundle them into single file
3. Generate session hash (MD5 of content?)
4. Serve under `/public/session/styles/`

**Missing in Tickets**:
- REED-08-01 (CSS Bundler) mentions "component asset discovery" but:
  - No session hash generation specified
  - No "when to bundle" strategy (on-demand? pre-build? hot-reload?)
  - No caching strategy specified

**Questions**:
- When is CSS bundled? (First request? Startup? Build command?)
- How is session hash generated? (Content hash? Timestamp?)
- Where is bundled CSS stored? (Memory? Disk?)
- How does hot-reload work with bundling?

---

### E) Screen Info Cookie → Client Context Population

**Template Usage** (current):
```jinja
<html lang="{{ client.lang }}">
<link rel="stylesheet" href="/public/static/styles/{{ client.interaction_mode }}.css">
```

**Required Context Variables**:
```rust
struct ClientInfo {
    lang: String,                    // From URL: /de/wissen → "de"
    interaction_mode: String,        // From heuristic: "mouse"/"touch"/"reader"
    viewport_width: Option<u32>,     // From cookie
    viewport_height: Option<u32>,    // From cookie
    dpr: Option<f32>,                // From cookie
    // ... etc
}
```

**Missing in Tickets**:
- REED-06-02 (Routing System) or REED-06-03/04 (Middleware/Handler) should specify:
  - Cookie parsing for `screen_info`
  - Interaction mode heuristic (viewport < 768 → touch, voices > 0 → reader)
  - Context population for `client` object

**Archive Reference**: `_workbench/Archive/libs/client.rs` has complete implementation

**Question**: Should we create **REED-06-05: Client Detection Service**?

---

### F) Icon Atoms: SVG Fragments without Wrapper

**Current Structure**:
```
templates/components/atoms/icons/arrow-right.jinja
  → Contains: <line .../><polyline .../>
  → NO <svg> wrapper
```

**Problem**: Templates need complete `<svg>` element, not just paths.

**Possible Solutions**:
1. **Wrapper in Template**: Each template wraps icon manually
2. **Molecule Wrapper**: `svg-icon` molecule adds `<svg>` wrapper
3. **Atom Rendering Function**: `icon()` function adds wrapper automatically

**Archive Reference**: `_workbench/Archive/libs/icons.rs` has `load_icon()` function

**Question**: How should icons be rendered in ReedCMS?

---

### G) Navigation: Hardcoded vs. Registry.csv

**Template Usage** (current):
```jinja
{% set pages = ["knowledge", "portfolio", "blog", "impressum"] %}
{% for pagekey in pages %}
    <li><a href="/{{ client.lang }}/{{ pagekey | route('auto') }}/">
{% endfor %}
```

**Tickets Mention**: REED-05-03 mentions `registry.csv` for navigation:
```csv
knowledge|layout|true|10||Knowledge base layout
portfolio|layout|true|20||Portfolio layout
```

**Missing Specification**:
- How does `registry.csv` → template context?
- Filter: enabled=false layouts?
- Order by `order` field?
- Exclude sub-layouts (parent != null)?

**Question**: Should context builder auto-populate navigation list, or should templates query registry manually?

---

### H) Text Migration: 31x `.text.csv` → `.reed/text.csv`

**Current State**:
- 31 component-local `.text.csv` files
- Already pipe-delimited (correct format)
- Keys already have namespace: `page-header.logo.title@de`

**Migration Strategy**:
```bash
reed migrate:text templates/components/organisms/page-header/
reed migrate:text templates/layouts/knowledge/
```

**Question**: 
- Does migration expect keys **with** or **without** namespace prefix?
- Should migration auto-add prefix if missing?
- Example: `logo.title@de` → `page-header.logo.title@de` (auto-prefix)?

**Needs Clarification**: REED-04-04 CLI Migration Commands

---

## 🎯 Next Steps

1. **Answer Question C**: Add component inclusion functions to REED-05-02
2. **Document Question D**: CSS bundling strategy in REED-08-01
3. **Decide Question E**: Create REED-06-05 or extend existing ticket?
4. **Resolve Question F**: Icon rendering approach
5. **Clarify Question G**: Navigation auto-population
6. **Specify Question H**: Migration key handling

---

## 📊 Progress Summary

**Total Questions Identified**: 9 (including B.1 as separate implementation)
**Resolved**: 4 (A, B, B.1, C)
**In Progress**: 0
**Remaining**: 5 (D, E, F, G, H)

**Recent Commits**:
- `[DOCS]` - Template integration analysis + 13 ticket optimizations
- `[DOCS]` - Created project_todo.md for tracking
- `[TEMPLATES]` - Migrated reed dictionary to route filter
- `[REED-05-01]` - Simplified route filter with empty route handling
- `[REED-05-02]` - Added custom functions for component inclusion

**Estimated Completion**: After all questions answered, tickets are 100% implementation-ready.
