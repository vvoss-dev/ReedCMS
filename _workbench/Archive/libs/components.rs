use std::collections::{HashMap, HashSet, VecDeque};
use std::fs;
use std::path::Path;
use std::sync::Arc;
use super::keybase;

/// Component types based on Atomic Design Pattern
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ComponentType {
    Layout,    // templates/layouts/*
    Atom,      // templates/components/atoms/*
    Molecule,  // templates/components/molecules/*
    Organism,  // templates/components/organisms/*
}

/// Information about a discovered component
#[derive(Debug, Clone)]
pub struct ComponentInfo {
    pub name: String,           // Component name (e.g., "knowledge")
    pub path: String,           // Full path (e.g., "templates/layouts/knowledge")
    pub component_type: ComponentType, // Layout, Atom, Molecule, Organism
    // System CSV removed - pageconf() system replaces metadata
    pub has_text_csv: bool,     // knowledge.text.csv exists
    pub has_template: bool,     // knowledge.jinja exists
    pub has_mouse_css: bool,    // knowledge.mouse.css exists
    pub has_touch_css: bool,    // knowledge.touch.css exists
    pub has_reader_css: bool,   // knowledge.reader.css exists
    pub has_js: bool,           // knowledge.js exists
}

/// Loaded CSV data for a component
#[derive(Debug, Clone)]
pub struct ComponentData {
    pub name: String,
    pub component_type: ComponentType,
    pub system: HashMap<String, HashMap<String, String>>, // Environment-aware system data
    pub text: HashMap<String, HashMap<String, String>>,   // Environment-aware text data
}

/// Complete component with info, data, and template path
#[derive(Debug, Clone)]
pub struct Component {
    pub info: ComponentInfo,        // From S05-01
    pub data: ComponentData,        // From S05-02
    pub template_path: String,      // Full path to .jinja file
}

/// Component registry with all discovered components
#[derive(Debug, Clone)]
pub struct ComponentRegistry {
    pub components: HashMap<String, Component>,
    pub environment: String,
}

/// Dependencies parsed from component system.csv
#[derive(Debug, Clone, PartialEq)]
pub struct ComponentDependencies {
    pub atoms: Vec<String>,
    pub molecules: Vec<String>,
    pub organisms: Vec<String>,
}

/// Resolved dependencies with load order
#[derive(Debug, Clone)]
pub struct ResolvedDependencies {
    pub root_component: String,
    pub atoms: Vec<Component>,
    pub molecules: Vec<Component>,
    pub organisms: Vec<Component>,
    pub load_order: Vec<String>, // Topologically sorted
}

/// Dependency resolution errors
#[derive(Debug)]
pub enum DependencyError {
    ComponentNotFound(String),
    CircularDependency(Vec<String>), // Path showing the cycle
    InvalidDependencyType(String),
}

/// Asset bundle containing device-specific CSS and JS paths
#[derive(Debug, Clone)]
pub struct AssetBundle {
    pub mouse_css: Vec<String>,    // Paths to .mouse.css files
    pub touch_css: Vec<String>,    // Paths to .touch.css files
    pub reader_css: Vec<String>,   // Paths to .reader.css files
    pub javascript: Vec<String>,   // Paths to .js files
}



/// Dependency resolver for component dependency graphs
pub struct DependencyResolver {
    pub registry: Arc<ComponentRegistry>,
}

impl std::fmt::Display for DependencyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DependencyError::ComponentNotFound(name) => write!(f, "Component not found: {}", name),
            DependencyError::CircularDependency(path) => write!(f, "Circular dependency detected: {}", path.join(" -> ")),
            DependencyError::InvalidDependencyType(msg) => write!(f, "Invalid dependency type: {}", msg),
        }
    }
}

impl std::error::Error for DependencyError {}

/// Scan component directories and discover component structure
/// Pure filesystem scanning with no dependencies on other S05 parts
pub fn scan_component_directories(base_path: &str) -> Vec<ComponentInfo> {
    let mut components = Vec::new();

    // Define component type directories
    let component_dirs = [
        ("layouts", ComponentType::Layout),
        ("components/atoms", ComponentType::Atom),
        ("components/molecules", ComponentType::Molecule),
        ("components/organisms", ComponentType::Organism),
    ];

    for (subdir, component_type) in component_dirs {
        let full_path = format!("{}/{}", base_path, subdir);

        if let Ok(entries) = fs::read_dir(&full_path) {
            for entry in entries.flatten() {
                if entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
                    if let Some(component_name) = entry.file_name().to_str() {
                        let component_path = entry.path();
                        let component_info = scan_component_files(
                            component_name,
                            &component_path.to_string_lossy(),
                            component_type.clone()
                        );
                        components.push(component_info);
                    }
                }
            }
        }
    }

    components
}

