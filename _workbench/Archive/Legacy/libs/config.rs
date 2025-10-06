use std::collections::HashMap;
use std::sync::OnceLock;
use super::keybase;

/// Ultimate flexible configuration using keybase registry system
#[derive(Clone)]
pub struct Config {
    // Environment-aware registry using keybase system
    registry: HashMap<String, HashMap<String, String>>,
    current_env: String,
    // Runtime session hash for bundle cache busting
    session_hash: Option<String>,
}


/// SYSTEM constants registry using keybase system
#[derive(Clone)]
pub struct System {
    registry: HashMap<String, HashMap<String, String>>,
    current_env: String,
}

/// TEXT constants registry using keybase system with layout-prefixed keys
#[derive(Clone)]
pub struct Text {
    registry: HashMap<String, HashMap<String, String>>,
    current_env: String,
}

impl System {
    /// Load SYSTEM constants from single CSV file using keybase registry system
    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let registry = keybase::load_environment_csv(path)?;
        let current_env = std::env::var("ENVIRONMENT").unwrap_or_else(|_| "PROD".to_string());

        Ok(System {
            registry,
            current_env,
        })
    }

    /// Load SYSTEM constants from scope-based CSV directory using keybase registry system
    pub fn from_directory(dir_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let registry = keybase::load_environment_csv_directory(dir_path)?;
        let current_env = std::env::var("ENVIRONMENT").unwrap_or_else(|_| "PROD".to_string());

        Ok(System {
            registry,
            current_env,
        })
    }

    // Path constants - fallback values indicate missing configuration
    pub fn get_path_config(&self) -> String {
        keybase::get_env_string(&self.registry, &self.current_env, "PATH_CONFIG", "MISSING_PATH_CONFIG")
    }

    pub fn get_path_translations(&self) -> String {
        keybase::get_env_string(&self.registry, &self.current_env, "PATH_TRANSLATIONS", "MISSING_PATH_TRANSLATIONS")
    }

    pub fn get_path_routings(&self) -> String {
        keybase::get_env_string(&self.registry, &self.current_env, "PATH_ROUTINGS", "MISSING_PATH_ROUTINGS")
    }

    pub fn get_path_templates(&self) -> String {
        keybase::get_env_string(&self.registry, &self.current_env, "PATH_TEMPLATES", "MISSING_PATH_TEMPLATES")
    }

    pub fn get_path_icon_templates(&self) -> String {
        keybase::get_env_string(&self.registry, &self.current_env, "PATH_ICON_TEMPLATES", "MISSING_PATH_ICON_TEMPLATES")
    }

    // Breakpoint constants - fallback values indicate missing configuration
    pub fn get_breakpoint_mobile_min(&self) -> u32 {
        keybase::get_env_u32(&self.registry, &self.current_env, "BREAKPOINT_MOBILE_MIN", 99999)
    }

    pub fn get_breakpoint_mobile_max(&self) -> u32 {
        keybase::get_env_u32(&self.registry, &self.current_env, "BREAKPOINT_MOBILE_MAX", 99999)
    }

    pub fn get_breakpoint_tablet_min(&self) -> u32 {
        keybase::get_env_u32(&self.registry, &self.current_env, "BREAKPOINT_TABLET_MIN", 99999)
    }

    pub fn get_breakpoint_tablet_max(&self) -> u32 {
        keybase::get_env_u32(&self.registry, &self.current_env, "BREAKPOINT_TABLET_MAX", 99999)
    }

    pub fn get_breakpoint_screen_min(&self) -> u32 {
        keybase::get_env_u32(&self.registry, &self.current_env, "BREAKPOINT_SCREEN_MIN", 99999)
    }

    pub fn get_breakpoint_screen_max(&self) -> u32 {
        keybase::get_env_u32(&self.registry, &self.current_env, "BREAKPOINT_SCREEN_MAX", 99999)
    }

    pub fn get_breakpoint_wide_min(&self) -> u32 {
        keybase::get_env_u32(&self.registry, &self.current_env, "BREAKPOINT_WIDE_MIN", 99999)
    }

    // Cookie constants - fallback values indicate missing configuration
    pub fn get_cookie_locale(&self) -> String {
        keybase::get_env_string(&self.registry, &self.current_env, "COOKIE_LOCALE", "MISSING_COOKIE_LOCALE")
    }

    pub fn get_cookie_screen(&self) -> String {
        keybase::get_env_string(&self.registry, &self.current_env, "COOKIE_SCREEN", "MISSING_COOKIE_SCREEN")
    }

    pub fn get_cookie_max_age_days(&self) -> u32 {
        keybase::get_env_u32(&self.registry, &self.current_env, "COOKIE_MAX_AGE_DAYS", 0)
    }

    pub fn get_cookie_path(&self) -> String {
        keybase::get_env_string(&self.registry, &self.current_env, "COOKIE_PATH", "MISSING_COOKIE_PATH")
    }

    // Theme constants - fallback values indicate missing configuration
    pub fn get_theme_icon_default_size(&self) -> u32 {
        keybase::get_env_u32(&self.registry, &self.current_env, "THEME_ICON_DEFAULT_SIZE", 0)
    }

    pub fn get_theme_icon_default_stroke(&self) -> u32 {
        keybase::get_env_u32(&self.registry, &self.current_env, "THEME_ICON_DEFAULT_STROKE", 0)
    }

    pub fn get_theme_icon_viewbox(&self) -> String {
        keybase::get_env_string(&self.registry, &self.current_env, "THEME_ICON_VIEWBOX", "MISSING_VIEWBOX")
    }

    pub fn get_theme_template_suffix(&self) -> String {
        keybase::get_env_string(&self.registry, &self.current_env, "THEME_TEMPLATE_SUFFIX", "MISSING_SUFFIX")
    }

    // CSV constants - fallback values indicate missing configuration
    pub fn get_csv_delimiter(&self) -> String {
        keybase::get_env_string(&self.registry, &self.current_env, "CSV_DELIMITER", "MISSING_DELIMITER")
    }

    pub fn get_csv_header_skip(&self) -> u32 {
        keybase::get_env_u32(&self.registry, &self.current_env, "CSV_HEADER_SKIP", 0)
    }

    pub fn get_csv_min_columns(&self) -> u32 {
        keybase::get_env_u32(&self.registry, &self.current_env, "CSV_MIN_COLUMNS", 0)
    }

    // Bundle system - fallback values indicate missing configuration
    pub fn get_tar_bundles(&self) -> bool {
        keybase::get_env_bool(&self.registry, &self.current_env, "TAR_BUNDLES", false)
    }
}

