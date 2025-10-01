use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Parse environment-aware key into (registry_key, environment)
/// Examples: "SERVER_PORT@DEV" → ("SERVER_PORT", "DEV")
///          "SERVER_PORT" → ("SERVER_PORT", "DEFAULT")
pub fn parse_env_key(key: &str) -> (&str, &str) {
    if let Some(at_pos) = key.find('@') {
        (&key[..at_pos], &key[at_pos + 1..])
    } else {
        (key, "DEFAULT")
    }
}

/// Generic CSV loader for environment-aware registries
/// Returns nested HashMap: environment → registry_key → value
/// Parses CSV with format: key;value;comment (comment optional)
/// Supports environment suffixes: key@ENV;value;comment
pub fn load_environment_csv(path: &str) -> Result<HashMap<String, HashMap<String, String>>, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;
    let mut registry: HashMap<String, HashMap<String, String>> = HashMap::new();

    // Parse CSV and populate environment-aware HashMaps
    for line in content.lines().skip(1) { // Skip header
        if line.trim().is_empty() {
            continue;
        }

        let parts: Vec<&str> = line.split(';').collect();
        if parts.len() >= 2 {
            let key = parts[0].trim();
            let value = parts[1].trim();

            // Parse key@environment or base key
            let (registry_key, env) = parse_env_key(key);

            // Store in nested HashMap structure: env → key → value
            registry.entry(env.to_string())
                .or_insert_with(HashMap::new)
                .insert(registry_key.to_string(), value.to_string());
        }
    }

    Ok(registry)
}

/// Get environment-aware value with automatic fallback to DEFAULT
pub fn get_env_value(
    registry: &HashMap<String, HashMap<String, String>>,
    current_env: &str,
    key: &str,
) -> Option<String> {
    registry.get(current_env)
        .and_then(|env_map| env_map.get(key))
        .or_else(|| {
            registry.get("DEFAULT")
                .and_then(|default_map| default_map.get(key))
        })
        .cloned()
}