/// Scan individual component directory for expected files
fn scan_component_files(name: &str, path: &str, component_type: ComponentType) -> ComponentInfo {
    ComponentInfo {
        name: name.to_string(),
        path: path.to_string(),
        component_type,
        // has_system_csv removed - pageconf() replaces system metadata
        has_text_csv: file_exists(path, &format!("{}.text.csv", name)),
        has_template: file_exists(path, &format!("{}.jinja", name)),
        has_mouse_css: file_exists(path, &format!("{}.mouse.css", name)),
        has_touch_css: file_exists(path, &format!("{}.touch.css", name)),
        has_reader_css: file_exists(path, &format!("{}.reader.css", name)),
        has_js: file_exists(path, &format!("{}.js", name)),
    }
}

/// Check if file exists in directory
pub fn file_exists(dir_path: &str, filename: &str) -> bool {
    Path::new(dir_path).join(filename).exists()
}

/// Parse CSV list from system data (e.g., "svg-icon,button" → ["svg-icon", "button"])
fn parse_csv_list(system_data: &HashMap<String, HashMap<String, String>>, key: &str) -> Vec<String> {
    let current_env = std::env::var("ENVIRONMENT").unwrap_or_else(|_| "PROD".to_string());

    if let Some(value) = keybase::get_env_value(system_data, &current_env, key) {
        if value.trim().is_empty() {
            Vec::new()
        } else {
            value.split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        }
    } else {
        Vec::new()
    }
}

/// Load CSV data for a single component using existing keybase system
/// Builds on S05-01 ComponentInfo to load actual CSV data
pub fn load_component_csv_data(component_info: &ComponentInfo) -> Result<ComponentData, Box<dyn std::error::Error>> {
    let system_data = HashMap::new(); // Empty - pageconf() replaces system metadata
    let mut text_data = HashMap::new();

    // System CSV loading removed - pageconf() system replaces component metadata

    // Load text CSV if it exists
    if component_info.has_text_csv {
        let text_csv_path = format!("{}/{}.text.csv", component_info.path, component_info.name);
        match keybase::load_environment_csv(&text_csv_path) {
            Ok(csv_data) => text_data = csv_data,
            Err(e) => eprintln!("Warning: Failed to load {}: {}", text_csv_path, e),
        }
    }

    Ok(ComponentData {
        name: component_info.name.clone(),
        component_type: component_info.component_type.clone(),
        system: system_data,
        text: text_data,
    })
}

/// Load CSV data for all components in batch
/// Processes list of ComponentInfo from S05-01 scanner
pub fn load_all_component_data(component_infos: &[ComponentInfo]) -> Vec<ComponentData> {
    let mut component_data_list = Vec::new();

    for component_info in component_infos {
        match load_component_csv_data(component_info) {
            Ok(component_data) => component_data_list.push(component_data),
            Err(e) => eprintln!("Error loading component '{}': {}", component_info.name, e),
        }
    }

    component_data_list
}

/// Create complete component registry combining S05-01 scanner and S05-02 CSV loader
/// This is the main entry point for S05-03 functionality
pub fn create_component_registry(base_path: &str) -> Result<ComponentRegistry, Box<dyn std::error::Error>> {
    // Step 1: Scan for components using S05-01
    let component_infos = scan_component_directories(base_path);

    // Step 2: Load CSV data using S05-02
    let component_data_list = load_all_component_data(&component_infos);

    // Step 3: Combine into complete components
    let mut components = HashMap::new();
    let current_env = std::env::var("ENVIRONMENT").unwrap_or_else(|_| "PROD".to_string());

    // Create component info lookup
    let mut info_map = HashMap::new();
    for info in component_infos {
        info_map.insert(info.name.clone(), info);
    }

    // Combine info and data into complete components
    for data in component_data_list {
        if let Some(info) = info_map.remove(&data.name) {
            let template_path = if info.has_template {
                format!("{}/{}.jinja", info.path, info.name)
            } else {
                String::new()
            };

            let component = Component {
                info: info.clone(),
                data,
                template_path,
            };

            components.insert(info.name, component);
        }
    }

    Ok(ComponentRegistry {
        components,
        environment: current_env,
    })
}

impl Component {
    /// Parse dependencies from system.csv data
    /// Parses ATOMS, MOLECULES, ORGANISMS keys from component system data
    pub fn get_dependencies(&self) -> ComponentDependencies {
        let atoms = parse_csv_list(&self.data.system, "ATOMS");
        let molecules = parse_csv_list(&self.data.system, "MOLECULES");
        let organisms = parse_csv_list(&self.data.system, "ORGANISMS");

        ComponentDependencies {
            atoms,
            molecules,
            organisms,
        }
    }
}

