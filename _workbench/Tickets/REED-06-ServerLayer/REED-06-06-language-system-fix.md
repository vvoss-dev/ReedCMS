# REED-06-06: Language System Fix - Default Language & URL Prefix Routing

## MANDATORY Development Standards

**CRITICAL**: Every implementation MUST follow these standards:

- **Language**: All documentation and code comments in BBC English
- **Principle**: KISS (Keep It Simple, Stupid) - minimal code and professional inline documentation
- **File Naming**: File name = Unique theme - crystal clear what single topic this file handles
- **Files**: One file = One responsibility
- **Functions**: One function = One distinctive job
- **Shared Functions**: Same patterns = One shared function (CONFIG and SYSTEM use identical logic)
- **Testing**: Separate test files as `{name}.test.rs` - never inline `#[cfg(test)]` modules
- **Avoid**: Avoid Swiss Army knife functions
- **Avoid**: Generic file names like `handler.rs`, `middleware.rs`, `utils.rs` - every topic (auth, login,...) has its own scoped rs service file
- **Templates**: See `_workbench/Tickets/templates/service-template.md` for complete implementation guide
- **Testing**: See `_workbench/Tickets/templates/service-template.test.md` for test structure

## Ticket Information
- **ID**: REED-06-06
- **Title**: Language System Fix - Default Language & URL Prefix Routing
- **Layer**: Server Layer (REED-06)
- **Priority**: Critical
- **Status**: Open
- **Complexity**: Medium
- **Dependencies**: REED-06-01, REED-06-02, REED-06-05

## Summary Reference
- **Section**: Language Detection and URL Routing
- **Lines**: To be added to project_summary.md after implementation
- **Key Concepts**: Default language configuration, URL prefix routing, language switcher integration

## Objective

Fix two critical language system issues:
1. **No default language configured**: Missing `project.default_language` causes browser Accept-Language header to override intended default
2. **Language switching not working**: Templates expect `/de/...` and `/en/...` URL structure, but routes.csv has no language prefixes

## Problem Analysis

### Problem 1: Missing Default Language Configuration

**Current State** (`.reed/project.csv`):
```csv
project.languages|de,en|Available languages
```

**Missing**: `project.default_language|de|Default site language`

**Impact**:
- Browser with `Accept-Language: en-US,en;q=0.9` shows English content
- No way to enforce German as default language
- `language.rs:detect_language()` prioritises Accept-Language over default

**Root Cause** (`src/reedcms/routing/language.rs:14-23`):
```rust
pub fn detect_language(req: &HttpRequest) -> String {
    // 1. Try URL path first
    if let Some(lang) = extract_language_from_path(req.path()) {
        return lang;
    }

    // 2. Try Accept-Language header ← PROBLEM HERE
    if let Some(lang) = parse_accept_language_header(req) {
        return lang;  // Browser says "en" → used before default
    }

    // 3. Fall back to default ← Never reached when Accept-Language present
    get_default_language().unwrap_or_else(|| "de".to_string())
}
```

### Problem 2: URL Prefix Mismatch

**Templates Expect** (`page-header.mouse.jinja:42-44`):
```jinja
<a href="/{{ client.lang }}/{{ pagekey | route('auto') }}/">
```

**Routes.csv Provides**:
```csv
knowledge@de|wissen|Knowledge German (no /de/ prefix!)
knowledge@en|knowledge|Knowledge English (no /en/ prefix!)
```

**Result**: Language switcher generates `/de/wissen/` → **404 Not Found**

**Legacy System** (`_workbench/Archive/libs/routing.rs:54-61`):
```rust
pub fn get_path(&self, key: &str, lang: &str) -> Option<String> {
    get_env_value(&self.registry, &lang_env, key)
        .map(|path| {
            if path.is_empty() {
                format!("/{}/", lang)  // ← ALWAYS with language prefix!
            } else {
                format!("/{}/{}", lang, path)  // ← /de/wissen, /en/knowledge
            }
        })
}
```

## Solution

### Part 1: Configure Default Language

**File**: `.reed/project.csv`

**Changes**:
```csv
# Add default language configuration
project.default_language|de|Default website language
```

### Part 2: Migrate Routes to URL Prefix System

**File**: `.reed/routes.csv`