/// Generic typed getter functions for common value types
pub fn get_env_bool(
    registry: &HashMap<String, HashMap<String, String>>,
    current_env: &str,
    key: &str,
    default: bool,
) -> bool {
    get_env_value(registry, current_env, key)
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

pub fn get_env_u32(
    registry: &HashMap<String, HashMap<String, String>>,
    current_env: &str,
    key: &str,
    default: u32,
) -> u32 {
    get_env_value(registry, current_env, key)
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

pub fn get_env_u64(
    registry: &HashMap<String, HashMap<String, String>>,
    current_env: &str,
    key: &str,
    default: u64,
) -> u64 {
    get_env_value(registry, current_env, key)
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

pub fn get_env_string(
    registry: &HashMap<String, HashMap<String, String>>,
    current_env: &str,
    key: &str,
    default: &str,
) -> String {
    get_env_value(registry, current_env, key)
        .unwrap_or_else(|| default.to_string())
}

pub fn get_env_vec_string(
    registry: &HashMap<String, HashMap<String, String>>,
    current_env: &str,
    key: &str,
    default: Vec<String>,
) -> Vec<String> {
    get_env_value(registry, current_env, key)
        .map(|v| v.split(',').map(|s| s.trim().to_string()).collect())
        .unwrap_or(default)
}

/// Load multiple CSV files from a directory and merge them into a single registry
/// Useful for scope-based configuration (e.g., keybase/system/*.csv)
pub fn load_environment_csv_directory(dir_path: &str) -> Result<HashMap<String, HashMap<String, String>>, Box<dyn std::error::Error>> {
    let mut merged_registry: HashMap<String, HashMap<String, String>> = HashMap::new();

    // Check if directory exists
    if !Path::new(dir_path).is_dir() {
        return Err(format!("Directory not found: {}", dir_path).into());
    }

    // Read all CSV files in directory
    let entries = fs::read_dir(dir_path)?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        // Only process .csv files
        if let Some(extension) = path.extension() {
            if extension == "csv" {
                if let Some(file_path_str) = path.to_str() {
                    // Load individual CSV file
                    match load_environment_csv(file_path_str) {
                        Ok(file_registry) => {
                            // Merge into main registry
                            for (env_key, env_map) in file_registry {
                                let merged_env_map = merged_registry.entry(env_key).or_insert_with(HashMap::new);
                                for (reg_key, value) in env_map {
                                    merged_env_map.insert(reg_key, value);
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("Warning: Failed to load {}: {}", file_path_str, e);
                        }
                    }
                }
            }
        }
    }

    Ok(merged_registry)
}

/// Load all layout text CSV files (keys already properly prefixed)
/// Scans only layout directories and loads CSV files directly
/// Keys in CSV files are already prefixed: LANDING_PROBLEMS_TITLE_DE
pub fn load_layout_text_csv_files(base_path: &str) -> Result<HashMap<String, HashMap<String, String>>, Box<dyn std::error::Error>> {
    let mut text_registry: HashMap<String, HashMap<String, String>> = HashMap::new();

    // Component directories to scan
    let component_dirs = vec![
        "templates/layouts",
        "templates/components/atoms",
        "templates/components/molecules",
        "templates/components/organisms"
    ];

    for component_dir in component_dirs {
        let full_path = format!("{}/{}", base_path, component_dir);

        // Skip if directory doesn't exist
        if !Path::new(&full_path).is_dir() {
            continue;
        }

        // Scan all component directories
        let component_entries = fs::read_dir(&full_path)?;

        for component_entry in component_entries {
            let component_entry = component_entry?;
            let component_path = component_entry.path();

            if component_path.is_dir() {
                if let Some(component_name) = component_path.file_name().and_then(|name| name.to_str()) {
                    // Check for component-name.text.csv file
                    let text_csv_path = component_path.join(format!("{}.text.csv", component_name));

                    if text_csv_path.exists() {
                        if let Some(text_csv_str) = text_csv_path.to_str() {
                            // Load CSV without additional prefixing (keys already prefixed by migration)
                            match load_environment_csv(text_csv_str) {
                                Ok(component_text_registry) => {
                                    // Merge prefixed keys into main text registry
                                    for (env_key, env_map) in component_text_registry {
                                        let merged_env_map = text_registry.entry(env_key).or_insert_with(HashMap::new);
                                        for (prefixed_key, value) in env_map {
                                            merged_env_map.insert(prefixed_key, value);
                                        }
                                    }
                                }
                                Err(e) => {
                                    eprintln!("Warning: Failed to load {}: {}", text_csv_str, e);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(text_registry)
}

/// Load single CSV file with automatic key prefixing
/// All keys get prefixed with layout name: KEY_DE → LAYOUT_KEY_DE
pub fn load_environment_csv_with_prefix(path: &str, prefix: &str) -> Result<HashMap<String, HashMap<String, String>>, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;
    let mut registry: HashMap<String, HashMap<String, String>> = HashMap::new();

    // Parse CSV and populate environment-aware HashMaps with prefixed keys
    for line in content.lines().skip(1) { // Skip header
        if line.trim().is_empty() {
            continue;
        }

        let parts: Vec<&str> = line.split(';').collect();
        if parts.len() >= 2 {
            let original_key = parts[0].trim();
            let value = parts[1].trim();

            // Parse original_key@environment or base key
            let (registry_key, env) = parse_env_key(original_key);

            // Create prefixed key: METRICS_TITLE_DE → LANDING_METRICS_TITLE_DE
            let prefixed_key = format!("{}_{}", prefix, registry_key);

            // Store in nested HashMap structure: env → prefixed_key → value
            registry.entry(env.to_string())
                .or_insert_with(HashMap::new)
                .insert(prefixed_key, value.to_string());
        }
    }

    Ok(registry)
}