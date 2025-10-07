// Copyright 2025 Vivian Voss. Licensed under the Apache Licence, Version 2.0.
// SPDX-Licence-Identifier: Apache-2.0

//! Template context building system.
//!
//! Prepares data for template rendering with ReedBase integration.

use crate::reedcms::reedstream::{ReedError, ReedRequest, ReedResult};
use crate::reedcms::server::client_detection::ClientInfo;
use std::collections::HashMap;

/// Builds template context for rendering.
///
/// ## Arguments
/// - layout: Layout name (e.g., "knowledge")
/// - language: Language code (e.g., "en", "de")
/// - client_info: Client detection information (breakpoint, interaction_mode, etc.)
///
/// ## Context Variables
/// - layout: Current layout name
/// - lang: Current language code
/// - interaction_mode: Current interaction mode (mouse/touch/reader)
/// - site_name: Project name from config
/// - site_url: Base URL from config
/// - languages: Available languages
/// - current_year: Current year for copyright
/// - version: ReedCMS version
///
/// ## Performance
/// - Context building: < 5ms
/// - Memory usage: < 1KB per context
///
/// ## Output
/// - HashMap ready for MiniJinja rendering
pub fn build_context(
    layout: &str,
    language: &str,
    client_info: &ClientInfo,
) -> ReedResult<HashMap<String, serde_json::Value>> {
    let mut ctx = HashMap::new();

    // Core variables
    ctx.insert("pagekey".to_string(), serde_json::json!(layout));
    ctx.insert("lang".to_string(), serde_json::json!(language));
    ctx.insert(
        "interaction_mode".to_string(),
        serde_json::json!(&client_info.interaction_mode),
    );

    // Client object for template compatibility
    let mut client = HashMap::new();
    client.insert("lang", serde_json::json!(language));
    client.insert(
        "interaction_mode",
        serde_json::json!(&client_info.interaction_mode),
    );
    client.insert("breakpoint", serde_json::json!(&client_info.breakpoint));
    client.insert("device_type", serde_json::json!(&client_info.device_type));

    // Optional screen dimensions from cookie
    if let Some(vw) = client_info.viewport_width {
        client.insert("viewport_width", serde_json::json!(vw));
    }
    if let Some(vh) = client_info.viewport_height {
        client.insert("viewport_height", serde_json::json!(vh));
    }
    if let Some(sw) = client_info.screen_width {
        client.insert("screen_width", serde_json::json!(sw));
    }
    if let Some(sh) = client_info.screen_height {
        client.insert("screen_height", serde_json::json!(sh));
    }
    if let Some(dpr) = client_info.dpr {
        client.insert("dpr", serde_json::json!(dpr));
    }

    ctx.insert("client".to_string(), serde_json::json!(client));

    // Layout name
    ctx.insert("layout_name".to_string(), serde_json::json!(layout));

    // Config object
    let mut config = HashMap::new();
    // Get actual session hash from ReedBase
    let session_hash = crate::reedcms::assets::css::session_hash::get_session_hash()
        .unwrap_or_else(|_| "dev42".to_string());
    config.insert("session_hash", serde_json::json!(session_hash));
    config.insert("dev_mode", serde_json::json!(cfg!(debug_assertions))); // True in debug builds
    ctx.insert("config".to_string(), serde_json::json!(config));

    // Optional page metadata (templates use these with fallbacks)
    ctx.insert("page_title".to_string(), serde_json::Value::Null);
    ctx.insert("page_description".to_string(), serde_json::Value::Null);
    ctx.insert("current_pagekey".to_string(), serde_json::json!(layout));

    // Page object (for legacy template compatibility)
    let mut page = HashMap::new();
    // Get latest update from git or use current date
    let latest_update = std::process::Command::new("git")
        .args(&["log", "-1", "--format=%cd", "--date=short"])
        .output()
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| chrono::Local::now().format("%Y-%m-%d").to_string());
    page.insert("latest_update", serde_json::json!(latest_update));
    ctx.insert("page".to_string(), serde_json::json!(page));

    // Add globals
    add_globals(&mut ctx)?;

    // Add layout-specific data
    add_layout_data(&mut ctx, layout, language)?;

    Ok(ctx)
}

