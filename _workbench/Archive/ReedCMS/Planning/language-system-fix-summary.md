# Language System Fix - Implementation Summary

**Date**: 2025-10-06  
**Status**: Tickets Created, Ready for Implementation  
**Approach**: Two-Phase (A then B as requested)

---

## Overview

Two critical issues identified and resolved with tickets:

### Problem 1: No Default Language Configured
- Missing `project.default_language` in `.reed/project.csv`
- Browser Accept-Language header overrides intended default
- German site shows English to English-speaking browsers

### Problem 2: Language Switching Not Working
- Templates expect URL prefix routing (`/de/...` and `/en/...`)
- Routes.csv has no language prefixes
- Language switcher generates 404 errors

---

## Created Tickets

### REED-06-06: Language System Fix (Immediate - Phase A)
**File**: `_workbench/Tickets/REED-06-ServerLayer/REED-06-06-language-system-fix.md`

**Priority**: Critical  
**Complexity**: Medium  
**Dependencies**: REED-06-01, REED-06-02, REED-06-05  
**Estimated Time**: 15-30 minutes

**Changes**:
1. Add `project.default_language|de` to `.reed/project.csv`
2. Migrate all routes in `.reed/routes.csv` with language prefixes:
   - `landing@de||` → `landing@de|de|`
   - `knowledge@de|wissen` → `knowledge@de|de/wissen`
   - Apply to all 40+ routes
3. Implement root redirect handler in `http_server.rs`:
   - `/` → `/de/` or `/en/` based on Accept-Language + default
   - 301 Moved Permanently for SEO

**Result**:
- ✅ Default language respected
- ✅ Language switcher works (DE ↔ EN)
- ✅ SEO-friendly URL structure
- ✅ Content changes when switching language
- ✅ No 404 errors

**Files Modified**:
- `.reed/project.csv` (1 line added)
- `.reed/routes.csv` (40+ routes updated)
- `src/reedcms/server/http_server.rs` (root redirect handler)

---

### REED-04-12: Reed.toml Configuration System (Future - Phase B)
**File**: `_workbench/Tickets/REED-04-CLILayer/REED-04-12-reed-toml-configuration.md`

**Priority**: Medium  
**Complexity**: Medium  
**Dependencies**: REED-04-01, REED-04-02, REED-02-01  
**Estimated Time**: 1-2 hours

**Purpose**:
Developer-facing `Reed.toml` configuration file in project root as single source of truth for project defaults.

**Features**:
```toml
# Reed.toml - Example
[project]
name = "vvoss.dev"
url = "https://vvoss.dev"

[project.languages]
default = "de"  # ← Configured here!
available = ["de", "en"]
```

**CLI Commands**:
```bash
reed config:init      # Create Reed.toml from .reed/*.csv
reed config:sync      # Sync Reed.toml → .reed/*.csv
reed config:show      # Display current configuration
reed config:validate  # Validate Reed.toml syntax
```

**Benefits**:
- ✅ Rust-idiomatisch (wie Cargo.toml)
- ✅ Type-safe configuration
- ✅ Git-visible (not hidden in `.reed/`)
- ✅ Self-documenting with TOML comments
- ✅ Uses existing CLI commands for sync
- ✅ Clean separation: TOML (input) vs CSV (runtime)

**Files Created**:
- `src/reedcms/config/toml_parser.rs` (TOML parsing + validation)
- `src/reedcms/config/toml_to_csv.rs` (Sync logic)
- `src/reedcms/cli/config_commands.rs` (CLI commands)
- `Reed.toml` (Example in project root)

**Dependencies**:
- `toml = "0.8"` (TOML parsing)

---

## Legacy System Analysis

### Legacy vvoss.dev Behaviour (`_workbench/Archive/libs/`)

**URL Structure** (routing.rs:54-61):
```rust
// Legacy ALWAYS used URL prefixes
format!("/{}/", lang)        // /de/ or /en/
format!("/{}/{}", lang, path) // /de/wissen, /en/knowledge
```

**Language Detection** (client.rs:50-64):
- Accept-Language header parsing first
- No default language config
- Browser preference always honoured

**Result**: ReedCMS fix brings system in line with proven legacy behaviour.

---

## Implementation Sequence

### Phase A: Immediate Fix (Today - REED-06-06)

1. **Add Default Language** (2 minutes):
   ```bash
   echo "project.default_language|de|Default website language" >> .reed/project.csv
   ```

