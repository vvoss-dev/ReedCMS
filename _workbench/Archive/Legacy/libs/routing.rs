use std::collections::HashMap;
use crate::libs::keybase::{load_environment_csv, get_env_value};

#[derive(Clone, Debug)]
pub struct Route {
    pub key: String,
    pub path: String,
    pub lang: String,
}

#[derive(Clone)]
pub struct Routing {
    // Uses environment-aware registry: environment → route_key → path
    registry: HashMap<String, HashMap<String, String>>,
    // Legacy routes field for compatibility with existing handlers
    pub routes: HashMap<String, HashMap<String, String>>,
}

impl Routing {
    /// Create empty routing for S06-02 template system
    pub fn new() -> Self {
        Self {
            registry: HashMap::new(),
            routes: HashMap::new(),
        }
    }

    /// Load routing from CSV file with @DE/@EN format (key;value;comment)
    pub fn from_csv(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let registry = load_environment_csv(path)?;

        // Build legacy routes format for compatibility
        let mut routes: HashMap<String, HashMap<String, String>> = HashMap::new();

        // Extract all route keys from the registry
        let mut all_keys = std::collections::HashSet::new();
        for env_map in registry.values() {
            for key in env_map.keys() {
                all_keys.insert(key.clone());
            }
        }

        // Build routes in legacy format: route_key → lang → path
        for key in all_keys {
            let mut lang_paths = HashMap::new();

            if let Some(de_path) = get_env_value(&registry, "DE", &key) {
                lang_paths.insert("de".to_string(), de_path);
            }
            if let Some(en_path) = get_env_value(&registry, "EN", &key) {
                lang_paths.insert("en".to_string(), en_path);
            }

            if !lang_paths.is_empty() {
                routes.insert(key, lang_paths);
            }
        }

        Ok(Routing { registry, routes })
    }
    
    /// Get full URL path for a route key in a specific language
    pub fn get_path(&self, key: &str, lang: &str) -> Option<String> {
        let lang_env = lang.to_uppercase();
        get_env_value(&self.registry, &lang_env, key)
            .map(|path| {
                if path.is_empty() {
                    format!("/{}/", lang)
                } else {
                    format!("/{}/{}", lang, path)
                }
            })
    }

    /// Get all language URLs for a route key
    pub fn get_lang_urls(&self, key: &str) -> HashMap<String, String> {
        let mut urls = HashMap::new();

        // Check both DE and EN environments
        for lang in ["de", "en"] {
            let lang_env = lang.to_uppercase();
            if let Some(path) = get_env_value(&self.registry, &lang_env, key) {
                let url = if path.is_empty() {
                    format!("/{}/", lang)
                } else {
                    format!("/{}/{}", lang, path)
                };
                urls.insert(lang.to_string(), url);
            }
        }

        urls
    }

    /// Get route key from URL path (without language prefix)
    pub fn get_key_from_path_and_lang(&self, path: &str, lang: &str) -> Option<String> {
        let lang_env = lang.to_uppercase();

        // Search through all route keys in the environment
        if let Some(env_map) = self.registry.get(&lang_env) {
            for (key, stored_path) in env_map {
                if *stored_path == path {
                    return Some(key.clone());
                }
            }
        }

        None
    }

    /// Get route key from URL path (with language prefix)
    pub fn get_key_from_path(&self, path: &str) -> Option<String> {
        // Remove leading slash and language prefix
        let path_parts: Vec<&str> = path.trim_start_matches('/').split('/').collect();
        if path_parts.is_empty() {
            return None;
        }

        let lang = path_parts[0];
        let route_path = if path_parts.len() > 1 {
            path_parts[1..].join("/")
        } else {
            String::new()
        };

        self.get_key_from_path_and_lang(&route_path, lang)
    }
}