/// Adds global variables to context.
///
/// ## Global Variables
/// - site_name: Project name from config
/// - site_url: Base URL from config
/// - languages: Available languages
/// - current_year: Current year for copyright
/// - version: ReedCMS version
///
/// ## Example Context
/// ```jinja
/// {{ site_name }}           {# ReedCMS Documentation #}
/// {{ languages | join(", ") }} {# en, de, fr #}
/// {{ current_year }}        {# 2025 #}
/// ```
pub fn add_globals(ctx: &mut HashMap<String, serde_json::Value>) -> ReedResult<()> {
    // Site information from project config
    let site_name = get_config_value("name").unwrap_or_else(|_| "ReedCMS".to_string());
    let site_url = get_config_value("url").unwrap_or_else(|_| "https://example.com".to_string());
    let languages_str = get_config_value("languages").unwrap_or_else(|_| "en,de".to_string());

    let languages: Vec<String> = languages_str
        .split(',')
        .map(|s| s.trim().to_string())
        .collect();

    ctx.insert("site_name".to_string(), serde_json::json!(site_name));
    ctx.insert("site_url".to_string(), serde_json::json!(site_url));
    ctx.insert("languages".to_string(), serde_json::json!(languages));

    // System information
    use chrono::Datelike;
    ctx.insert(
        "current_year".to_string(),
        serde_json::json!(chrono::Utc::now().year()),
    );
    ctx.insert(
        "version".to_string(),
        serde_json::json!(env!("CARGO_PKG_VERSION")),
    );

    Ok(())
}

/// Adds layout-specific data to context.
///
/// ## Layout Data
/// - layout_title: Title from text.csv
/// - layout_description: Description from text.csv
/// - cache_ttl: Cache TTL from meta.csv
///
/// ## Example Context
/// ```jinja
/// {{ layout_title }}        {# Knowledge Base #}
/// {{ layout_description }}  {# Comprehensive documentation #}
/// ```
pub fn add_layout_data(
    ctx: &mut HashMap<String, serde_json::Value>,
    layout: &str,
    language: &str,
) -> ReedResult<()> {
    // Layout title
    let title_key = format!("{}.title", layout);
    if let Ok(title) = get_text_value(&title_key, language) {
        ctx.insert("layout_title".to_string(), serde_json::json!(title));
    }

    // Layout description
    let desc_key = format!("{}.description", layout);
    if let Ok(description) = get_text_value(&desc_key, language) {
        ctx.insert(
            "layout_description".to_string(),
            serde_json::json!(description),
        );
    }

    // Layout meta data
    let cache_ttl_key = format!("{}.cache.ttl", layout);
    if let Ok(ttl) = get_meta_value(&cache_ttl_key) {
        ctx.insert("cache_ttl".to_string(), serde_json::json!(ttl));
    }

    Ok(())
}

/// Gets text value from ReedBase.
fn get_text_value(key: &str, language: &str) -> ReedResult<String> {
    let req = ReedRequest {
        key: key.to_string(),
        language: Some(language.to_string()),
        environment: None,
        context: None,
        value: None,
        description: None,
    };

    match crate::reedcms::reedbase::get::text(&req) {
        Ok(response) => Ok(response.data),
        Err(e) => Err(e),
    }
}

/// Gets meta value from ReedBase.
fn get_meta_value(key: &str) -> ReedResult<String> {
    let req = ReedRequest {
        key: key.to_string(),
        language: None,
        environment: None,
        context: None,
        value: None,
        description: None,
    };

    match crate::reedcms::reedbase::get::meta(&req) {
        Ok(response) => Ok(response.data),
        Err(e) => Err(e),
    }
}

/// Gets config value from ReedBase.
///
/// ## Auto-Detection
/// Tries project.{key} first, then server.{key}
fn get_config_value(key: &str) -> ReedResult<String> {
    // Try project first
    let project_key = format!("project.{}", key);
    let req = ReedRequest {
        key: project_key.clone(),
        language: None,
        environment: None,
        context: None,
        value: None,
        description: None,
    };

    if let Ok(response) = crate::reedcms::reedbase::get::project(&req) {
        return Ok(response.data);
    }

    // Try server second
    let server_key = format!("server.{}", key);
    let req = ReedRequest {
        key: server_key.clone(),
        language: None,
        environment: None,
        context: None,
        value: None,
        description: None,
    };

    match crate::reedcms::reedbase::get::server(&req) {
        Ok(response) => Ok(response.data),
        Err(_) => Err(ReedError::NotFound {
            resource: format!("Config key '{}'", key),
            context: Some(format!("Tried project.{} and server.{}", key, key)),
        }),
    }
}
