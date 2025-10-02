// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Screen detection HTML generation for first-visit detection.
//!
//! Generates minimal HTML page with JavaScript that:
//! 1. Detects screen dimensions and device information
//! 2. Sets screen_info cookie
//! 3. Reloads page for actual content delivery
//!
//! ## Performance
//! - One-time delay on first visit only (< 100ms)
//! - Cookie persists for 1 year
//! - Subsequent visits: No detection needed

use crate::reedcms::server::client_detection::parse_screen_info_cookie;
use actix_web::HttpRequest;

/// Generates minimal HTML page for screen detection.
///
/// ## Purpose
/// - First visit without screen_info cookie
/// - JavaScript detects screen dimensions
/// - Sets cookie and reloads immediately
///
/// ## Performance
/// - HTML generation: < 1ms
/// - JavaScript execution: < 100ms
/// - Total first-visit delay: < 200ms
///
/// ## Cookie Set
/// - Name: screen_info
/// - Value: URL-encoded JSON
/// - Max-Age: 31536000 (1 year)
/// - Path: /
/// - SameSite: Lax
///
/// ## Example Usage
/// ```rust
/// if needs_screen_detection(&req) {
///     return Ok(HttpResponse::Ok()
///         .content_type("text/html; charset=utf-8")
///         .body(generate_screen_detection_html()));
/// }
/// ```
pub fn generate_screen_detection_html() -> String {
    r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Loading...</title>
    <style>
        body {
            margin: 0;
            padding: 20px;
            font-family: system-ui, -apple-system, sans-serif;
            background: #fff;
            color: #333;
            text-align: center;
        }
        .loader {
            margin: 50px auto;
            font-size: 18px;
        }
    </style>
    <script>
    (function(){
        var d = {
            width: screen.width,
            height: screen.height,
            dpr: window.devicePixelRatio || 1,
            viewport_width: window.innerWidth || document.documentElement.clientWidth,
            viewport_height: window.innerHeight || document.documentElement.clientHeight,
            active_voices: 0
        };

        // Detect screen reader voices (if available)
        if (window.speechSynthesis) {
            try {
                d.active_voices = window.speechSynthesis.getVoices().length;
            } catch(e) {
                d.active_voices = 0;
            }
        }

        // Set cookie
        document.cookie = 'screen_info=' +
            encodeURIComponent(JSON.stringify(d)) +
            ';path=/;max-age=31536000;SameSite=Lax';

        // Reload page
        location.reload();
    })();
    </script>
</head>
<body>
    <div class="loader">
        <p>Detecting screen dimensions...</p>
        <p><small>This happens only once.</small></p>
    </div>
</body>
</html>"#
        .to_string()
}

/// Checks if screen detection is needed.
///
/// ## Returns
/// - true: No screen_info cookie present, send detection HTML
/// - false: Cookie exists, proceed with normal rendering
///
/// ## Performance
/// - < 1ms cookie check
///
/// ## Example Usage
/// ```rust
/// if needs_screen_detection(&req) && !is_bot {
///     // Send detection HTML
/// } else {
///     // Normal rendering
/// }
/// ```
pub fn needs_screen_detection(req: &HttpRequest) -> bool {
    parse_screen_info_cookie(req).ok().flatten().is_none()
}
