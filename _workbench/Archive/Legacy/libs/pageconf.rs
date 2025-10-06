use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};
use std::process::Command;
use std::path::PathBuf;
use serde_json::{Value as JsonValue};

/// Global PageConfig singleton for S06-03 static pageconf system
/// Populated once at startup by scan_layout_pageconf()
static GLOBAL_PAGE_CONFIG: OnceLock<PageConfig> = OnceLock::new();

/// Global route-to-layout mappings from S06-03 static scanning
/// Populated once at startup by scan_layout_pageconf()
static GLOBAL_ROUTE_MAPPINGS: OnceLock<HashMap<String, String>> = OnceLock::new();

/// Get reference to global PageConfig singleton
pub fn get_global_page_config() -> &'static PageConfig {
    GLOBAL_PAGE_CONFIG.get().expect("Global PageConfig not initialised - call scan_layout_pageconf() at startup")
}

/// Get reference to global route mappings
pub fn get_global_route_mappings() -> &'static HashMap<String, String> {
    GLOBAL_ROUTE_MAPPINGS.get().expect("Global route mappings not initialised - call scan_layout_pageconf() at startup")
}

/// Get pagekey from route by reverse lookup of pageroute@pagekey values
/// Returns the pagekey that has the specified route as its pageroute value
pub fn get_pagekey_from_route(route: &str) -> Option<String> {
    let global_page_config = get_global_page_config();
    let storage = global_page_config.storage.lock().unwrap();

    for (scoped_key, value) in storage.iter() {
        if scoped_key.starts_with("pageroute@") {
            // Handle ROOT route mapping: empty route "" matches "ROOT" value
            let route_matches = if route.is_empty() && value == "ROOT" {
                true
            } else {
                value == route
            };

            if route_matches {
                // Extract pagekey from "pageroute@test" → "test"
                if let Some(pagekey) = scoped_key.strip_prefix("pageroute@") {
                    return Some(pagekey.to_string());
                }
            }
        }
    }
    None
}

/// Build reed context with auto-resolution for a specific pagekey
/// Creates a JSON object with both scoped and auto-resolved values for template access
pub fn build_reed_context_for_pagekey(pagekey: &str) -> serde_json::Value {
    let global_page_config = get_global_page_config();
    let storage = global_page_config.storage.lock().unwrap();
    let mut result_map = std::collections::BTreeMap::new();

    // Add all scoped values directly
    for (scoped_key, value) in storage.iter() {
        result_map.insert(scoped_key.clone(), serde_json::Value::String(value.clone()));
    }

    // Add auto-resolved values for specified pagekey
    for (scoped_key, value) in storage.iter() {
        if scoped_key.ends_with(&format!("@{}", pagekey)) {
            // Extract base key from "pagekey@test" → "pagekey"
            if let Some(base_key) = scoped_key.strip_suffix(&format!("@{}", pagekey)) {
                // Only add if not already present (scoped access takes precedence)
                if !result_map.contains_key(base_key) {
                    result_map.insert(base_key.to_string(), serde_json::Value::String(value.clone()));
                }
            }
        }
    }

    serde_json::Value::Object(result_map.into_iter().collect())
}

/// Global page configuration storage for reed.* access pattern with @pagekey scoping
/// Thread-safe storage for page metadata set by pageconf() function
///
/// Storage format:
/// - Scoped keys: "pagekey@landing" → "landing"
/// - Local access: "pagekey" auto-resolves to current page
/// - Global access: "pagekey@landing" for specific page
#[derive(Debug, Clone)]
pub struct PageConfig {
    /// Configuration storage: scoped_key → value
    /// Format: "key@pagekey" → "value"
    storage: Arc<Mutex<HashMap<String, String>>>,
    /// Current pagekey for auto-resolution of local access
    current_pagekey: Arc<Mutex<Option<String>>>,
}

impl PageConfig {
    /// Create new empty page configuration
    pub fn new() -> Self {
        Self {
            storage: Arc::new(Mutex::new(HashMap::new())),
            current_pagekey: Arc::new(Mutex::new(None)),
        }
    }

    /// Set a configuration value with @pagekey scoping (called by pageconf() function)
    pub fn set(&self, key: &str, value: &str) {
        let mut storage = self.storage.lock().unwrap();
        let current_pagekey = self.current_pagekey.lock().unwrap();

        // Handle pagekey setting specially - this sets the current pagekey
        if key == "pagekey" {
            drop(current_pagekey); // Release lock before acquiring again
            let mut pagekey_lock = self.current_pagekey.lock().unwrap();
            *pagekey_lock = Some(value.to_string());
            drop(pagekey_lock);

            // Store with scoping: pagekey@landing = "landing"
            let scoped_key = format!("{}@{}", key, value);
            storage.insert(scoped_key, value.to_string());
        } else {
            // For other keys, use current pagekey for scoping
            if let Some(ref pagekey) = *current_pagekey {
                let scoped_key = format!("{}@{}", key, pagekey);
                storage.insert(scoped_key, value.to_string());
            } else {
                // Fallback: store without scoping if no current pagekey
                storage.insert(key.to_string(), value.to_string());
            }
        }
    }

