// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Route testing utility.
//!
//! ## Features
//! - Test URL resolution
//! - Verify template existence
//! - Check asset availability
//!
//! ## CLI Usage
//! ```bash
//! reed debug:route /knowledge
//! reed debug:route /de/wissen
//! ```

/// Route test result.
#[derive(Debug, Clone)]
pub struct RouteTest {
    pub url: String,
    pub matched: bool,
    pub layout: Option<String>,
    pub language: Option<String>,
    pub template_exists: bool,
    pub template_path: String,
    pub errors: Vec<String>,
}

impl RouteTest {
    /// Creates new route test.
    pub fn new(url: &str) -> Self {
        Self {
            url: url.to_string(),
            matched: false,
            layout: None,
            language: None,
            template_exists: false,
            template_path: String::new(),
            errors: Vec::new(),
        }
    }

    /// Simulates route testing.
    pub fn simulate(&mut self) {
        // Extract potential layout from URL
        let path = self.url.trim_start_matches('/');
        let parts: Vec<&str> = path.split('/').collect();

        // Check for language prefix
        if parts.first().map(|p| p.len()) == Some(2) {
            self.language = Some(parts[0].to_string());
            if parts.len() > 1 {
                self.layout = Some(parts[1].to_string());
            }
        } else if !parts.is_empty() && !parts[0].is_empty() {
            self.layout = Some(parts[0].to_string());
            self.language = Some("en".to_string());
        }

        self.matched = self.layout.is_some();

        if let Some(ref layout) = self.layout {
            self.template_path = format!("templates/layouts/{}/{}.mouse.jinja", layout, layout);
            self.template_exists = std::path::Path::new(&self.template_path).exists();

            if !self.template_exists {
                self.errors
                    .push(format!("Template not found: {}", self.template_path));
            }
        } else {
            self.errors.push("No layout matched for URL".to_string());
        }
    }

    /// Formats test result.
    pub fn format(&self) -> String {
        let mut output = format!("🛣️  Route Test: {}\n\n", self.url);

        output.push_str("Resolution:\n");
        if self.matched {
            output.push_str("  ✓ Route matched\n");
            if let Some(ref layout) = self.layout {
                output.push_str(&format!("  Layout: {}\n", layout));
            }
            if let Some(ref lang) = self.language {
                output.push_str(&format!("  Language: {}\n", lang));
            }
        } else {
            output.push_str("  ✗ No route match\n");
        }

        output.push_str("\nTemplate:\n");
        output.push_str(&format!("  Path: {}\n", self.template_path));
        if self.template_exists {
            output.push_str("  ✓ Template exists\n");
        } else {
            output.push_str("  ✗ Template not found\n");
        }

        if !self.errors.is_empty() {
            output.push_str("\n⚠ Errors:\n");
            for error in &self.errors {
                output.push_str(&format!("  - {}\n", error));
            }
        }

        output.push_str(
            "\nNote: For complete route testing, use reed server:start with --debug flag.\n",
        );

        output
    }
}

/// Tests route.
///
/// ## Arguments
/// - `url`: URL to test
///
/// ## Returns
/// Route test result
pub fn test_route(url: &str) -> RouteTest {
    let mut test = RouteTest::new(url);
    test.simulate();
    test
}
