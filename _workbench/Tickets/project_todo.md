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
**Status**: ✅ Resolved

**Decision**:
- **NO** `reed` Dictionary in context (legacy pattern from vvoss.dev)
- **YES** Use filter system consistently: `{{ pagekey | route('auto') }}`
- Migration needed: Replace all `reed['pageroute@...']` with `| route('auto')`

**Rationale**:
- Consistent with Filter-System
- No Dictionary-Overhead
- Lazy evaluation
- Clear data flow

---

## 🔄 Current Question (In Progress)

### C) Component Inclusion Functions: `organism()`, `molecule()`, `atom()`, `layout()`

**Template Usage** (current):
```jinja
{% extends layout("page") %}
{% include organism("page-header") %}
{% include organism("landing-hero") %}
```

**What These Must Do**:
Resolve to variant-specific paths based on `client.interaction_mode`:

```
organism("page-header") 
  + client.interaction_mode="mouse" 
  → "templates/components/organisms/page-header/page-header.mouse.jinja"

organism("page-header") 
  + client.interaction_mode="touch" 
  → "templates/components/organisms/page-header/page-header.touch.jinja"
```

**Missing in Tickets**:
- REED-05-02 (Template Engine Setup) has NO specification for these custom functions
- Only `path_loader("templates/")` is specified
- Variant resolution logic completely missing

**Required Implementation**:
```rust
// In REED-05-02 Template Engine Setup
pub fn make_organism_function(interaction_mode: String) -> impl Function {
    move |name: &str| -> Result<String> {
        format!(
            "templates/components/organisms/{}/{}.{}.jinja",
            name, name, interaction_mode
        )
    }
}

env.add_function("organism", make_organism_function(client.interaction_mode));
env.add_function("molecule", make_molecule_function(client.interaction_mode));
env.add_function("atom", make_atom_function(client.interaction_mode));
env.add_function("layout", make_layout_function()); // No variant, just path resolution
```

**Question**: Should we add this to REED-05-02 now?

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

**Total Questions Identified**: 8
**Resolved**: 2 (A, B)
**In Progress**: 1 (C)
**Remaining**: 5 (D, E, F, G, H)

**Estimated Completion**: After all questions answered, tickets are 100% implementation-ready.