    /// Get a configuration value with auto-resolution
    ///
    /// Access patterns:
    /// - "pagekey" → auto-resolves to current page (reed.pagekey)
    /// - "pagekey@landing" → specific page access (reed.pagekey@landing)
    pub fn get(&self, key: &str) -> Option<String> {
        let storage = self.storage.lock().unwrap();

        // Direct scoped access (e.g., "pagekey@landing")
        if key.contains('@') {
            return storage.get(key).cloned();
        }

        // Auto-resolution for local access
        let current_pagekey = self.current_pagekey.lock().unwrap();
        if let Some(ref pagekey) = *current_pagekey {
            let scoped_key = format!("{}@{}", key, pagekey);
            if let Some(value) = storage.get(&scoped_key) {
                return Some(value.clone());
            }
        }

        // Fallback: direct key lookup (for backward compatibility)
        storage.get(key).cloned()
    }

    /// Get all configuration as JSON Value for template context with auto-resolution
    ///
    /// Returns both scoped and auto-resolved values:
    /// - "pagekey": "landing" (auto-resolved for current page)
    /// - "pagekey@landing": "landing" (explicit scoped access)
    /// - "pagetitle@knowledge": "Knowledge Page" (cross-page access)
    pub fn to_json_value(&self) -> JsonValue {
        let storage = self.storage.lock().unwrap();
        let current_pagekey = self.current_pagekey.lock().unwrap();
        let mut result_map = std::collections::BTreeMap::new();

        // Add all scoped values directly
        for (scoped_key, value) in storage.iter() {
            result_map.insert(scoped_key.clone(), JsonValue::String(value.clone()));
        }

        // Add auto-resolved values for current page (if current pagekey is set)
        if let Some(ref pagekey) = *current_pagekey {
            for (scoped_key, value) in storage.iter() {
                if scoped_key.ends_with(&format!("@{}", pagekey)) {
                    // Extract base key from "pagekey@landing" → "pagekey"
                    if let Some(base_key) = scoped_key.strip_suffix(&format!("@{}", pagekey)) {
                        // Only add if not already present (scoped access takes precedence)
                        if !result_map.contains_key(base_key) {
                            result_map.insert(base_key.to_string(), JsonValue::String(value.clone()));
                        }
                    }
                }
            }
        }

        JsonValue::Object(result_map.into_iter().collect())
    }

    /// Clear all configuration (for new requests)
    pub fn clear(&self) {
        let mut storage = self.storage.lock().unwrap();
        storage.clear();

        let mut current_pagekey = self.current_pagekey.lock().unwrap();
        *current_pagekey = None;
    }

    /// Check if a key exists
    pub fn contains_key(&self, key: &str) -> bool {
        let storage = self.storage.lock().unwrap();
        storage.contains_key(key)
    }

    /// Get all keys
    pub fn keys(&self) -> Vec<String> {
        let storage = self.storage.lock().unwrap();
        storage.keys().cloned().collect()
    }
}

impl Default for PageConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Create pageconf() function for MiniJinja templates
/// Only callable in layout templates - sets page configuration values
///
/// Usage: {{ pageconf("pagekey", "landing") }}
/// Usage: {{ pageconf("pagetitle", "Welcome Page") }}
pub fn make_pageconf_function(
    page_config: PageConfig,
) -> impl Fn(String, String) -> Result<String, minijinja::Error> + Send + Sync + 'static {
    move |key: String, value: String| -> Result<String, minijinja::Error> {
        // Set the configuration value
        page_config.set(&key, &value);

        // Return empty string (silent function - no output)
        Ok(String::new())
    }
}

/// Enhanced page configuration with fallback strategy
/// Provides intelligent lookup with hierarchy:
/// 1. PageConfig (from pageconf() calls)
/// 2. Component metadata (system.csv)
/// 3. TEXT registry (global text constants)
/// 4. Fallback key
#[allow(dead_code)]
pub fn get_page_config_with_fallback(
    page_config: &PageConfig,
    key: &str,
    _component_name: Option<&str>,
    _language: &str,
    fallback: Option<&str>,
) -> String {
    // 1. Check page_config storage first (from pageconf() calls)
    if let Some(value) = page_config.get(key) {
        return value;
    }

    // 2. Check component system.csv for META_* keys
    // TODO: Implement component registry lookup when needed
    // let meta_key = format!("META_{}", key.to_uppercase());
    // if let Some(component) = component_name {
    //     if let Some(value) = get_component_meta(component, &meta_key, language) {
    //         return value;
    //     }
    // }

    // 3. Check TEXT registry for PAGE_* keys
    // TODO: Implement TEXT registry lookup when needed
    // let text_key = format!("PAGE_{}", key.to_uppercase());
    // if let Some(value) = crate::TEXT!().get_with_language(&text_key, language) {
    //     return value;
    // }

    // 4. Return fallback or key itself
    fallback.unwrap_or(key).to_string()
}

