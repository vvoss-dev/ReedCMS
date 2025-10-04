# ReedCMS Debugging Sessions

This document tracks debugging sessions with detailed checkpoints and analyses.

---

## Session 2025-01-04: Template Context Variables & Dev Mode

### Context
Working on REED-09 Build Layer to make the website fully startable via `reed` command. Server starts correctly but templates fail to render.

### Initial Problem
- Server returns 500 Internal Server Error
- Template error in `page-footer.mouse.jinja`
- Missing context variables: `current_year`, `config.dev_mode`

### Debug Approach Applied
Following CLAUDE.md rules: **Comment out, never delete**

### Checkpoint 1: Identify Missing Variables
**Action**: Commented out problematic template sections to isolate errors
**File**: `templates/components/organisms/page-footer/page-footer.mouse.jinja`

**Commented out**:
```jinja
{# © {{ current_year }} Vivian Voss. #}
© 2025 Vivian Voss. {# Hardcoded for testing #}

{# {% if config.dev_mode %} ... {% endif %} #}
```

**Result**: ✅ Page rendered successfully → Proves issue is missing context variables

### Checkpoint 2: Verify Context System
**Action**: Read `src/reedcms/templates/context.rs`
**Discovery**: 
- `current_year` already implemented in `add_globals()` using `chrono::Utc::now().year()`
- `config.dev_mode` **NOT** implemented in config object

**Root Cause Identified**: Template expects `config.dev_mode` but it's not provided by Rust context builder.

### Checkpoint 3: Add Missing Context Variable
**Action**: Add `config.dev_mode` to context builder
**File**: `src/reedcms/templates/context.rs`

**Change**:
```rust
// Config object
let mut config = HashMap::new();
config.insert("session_hash", serde_json::json!("dev42"));
config.insert("dev_mode", serde_json::json!(cfg!(debug_assertions))); // ← Added
ctx.insert("config".to_string(), serde_json::json!(config));
```

**Rationale**: Use Rust's `cfg!(debug_assertions)` to automatically detect debug/release mode

### Checkpoint 4: Re-enable Template Code
**Action**: Uncommented previously disabled template sections
**File**: `templates/components/organisms/page-footer/page-footer.mouse.jinja`

**Re-enabled**:
```jinja
© {{ current_year }} Vivian Voss.

{% if config.dev_mode %}
<div class="footer-dev">
    <!-- dev info -->
</div>
{% endif %}
```

### Checkpoint 5: Test Changes
**Action**: Rebuild, restart server, test with curl

**Commands**:
```bash
cargo build
./target/debug/reed server:start --port 3000
curl -H "Cookie: screen_info=..." http://localhost:3000
```

**Result**: ❌ Still 500 error in `page-footer.mouse.jinja`

**Log Output**:
```
Server error: TemplateError { template: "layouts/landing/landing.jinja", 
  reason: "could not render include: error in page-footer.mouse.jinja" }
```

### Checkpoint 6: Identify Additional Client Variable Issues
**Action**: Checked client variable usage in template

**File**: `templates/components/organisms/page-footer/page-footer.mouse.jinja`
**Lines 34-45**: Uses `client.lang`, `client.device_type`, `client.breakpoint`, etc.

**Discovery**: Template uses `client.*` variables but error message doesn't specify which one is missing.

**Hypothesis**: One or more client variables expected in dev mode section are not provided by context.

### Current Status
- ✅ `current_year` is provided by context
- ✅ `config.dev_mode` is now provided by context
- ❌ Template still fails to render
- 🔍 **Next step**: Verify all `client.*` variables are provided by context builder

### Variables Used in page-footer.mouse.jinja
**Required by template**:
- `client.lang` (line 34)
- `client.device_type` (line 35)
- `client.breakpoint` (line 36)
- `client.interaction_mode` (line 37)
- `client.screen_width` (optional, line 38)
- `client.screen_height` (optional, line 39)
- `client.viewport_width` (optional, line 41)
- `client.viewport_height` (optional, line 42)
- `client.dpr` (optional, line 44)

**Provided by context.rs build_context()**:
```rust
let mut client = HashMap::new();
client.insert("lang", serde_json::json!(language));
client.insert("interaction_mode", serde_json::json!(&client_info.interaction_mode));
client.insert("breakpoint", serde_json::json!(&client_info.breakpoint));
client.insert("device_type", serde_json::json!(&client_info.device_type));
```

