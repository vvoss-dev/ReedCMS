// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

#[cfg(test)]
mod tests {
    use super::super::layout_commands::*;
    use std::collections::HashMap;
    use std::fs;
    use std::path::Path;

    /// Cleanup test files and directories.
    fn cleanup_test_files(layout: &str) {
        let layout_dir = format!("templates/layouts/{}", layout);
        fs::remove_dir_all(&layout_dir).ok();

        // Remove test entries from CSV files
        cleanup_csv_entries(layout);
    }

    /// Cleanup CSV entries for test layout.
    fn cleanup_csv_entries(layout: &str) {
        use crate::reedcms::csv::{read_csv, write_csv};

        // Clean registry
        if let Ok(mut records) = read_csv(".reed/registry.csv") {
            records.retain(|r| !r.key.starts_with(layout));
            write_csv(".reed/registry.csv", &records).ok();
        }

        // Clean routes
        if let Ok(mut records) = read_csv(".reed/routes.csv") {
            records.retain(|r| !r.key.starts_with(layout));
            write_csv(".reed/routes.csv", &records).ok();
        }

        // Clean text
        if let Ok(mut records) = read_csv(".reed/text.csv") {
            records.retain(|r| !r.key.starts_with(layout));
            write_csv(".reed/text.csv", &records).ok();
        }

        // Clean meta
        if let Ok(mut records) = read_csv(".reed/meta.csv") {
            records.retain(|r| !r.key.starts_with(layout));
            write_csv(".reed/meta.csv", &records).ok();
        }
    }

    #[test]
    fn test_template_variant_from_str() {
        assert_eq!(
            TemplateVariant::from_str("mouse").unwrap(),
            TemplateVariant::Mouse
        );
        assert_eq!(
            TemplateVariant::from_str("touch").unwrap(),
            TemplateVariant::Touch
        );
        assert_eq!(
            TemplateVariant::from_str("reader").unwrap(),
            TemplateVariant::Reader
        );
        assert!(TemplateVariant::from_str("invalid").is_err());
    }

    #[test]
    fn test_template_variant_as_str() {
        assert_eq!(TemplateVariant::Mouse.as_str(), "mouse");
        assert_eq!(TemplateVariant::Touch.as_str(), "touch");
        assert_eq!(TemplateVariant::Reader.as_str(), "reader");
    }

    #[test]
    fn test_validate_layout_name_success() {
        assert!(validate_layout_name("knowledge").is_ok());
        assert!(validate_layout_name("blog-detail").is_ok());
        assert!(validate_layout_name("my_layout").is_ok());
    }

    #[test]
    fn test_validate_layout_name_too_short() {
        assert!(validate_layout_name("ab").is_err());
    }

    #[test]
    fn test_validate_layout_name_too_long() {
        assert!(validate_layout_name("a".repeat(33).as_str()).is_err());
    }

    #[test]
    fn test_validate_layout_name_invalid_start() {
        assert!(validate_layout_name("123layout").is_err());
        assert!(validate_layout_name("-layout").is_err());
    }

    #[test]
    fn test_validate_layout_name_invalid_chars() {
        assert!(validate_layout_name("layout@name").is_err());
        assert!(validate_layout_name("layout name").is_err());
        assert!(validate_layout_name("layout.name").is_err());
    }

    #[test]
    fn test_validate_layout_name_reserved() {
        assert!(validate_layout_name("admin").is_err());
        assert!(validate_layout_name("system").is_err());
        assert!(validate_layout_name("api").is_err());
    }

    #[test]
    fn test_init_layout_no_args() {
        let args = vec![];
        let flags = HashMap::new();
        let result = init_layout(&args, &flags);
        assert!(result.is_err());
    }

    #[test]
    fn test_init_layout_minimal() {
        let layout = "testlayout1";
        cleanup_test_files(layout);

        let args = vec![layout.to_string()];
        let flags = HashMap::new();

        let result = init_layout(&args, &flags);
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(response.data.contains("✓ Created 1 layout"));
        assert!(response.data.contains(layout));

        // Check files exist
        assert!(Path::new(&format!("templates/layouts/{}", layout)).exists());
        assert!(Path::new(&format!(
            "templates/layouts/{}/{}.mouse.jinja",
            layout, layout
        ))
        .exists());
        assert!(Path::new(&format!(
            "templates/layouts/{}/{}.touch.jinja",
            layout, layout
        ))
        .exists());
        assert!(Path::new(&format!(
            "templates/layouts/{}/{}.reader.jinja",
            layout, layout
        ))
        .exists());

        cleanup_test_files(layout);
    }

    #[test]
    fn test_init_layout_with_languages() {
        let layout = "testlayout2";
        cleanup_test_files(layout);

        let args = vec![layout.to_string()];
        let mut flags = HashMap::new();
        flags.insert("languages".to_string(), "de,en,fr".to_string());

        let result = init_layout(&args, &flags);
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(response.data.contains("de, en, fr"));

        cleanup_test_files(layout);
    }

