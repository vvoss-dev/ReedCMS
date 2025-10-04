// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Tests for CSS Minifier

#[cfg(test)]
mod tests {
    use super::super::minifier::*;

    #[test]
    fn test_minify_css_basic() {
        let css = r#"
body {
    margin: 0px;
    padding: 0px;
}
"#;
        let result = minify_css(css).unwrap();
        assert!(result.contains("margin:0"));
        assert!(result.contains("padding:0"));
        assert!(!result.contains("0px"));
    }

    #[test]
    fn test_minify_css_removes_comments() {
        let css = r#"
/* This is a comment */
body {
    margin: 0;
}
/* Multi-line
   comment */
"#;
        let result = minify_css(css).unwrap();
        assert!(!result.contains("/*"));
        assert!(!result.contains("*/"));
        assert!(!result.contains("comment"));
    }

    #[test]
    fn test_minify_css_removes_whitespace() {
        let css = "body {  margin:  0;  padding:  0;  }";
        let result = minify_css(css).unwrap();
        assert_eq!(result, "body{margin:0;padding:0}");
    }

    #[test]
    fn test_minify_css_removes_unnecessary_semicolons() {
        let css = "body { margin: 0; }";
        let result = minify_css(css).unwrap();
        assert_eq!(result, "body{margin:0}");
    }

    #[test]
    fn test_minify_css_shortens_hex_colours() {
        let css = "body { color: #ffffff; background: #000000; border: #ff0000; }";
        let result = minify_css(css).unwrap();
        assert!(result.contains("#fff"));
        assert!(result.contains("#000"));
        assert!(result.contains("#f00"));
    }

    #[test]
    fn test_minify_css_preserves_non_shortenable_hex() {
        let css = "body { color: #123456; }";
        let result = minify_css(css).unwrap();
        assert!(result.contains("#123456"));
    }

    #[test]
    fn test_minify_css_removes_zero_units() {
        let css = "body { margin: 0px; padding: 0em; width: 0rem; }";
        let result = minify_css(css).unwrap();
        assert!(result.contains("margin:0"));
        assert!(result.contains("padding:0"));
        assert!(result.contains("width:0"));
        assert!(!result.contains("0px"));
        assert!(!result.contains("0em"));
        assert!(!result.contains("0rem"));
    }

    #[test]
    fn test_minify_css_complex_example() {
        let css = r#"
/* Header styles */
.header {
    margin: 0px;
    padding: 0px 10px;
    background: #ffffff;
    color: #000000;
}

/* Body styles */
body {
    font-size: 16px;
    line-height: 1.5;
}
"#;
        let result = minify_css(css).unwrap();

        // Verify minification
        assert!(!result.contains("/*"));
        assert!(!result.contains("\n"));
        assert!(result.contains("margin:0"));
        assert!(result.contains("#fff"));
        assert!(result.contains("#000"));
        assert!(result.len() < css.len());
    }

    #[test]
    fn test_minify_css_preserves_strings() {
        let css = r#"body { content: "  Hello  World  "; }"#;
        let result = minify_css(css).unwrap();
        // Whitespace inside strings should be preserved
        assert!(result.contains("\"  Hello  World  \""));
    }

    #[test]
    fn test_calculate_reduction() {
        assert_eq!(calculate_reduction(1000, 300), 70);
        assert_eq!(calculate_reduction(1000, 500), 50);
        assert_eq!(calculate_reduction(1000, 900), 10);
        assert_eq!(calculate_reduction(1000, 1000), 0);
    }

    #[test]
    fn test_calculate_reduction_zero_original() {
        assert_eq!(calculate_reduction(0, 0), 0);
    }

    #[test]
    fn test_minify_css_size_reduction() {
        let css = r#"
/* Large CSS file with lots of whitespace and comments */
body {
    margin: 0px;
    padding: 0px;
    background: #ffffff;
}

.header {
    color: #000000;
    font-size: 16px;
}
"#;
        let original_size = css.len();
        let result = minify_css(css).unwrap();
        let minified_size = result.len();

        let reduction = calculate_reduction(original_size, minified_size);
        assert!(
            reduction > 50,
            "Reduction should be > 50%, got {}",
            reduction
        );
    }

    #[test]
    fn test_minify_css_empty_input() {
        let css = "";
        let result = minify_css(css).unwrap();
        assert_eq!(result, "");
    }

    #[test]
    fn test_minify_css_only_comments() {
        let css = "/* Only a comment */";
        let result = minify_css(css).unwrap();
        assert_eq!(result, "");
    }

    #[test]
    fn test_minify_css_media_queries() {
        let css = r#"
@media (min-width: 768px) {
    body {
        margin: 0px;
    }
}
"#;
        let result = minify_css(css).unwrap();
        assert!(result.contains("@media"));
        assert!(result.contains("min-width:768px"));
        assert!(result.contains("margin:0"));
    }
}
