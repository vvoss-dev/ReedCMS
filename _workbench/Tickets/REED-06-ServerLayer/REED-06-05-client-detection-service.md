# REED-06-05: Client Detection Service

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
- **ID**: REED-06-05
- **Title**: Client Detection Service
- **Layer**: Server Layer (REED-06)
- **Priority**: High
- **Status**: Open
- **Complexity**: Medium
- **Dependencies**: REED-06-01

## Summary Reference
- **Section**: Client Detection
- **Key Concepts**: Screen info cookie, interaction mode detection, device type detection, breakpoint detection

## Objective
Implement client detection service that extracts device information from cookies and User-Agent headers, determines interaction mode (mouse/touch/reader), populates client context for templates, and handles initial screen detection.

## Requirements

### Client Context Structure

```rust
/// Client information for template context.
///
/// ## Fields
/// - lang: Language code from URL (/de/wissen → "de")
/// - interaction_mode: Detected interaction mode (mouse/touch/reader)
/// - device_type: Device classification (mobile/tablet/desktop/bot)
/// - breakpoint: CSS breakpoint (phone/tablet/screen/wide)
/// - viewport_width: Browser viewport width (from cookie)
/// - viewport_height: Browser viewport height (from cookie)
/// - screen_width: Physical screen width (from cookie)
/// - screen_height: Physical screen height (from cookie)
/// - dpr: Device pixel ratio (from cookie)
/// - active_voices: Screen reader voice count (from cookie)
/// - is_bot: Bot/crawler detection flag
#[derive(Debug, Clone, Serialize)]
pub struct ClientInfo {
    pub lang: String,
    pub interaction_mode: String,
    pub device_type: String,
    pub breakpoint: String,
    pub viewport_width: Option<u32>,
    pub viewport_height: Option<u32>,
    pub screen_width: Option<u32>,
    pub screen_height: Option<u32>,
    pub dpr: Option<f32>,
    pub active_voices: Option<u32>,
    pub is_bot: bool,
}
```

### Screen Info Cookie Format

**Cookie Name**: `screen_info`  
**Format**: URL-encoded JSON  
**Max Age**: 1 year (31536000 seconds)  
**SameSite**: Lax

**Cookie Value Example**:
```json
{
  "width": 1920,
  "height": 1080,
  "dpr": 2.0,
  "viewport_width": 1920,
  "viewport_height": 937,
  "active_voices": 0
}
```

### Implementation (`src/reedcms/server/client_detection.rs`)

