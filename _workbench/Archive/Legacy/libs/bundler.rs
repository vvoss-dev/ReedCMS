use std::fs;
use std::path::Path;
use super::components::{ComponentRegistry, AssetBundle};

/// Asset bundling and concatenation system
pub struct AssetBundler {
    pub output_dir: String,
    pub session_hash: String,
}

/// Generated asset bundle paths for template integration
#[derive(Debug, Clone)]
pub struct GeneratedBundles {
    pub mouse_css_path: String,    // /public/session/component-mouse.css
    pub touch_css_path: String,    // /public/session/component-touch.css
    pub reader_css_path: String,   // /public/session/component-reader.css
    pub javascript_path: String,   // /public/session/component.js
}

/// Template migration tools for converting legacy templates to component structure
pub struct TemplateMigrator {
    pub source_path: String,
    pub target_path: String,
}

/// CSV migration tools for extracting component data from legacy CSV files
pub struct CsvMigrator {
    pub source_content_csv: String,
    pub source_routing_csv: String,
}

/// Migration validation result
#[derive(Debug)]
pub struct ValidationResult {
    pub success: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl AssetBundler {
    /// Create new asset bundler with output directory and session hash
    pub fn new(output_dir: &str, session_hash: &str) -> Self {
        AssetBundler {
            output_dir: output_dir.to_string(),
            session_hash: session_hash.to_string(),
        }
    }

    /// Generate CSS/JS bundles from asset bundle
    /// Creates device-specific CSS bundles and single JS bundle
    pub fn generate_bundles(&self, asset_bundle: &AssetBundle, component_name: &str) -> Result<GeneratedBundles, Box<dyn std::error::Error>> {
        // Create output directory if it doesn't exist
        fs::create_dir_all(&self.output_dir)?;

        // Generate bundle name based on component
        let bundle_name = self.generate_bundle_name(component_name, asset_bundle);

        // Generate device-specific CSS bundles
        let mouse_css_path = self.create_css_bundle(&asset_bundle.mouse_css, &bundle_name, "mouse")?;
        let touch_css_path = self.create_css_bundle(&asset_bundle.touch_css, &bundle_name, "touch")?;
        let reader_css_path = self.create_css_bundle(&asset_bundle.reader_css, &bundle_name, "reader")?;

        // Generate JavaScript bundle
        let javascript_path = self.create_js_bundle(&asset_bundle.javascript, &bundle_name)?;

        Ok(GeneratedBundles {
            mouse_css_path,
            touch_css_path,
            reader_css_path,
            javascript_path,
        })
    }

    /// Generate bundle name based on layout component name
    fn generate_bundle_name(&self, component_name: &str, _asset_bundle: &AssetBundle) -> String {
        // Always use the layout component name (e.g., "landing", "knowledge", "portfolio")
        // Bundle name should be simple and predictable: layout.{session_hash}.{device}.css
        component_name.to_string()
    }

    /// Extract component name from asset file path
    /// e.g., "templates/layouts/knowledge/knowledge.mouse.css" → "knowledge"
    fn extract_component_name_from_path(&self, path: &str) -> Option<String> {
        let path_obj = Path::new(path);
        let filename = path_obj.file_stem()?.to_str()?;

        // Remove device suffix (.mouse, .touch, .reader)
        if let Some(component_name) = filename.strip_suffix(".mouse")
            .or_else(|| filename.strip_suffix(".touch"))
            .or_else(|| filename.strip_suffix(".reader")) {
            Some(component_name.to_string())
        } else {
            // For JS files without device suffix
            Some(filename.to_string())
        }
    }

    /// Create CSS bundle by concatenating CSS files
    fn create_css_bundle(&self, css_files: &[String], bundle_name: &str, device: &str) -> Result<String, Box<dyn std::error::Error>> {
        if css_files.is_empty() {
            return Ok(String::new());
        }

        let bundle_filename = format!("{}.{}.{}.css", bundle_name, self.session_hash, device);
        let bundle_path = format!("{}/{}", self.output_dir, bundle_filename);

        let mut bundle_content = String::new();

        // Add bundle header
        bundle_content.push_str(&format!(
            "/* Auto-generated CSS bundle: {} for {} device */\n",
            bundle_name, device
        ));
        bundle_content.push_str(&format!(
            "/* Session: {} */\n",
            self.session_hash
        ));
        bundle_content.push_str(&format!(
            "/* Components: {} */\n\n",
            css_files.iter()
                .filter_map(|path| self.extract_component_name_from_path(path))
                .collect::<std::collections::HashSet<_>>()
                .into_iter()
                .collect::<Vec<_>>()
                .join(", ")
        ));

        // Concatenate CSS files
        for css_file in css_files {
            if Path::new(css_file).exists() {
                bundle_content.push_str(&format!("/* From: {} */\n", css_file));
                match fs::read_to_string(css_file) {
                    Ok(content) => {
                        bundle_content.push_str(&content);
                        bundle_content.push_str("\n\n");
                    }
                    Err(e) => {
                        eprintln!("Warning: Failed to read {}: {}", css_file, e);
                    }
                }
            } else {
                eprintln!("Warning: CSS file not found: {}", css_file);
            }
        }

        // Write bundle file
        fs::write(&bundle_path, bundle_content)?;

        Ok(bundle_filename)
    }

    /// Create JavaScript bundle by concatenating JS files
    fn create_js_bundle(&self, js_files: &[String], bundle_name: &str) -> Result<String, Box<dyn std::error::Error>> {
        if js_files.is_empty() {
            return Ok(String::new());
        }

        let bundle_filename = format!("{}.{}.js", bundle_name, self.session_hash);
        let bundle_path = format!("{}/{}", self.output_dir, bundle_filename);

        let mut bundle_content = String::new();

        // Add bundle header
        bundle_content.push_str(&format!(
            "/* Auto-generated JavaScript bundle: {} */\n",
            bundle_name
        ));
        bundle_content.push_str(&format!(
            "/* Session: {} */\n",
            self.session_hash
        ));
        bundle_content.push_str(&format!(
            "/* Components: {} */\n\n",
            js_files.iter()
                .filter_map(|path| self.extract_component_name_from_path(path))
                .collect::<std::collections::HashSet<_>>()
                .into_iter()
                .collect::<Vec<_>>()
                .join(", ")
        ));

        // Concatenate JS files
        for js_file in js_files {
            if Path::new(js_file).exists() {
                bundle_content.push_str(&format!("/* From: {} */\n", js_file));
                match fs::read_to_string(js_file) {
                    Ok(content) => {
                        bundle_content.push_str(&content);
                        bundle_content.push_str(";\n\n"); // Add semicolon for safety
                    }
                    Err(e) => {
                        eprintln!("Warning: Failed to read {}: {}", js_file, e);
                    }
                }
            } else {
                eprintln!("Warning: JavaScript file not found: {}", js_file);
            }
        }

        // Write bundle file
        fs::write(&bundle_path, bundle_content)?;

        Ok(bundle_filename)
    }

    /// Clean old bundle files (remove files with different session hashes)
    pub fn clean_old_bundles(&self) -> Result<(), Box<dyn std::error::Error>> {
        if !Path::new(&self.output_dir).exists() {
            return Ok(());
        }

        let entries = fs::read_dir(&self.output_dir)?;

        for entry in entries.flatten() {
            if let Some(filename) = entry.file_name().to_str() {
                // Check if file is a bundle but not from current session
                if (filename.ends_with(".css") || filename.ends_with(".js"))
                    && filename.contains('.')
                    && !filename.contains(&self.session_hash) {

                    if let Err(e) = fs::remove_file(entry.path()) {
                        eprintln!("Warning: Failed to remove old bundle {}: {}", filename, e);
                    }
                }
            }
        }

        Ok(())
    }
}

impl TemplateMigrator {
    /// Create new template migrator
    pub fn new(source_path: &str, target_path: &str) -> Self {
        TemplateMigrator {
            source_path: source_path.to_string(),
            target_path: target_path.to_string(),
        }
    }