// Global instances
static CONFIG: OnceLock<Config> = OnceLock::new();
static SYSTEM: OnceLock<System> = OnceLock::new();
static TEXT: OnceLock<Text> = OnceLock::new();

/// Initialise the global configuration from CSV file
pub fn init_config(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::from_file(path)?;
    CONFIG.set(config).map_err(|_| "Configuration already initialised")?;
    Ok(())
}

/// Initialise the global configuration with session hash
pub fn init_config_with_session_hash(path: &str, session_hash: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut config = Config::from_file(path)?;
    config.session_hash = Some(session_hash.to_string());
    CONFIG.set(config).map_err(|_| "Configuration already initialised")?;
    Ok(())
}

/// Initialise the global SYSTEM constants from CSV file
pub fn init_system(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let system = System::from_file(path)?;
    SYSTEM.set(system).map_err(|_| "SYSTEM already initialised")?;
    Ok(())
}

/// Initialise the global SYSTEM constants from scope-based CSV directory
pub fn init_system_directory(dir_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let system = System::from_directory(dir_path)?;
    SYSTEM.set(system).map_err(|_| "SYSTEM already initialised")?;
    Ok(())
}

/// Initialise the global TEXT constants from layout text CSV files with automatic key prefixing
pub fn init_text_layouts(base_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let text = Text::from_layout_csvs(base_path)?;
    TEXT.set(text).map_err(|_| "TEXT already initialised")?;
    Ok(())
}

/// Get reference to the global configuration
pub fn get_config() -> &'static Config {
    CONFIG.get().expect("Configuration not initialised. Call init_config() first.")
}

/// Get reference to the global SYSTEM constants
pub fn get_system() -> &'static System {
    SYSTEM.get().expect("SYSTEM not initialised. Call init_system() first.")
}

/// Get reference to the global TEXT constants
pub fn get_text() -> &'static Text {
    TEXT.get().expect("TEXT not initialised. Call init_text_layouts() first.")
}

// Convenience macros for accessing configuration and system
#[macro_export]
macro_rules! CONFIG {
    () => {
        $crate::libs::config::get_config()
    };
}

#[macro_export]
macro_rules! SYSTEM {
    () => {
        $crate::libs::config::get_system()
    };
}

#[macro_export]
macro_rules! TEXT {
    () => {
        $crate::libs::config::get_text()
    };
}

impl Config {
    /// Load configuration from CSV file using keybase registry system
    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let registry = keybase::load_environment_csv(path)?;
        let current_env = std::env::var("ENVIRONMENT").unwrap_or_else(|_| "PROD".to_string());

