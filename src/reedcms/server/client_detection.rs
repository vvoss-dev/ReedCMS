// Copyright 2025 Vivian Voss. Licensed under the Apache Licence, Version 2.0.
// SPDX-Licence-Identifier: Apache-2.0

//! Client detection service for ReedCMS.
//!
//! Detects device information from screen_info cookie and User-Agent fallback.
//! Determines interaction mode (mouse/touch/reader) for template variant selection.
//!
//! ## Detection Strategy
//! 1. Parse screen_info cookie (viewport dimensions, DPR, screen reader voices)
//! 2. Detect device type (mobile/tablet/desktop/bot)
//! 3. Determine CSS breakpoint (phone/tablet/screen/wide)
//! 4. Select interaction mode (mouse/touch/reader)
//! 5. Detect bots and crawlers
//!
//! ## Performance
//! - With cookie: < 5ms
//! - User-Agent fallback: < 10ms
//! - Bot detection: < 1ms

use crate::reedcms::reedstream::ReedResult;
use actix_web::HttpRequest;
use serde::{Deserialize, Serialize};

/// Client information for template context.
///
/// Contains all detected client information for template rendering.
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

/// Screen information from cookie.
///
/// ## Cookie Format
/// - Name: screen_info
/// - Value: URL-encoded JSON
/// - Max-Age: 31536000 (1 year)
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
/// ## Arguments
/// - req: HTTP request
/// - lang: Language code from routing layer
///
/// ## Returns
/// - ClientInfo struct with all detected information
///
/// ## Performance
/// - < 5ms with screen_info cookie
/// - < 10ms with User-Agent fallback
///
/// ## Example Usage
/// ```rust
/// let client_info = detect_client_info(&req, "de")?;
/// // ClientInfo { lang: "de", interaction_mode: "mouse", ... }
/// ```
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
/// ## Returns
/// - Some(ScreenInfo) if cookie present and valid
/// - None if cookie missing or invalid
///
/// ## Performance
/// - < 2ms cookie parsing
pub fn parse_screen_info_cookie(req: &HttpRequest) -> ReedResult<Option<ScreenInfo>> {
    // Get cookie header
    let cookie_header = req.headers().get("cookie").and_then(|h| h.to_str().ok());

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
/// ## Detection Rules
/// - viewport_width < 560px → mobile
/// - viewport_width < 960px → tablet
/// - viewport_width >= 960px → desktop
/// - User-Agent fallback if no screen_info
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
            if ua_lower.contains("bot")
                || ua_lower.contains("crawler")
                || ua_lower.contains("spider")
                || ua_lower.contains("googlebot")
            {
                return "bot".to_string();
            }

            // Mobile detection
            if ua_lower.contains("mobile")
                || ua_lower.contains("android")
                || ua_lower.contains("iphone")
                || ua_lower.contains("windows phone")
                || ua_lower.contains("blackberry")
            {
                return "mobile".to_string();
            }

            // Tablet detection
            if ua_lower.contains("ipad")
                || ua_lower.contains("tablet")
                || ua_lower.contains("kindle")
            {
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
/// ## Performance
/// - O(1) comparison
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
/// ## Performance
/// - O(1) decision tree
fn detect_interaction_mode(
    screen_info: &Option<ScreenInfo>,
    device_type: &str,
    breakpoint: &str,
    is_bot: bool,
) -> String {
    // Priority 1: Reader mode
    // - No screen info available
    // - Bot/crawler
    // - Screen reader detected (active_voices > 0)
    if screen_info.is_none()
        || is_bot
        || device_type == "bot"
        || screen_info.as_ref().map_or(false, |s| {
            s.viewport_width < 1 || s.active_voices.unwrap_or(0) > 0
        })
    {
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
/// - Checks User-Agent header for bot keywords
///
/// ## Performance
/// - < 1ms string search
pub fn is_bot_request(req: &HttpRequest) -> bool {
    req.headers()
        .get("user-agent")
        .and_then(|h| h.to_str().ok())
        .map(|s| {
            let lower = s.to_lowercase();
            lower.contains("bot") || lower.contains("crawler") || lower.contains("spider")
        })
        .unwrap_or(false)
}