**Current Format**:
```csv
landing@de||German homepage route (landing/root)
landing@en||English homepage route (landing/root)
knowledge@de|wissen|German knowledge section route
knowledge@en|knowledge|English knowledge section route
```

**New Format** (with language prefixes):
```csv
landing@de|de|German homepage (/de/)
landing@en|en|English homepage (/en/)
knowledge@de|de/wissen|Knowledge German (/de/wissen)
knowledge@en|en/knowledge|Knowledge English (/en/knowledge)
portfolio@de|de/portfolio|Portfolio German
portfolio@en|en/portfolio|Portfolio English
blog@de|de/blog|Blog German
blog@en|en/blog|Blog English
impressum@de|de/impressum|Impressum German
impressum@en|en/impressum|Impressum English
contact@de|de/contact|Contact German
contact@en|en/contact|Contact English

# Sub-pages with full language prefix
actix-web@de|de/wissen/actix-web-framework|Actix Web DE
actix-web@en|en/knowledge/actix-web-framework|Actix Web EN
agility@de|de/wissen/agilitaet|Agility DE
agility@en|en/knowledge/agility|Agility EN
aeo@de|de/wissen/answer-engine-optimization|AEO DE
aeo@en|en/knowledge/answer-engine-optimization|AEO EN
ai-visibility@de|de/wissen/ai-visibility|AI Visibility DE
ai-visibility@en|en/knowledge/ai-visibility|AI Visibility EN
apache@de|de/wissen/apache-webserver|Apache DE
apache@en|en/knowledge/apache-webserver|Apache EN
apache-license@de|de/wissen/apache-lizenz|Apache License DE
apache-license@en|en/knowledge/apache-license|Apache License EN
api@de|de/wissen/api|API DE
api@en|en/knowledge/api|API EN
bastille@de|de/wissen/bastille-jail-verwaltung|Bastille DE
bastille@en|en/knowledge/bastille-jail-management|Bastille EN
geo@de|de/wissen/generative-engine-optimization|GEO DE
geo@en|en/knowledge/generative-engine-optimization|GEO EN
lost-in-the-middle@de|de/wissen/lost-in-the-middle|Lost in the Middle DE
lost-in-the-middle@en|en/knowledge/lost-in-the-middle|Lost in the Middle EN
seo@de|de/wissen/suchmaschinenoptimierung|SEO DE
seo@en|en/knowledge/search-engine-optimization|SEO EN
```

### Part 3: Implement Root Redirect Handler

**File**: `src/reedcms/server/http_server.rs`

**New Function**:
```rust
/// Handles root URL redirect to language-specific landing page.
///
/// ## Process
/// 1. Detect language from Accept-Language header or default
/// 2. Redirect / → /de/ or /en/
/// 3. Use 301 Moved Permanently for SEO
///
/// ## Language Detection
/// - Accept-Language header first (user preference)
/// - Default language from config as fallback
///
/// ## SEO Benefit
/// - Proper language URL structure
/// - Search engines can index language variants separately
///
/// ## Performance
/// - < 5ms redirect response
async fn handle_root_redirect(req: HttpRequest) -> HttpResponse {
    use crate::reedcms::routing::language::detect_language;
    
    let lang = detect_language(&req);
    
    HttpResponse::MovedPermanently()
        .append_header(("Location", format!("/{}/", lang)))
        .finish()
}
```

**Route Configuration**:
```rust
fn configure_routes(cfg: &mut web::ServiceConfig) {
    configure_public_routes(cfg);
    
    // Root redirect to language-specific landing page
    cfg.service(web::resource("/").route(web::get().to(handle_root_redirect)));
    
    // All other routes
    cfg.service(web::resource("/{path:.*}").route(web::get().to(handle_request)));
}
```

### Part 4: Update Language Detection Priority (Optional Enhancement)

**File**: `src/reedcms/routing/language.rs`

**Note**: Current implementation is already correct (URL prefix has highest priority), but we document the detection order:

```rust
/// Detects language from HTTP request.
///
/// ## Detection Order (unchanged)
/// 1. **URL path prefix** (highest priority): /de/wissen → "de"
/// 2. **Accept-Language header**: Browser preference
/// 3. **Default language**: From project.default_language config
/// 4. **Hard-coded fallback**: "de"
///
/// ## Why This Order?
/// - URL prefix = Explicit user choice (clicked language switcher)
/// - Accept-Language = Browser preference (first visit)
/// - Default config = Site owner preference
/// - Fallback = Safety net
pub fn detect_language(req: &HttpRequest) -> String {
    // Implementation unchanged - already correct!
    // URL path already has highest priority
}
```

## Implementation Steps

### Step 1: Add Default Language to Project Config

```bash
# Manual edit or via reed command when available
echo "project.default_language|de|Default website language" >> .reed/project.csv
```

### Step 2: Migrate All Routes with Language Prefixes

**File**: `.reed/routes.csv`

1. Open `.reed/routes.csv`
2. For each route, add language prefix:
   - `landing@de||` → `landing@de|de|`
   - `landing@en||` → `landing@en|en|`
   - `knowledge@de|wissen` → `knowledge@de|de/wissen`
   - `knowledge@en|knowledge` → `knowledge@en|en/knowledge`
3. Apply to all 40+ routes

### Step 3: Implement Root Redirect Handler

**File**: `src/reedcms/server/http_server.rs`

1. Add `handle_root_redirect()` function
2. Update `configure_routes()` to register root handler
3. Test redirect: `curl -I http://localhost:8333/`

### Step 4: Test Language System

**Manual Tests**:
```bash
# Test root redirect
curl -I http://localhost:8333/
# Expected: 301 → /de/ or /en/ based on Accept-Language

# Test German landing page
curl http://localhost:8333/de/
# Expected: German content

# Test English landing page
curl http://localhost:8333/en/
# Expected: English content

# Test knowledge section
curl http://localhost:8333/de/wissen/
curl http://localhost:8333/en/knowledge/
# Expected: Knowledge content in respective language

# Test language switcher
# Visit http://localhost:8333/de/ in browser
# Click "EN" link → should go to /en/
# Content should change to English
```

## Acceptance Criteria

- [ ] `project.default_language` configured in `.reed/project.csv`
- [ ] All routes in `.reed/routes.csv` have language prefixes
- [ ] Root URL `/` redirects to `/de/` or `/en/` based on Accept-Language
- [ ] Landing pages work: `/de/` shows German, `/en/` shows English
- [ ] All knowledge articles work with prefixes: `/de/wissen/...` and `/en/knowledge/...`
- [ ] Language switcher in page-header works (DE ↔ EN)
- [ ] Content changes when switching language
- [ ] No 404 errors when clicking language switcher
- [ ] Browser Accept-Language respected on first visit
- [ ] Default language used when no Accept-Language header

## Files Modified

1. `.reed/project.csv` - Add default language
2. `.reed/routes.csv` - Migrate all routes with language prefixes
3. `src/reedcms/server/http_server.rs` - Add root redirect handler

## Performance Impact

- Root redirect: < 5ms (simple 301 response)
- No change to language detection (< 1ms)
- No change to route resolution (same O(n) linear scan)

## SEO Benefits

- Proper language URL structure: `/de/...` vs `/en/...`
- Search engines can index language variants separately
- hreflang tags work correctly with explicit language URLs
- Clean URL structure for international SEO

## Legacy System Comparison

**Legacy** (`_workbench/Archive/libs/routing.rs`):
- ✅ Always used URL prefixes: `/de/...` and `/en/...`
- ✅ Empty routes became `/{lang}/` (e.g., `/de/`)
- ✅ Accept-Language header for first visit detection

**New ReedCMS** (after this fix):
- ✅ URL prefixes: `/de/...` and `/en/...`
- ✅ Default language configuration
- ✅ Accept-Language header detection
- ✅ Professional language switching

## Related Decisions

See `_workbench/Tickets/project_optimisations.md` for:
- **Decision D048**: Default language configuration priority
- **Decision D049**: URL prefix routing for language variants
- **Decision D050**: Root redirect strategy for language detection

## Notes

- This fix brings ReedCMS in line with legacy vvoss.dev behaviour
- Templates already expect this URL structure (no template changes needed)
- Language switcher will work immediately after routes migration
- No breaking changes to existing language detection logic
- Enhancement for REED-04-12 (Reed.yaml) will make default language configuration more convenient