```rust
/// Screen information from cookie.
#[derive(Debug, Clone, Deserialize)]
pub struct ScreenInfo {
    pub width: u32,
    pub height: u32,
    pub dpr: f32,
    pub viewport_width: u32,
    pub viewport_height: u32,
    #[serde(default)]
    pub active_voices: Option<u32>,
}

/// Detects client information from HTTP request.
///
/// ## Process
/// 1. Parse screen_info cookie (if present)
/// 2. Detect device type (from cookie or User-Agent fallback)
/// 3. Determine CSS breakpoint (based on viewport width)
/// 4. Detect interaction mode (based on breakpoint and screen info)
/// 5. Check for bot/crawler
/// 6. Extract language from URL (set by routing layer)
///
/// ## Performance
/// - Detection: < 5ms with cookie
/// - Fallback (User-Agent): < 10ms
/// - Bot detection: < 1ms
///
/// ## Output
/// - ClientInfo struct ready for template context
pub fn detect_client_info(req: &HttpRequest, lang: &str) -> ReedResult<ClientInfo> {
    // 1. Parse screen info cookie
    let screen_info = parse_screen_info_cookie(req)?;
    
    // 2. Detect bot status
    let is_bot = is_bot_request(req);
    
    // 3. Detect device type
    let device_type = detect_device_type(req, &screen_info);
    
    // 4. Detect CSS breakpoint
    let breakpoint = detect_breakpoint(&screen_info, &device_type);
    
    // 5. Detect interaction mode
    let interaction_mode = detect_interaction_mode(&screen_info, &device_type, &breakpoint, is_bot);
    
    Ok(ClientInfo {
        lang: lang.to_string(),
        interaction_mode,
        device_type,
        breakpoint,
        viewport_width: screen_info.as_ref().map(|s| s.viewport_width),
        viewport_height: screen_info.as_ref().map(|s| s.viewport_height),
        screen_width: screen_info.as_ref().map(|s| s.width),
        screen_height: screen_info.as_ref().map(|s| s.height),
        dpr: screen_info.as_ref().map(|s| s.dpr),
        active_voices: screen_info.as_ref().and_then(|s| s.active_voices),
        is_bot,
    })
}

/// Parses screen_info cookie from HTTP request.
///
/// ## Cookie Format
/// - Name: screen_info
/// - Value: URL-encoded JSON with ScreenInfo structure
///
/// ## Process
/// 1. Extract Cookie header
/// 2. Find screen_info cookie
/// 3. URL-decode value
/// 4. Parse JSON to ScreenInfo struct
///
/// ## Returns
/// - Some(ScreenInfo) if cookie present and valid
/// - None if cookie missing or parsing failed
fn parse_screen_info_cookie(req: &HttpRequest) -> ReedResult<Option<ScreenInfo>> {
    // Get cookie header
    let cookie_header = req.headers()
        .get("cookie")
        .and_then(|h| h.to_str().ok());
    
    if let Some(cookies_str) = cookie_header {
        // Parse cookies
        for cookie_part in cookies_str.split(';') {
            let trimmed = cookie_part.trim();
            
            if trimmed.starts_with("screen_info=") {
                let value = &trimmed[12..]; // Skip "screen_info="
                
                // URL-decode
                if let Ok(decoded) = urlencoding::decode(value) {
                    // Parse JSON
                    if let Ok(screen_info) = serde_json::from_str::<ScreenInfo>(&decoded) {
                        return Ok(Some(screen_info));
                    }
                }
            }
        }
    }
    
    Ok(None)
}

/// Detects device type from screen info or User-Agent fallback.
///
/// ## Detection Logic
/// - If screen_info present: Use viewport width
///   - < 560px → mobile
///   - < 960px → tablet
///   - >= 960px → desktop
/// - If no screen_info: Parse User-Agent
///   - Mobile keywords → mobile
///   - Tablet keywords → tablet
///   - Bot keywords → bot
///   - Default → desktop
///
/// ## Performance
/// - With screen_info: O(1)
/// - User-Agent fallback: O(n) string search
fn detect_device_type(req: &HttpRequest, screen_info: &Option<ScreenInfo>) -> String {
    // Priority 1: Screen info (most reliable)
    if let Some(info) = screen_info {
        if info.viewport_width < 560 {
            return "mobile".to_string();
        } else if info.viewport_width < 960 {
            return "tablet".to_string();
        } else {
            return "desktop".to_string();
        }
    }
    
    // Priority 2: User-Agent fallback
    if let Some(user_agent) = req.headers().get("user-agent") {
        if let Ok(ua_str) = user_agent.to_str() {
            let ua_lower = ua_str.to_lowercase();
            
            // Bot detection
            if ua_lower.contains("bot") || 
               ua_lower.contains("crawler") ||
               ua_lower.contains("spider") ||
               ua_lower.contains("googlebot") {
                return "bot".to_string();
            }
            
            // Mobile detection
            if ua_lower.contains("mobile") || 
               ua_lower.contains("android") || 
               ua_lower.contains("iphone") ||
               ua_lower.contains("windows phone") ||
               ua_lower.contains("blackberry") {
                return "mobile".to_string();
            }
            
            // Tablet detection
            if ua_lower.contains("ipad") || 
               ua_lower.contains("tablet") ||
               ua_lower.contains("kindle") {
                return "tablet".to_string();
            }
        }
    }
    
    // Default fallback
    "desktop".to_string()
}

/// Determines CSS breakpoint based on viewport width.
///
/// ## Breakpoint Ranges
/// - phone: 0-559px
/// - tablet: 560-959px
/// - screen: 960-1259px
/// - wide: 1260px+
///
/// ## Fallback Logic
/// If no screen info, use device type:
/// - mobile → phone
/// - tablet → tablet
/// - desktop/bot → screen
fn detect_breakpoint(screen_info: &Option<ScreenInfo>, device_type: &str) -> String {
    if let Some(info) = screen_info {
        if info.viewport_width <= 559 {
            return "phone".to_string();
        } else if info.viewport_width <= 959 {
            return "tablet".to_string();
        } else if info.viewport_width <= 1259 {
            return "screen".to_string();
        } else {
            return "wide".to_string();
        }
    }
    
    // Fallback based on device type
    match device_type {
        "mobile" => "phone".to_string(),
        "tablet" => "tablet".to_string(),
        "bot" => "screen".to_string(),
        _ => "screen".to_string(),
    }
}

/// Detects interaction mode for template variant selection.
///
/// ## Interaction Modes
/// - reader: No viewport, bot, or screen reader (active_voices > 0)
/// - touch: phone or tablet breakpoint
/// - mouse: screen or wide breakpoint
///
/// ## Detection Priority
/// 1. Reader mode: No screen info OR active_voices > 0 OR is_bot
/// 2. Touch mode: phone or tablet breakpoint
/// 3. Mouse mode: screen or wide breakpoint
///
/// ## Rationale
/// - Reader mode for accessibility and bots (text-only rendering)
/// - Touch mode for mobile/tablet (larger tap targets, swipe gestures)
/// - Mouse mode for desktop (hover states, precise clicking)
fn detect_interaction_mode(
    screen_info: &Option<ScreenInfo>,
    device_type: &str,
    breakpoint: &str,
    is_bot: bool
) -> String {
    // Priority 1: Reader mode
    // - No screen info available
    // - Bot/crawler
    // - Screen reader detected (active_voices > 0)
    if screen_info.is_none() || 
       is_bot ||
       device_type == "bot" ||
       screen_info.as_ref().map_or(false, |s| {
           s.viewport_width < 1 || 
           s.active_voices.unwrap_or(0) > 0
       }) {
        return "reader".to_string();
    }
    
    // Priority 2: Breakpoint-based detection (most reliable)
    match breakpoint {
        "phone" | "tablet" => "touch".to_string(),
        "screen" | "wide" => "mouse".to_string(),
        _ => {
            // Fallback to device type
            if device_type == "mobile" || device_type == "tablet" {
                "touch".to_string()
            } else {
                "mouse".to_string()
            }
        }
    }
}

/// Checks if request is from a bot or crawler.
///
/// ## Detection Method
/// - Checks User-Agent header for bot keywords:
///   - "bot", "crawler", "spider", "googlebot", etc.
///
/// ## Performance
/// - < 1ms (simple string search)
fn is_bot_request(req: &HttpRequest) -> bool {
    req.headers()
        .get("user-agent")
        .and_then(|h| h.to_str().ok())
        .map(|s| {
            let lower = s.to_lowercase();
            lower.contains("bot") || 
            lower.contains("crawler") || 
            lower.contains("spider")
        })
        .unwrap_or(false)
}
```