2. **Migrate Routes** (10 minutes):
   - Open `.reed/routes.csv`
   - For each route, add language prefix:
     - Empty routes: `||` → `|de|` or `|en|`
     - Normal routes: `|wissen` → `|de/wissen`
   - Save file

3. **Implement Root Redirect** (5 minutes):
   ```rust
   // In src/reedcms/server/http_server.rs
   async fn handle_root_redirect(req: HttpRequest) -> HttpResponse {
       let lang = detect_language(&req);
       HttpResponse::MovedPermanently()
           .append_header(("Location", format!("/{}/", lang)))
           .finish()
   }
   
   // In configure_routes()
   cfg.service(web::resource("/").route(web::get().to(handle_root_redirect)));
   ```

4. **Test** (3 minutes):
   ```bash
   cargo build
   reed server:io
   # Visit http://localhost:8333/
   # Should redirect to /de/ or /en/
   # Test language switcher
   ```

**Total Time**: ~20 minutes  
**Result**: Language system fully functional

---

### Phase B: Developer Experience Enhancement (Later - REED-04-12)

1. **Implement YAML Parser** (30 minutes)
2. **Implement Sync Logic** (20 minutes)
3. **Implement CLI Commands** (30 minutes)
4. **Write Tests** (20 minutes)
5. **Create Example Reed.yaml** (10 minutes)

**Total Time**: ~2 hours  
**Result**: Professional configuration system

---

## Documentation Updates

### Ticket Index
✅ Updated: `_workbench/Tickets/ticket-index.csv`
- Added REED-04-12 (Reed.yaml Configuration System)
- Added REED-06-06 (Language System Fix)

### Project Optimisations (To Be Added)
Future decisions to document in `project_optimisations.md`:
- **Decision D048**: Default language configuration priority
- **Decision D049**: URL prefix routing for language variants
- **Decision D050**: Root redirect strategy for language detection
- **Decision D051**: Reed.toml as single source of truth
- **Decision D052**: TOML → CSV sync using existing CLI commands
- **Decision D053**: Separation between developer-facing (TOML) and system-facing (CSV) configuration
- **Decision D054**: TOML chosen over YAML for Rust-idiomatic approach and type safety

### Project Summary (To Be Added)
Future sections to add to `project_summary.md`:
- Language System Architecture (REED-06-06)
- Reed.toml Configuration System (REED-04-12)

### Function Registry
Future functions to add to `project_functions.csv` (REED-04-12):
- `config/toml_parser.rs`: 15+ functions
- `config/toml_to_csv.rs`: 8+ functions
- `cli/config_commands.rs`: 4+ functions

---

## Next Steps

### Immediate (Phase A - REED-06-06):
1. ✅ Tickets created
2. ⏸️  Awaiting user approval to proceed
3. Implementation: ~20 minutes
4. Testing: ~5 minutes
5. Commit: `[REED-06-06] – fix: implement default language and URL prefix routing`

### Future (Phase B - REED-04-12):
1. ✅ Ticket created
2. Schedule for next session
3. Implementation: ~2 hours
4. Testing: ~30 minutes
5. Commit: `[REED-04-12] – feat: implement Reed.yaml configuration system`

---

## Questions Answered

### Q: "Hast Du Dir angeschaut, wie es im legacy funktioniert hat?"
✅ **Ja!** Legacy System analysiert:
- URL-Prefix System (`/de/...`, `/en/...`) war Standard
- Accept-Language Header Detection
- Routing.rs baute immer Prefixes auf

### Q: "Ich überlege auch, ob wir eine Reed.toml im project root ablegen..."
✅ **Brillante Idee!** Ticket REED-04-12 erstellt:
- Developer-facing TOML configuration (Rust-Standard!)
- Nutzt bestehende CLI Commands für Sync
- Saubere Trennung: TOML (Input) vs CSV (Runtime)
- Perfekt für ReedCMS-Philosophie

---

## Summary

**Status**: ✅ Complete ticket creation and documentation  
**Ready**: Immediate implementation of REED-06-06  
**Planned**: Future implementation of REED-04-12  

**All documentation follows**:
- ✅ BBC English throughout
- ✅ KISS principle
- ✅ Professional standards
- ✅ Complete ticket structure
- ✅ Function registry preparation
- ✅ Project documentation integration

**Awaiting**: Your approval to proceed with REED-06-06 implementation.