impl ComponentRegistry {
    /// Get component by name
    pub fn get_component(&self, name: &str) -> Option<&Component> {
        self.components.get(name)
    }

    /// Get route for component in specific language
    pub fn get_route(&self, component_name: &str, lang: &str) -> Option<String> {
        let component = self.get_component(component_name)?;
        let route_key = format!("ROUTE_{}", lang.to_uppercase());

        // For root routes (landing page), empty value should return empty string, not None
        if let Some(route) = keybase::get_env_value(&component.data.system, &self.environment, &route_key) {
            Some(route)
        } else {
            // Check if key exists with empty value (for root routes)
            if component.data.system.get(&self.environment)
                .and_then(|env_map| env_map.get(&route_key))
                .map(|v| v.is_empty())
                .unwrap_or(false)
            {
                Some(String::new()) // Return empty string for root route
            } else {
                None
            }
        }
    }

    /// Get text for component key in specific language (DEPRECATED - use global TEXT!() instead)
    /// This method is kept for backward compatibility but TEXT!() registry should be used
    pub fn get_text(&self, component_name: &str, key: &str, lang: &str) -> Option<String> {
        let component = self.get_component(component_name)?;
        let text_key = format!("{}_{}", key, lang.to_uppercase());
        keybase::get_env_value(&component.data.text, &self.environment, &text_key)
    }

    /// Get all components of specific type
    pub fn get_components_by_type(&self, component_type: ComponentType) -> Vec<&Component> {
        self.components
            .values()
            .filter(|c| c.info.component_type == component_type)
            .collect()
    }
}

impl DependencyResolver {
    /// Create new dependency resolver with component registry
    pub fn new(registry: Arc<ComponentRegistry>) -> Self {
        DependencyResolver { registry }
    }

    /// Resolve all dependencies for a component with topological sorting
    pub fn resolve_dependencies(&self, component_name: &str) -> Result<ResolvedDependencies, DependencyError> {
        // Verify root component exists
        if !self.registry.components.contains_key(component_name) {
            return Err(DependencyError::ComponentNotFound(component_name.to_string()));
        }

        // Build complete dependency graph
        let mut dependency_graph = HashMap::new();
        let mut visited = HashSet::new();
        let mut visiting = HashSet::new();
        let mut path = Vec::new();

        self.build_dependency_graph(
            component_name,
            &mut dependency_graph,
            &mut visited,
            &mut visiting,
            &mut path,
        )?;

        // Perform topological sort to determine load order
        let load_order = self.topological_sort(&dependency_graph)?;

        // Collect components by type
        let mut atoms = Vec::new();
        let mut molecules = Vec::new();
        let mut organisms = Vec::new();

        let root_name = component_name;
        for comp_name in &load_order {
            if comp_name == root_name {
                continue; // Skip root component
            }

            if let Some(component) = self.registry.get_component(comp_name) {
                match component.info.component_type {
                    ComponentType::Atom => atoms.push(component.clone()),
                    ComponentType::Molecule => molecules.push(component.clone()),
                    ComponentType::Organism => organisms.push(component.clone()),
                    ComponentType::Layout => {}, // Layouts are not dependencies
                }
            }
        }

        Ok(ResolvedDependencies {
            root_component: component_name.to_string(),
            atoms,
            molecules,
            organisms,
            load_order,
        })
    }

    /// Recursively build dependency graph with cycle detection
    fn build_dependency_graph(
        &self,
        component_name: &str,
        graph: &mut HashMap<String, Vec<String>>,
        visited: &mut HashSet<String>,
        visiting: &mut HashSet<String>,
        path: &mut Vec<String>,
    ) -> Result<(), DependencyError> {
        // Check for circular dependency
        if visiting.contains(component_name) {
            // Build cycle path
            let cycle_start_pos = path.iter().position(|x| x == component_name);
            if let Some(start) = cycle_start_pos {
                let mut cycle_path = path[start..].to_vec();
                cycle_path.push(component_name.to_string());
                return Err(DependencyError::CircularDependency(cycle_path));
            }
        }

        // Skip if already fully processed
        if visited.contains(component_name) {
            return Ok(());
        }

        // Get component
        let component = self.registry.get_component(component_name)
            .ok_or_else(|| DependencyError::ComponentNotFound(component_name.to_string()))?;

        // Mark as being visited
        visiting.insert(component_name.to_string());
        path.push(component_name.to_string());

        // Get dependencies
        let deps = component.get_dependencies();
        let mut all_deps = Vec::new();
        all_deps.extend(deps.atoms);
        all_deps.extend(deps.molecules);
        all_deps.extend(deps.organisms);

        // Add to graph
        graph.insert(component_name.to_string(), all_deps.clone());

        // Recursively process dependencies
        for dep_name in &all_deps {
            self.build_dependency_graph(dep_name, graph, visited, visiting, path)?;
        }

        // Mark as fully processed
        visiting.remove(component_name);
        path.pop();
        visited.insert(component_name.to_string());

        Ok(())
    }

