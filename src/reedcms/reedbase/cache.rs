// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! ReedBase Cache System
//!
//! O(1) HashMap cache for text, route, and meta CSV data.
//! Initialized once at startup, read-only during runtime.
//!
//! ## Architecture
//! - OnceLock singletons for thread-safe lazy initialization
//! - Nested HashMap structure: language -> (key -> value)
//! - Environment fallback: key@env -> key
//! - No Mutex needed: read-only after init
//!
//! ## Performance
//! - Cache initialization: < 50ms for typical dataset
//! - Lookup: < 1μs (O(1) HashMap access)
//! - Memory: ~1-5MB depending on content size
//!
//! ## Usage
//! ```rust
//! // At server startup
//! init_text_cache()?;
//! init_route_cache()?;
//! init_meta_cache()?;
//!
//! // During request
//! let value = get_text("knowledge.title", "de", Some("dev"))?;
//! ```

use crate::reedcms::reedstream::{ReedError, ReedResult};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::sync::OnceLock;

/// Global text cache: language -> (key -> value)
/// Format: HashMap<"de", HashMap<"knowledge.title", "Wissen">>
static TEXT_CACHE: OnceLock<HashMap<String, HashMap<String, String>>> = OnceLock::new();

/// Global route cache: language -> (key -> value)
/// Format: HashMap<"de", HashMap<"knowledge", "wissen">>
static ROUTE_CACHE: OnceLock<HashMap<String, HashMap<String, String>>> = OnceLock::new();

/// Global meta cache: key -> value (no language)
/// Format: HashMap<"knowledge.cache.ttl", "3600">
static META_CACHE: OnceLock<HashMap<String, String>> = OnceLock::new();

/// Global project cache: key -> value
static PROJECT_CACHE: OnceLock<HashMap<String, String>> = OnceLock::new();

/// Global server cache: key -> value
static SERVER_CACHE: OnceLock<HashMap<String, String>> = OnceLock::new();

/// Initialize text cache from .reed/text.csv
///
/// ## Process
/// 1. Read CSV file line by line
/// 2. Parse format: key@lang|value|description
/// 3. Group by language into nested HashMap
/// 4. Store in OnceLock singleton
///
/// ## Performance
/// - < 20ms for 1000 entries
/// - < 50ms for 5000 entries
///
/// ## Example
/// ```
/// init_text_cache()?;
/// ```
pub fn init_text_cache() -> ReedResult<()> {
    let cache = load_language_csv(".reed/text.csv")?;
    TEXT_CACHE.set(cache).map_err(|_| ReedError::ConfigError {
        component: "TEXT_CACHE".to_string(),
        reason: "Cache already initialized".to_string(),
    })
}

/// Initialize route cache from .reed/routes.csv
pub fn init_route_cache() -> ReedResult<()> {
    let cache = load_language_csv(".reed/routes.csv")?;
    ROUTE_CACHE.set(cache).map_err(|_| ReedError::ConfigError {
        component: "ROUTE_CACHE".to_string(),
        reason: "Cache already initialized".to_string(),
    })
}

/// Initialize meta cache from .reed/meta.csv
pub fn init_meta_cache() -> ReedResult<()> {
    let cache = load_flat_csv(".reed/meta.csv")?;
    META_CACHE.set(cache).map_err(|_| ReedError::ConfigError {
        component: "META_CACHE".to_string(),
        reason: "Cache already initialized".to_string(),
    })
}

/// Initialize project cache from .reed/project.csv
pub fn init_project_cache() -> ReedResult<()> {
    let cache = load_flat_csv(".reed/project.csv")?;
    PROJECT_CACHE
        .set(cache)
        .map_err(|_| ReedError::ConfigError {
            component: "PROJECT_CACHE".to_string(),
            reason: "Cache already initialized".to_string(),
        })
}

/// Initialize server cache from .reed/server.csv
pub fn init_server_cache() -> ReedResult<()> {
    let cache = load_flat_csv(".reed/server.csv")?;
    SERVER_CACHE.set(cache).map_err(|_| ReedError::ConfigError {
        component: "SERVER_CACHE".to_string(),
        reason: "Cache already initialized".to_string(),
    })
}

/// Get text value from cache with language and environment fallback
///
/// ## Fallback Chain
/// 1. Try: key@lang with environment
/// 2. Try: key@lang without environment
/// 3. Return: None if not found
///
/// ## Performance
/// - < 1μs typical (O(1) HashMap lookup)
///
/// ## Example
/// ```
/// let value = get_text("knowledge.title", "de", Some("dev"))?;
/// ```
pub fn get_text(key: &str, language: &str, environment: Option<&str>) -> ReedResult<String> {
    let cache = TEXT_CACHE.get().ok_or_else(|| ReedError::ConfigError {
        component: "TEXT_CACHE".to_string(),
        reason: "Cache not initialized - call init_text_cache() first".to_string(),
    })?;

    lookup_with_env(cache, key, language, environment)
}

/// Get route value from cache
pub fn get_route(key: &str, language: &str, environment: Option<&str>) -> ReedResult<String> {
    let cache = ROUTE_CACHE.get().ok_or_else(|| ReedError::ConfigError {
        component: "ROUTE_CACHE".to_string(),
        reason: "Cache not initialized - call init_route_cache() first".to_string(),
    })?;

    lookup_with_env(cache, key, language, environment)
}