**Missing (not in context.rs)**:
- `client.screen_width`
- `client.screen_height`
- `client.viewport_width`
- `client.viewport_height`
- `client.dpr`

**Analysis**: Template uses `{% if client.screen_width %}` which should handle missing values gracefully. The error must be something else.

### Hypothesis Revision
Error is NOT about missing optional variables (they're in `{% if %}` blocks).
Error might be about:
1. MiniJinja `{% if %}` not working with missing hash keys?
2. Template syntax error elsewhere in the file?
3. Text filter failing for one of the dev info labels?

### Next Debug Steps
1. Check if MiniJinja requires explicit None/null for optional variables
2. Test text filter keys: `page.footer.dev.*` in `.reed/text.csv`
3. Simplify dev_mode block to minimal version to isolate issue

---

## Debug-Step Tracking System

All commented-out code sections MUST be tracked with Debug-Step IDs (DS001, DS002, etc.) for restoration after bug resolution.

### Active Debug-Steps

*None - All debug-steps resolved!*

---

### Completed Debug-Steps

#### DS005: current_year variable in page-footer.mouse.jinja
**Status**: ✅ Resolved (2025-01-04)  
**File**: `templates/components/organisms/page-footer/page-footer.mouse.jinja`  
**Reason**: Missing context variable `current_year`  
**Resolution**: Added `current_year` to `context.rs::add_globals()` using `chrono::Utc::now().year()`  
**Restoration**: Code uncommented and working

---

#### DS006: config.dev_mode section in page-footer.mouse.jinja
**Status**: ✅ Resolved → Led to DS007  
**File**: `templates/components/organisms/page-footer/page-footer.mouse.jinja`  
**Reason**: Missing context variable `config.dev_mode`  
**Resolution**: Added `config.dev_mode` to context.rs, but template still failed → DS007 created for deeper analysis

---

#### DS007: Optional client variables in dev_mode section
**Status**: ✅ Resolved (2025-01-04)  
**File**: `templates/components/organisms/page-footer/page-footer.mouse.jinja`  
**Reason**: Template uses `{% if client.screen_width %}` but optional client variables not provided in context  
**Root Cause**: `ClientInfo` struct has optional fields (`viewport_width`, `screen_width`, `dpr`) but these weren't transferred to template context  
**Solution**: 
```rust
// In src/reedcms/templates/context.rs
if let Some(vw) = client_info.viewport_width {
    client.insert("viewport_width", serde_json::json!(vw));
}
// ... same for viewport_height, screen_width, screen_height, dpr
```
**Restoration**: Full dev_mode section uncommented and working  
**Test Result**: Page renders with dev info panel showing all client variables

---

#### DS001-DS004: Icon Macro Rendering
**Status**: ✅ Resolved (2025-01-04)  
**Files**: 
- `templates/components/organisms/landing-contact/landing-contact.mouse.jinja`
- `templates/components/organisms/landing-contact/landing-contact.touch.jinja`
- `templates/components/organisms/page-header/page-header.touch.jinja`
- `templates/components/molecules/svg-icon/svg-icon.jinja`

**Reason**: Icon macros failed to render (produced empty output)  
**Root Cause**: MiniJinja `macros` feature was not enabled in Cargo.toml  
**Solution**: 
```toml
minijinja = { version = "2.0", features = ["builtins", "debug", "loader", "multi_template", "macros"] }
```

**Restoration Actions Completed**:
1. ✅ Added `"macros"` feature to Cargo.toml
2. ✅ Rebuilt project (27s for minijinja recompile)
3. ✅ Restored svg-icon macro with full icon atom inclusion logic
4. ✅ Restored icon imports in all templates: `{% from "components/molecules/svg-icon/svg-icon.jinja" import svg_icon %}`
5. ✅ Restored icon calls: `{{ svg_icon("mail", "32", "contact-icon", "Email") }}`
6. ✅ Tested rendering - mail icon renders correctly with full SVG path

**Test Result**: Icons render perfectly in mouse mode

---

### Debug-Step Workflow

**1. When commenting out code**:
```jinja
{# DEBUG DS00X: Brief reason for commenting out #}
{# Original code here #}
```

**2. Document in project_debugging.md**:
- DS ID and status
- File path
- Reason for commenting
- Restoration condition
- Restoration action

**3. After bug resolution**:
- Uncomment code
- Test thoroughly
- Move DS entry from "Active" to "Completed"
- Update status with resolution details

**4. If uncommented code still fails**:
- Re-comment if necessary
- Update DS status to "Re-activated"
- Create new DS for deeper investigation

---

## Debug Session 2: MiniJinja Macro Investigation (2025-01-04)

### Problem Statement
Icon rendering requires macros, but `{% macro %}` definitions produce no output despite MiniJinja documentation confirming support for macro syntax.

### Investigation Steps

#### Step 1: Verify MiniJinja Macro Support
**Action**: Web research on MiniJinja macro features  
**Finding**: MiniJinja v2.x supports both:
- `{% macro name(params) %}...{% endmacro %}` - Local macro definition
- `{% from "file.jinja" import macro_name %}` - Macro import from other templates
- `{% import "file.jinja" as helpers %}` - Full module import

**Conclusion**: Syntax is correct according to official documentation

#### Step 2: Test Local Macro in Component Template
**File**: `templates/components/organisms/landing-contact/landing-contact.mouse.jinja`  
**Test Code**:
```jinja
{% macro test_macro(text) %}<strong>TEST: {{ text }}</strong>{% endmacro %}
{{ test_macro("Local macro test") }}
```

**Result**: ❌ No output - macro call produces empty string  
**Server Log**: ✅ No errors - request successful  
**Conclusion**: Macros are silently ignored, not causing errors

#### Step 3: Test Macro Import from External File
**File**: `templates/components/molecules/svg-icon/svg-icon.jinja`  
**Macro Definition**:
```jinja
{% macro svg_icon(icon, size, class, alt) %}
<svg class="{{ class }}" width="{{ size }}" height="{{ size }}" viewBox="0 0 24 24">
<circle cx="12" cy="12" r="10"/>
</svg>
{% endmacro %}
```

**Import in Component**:
```jinja
{% from "components/molecules/svg-icon/svg-icon.jinja" import svg_icon %}
{{ svg_icon("mail", "32", "contact-icon", "Email") }}
```

**Result**: ❌ No output - imported macro also produces empty string  
**Conclusion**: Both local and imported macros fail

#### Step 4: Check MiniJinja Version and Features
**Command**: `cargo tree | grep minijinja`  
**Result**: `minijinja v2.12.0`  
**Cargo.toml Features**: `["builtins", "debug", "loader", "multi_template"]`  
**Conclusion**: Latest version with all required features enabled

#### Step 5: Analyze Template Engine Initialization
**File**: `src/reedcms/response/builder.rs`  
**Finding**: Template engine is **statically initialized once** using `OnceLock`:
```rust
fn get_template_engine() -> &'static Environment<'static> {
    static ENGINE: OnceLock<Environment<'static>> = OnceLock::new();
    ENGINE.get_or_init(|| {
        init_template_engine("en".to_string(), "mouse".to_string())
    })
}
```

**Analysis**:
- ✅ Static engine is good for performance
- ✅ Filters work (use closures with runtime language)
- ❓ Macros should work with static engine
- **Not the root cause**

#### Step 6: Test Macro in Main Layout (Outside Component)
**File**: `templates/layouts/landing/landing.jinja`  
**Test Location**: Before `{% extends %}` directive  
**Test Code**:
```jinja
{% macro test() %}MACRO_WORKS{% endmacro %}
{% block content %}
    <p>{{ test() }}</p>
{% endblock %}
```

**Result**: ❌ No output from macro  
**Additional Finding**: "DEBUG:" text also missing → entire `{% block content %}` not rendered  
**Hypothesis**: Macro defined **before** `{% extends %}` might be lost during template inheritance

#### Step 7: Test Macro Inside Block (After Template Inheritance)
**Action**: Move macro definition **inside** `{% block content %}`  
**Rationale**: Template inheritance with `{% extends %}` might discard macros defined before blocks  
**Test Code**:
```jinja
{% extends "layouts/page/page.jinja" %}
{% block content %}
    {% macro test() %}MACRO_WORKS{% endmacro %}
    <p>DEBUG: Before macro call</p>
    <p>{{ test() }}</p>
    <p>DEBUG: After macro call</p>
{% endblock %}
```

**Status**: ⏳ Test pending - awaiting server restart

#### Step 8: Search MiniJinja Documentation for Macro Features
**Action**: WebFetch on https://docs.rs/minijinja/latest/minijinja/  
**Critical Finding**: MiniJinja has a **separate `macros` feature flag**!

**From Documentation**:
> `macros`: When removed, the `{% macro %}` tag is not included

**Current Cargo.toml Features**: `["builtins", "debug", "loader", "multi_template"]`  
**Missing Feature**: `"macros"`  

**Root Cause Identified**: ✅ The `{% macro %}` tag is **disabled by default** and requires explicit feature activation!

#### Step 9: Enable Macros Feature and Test
**Action**: Added `"macros"` to Cargo.toml features  
**Change**:
```toml
minijinja = { version = "2.0", features = ["builtins", "debug", "loader", "multi_template", "macros"] }
```

**Rebuild**: `cargo build` (27s for minijinja recompile)  
**Server**: Restarted with new binary  
**Test**: `curl http://localhost:3000 | grep MACRO`  

**Result**: ✅ **SUCCESS!**
```html
<p>DEBUG: Before macro call</p>
<p>MACRO_WORKS</p>
<p>DEBUG: After macro call</p>
```

**Solution Confirmed**: Adding `"macros"` feature to MiniJinja enables `{% macro %}` tag support!

### Solution Summary

**Root Cause**: MiniJinja `macros` feature was not enabled in Cargo.toml  
**Solution**: Add `"macros"` to feature list  
**Result**: ✅ Macros now work (both local and imported)  
**Time to Solution**: ~2 hours of debugging

### Lessons From This Session

1. **Always check Cargo features first** - MiniJinja has granular feature flags
2. **Read official documentation** - Features are well-documented but easy to miss
3. **Silent failures are tricky** - Macros were simply ignored without errors
4. **Systematic debugging works** - Step-by-step isolation led to solution

### Next Steps

1. ✅ Macros working - feature enabled
2. ⏳ Remove DEBUG code from templates
3. ⏳ Restore icon macros (DS001-DS004)
4. ⏳ Test icon rendering end-to-end
5. ⏳ Document in project_optimisations.md
6. ⏳ Commit all fixes

### Files Modified for Testing

**Templates with DEBUG code**:
- `templates/layouts/landing/landing.jinja` - Macro placement test
- `templates/components/organisms/landing-contact/landing-contact.mouse.jinja` - Local macro test
- `templates/components/molecules/svg-icon/svg-icon.jinja` - Simplified macro definition
- `templates/test_macro.jinja` - Minimal test template (created)

**Note**: All DEBUG code will be removed once solution is found

---

## Debug Session 3: Server Auto-Stop Implementation (2025-01-04)

### Problem Statement
Multiple `reed server:start` calls should automatically stop existing instances instead of failing with "Server already running" error.

### Requirement
**User Request**: "jedes mal wenn man mit reed via server startet werden alle laufenden instanzen beendet - es darf immer nur einen aktiven geben"

### Current Behavior (Before Fix)
```bash
$ reed server:start
# Server starts successfully

$ reed server:start
# ERROR: Server already running (PID: 1234)
```

**Issue**: User had to manually run `reed server:stop` before starting again.

### Implementation Steps

#### Step 1: Analyze Existing Code
**File**: `src/reedcms/cli/server_commands.rs`  
**Finding**: `server_start()` checks for running instances but returns error instead of stopping them:

```rust
if is_process_running(&pid) {
    return Err(ReedError::ServerError {
        component: "server_start".to_string(),
        reason: format!("Server already running (PID: {})", pid),
    });
}
```

#### Step 2: Extract Stop Logic to Helper Function
**Action**: Created `stop_server_by_pid()` helper function from `server_stop()` logic

**Function Signature**:
```rust
fn stop_server_by_pid(pid: &str) -> ReedResult<()>
```

**Implementation**:
- Send SIGTERM signal for graceful shutdown
- Wait up to 5 seconds for process to stop
- Force SIGKILL if still running after timeout
- Remove PID file after successful stop
- Unix-only (returns error on non-Unix systems)

#### Step 3: Modify server_start() Behavior
**Change**: Replace error return with automatic stop

**Before**:
```rust
if is_process_running(&pid) {
    return Err(ReedError::ServerError { ... });
}
```

**After**:
```rust
if is_process_running(&pid) {
    output.push_str(&format!("⚠ Found running server (PID: {}), stopping it first...\n", pid));
    stop_server_by_pid(&pid)?;
    std::thread::sleep(std::time::Duration::from_millis(500));
    output.push_str("✓ Previous instance stopped\n");
}
```

**Wait Time**: Added 500ms sleep after stop to ensure clean shutdown before spawning new process.

#### Step 4: Test Auto-Stop Functionality

**Test 1**: Start with existing instance
```bash
$ reed server:start
🚀 Starting ReedCMS server in background...
⚠ Found running server (PID: 5976), stopping it first...
✓ Previous instance stopped
✓ Configuration validated
✓ Environment: PROD
✓ Server started in background
PID: 6961
```

**Result**: ✅ Existing instance stopped, new instance started

**Test 2**: Consecutive starts
```bash
$ reed server:start
# Stops PID 6961, starts 7066

$ reed server:start  
# Stops PID 7066, starts 7234
```

**Result**: ✅ Each call stops previous and starts fresh instance

### Solution Summary

**Root Cause**: `server_start()` treated existing instances as errors instead of stopping them  
**Solution**: Auto-stop existing instances before starting new one  
**Result**: ✅ Only one server instance can run at a time (enforced automatically)

**Benefits**:
- **User-Friendly**: No manual `server:stop` needed
- **Idempotent**: `reed server:start` always results in exactly one running instance
- **Safe**: Graceful shutdown with SIGTERM, force kill only if needed
- **Clear Feedback**: Shows when stopping previous instance

**Files Modified**:
- `src/reedcms/cli/server_commands.rs` - Added `stop_server_by_pid()` helper and modified `server_start()`

**Commit**: `[REED-09-01] – feat: auto-stop running server instances on start`

---

## Lessons Learned

### 1. Comment Out, Don't Delete
✅ **Worked perfectly**: Commenting out allowed systematic isolation of errors
- Identified that base template works
- Identified exactly which variables were missing
- No code was destroyed in the process

### 2. Cascading Template Errors
**Pattern observed**: MiniJinja reports errors from innermost template outward
- First error: `landing-contact.mouse.jinja`
- After fixing: Error moved to `page-footer.mouse.jinja`
- **Lesson**: Fix errors one at a time, don't assume first error is root cause

### 3. Context Variables Must Match Template Expectations
**Discovery**: Template and context builder must be in sync
- Templates expect variables → context.rs must provide them
- No automatic error reporting for missing required variables
- Optional variables should be wrapped in `{% if %}` blocks

### 4. Debug vs Release Mode Detection
**Solution found**: Use `cfg!(debug_assertions)` for automatic mode detection
- True in debug builds (`cargo build`)
- False in release builds (`cargo build --release`)
- No manual configuration needed

---

## Open Questions

1. **Why does `{% if client.screen_width %}` fail?** MiniJinja should handle missing hash keys gracefully in conditionals.

2. **Are text filter keys missing?** Need to verify all `page.footer.dev.*` keys exist in `.reed/text.csv`.

3. **Should optional client variables be null or omitted?** What's MiniJinja's expected behaviour?

---

## Files Modified This Session

### `src/reedcms/templates/context.rs`
**Change**: Added `config.dev_mode` to context
**Status**: ✅ Committed to working tree
**Test status**: Built successfully

### `templates/components/organisms/page-footer/page-footer.mouse.jinja`
**Change**: Re-enabled `current_year` and `config.dev_mode` sections
**Status**: ⚠️ Modified, causes error
**Next**: Need to debug further

---

## Commands Used

```bash
# Stop server
./target/debug/reed server:stop

# Rebuild with changes
cargo build

# Start server
./target/debug/reed server:start --port 3000

# Test without cookie (screen detection)
curl http://localhost:3000

# Test with cookie (actual render)
curl -H "Cookie: screen_info=%7B%22width%22%3A1920%2C%22height%22%3A1080%2C%22dpr%22%3A1%2C%22viewport_width%22%3A1920%2C%22viewport_height%22%3A1080%2C%22active_voices%22%3A0%7D" http://localhost:3000

# Check logs
tail -50 .reed/flow/server.log

# Find client variable usage
grep -n "client\." templates/components/organisms/page-footer/page-footer.mouse.jinja
```

---

## Time Tracking
- **Session start**: After continuation from context limit
- **Current duration**: ~20 minutes
- **Status**: In progress - debugging template render error