/// Helper function to find all .jinja files in layouts/ using fd
fn fd_jinja_files() -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    let output = Command::new("fd")
        .arg("-t")
        .arg("f")
        .arg("-e")
        .arg("jinja")
        .arg(".")
        .arg("templates/layouts/")
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("fd command failed: {}", stderr).into());
    }

    let paths = String::from_utf8(output.stdout)?
        .lines()
        .filter(|line| !line.is_empty())
        .map(PathBuf::from)
        .collect();

    Ok(paths)
}

/// Helper function to check if a file contains pageconf() calls
fn file_contains_pageconf(file_path: &PathBuf) -> Result<bool, Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string(file_path)?;
    Ok(content.contains("pageconf("))
}

/// Static scanner for layout templates to extract pageconf() calls at startup
/// Replaces routing.csv by scanning all layout/*.jinja files for pageconf() calls
pub fn scan_layout_pageconf() -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
    let mut route_to_layout = HashMap::new();

    log::info!("S06-03: Starting static pageconf scanning...");
    log::debug!("S06-03: Working directory: {:?}", std::env::current_dir().unwrap_or_default());

    // Create temporary PageConfig for accumulating all values
    let temp_page_config = PageConfig::new();

    // Step 1: Find all .jinja files in layouts/ using fd
    let all_jinja_files = fd_jinja_files()?;
    log::debug!("S06-03: Found {} total .jinja files in layouts/", all_jinja_files.len());

    // Step 2: Filter files that contain pageconf() calls
    let mut pageconf_files = Vec::new();
    for file_path in all_jinja_files {
        if file_contains_pageconf(&file_path)? {
            pageconf_files.push(file_path);
        }
    }

    log::info!("S06-03: Found {} files with pageconf calls", pageconf_files.len());

    // Step 3: For each file, extract ALL pageconf() values
    for file_path in pageconf_files {
        log::info!("S06-03: Processing file: {:?}", file_path);

        // Extract layout name from path: templates/layouts/landing/landing.jinja -> landing
        let layout_name = file_path.file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or("unknown");

        // Read the entire file to extract all pageconf() calls
        if let Ok(content) = std::fs::read_to_string(&file_path) {
            let lines: Vec<&str> = content.lines().collect();

            for line in lines {
                if line.contains("pageconf(") {
                    // Simple string parsing: {{ pageconf("key", "value") }}
                    if let Some(start) = line.find("pageconf(\"") {
                        let rest = &line[start + 10..]; // Skip 'pageconf("'
                        if let Some(key_end) = rest.find("\", \"") {
                            let key = &rest[..key_end];
                            let value_start = key_end + 4; // Skip '", "'
                            if let Some(value_end) = rest[value_start..].find("\")") {
                                let value = &rest[value_start..value_start + value_end];

                                // Set in temporary PageConfig (will handle scoping automatically)
                                temp_page_config.set(key, value);

                                // Build route mapping from pageroute values
                                if key == "pageroute" {
                                    let lookup_key = if value.is_empty() { "ROOT".to_string() } else { value.to_string() };
                                    route_to_layout.insert(lookup_key.clone(), layout_name.to_string());
                                    log::info!("S06-03: Mapped route '{}' -> layout '{}'", lookup_key, layout_name);
                                }

                                log::debug!("S06-03: Extracted pageconf {}='{}' from {}", key, value, layout_name);
                            }
                        }
                    }
                }
            }
        }
    }

    // Step 3: Initialise global PageConfig singleton with all collected values
    let global_config = PageConfig::new();

    // Copy all values from temp config to global config
    let temp_storage = temp_page_config.storage.lock().unwrap();
    for (key, value) in temp_storage.iter() {
        global_config.storage.lock().unwrap().insert(key.clone(), value.clone());
    }
    drop(temp_storage);

    // Set global singletons
    GLOBAL_PAGE_CONFIG.set(global_config).map_err(|_| "Failed to set global PageConfig singleton")?;
    GLOBAL_ROUTE_MAPPINGS.set(route_to_layout.clone()).map_err(|_| "Failed to set global route mappings singleton")?;

    log::info!("S06-03: Scanning complete. Found {} route mappings and populated global PageConfig", route_to_layout.len());
    Ok(route_to_layout)
}

/// Extract pageconf() value from template content using regex
fn extract_pageconf_value(content: &str, key: &str) -> Option<String> {
    // Look for: {{ pageconf("pageroute", "value") }}
    let pattern = format!(r#"{{\s*pageconf\s*\(\s*"{}"\s*,\s*"([^"]*)"\s*\)\s*}}"#, key);
    if let Ok(regex) = regex::Regex::new(&pattern) {
        if let Some(captures) = regex.captures(content) {
            return captures.get(1).map(|m| m.as_str().to_string());
        }
    }
    None
}


