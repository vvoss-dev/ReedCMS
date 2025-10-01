use std::fs;
use std::path::Path;
use minijinja::{Error as MiniJinjaError, Value as MiniJinjaValue, ErrorKind};

/// Generate a complete SVG icon with customisable attributes
pub fn load_icon(args: &[MiniJinjaValue]) -> std::result::Result<MiniJinjaValue, MiniJinjaError> {
    // Get the icon name (first argument)
    let name = args.get(0)
        .and_then(|v| v.as_str())
        .ok_or_else(|| MiniJinjaError::new(
            minijinja::ErrorKind::InvalidOperation,
            "icon function requires at least a name parameter"
        ))?;

    // Parse optional parameters from remaining arguments
    let mut size = 24i64;
    let mut stroke_width = 2i64;
    let mut class = "";

    // Arguments in MiniJinja come as positional values, but we can parse named-style calls
    // For now, we'll take them positionally: icon(name, size, stroke_width, class)
    if let Some(size_val) = args.get(1) {
        if let Some(s) = size_val.as_i64() {
            size = s;
        }
    }

    if let Some(stroke_val) = args.get(2) {
        if let Some(s) = stroke_val.as_i64() {
            stroke_width = s;
        }
    }

    if let Some(class_val) = args.get(3) {
        if let Some(c) = class_val.as_str() {
            class = c;
        }
    }

    // Construct the path to the icon atom template
    let icon_path = format!("templates/components/atoms/icons/{}.jinja", name);

    // Check if file exists
    if !Path::new(&icon_path).exists() {
        return Err(MiniJinjaError::new(
            ErrorKind::TemplateNotFound,
            format!("Icon '{}' not found", name)
        ));
    }

    // Read the icon content
    let content = fs::read_to_string(&icon_path)
        .map_err(|e| MiniJinjaError::new(
            ErrorKind::BadSerialization,
            format!("Failed to read icon '{}': {}", name, e)
        ))?;

    // Remove the comment line if present
    let inner_content = content
        .lines()
        .filter(|line| !line.trim().starts_with("{#"))
        .collect::<Vec<_>>()
        .join("\n");

    // Build the complete SVG
    let svg = format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg"
     width="{}"
     height="{}"
     viewBox="0 0 24 24"
     fill="none"
     stroke="currentColor"
     stroke-width="{}"
     stroke-linecap="round"
     stroke-linejoin="round"
     class="icon icon-{} {}">
{}
</svg>"#,
        size, size, stroke_width, name, class, inner_content
    );

    Ok(MiniJinjaValue::from_safe_string(svg))
}