    /// Perform topological sort using Kahn's algorithm
    /// Returns dependencies in load order (dependencies first, then dependents)
    fn topological_sort(&self, graph: &HashMap<String, Vec<String>>) -> Result<Vec<String>, DependencyError> {
        // Reverse the graph: if A depends on B, create edge B -> A
        // This way dependencies come first in topological order
        let mut reversed_graph = HashMap::new();
        let mut all_nodes = HashSet::new();

        // Collect all nodes
        for (node, deps) in graph {
            all_nodes.insert(node.clone());
            for dep in deps {
                all_nodes.insert(dep.clone());
            }
        }

        // Initialize reversed graph
        for node in &all_nodes {
            reversed_graph.insert(node.clone(), Vec::new());
        }

        // Build reversed graph: if A depends on B, add edge B -> A
        for (dependent, dependencies) in graph {
            for dependency in dependencies {
                reversed_graph.entry(dependency.clone())
                    .or_insert_with(Vec::new)
                    .push(dependent.clone());
            }
        }

        // Now perform topological sort on reversed graph
        let mut in_degree = HashMap::new();
        let mut queue = VecDeque::new();
        let mut result = Vec::new();

        // Calculate in-degrees for reversed graph
        for node in &all_nodes {
            in_degree.insert(node.clone(), 0);
        }

        for dependents in reversed_graph.values() {
            for dependent in dependents {
                *in_degree.entry(dependent.clone()).or_insert(0) += 1;
            }
        }

        // Find nodes with no incoming edges (these are the leaf dependencies)
        for (node, &degree) in &in_degree {
            if degree == 0 {
                queue.push_back(node.clone());
            }
        }

        // Process nodes in topological order
        while let Some(current) = queue.pop_front() {
            result.push(current.clone());

            // Reduce in-degree for dependent nodes
            if let Some(dependents) = reversed_graph.get(&current) {
                for dependent in dependents {
                    if let Some(degree) = in_degree.get_mut(dependent) {
                        *degree -= 1;
                        if *degree == 0 {
                            queue.push_back(dependent.clone());
                        }
                    }
                }
            }
        }

        // Check for cycles (should not happen due to earlier cycle detection)
        if result.len() != all_nodes.len() {
            return Err(DependencyError::CircularDependency(vec!["Unknown cycle".to_string()]));
        }

        Ok(result)
    }

    /// Collect assets from resolved dependencies for S05-05 asset pipeline
    /// Uses dependency load order to ensure correct CSS/JS loading sequence
    pub fn collect_assets(&self, resolved_deps: &ResolvedDependencies) -> AssetBundle {
        let mut asset_bundle = AssetBundle {
            mouse_css: Vec::new(),
            touch_css: Vec::new(),
            reader_css: Vec::new(),
            javascript: Vec::new(),
        };

        // Collect assets in dependency load order (dependencies first)
        for component_name in &resolved_deps.load_order {
            if let Some(component) = self.registry.get_component(component_name) {
                self.collect_component_assets(&component.info, &mut asset_bundle);
            }
        }

        asset_bundle
    }

    /// Collect assets from a single component
    fn collect_component_assets(&self, component_info: &ComponentInfo, bundle: &mut AssetBundle) {
        let component_path = &component_info.path;
        let component_name = &component_info.name;

        // Collect device-specific CSS files
        if component_info.has_mouse_css {
            bundle.mouse_css.push(format!("{}/{}.mouse.css", component_path, component_name));
        }

        if component_info.has_touch_css {
            bundle.touch_css.push(format!("{}/{}.touch.css", component_path, component_name));
        }

        if component_info.has_reader_css {
            bundle.reader_css.push(format!("{}/{}.reader.css", component_path, component_name));
        }

        // Collect JavaScript files
        if component_info.has_js {
            bundle.javascript.push(format!("{}/{}.js", component_path, component_name));
        }
    }
}

impl AssetBundle {
    /// Create empty asset bundle
    pub fn new() -> Self {
        AssetBundle {
            mouse_css: Vec::new(),
            touch_css: Vec::new(),
            reader_css: Vec::new(),
            javascript: Vec::new(),
        }
    }

    /// Check if bundle has any assets
    pub fn is_empty(&self) -> bool {
        self.mouse_css.is_empty()
            && self.touch_css.is_empty()
            && self.reader_css.is_empty()
            && self.javascript.is_empty()
    }

    /// Get total number of asset files
    pub fn total_assets(&self) -> usize {
        self.mouse_css.len() + self.touch_css.len() + self.reader_css.len() + self.javascript.len()
    }
}


// Tests have been moved to src/tests/components.rs for better code organization