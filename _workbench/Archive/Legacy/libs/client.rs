use actix_web::HttpRequest;
use serde::{Deserialize, Serialize};
use serde_json;

#[derive(Serialize, Clone)]
pub struct ClientInfo {
    pub language: String,
    pub screen_width: Option<u32>,
    pub screen_height: Option<u32>,
    pub viewport_width: Option<u32>,
    pub viewport_height: Option<u32>,
    pub dpr: Option<f32>,
    pub active_voices: Option<u32>,
    pub device_type: String,
    pub breakpoint: String,
    pub interaction_mode: String,
    pub is_bot: bool,
    pub lang: String,
}

#[derive(Deserialize)]
pub struct ScreenInfo {
    pub width: u32,
    pub height: u32,
    pub dpr: f32,
    pub viewport_width: u32,
    pub viewport_height: u32,
    #[serde(default)]
    pub active_voices: Option<u32>,
}

/// Parse screen info from cookie
pub fn parse_screen_info(req: &HttpRequest) -> Option<ScreenInfo> {
    if let Some(cookie_header) = req.headers().get("cookie") {
        if let Ok(cookies_str) = cookie_header.to_str() {
            for cookie_part in cookies_str.split(';') {
                let trimmed = cookie_part.trim();
                if trimmed.starts_with("screen_info=") {
                    let value = &trimmed[12..];
                    if let Ok(decoded) = urlencoding::decode(value) {
                        if let Ok(screen_info) = serde_json::from_str::<ScreenInfo>(&decoded) {
                            return Some(screen_info);
                        }
                    }
                }
            }
        }
    }
    None
}

/// Detect client information from headers and cookies
pub fn detect_client_info(req: &HttpRequest) -> ClientInfo {
    let screen_info = parse_screen_info(req);
    
    // Detect language from Accept-Language header
    let mut language = "en-EN".to_string();
    
    if let Some(accept_lang) = req.headers().get("accept-language") {
        if let Ok(lang_str) = accept_lang.to_str() {
            for lang_part in lang_str.split(',') {
                let lang = lang_part.split(';').next().unwrap_or("").trim();
                
                if lang.starts_with("de") {
                    language = "de-DE".to_string();
                    break;
                } else if lang.starts_with("en") {
                    language = "en-EN".to_string();
                    break;
                }
            }
        }
    }
    
    let is_bot = is_bot_request(req);
    let device_type = detect_device_type(req, &screen_info);
    let breakpoint = detect_breakpoint(req, &screen_info);
    let interaction_mode = detect_interaction_mode(req, &screen_info, &device_type, &breakpoint);
    
    ClientInfo {
        language,
        screen_width: screen_info.as_ref().map(|s| s.width),
        screen_height: screen_info.as_ref().map(|s| s.height),
        viewport_width: screen_info.as_ref().map(|s| s.viewport_width),
        viewport_height: screen_info.as_ref().map(|s| s.viewport_height),
        dpr: screen_info.as_ref().map(|s| s.dpr),
        active_voices: screen_info.as_ref().and_then(|s| s.active_voices),
        device_type,
        breakpoint,
        interaction_mode,
        is_bot,
        lang: String::new(), // Will be set in handler
    }
}

/// Detect device type from User-Agent and screen info
pub fn detect_device_type(req: &HttpRequest, screen_info: &Option<ScreenInfo>) -> String {
    // If we have screen info, use it for better detection
    if let Some(info) = screen_info {
        if info.viewport_width < 560 {
            return "mobile".to_string();
        } else if info.viewport_width < 960 {
            return "tablet".to_string();
        } else {
            return "desktop".to_string();
        }
    }
    
    // Fallback to User-Agent detection
    if let Some(user_agent) = req.headers().get("user-agent") {
        if let Ok(ua_str) = user_agent.to_str() {
            let ua_lower = ua_str.to_lowercase();
            
            if ua_lower.contains("bot") || 
               ua_lower.contains("crawler") ||
               ua_lower.contains("spider") ||
               ua_lower.contains("googlebot") {
                return "bot".to_string();
            }
            
            if ua_lower.contains("mobile") || 
               ua_lower.contains("android") || 
               ua_lower.contains("iphone") ||
               ua_lower.contains("windows phone") ||
               ua_lower.contains("blackberry") {
                return "mobile".to_string();
            }
            
            if ua_lower.contains("ipad") || 
               ua_lower.contains("tablet") ||
               ua_lower.contains("kindle") ||
               ua_lower.contains("silk") {
                return "tablet".to_string();
            }
            
            if ua_lower.contains("smart-tv") ||
               ua_lower.contains("smarttv") ||
               ua_lower.contains("googletv") ||
               ua_lower.contains("appletv") {
                return "tv".to_string();
            }
            
            if ua_lower.contains("playstation") ||
               ua_lower.contains("xbox") ||
               ua_lower.contains("nintendo") {
                return "console".to_string();
            }
        }
    }
    
    "desktop".to_string()
}

/// Determine interaction mode based on device type, screen info and breakpoint
pub fn detect_interaction_mode(req: &HttpRequest, screen_info: &Option<ScreenInfo>, device_type: &str, breakpoint: &str) -> String {
    // Check for reader mode: no viewport or bot/crawler or screenreader
    if screen_info.is_none() || 
       (screen_info.as_ref().map_or(false, |s| {
           s.viewport_width < 1 || 
           (s.viewport_width < 1 && s.active_voices.unwrap_or(0) > 0)
       })) ||
       device_type == "bot" || 
       is_bot_request(req) {
        return "reader".to_string();
    }
    
    // Breakpoint-based detection is most reliable
    // phone/tablet = touch, screen/wide = mouse
    match breakpoint {
        "phone" | "tablet" => "touch".to_string(),
        "screen" | "wide" => "mouse".to_string(),
        _ => {
            // Fallback to device type detection if breakpoint is unknown
            if device_type == "mobile" || device_type == "tablet" {
                "touch".to_string()
            } else {
                "mouse".to_string()
            }
        }
    }
}

/// Determine CSS breakpoint based on device type and screen info
pub fn detect_breakpoint(req: &HttpRequest, screen_info: &Option<ScreenInfo>) -> String {
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
    
    // Fallback to device type detection
    let device = detect_device_type(req, screen_info);
    match device.as_str() {
        "mobile" => "phone".to_string(),
        "tablet" => "tablet".to_string(),
        "wide" | "tv" | "console" => "wide".to_string(),
        "bot" => "screen".to_string(),
        _ => "screen".to_string(),
    }
}

/// Helper to check if request is from a bot
pub fn is_bot_request(req: &HttpRequest) -> bool {
    req.headers().get("user-agent")
        .and_then(|h| h.to_str().ok())
        .map(|s| {
            let lower = s.to_lowercase();
            lower.contains("bot") || lower.contains("crawler") || lower.contains("spider")
        })
        .unwrap_or(false)
}

/// Generate screen detection HTML
pub fn generate_screen_detection_html() -> String {
    r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <style>body{margin:0;padding:0;}</style>
    <script>
    (function(){
        var d={
            width:screen.width,
            height:screen.height,
            dpr:window.devicePixelRatio||1,
            viewport_width:window.innerWidth||document.documentElement.clientWidth,
            viewport_height:window.innerHeight||document.documentElement.clientHeight
        };
        document.cookie='screen_info='+encodeURIComponent(JSON.stringify(d))+';path=/;max-age=31536000;SameSite=Lax';
        location.reload();
    })();
    </script>
</head>
<body></body>
</html>"#.to_string()
}