### Screen Detection HTML (`src/reedcms/server/screen_detection.rs`)

```rust
/// Generates minimal HTML page for screen detection.
///
/// ## Purpose
/// - First visit: No screen_info cookie
/// - Send this HTML to detect screen dimensions
/// - JavaScript sets cookie and reloads page
/// - Second request: Cookie present, serve actual content
///
/// ## Performance
/// - One-time delay on first visit only
/// - < 100ms JavaScript execution
/// - Subsequent visits: No detection needed
///
/// ## Output
/// - Minimal HTML with inline JavaScript
/// - Sets screen_info cookie
/// - Immediate reload after cookie set
pub fn generate_screen_detection_html() -> String {
    r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Loading...</title>
    <style>body{margin:0;padding:0;background:#fff;}</style>
    <script>
    (function(){
        var d={
            width:screen.width,
            height:screen.height,
            dpr:window.devicePixelRatio||1,
            viewport_width:window.innerWidth||document.documentElement.clientWidth,
            viewport_height:window.innerHeight||document.documentElement.clientHeight,
            active_voices:window.speechSynthesis?window.speechSynthesis.getVoices().length:0
        };
        document.cookie='screen_info='+encodeURIComponent(JSON.stringify(d))+';path=/;max-age=31536000;SameSite=Lax';
        location.reload();
    })();
    </script>
</head>
<body>
    <p>Detecting screen...</p>
</body>
</html>"#.to_string()
}

/// Checks if screen detection is needed.
///
/// ## Returns
/// - true: No screen_info cookie, send detection HTML
/// - false: Cookie present, proceed with normal rendering
pub fn needs_screen_detection(req: &HttpRequest) -> bool {
    parse_screen_info_cookie(req)
        .ok()
        .flatten()
        .is_none()
}
```