        Ok(Config {
            registry,
            current_env,
            session_hash: None,
        })
    }

    /// Get environment-aware values using keybase functions
    /// Fallback values indicate missing configuration and should trigger investigation
    pub fn get_server_endpoint(&self) -> String {
        keybase::get_env_string(&self.registry, &self.current_env, "SERVER_ENDPOINT", "MISSING_SERVER_ENDPOINT")
    }

    pub fn get_server_auth_enabled(&self) -> bool {
        keybase::get_env_bool(&self.registry, &self.current_env, "SERVER_AUTH_ENABLED", false)
    }

    pub fn get_site_languages(&self) -> Vec<String> {
        keybase::get_env_vec_string(&self.registry, &self.current_env, "SITE_LANGUAGES", vec!["ERROR_MISSING_LANGUAGES".to_string()])
    }

    pub fn get_server_auth_username(&self) -> String {
        keybase::get_env_string(&self.registry, &self.current_env, "SERVER_AUTH_USERNAME", "MISSING_USERNAME")
    }

    pub fn get_server_auth_password(&self) -> String {
        keybase::get_env_string(&self.registry, &self.current_env, "SERVER_AUTH_PASSWORD", "MISSING_PASSWORD")
    }

    pub fn get_server_workers(&self) -> u32 {
        keybase::get_env_u32(&self.registry, &self.current_env, "SERVER_WORKERS", 0)
    }

    pub fn get_server_logging_level(&self) -> String {
        keybase::get_env_string(&self.registry, &self.current_env, "SERVER_LOGGING_LEVEL", "MISSING_LOG_LEVEL")
    }

    pub fn get_site_template_path(&self) -> String {
        keybase::get_env_string(&self.registry, &self.current_env, "SITE_TEMPLATE_PATH", "MISSING_TEMPLATE_PATH")
    }

    pub fn get_site_template_cache(&self) -> bool {
        keybase::get_env_bool(&self.registry, &self.current_env, "SITE_TEMPLATE_CACHE", false)
    }

    pub fn get_site_public_path(&self) -> String {
        keybase::get_env_string(&self.registry, &self.current_env, "SITE_PUBLIC_PATH", "MISSING_PUBLIC_PATH")
    }

    pub fn get_site_public_cache_max_age(&self) -> u64 {
        keybase::get_env_u64(&self.registry, &self.current_env, "SITE_PUBLIC_CACHE_MAX_AGE", 0)
    }

    /// Get session hash for CSS bundle loading
    pub fn get_session_hash(&self) -> String {
        self.session_hash.clone().unwrap_or_else(|| "dev".to_string())
    }

    // S05-06-Z: Component path getters
    pub fn get_site_layout_path(&self) -> String {
        keybase::get_env_string(&self.registry, &self.current_env, "SITE_LAYOUT_PATH", "templates/layouts")
    }

    pub fn get_site_atoms_path(&self) -> String {
        keybase::get_env_string(&self.registry, &self.current_env, "SITE_ATOMS_PATH", "templates/components/atoms")
    }

    pub fn get_site_molecules_path(&self) -> String {
        keybase::get_env_string(&self.registry, &self.current_env, "SITE_MOLECULES_PATH", "templates/components/molecules")
    }

    pub fn get_site_organisms_path(&self) -> String {
        keybase::get_env_string(&self.registry, &self.current_env, "SITE_ORGANISMS_PATH", "templates/components/organisms")
    }
}

impl Text {
    /// Load TEXT constants from layout CSV files with automatic key prefixing using keybase registry system
    pub fn from_layout_csvs(base_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let registry = keybase::load_layout_text_csv_files(base_path)?;
        let current_env = std::env::var("ENVIRONMENT").unwrap_or_else(|_| "PROD".to_string());

        Ok(Text {
            registry,
            current_env,
        })
    }

    /// Get text value for layout-prefixed key in specific language
    /// Usage: TEXT!().get("LANDING_METRICS_TITLE_DE")
    pub fn get(&self, key: &str) -> Option<String> {
        keybase::get_env_value(&self.registry, &self.current_env, key)
    }

    /// Get text value with fallback
    /// Usage: TEXT!().get_or("LANDING_METRICS_TITLE_DE", "Missing Text")
    pub fn get_or(&self, key: &str, fallback: &str) -> String {
        keybase::get_env_string(&self.registry, &self.current_env, key, fallback)
    }

    /// Get text with language-aware lookup: KEY@language or KEY@AUTO fallback
    /// Usage: TEXT!().get_with_language("PAGE_HEADER_KNOWLEDGE_TEXT", "DE")
    pub fn get_with_language(&self, key: &str, language: &str) -> Option<String> {
        // Convert to uppercase for consistency with @DEV/@PROD/@DEFAULT pattern
        let lang_upper = language.to_uppercase();
        // First try with specific language (KEY@DE, KEY@EN)
        keybase::get_env_value(&self.registry, &lang_upper, key)
            .or_else(|| {
                // Fallback to @AUTO for universal fallback
                keybase::get_env_value(&self.registry, "AUTO", key)
            })
    }

    /// Get text with language-aware lookup or return fallback
    /// Usage: TEXT!().get_with_language_or("PAGE_HEADER_KNOWLEDGE_TEXT", "de", "Missing")
    pub fn get_with_language_or(&self, key: &str, language: &str, fallback: &str) -> String {
        self.get_with_language(key, language).unwrap_or_else(|| fallback.to_string())
    }
}