/// Get meta value from cache (no language)
pub fn get_meta(key: &str, environment: Option<&str>) -> ReedResult<String> {
    let cache = META_CACHE.get().ok_or_else(|| ReedError::ConfigError {
        component: "META_CACHE".to_string(),
        reason: "Cache not initialized - call init_meta_cache() first".to_string(),
    })?;

    lookup_flat_with_env(cache, key, environment)
}

/// Get project config value from cache
pub fn get_project(key: &str, environment: Option<&str>) -> ReedResult<String> {
    let cache = PROJECT_CACHE.get().ok_or_else(|| ReedError::ConfigError {
        component: "PROJECT_CACHE".to_string(),
        reason: "Cache not initialized - call init_project_cache() first".to_string(),
    })?;

    lookup_flat_with_env(cache, key, environment)
}

/// Get server config value from cache
pub fn get_server(key: &str, environment: Option<&str>) -> ReedResult<String> {
    let cache = SERVER_CACHE.get().ok_or_else(|| ReedError::ConfigError {
        component: "SERVER_CACHE".to_string(),
        reason: "Cache not initialized - call init_server_cache() first".to_string(),
    })?;

    lookup_flat_with_env(cache, key, environment)
}

/// Check if caches are initialized
pub fn is_initialized() -> bool {
    TEXT_CACHE.get().is_some()
        && ROUTE_CACHE.get().is_some()
        && META_CACHE.get().is_some()
        && PROJECT_CACHE.get().is_some()
        && SERVER_CACHE.get().is_some()
}

/// Load language-aware CSV into nested HashMap
///
/// ## Input Format
/// ```csv
/// key@lang|value|description
/// knowledge.title@de|Wissen|German title
/// knowledge.title@en|Knowledge|English title
/// ```
///
/// ## Output Structure
/// ```rust
/// HashMap {
///   "de" => HashMap { "knowledge.title" => "Wissen" },
///   "en" => HashMap { "knowledge.title" => "Knowledge" }
/// }
/// ```
fn load_language_csv(path: &str) -> ReedResult<HashMap<String, HashMap<String, String>>> {
    let file = File::open(path).map_err(|e| ReedError::IoError {
        operation: "open".to_string(),
        path: path.to_string(),
        reason: e.to_string(),
    })?;

    let reader = BufReader::new(file);
    let mut cache: HashMap<String, HashMap<String, String>> = HashMap::new();

    for line_result in reader.lines() {
        let line = line_result.map_err(|e| ReedError::IoError {
            operation: "read_line".to_string(),
            path: path.to_string(),
            reason: e.to_string(),
        })?;

        // Skip empty lines and comments
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        // Parse: key@lang|value|description
        let parts: Vec<&str> = line.split('|').collect();
        if parts.len() < 2 {
            continue; // Skip invalid lines
        }

        let key_with_lang = parts[0].trim();
        let value = parts[1].trim();

        // Extract language from key@lang
        if let Some((base_key, lang)) = key_with_lang.rsplit_once('@') {
            cache
                .entry(lang.to_string())
                .or_default()
                .insert(base_key.to_string(), value.to_string());
        } else {
            // No language suffix - store in "default" language
            cache
                .entry("default".to_string())
                .or_default()
                .insert(key_with_lang.to_string(), value.to_string());
        }
    }

    Ok(cache)
}

/// Load flat CSV into simple HashMap (no language)
///
/// ## Input Format
/// ```csv
/// key|value|description
/// knowledge.cache.ttl|3600|Cache seconds
/// ```
///
/// ## Output Structure
/// ```rust
/// HashMap { "knowledge.cache.ttl" => "3600" }
/// ```
fn load_flat_csv(path: &str) -> ReedResult<HashMap<String, String>> {
    let file = File::open(path).map_err(|e| ReedError::IoError {
        operation: "open".to_string(),
        path: path.to_string(),
        reason: e.to_string(),
    })?;

    let reader = BufReader::new(file);
    let mut cache: HashMap<String, String> = HashMap::new();

    for line_result in reader.lines() {
        let line = line_result.map_err(|e| ReedError::IoError {
            operation: "read_line".to_string(),
            path: path.to_string(),
            reason: e.to_string(),
        })?;

        // Skip empty lines and comments
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        // Parse: key|value|description
        let parts: Vec<&str> = line.split('|').collect();
        if parts.len() < 2 {
            continue;
        }

        let key = parts[0].trim();
        let value = parts[1].trim();

        cache.insert(key.to_string(), value.to_string());
    }

    Ok(cache)
}

/// Lookup with environment fallback in nested HashMap
///
/// ## Fallback Chain
/// 1. key with environment: lookup in language map
/// 2. key without environment: lookup in language map
/// 3. NotFound error
///
/// ## Note
/// This function now uses the extracted environment module.
fn lookup_with_env(
    cache: &HashMap<String, HashMap<String, String>>,
    key: &str,
    language: &str,
    environment: Option<&str>,
) -> ReedResult<String> {
    use crate::reedcms::reedbase::environment;
    environment::resolve_with_fallback(cache, key, language, environment)
}

/// Lookup with environment fallback in flat HashMap
///
/// ## Note
/// This function now uses the extracted environment module.
fn lookup_flat_with_env(
    cache: &HashMap<String, String>,
    key: &str,
    environment: Option<&str>,
) -> ReedResult<String> {
    use crate::reedcms::reedbase::environment;
    environment::resolve_flat_with_fallback(cache, key, environment)
}