### Integration with Request Handler (`src/reedcms/server/handler.rs`)

```rust
/// Main request handler with client detection.
///
/// ## Process
/// 1. Check if screen detection needed
/// 2. If YES: Send screen detection HTML (first visit)
/// 3. If NO: Detect client info, render template
///
/// ## Example
/// ```rust
/// // First visit (no cookie)
/// GET /de/wissen
/// → Sends screen detection HTML
/// → JavaScript sets cookie, reloads
///
/// // Second request (cookie present)
/// GET /de/wissen
/// → Detects: mouse, desktop, screen breakpoint
/// → Renders: knowledge.mouse.jinja
/// ```
pub async fn handle_request(req: HttpRequest) -> Result<HttpResponse, Error> {
    // 1. Check for screen detection
    if needs_screen_detection(&req) && !is_bot_request(&req) {
        return Ok(HttpResponse::Ok()
            .content_type("text/html; charset=utf-8")
            .body(generate_screen_detection_html()));
    }
    
    // 2. Route resolution (from REED-06-02)
    let route_info = resolve_route(&req)?;
    
    // 3. Client detection
    let client_info = detect_client_info(&req, &route_info.language)?;
    
    // 4. Template context building (from REED-05-03)
    let mut context = build_context(
        &route_info.layout,
        &route_info.language,
        &get_environment(),
        &client_info.interaction_mode
    )?;
    
    // 5. Add client info to context
    context.insert("client", &client_info);
    
    // 6. Render template
    let html = render_template(
        &route_info.layout,
        &client_info.interaction_mode,
        context
    )?;
    
    // 7. Build response
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html))
}
```

### Template Usage

**Template Context**:
```jinja
<!DOCTYPE html>
<html lang="{{ client.lang }}">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{{ "page.title" | text(client.lang) }}</title>
    
    {# Asset bundle with automatic variant selection #}
    <link rel="stylesheet" href="{{ asset_css }}">
</head>
<body class="interaction-{{ client.interaction_mode }} device-{{ client.device_type }} breakpoint-{{ client.breakpoint }}">
    
    {# Conditional rendering based on interaction mode #}
    {% if client.interaction_mode == "reader" %}
        {# Text-only content for screen readers and bots #}
        {% include organism("content-text-only") %}
    {% else %}
        {# Full visual content #}
        {% include organism("page-header") %}
        {% include organism("content-main") %}
        {% include organism("page-footer") %}
    {% endif %}
    
    {# Viewport information for debugging (DEV only) #}
    {% if environment == "dev" %}
        <div style="position:fixed;bottom:0;right:0;background:rgba(0,0,0,0.8);color:#fff;padding:8px;font-size:12px;">
            {{ client.interaction_mode }} | {{ client.device_type }} | {{ client.breakpoint }}<br>
            Viewport: {{ client.viewport_width }}x{{ client.viewport_height }}
            {% if client.dpr %} | DPR: {{ client.dpr }}{% endif %}
        </div>
    {% endif %}
    
    <script src="{{ asset_js }}" defer></script>
</body>
</html>
```

**Available Client Variables**:
- `client.lang` - Language code (e.g., "de", "en")
- `client.interaction_mode` - Interaction mode (mouse/touch/reader)
- `client.device_type` - Device classification (mobile/tablet/desktop/bot)
- `client.breakpoint` - CSS breakpoint (phone/tablet/screen/wide)
- `client.viewport_width` - Browser viewport width (optional)
- `client.viewport_height` - Browser viewport height (optional)
- `client.screen_width` - Physical screen width (optional)
- `client.screen_height` - Physical screen height (optional)
- `client.dpr` - Device pixel ratio (optional)
- `client.active_voices` - Screen reader voices (optional)
- `client.is_bot` - Bot detection flag (boolean)

## Implementation Files

### Primary Implementation
- `src/reedcms/server/client_detection.rs` - Client detection logic
- `src/reedcms/server/screen_detection.rs` - Screen detection HTML
- `src/reedcms/server/handler.rs` - Request handler integration

### Test Files
- `src/reedcms/server/client_detection.test.rs`
- `src/reedcms/server/screen_detection.test.rs`
- `src/reedcms/server/handler.test.rs`

## File Structure
```
src/reedcms/server/
├── client_detection.rs         # Client info detection
├── client_detection.test.rs    # Detection tests
├── screen_detection.rs          # Screen detection HTML
├── screen_detection.test.rs     # Detection HTML tests
├── handler.rs                   # Request handler
└── handler.test.rs              # Handler tests
```

## Testing Requirements

### Unit Tests
- [ ] Test screen_info cookie parsing
- [ ] Test device type detection (with screen info)
- [ ] Test device type fallback (User-Agent only)
- [ ] Test breakpoint detection
- [ ] Test interaction mode detection
- [ ] Test bot detection
- [ ] Test screen detection HTML generation
- [ ] Test detection bypass for bots

### Integration Tests
- [ ] Test first visit (no cookie) → detection HTML
- [ ] Test second visit (with cookie) → normal rendering
- [ ] Test bot request (no detection)
- [ ] Test reader mode activation
- [ ] Test touch mode on mobile
- [ ] Test mouse mode on desktop
- [ ] Test client context in template

### Edge Cases
- [ ] Invalid JSON in cookie → fallback to User-Agent
- [ ] Missing cookie fields → graceful degradation
- [ ] Bot with valid cookie → still reader mode
- [ ] Very small viewport (< 1px) → reader mode
- [ ] Active voices detected → reader mode

### Performance Tests
- [ ] Client detection: < 5ms with cookie
- [ ] User-Agent fallback: < 10ms
- [ ] Bot detection: < 1ms
- [ ] Screen detection HTML: < 1ms generation

## Acceptance Criteria
- [ ] Client detection working from screen_info cookie
- [ ] User-Agent fallback functional
- [ ] Interaction mode correctly detected (mouse/touch/reader)
- [ ] Device type and breakpoint detection accurate
- [ ] Bot detection working
- [ ] Screen detection HTML functional
- [ ] First visit detection seamless
- [ ] Client context available in all templates
- [ ] All tests pass with 100% coverage
- [ ] Performance benchmarks met
- [ ] Documentation complete
- [ ] BBC English throughout

## Dependencies
- **Requires**: REED-06-01 (Server Foundation)

## Blocks
- REED-05-03 (Context Builder needs client info)
- REED-06-04 (Response Builder needs client info)

## References
- Service Template: `_workbench/Tickets/templates/service-template.md`
- Service Test Template: `_workbench/Tickets/templates/service-template.test.md`
- Archive Reference: `_workbench/Archive/libs/client.rs`

## Notes
Client detection enables responsive rendering at the server level, delivering the appropriate template variant (mouse/touch/reader) on first request. The screen_info cookie provides accurate viewport dimensions, avoiding client-side detection overhead. Bot and screen reader detection ensures accessibility and SEO optimisation. One-time screen detection on first visit creates minimal UX impact while providing comprehensive device information for all subsequent requests.