    #[test]
    fn test_init_layout_with_variants() {
        let layout = "testlayout3";
        cleanup_test_files(layout);

        let args = vec![layout.to_string()];
        let mut flags = HashMap::new();
        flags.insert("variants".to_string(), "mouse,touch".to_string());

        let result = init_layout(&args, &flags);
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(response.data.contains("mouse, touch"));
        assert!(!response.data.contains("reader"));

        // Check only mouse and touch files exist
        assert!(Path::new(&format!(
            "templates/layouts/{}/{}.mouse.jinja",
            layout, layout
        ))
        .exists());
        assert!(Path::new(&format!(
            "templates/layouts/{}/{}.touch.jinja",
            layout, layout
        ))
        .exists());
        assert!(!Path::new(&format!(
            "templates/layouts/{}/{}.reader.jinja",
            layout, layout
        ))
        .exists());

        cleanup_test_files(layout);
    }

    #[test]
    fn test_init_layout_with_routes() {
        let layout = "testlayout4";
        cleanup_test_files(layout);

        let args = vec![layout.to_string()];
        let mut flags = HashMap::new();
        flags.insert("languages".to_string(), "de,en".to_string());
        flags.insert("routes".to_string(), "de:wissen,en:knowledge".to_string());

        let result = init_layout(&args, &flags);
        assert!(result.is_ok());

        cleanup_test_files(layout);
    }

    #[test]
    fn test_init_layout_with_parent() {
        let layout = "testlayout5";
        cleanup_test_files(layout);

        let args = vec![layout.to_string()];
        let mut flags = HashMap::new();
        flags.insert("parent".to_string(), "blog".to_string());

        let result = init_layout(&args, &flags);
        assert!(result.is_ok());

        cleanup_test_files(layout);
    }

    #[test]
    fn test_init_layout_multiple() {
        let layouts = vec!["testlayout6", "testlayout7", "testlayout8"];

        for layout in &layouts {
            cleanup_test_files(layout);
        }

        let args = layouts
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<String>>();
        let flags = HashMap::new();

        let result = init_layout(&args, &flags);
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(response.data.contains("✓ Created 3 layouts"));

        for layout in &layouts {
            assert!(Path::new(&format!("templates/layouts/{}", layout)).exists());
            cleanup_test_files(layout);
        }
    }

    #[test]
    fn test_init_layout_duplicate() {
        let layout = "testlayout9";
        cleanup_test_files(layout);

        let args = vec![layout.to_string()];
        let flags = HashMap::new();

        // First creation
        let result1 = init_layout(&args, &flags);
        assert!(result1.is_ok());

        // Second creation (should fail)
        let result2 = init_layout(&args, &flags);
        assert!(result2.is_err());

        cleanup_test_files(layout);
    }

    #[test]
    fn test_init_layout_invalid_variant() {
        let layout = "testlayout10";
        cleanup_test_files(layout);

        let args = vec![layout.to_string()];
        let mut flags = HashMap::new();
        flags.insert("variants".to_string(), "mouse,invalid".to_string());

        let result = init_layout(&args, &flags);
        assert!(result.is_err());

        cleanup_test_files(layout);
    }

    #[test]
    fn test_init_layout_invalid_route_format() {
        let layout = "testlayout11";
        cleanup_test_files(layout);

        let args = vec![layout.to_string()];
        let mut flags = HashMap::new();
        flags.insert("routes".to_string(), "de-wissen".to_string()); // Missing colon

        let result = init_layout(&args, &flags);
        assert!(result.is_err());

        cleanup_test_files(layout);
    }

    #[test]
    fn test_layout_exists() {
        let layout = "testlayout12";
        cleanup_test_files(layout);

        assert!(!layout_exists(layout).unwrap());

        let args = vec![layout.to_string()];
        let flags = HashMap::new();
        init_layout(&args, &flags).unwrap();

        assert!(layout_exists(layout).unwrap());

        cleanup_test_files(layout);
    }

    #[test]
    fn test_generated_template_content() {
        let layout = "testlayout13";
        cleanup_test_files(layout);

        let args = vec![layout.to_string()];
        let flags = HashMap::new();
        init_layout(&args, &flags).unwrap();

        // Check template content
        let template_path = format!("templates/layouts/{}/{}.mouse.jinja", layout, layout);
        let content = fs::read_to_string(&template_path).unwrap();

        assert!(content.contains("<!DOCTYPE html>"));
        assert!(content.contains(&format!("{}.title", layout)));
        assert!(content.contains(&format!("{}.heading", layout)));
        assert!(content.contains("Copyright 2025 Vivian Voss"));

        cleanup_test_files(layout);
    }

    #[test]
    fn test_generated_css_content() {
        let layout = "testlayout14";
        cleanup_test_files(layout);

        let args = vec![layout.to_string()];
        let flags = HashMap::new();
        init_layout(&args, &flags).unwrap();

        // Check CSS content
        let css_path = format!("templates/layouts/{}/{}.mouse.css", layout, layout);
        let content = fs::read_to_string(&css_path).unwrap();

        assert!(content.contains(&format!("/* Layout: {} */", layout)));
        assert!(content.contains("/* Variant: mouse */"));
        assert!(content.contains("body.mouse"));
        assert!(content.contains("Copyright 2025 Vivian Voss"));

        cleanup_test_files(layout);
    }
}