    /// Migrate template from legacy structure to component structure
    pub fn migrate_template(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Create target directory
        if let Some(parent) = Path::new(&self.target_path).parent() {
            fs::create_dir_all(parent)?;
        }

        // Copy template file
        fs::copy(&self.source_path, &self.target_path)?;

        Ok(())
    }

    /// Create component directory structure with placeholder files
    pub fn create_component_structure(&self, component_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        let component_dir = Path::new(&self.target_path).parent()
            .ok_or("Invalid target path")?;

        // Create directory
        fs::create_dir_all(component_dir)?;

        // Create placeholder CSV files
        let system_csv_path = component_dir.join(format!("{}.system.csv", component_name));
        let text_csv_path = component_dir.join(format!("{}.text.csv", component_name));

        // Create system.csv with basic structure
        let system_csv_content = "key;value;comment\nROUTE_DE;route-placeholder;Component route placeholder\nROUTE_EN;route-placeholder;Component route placeholder\n";
        fs::write(system_csv_path, system_csv_content)?;

        // Create text.csv with basic structure
        let text_csv_content = "key;value;comment\nTITLE_DE;Titel;German title placeholder\nTITLE_EN;Title;English title placeholder\n";
        fs::write(text_csv_path, text_csv_content)?;

        // Create placeholder CSS files
        for device in &["mouse", "touch", "reader"] {
            let css_path = component_dir.join(format!("{}.{}.css", component_name, device));
            let css_content = format!("/* Component CSS for {} - {} device */\n", component_name, device);
            fs::write(css_path, css_content)?;
        }

        Ok(())
    }
}

impl CsvMigrator {
    /// Create new CSV migrator
    pub fn new(source_content_csv: &str, source_routing_csv: &str) -> Self {
        CsvMigrator {
            source_content_csv: source_content_csv.to_string(),
            source_routing_csv: source_routing_csv.to_string(),
        }
    }

    /// Extract component-specific entries from legacy CSV files
    pub fn extract_component_data(&self, _component_name: &str, prefix_filter: &str) -> Result<(String, String), Box<dyn std::error::Error>> {
        let mut system_csv_content = String::from("key;value;comment\n");
        let mut text_csv_content = String::from("key;value;comment\n");

        // Extract from routing CSV (if exists)
        if Path::new(&self.source_routing_csv).exists() {
            let routing_content = fs::read_to_string(&self.source_routing_csv)?;
            for line in routing_content.lines().skip(1) { // Skip header
                if line.contains(&format!("{}_", prefix_filter.to_uppercase())) {
                    // Convert routing entry to system entry
                    let parts: Vec<&str> = line.split(';').collect();
                    if parts.len() >= 3 {
                        let key = parts[0].replace(&format!("{}_", prefix_filter.to_uppercase()), "");
                        system_csv_content.push_str(&format!("{};{};{}\n", key, parts[1], parts[2]));
                    }
                }
            }
        }

        // Extract from content CSV (if exists)
        if Path::new(&self.source_content_csv).exists() {
            let content_content = fs::read_to_string(&self.source_content_csv)?;
            for line in content_content.lines().skip(1) { // Skip header
                if line.starts_with(&format!("{}.", prefix_filter)) {
                    // Convert content entry to text entry
                    let parts: Vec<&str> = line.split(';').collect();
                    if parts.len() >= 3 {
                        let key = parts[0].replace(&format!("{}.", prefix_filter), "")
                            .to_uppercase()
                            .replace('.', "_");
                        text_csv_content.push_str(&format!("{};{};{}\n", key, parts[1], parts[2]));
                    }
                }
            }
        }

        Ok((system_csv_content, text_csv_content))
    }
}

impl ValidationResult {
    /// Create new validation result
    pub fn new() -> Self {
        ValidationResult {
            success: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    /// Add error to validation result
    pub fn add_error(&mut self, error: &str) {
        self.success = false;
        self.errors.push(error.to_string());
    }

    /// Add warning to validation result
    pub fn add_warning(&mut self, warning: &str) {
        self.warnings.push(warning.to_string());
    }

    /// Check if validation passed (no errors)
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }

    /// Get summary of validation result
    pub fn summary(&self) -> String {
        if self.success && self.errors.is_empty() {
            format!("Validation passed with {} warnings", self.warnings.len())
        } else {
            format!("Validation failed with {} errors and {} warnings",
                    self.errors.len(), self.warnings.len())
        }
    }
}

/// Validate component migration and asset pipeline
pub fn validate_component_system(registry: &ComponentRegistry) -> ValidationResult {
    let mut result = ValidationResult::new();

    // Validate each component
    for (name, component) in &registry.components {
        // Check template file exists
        if !component.template_path.is_empty() && !Path::new(&component.template_path).exists() {
            result.add_error(&format!("Template file missing for component '{}': {}", name, component.template_path));
        }

        // System CSV validation removed - pageconf() system replaces route metadata

        // Check dependencies exist
        let deps = component.get_dependencies();
        for dep in [deps.atoms, deps.molecules, deps.organisms].concat() {
            if !registry.components.contains_key(&dep) {
                result.add_error(&format!("Component '{}' depends on missing component '{}'", name, dep));
            }
        }
    }

    result
}