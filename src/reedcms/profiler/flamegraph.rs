// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Flame graph data generator.
//!
//! ## Features
//! - Collapsed stack format for flame graph tools
//! - Compatible with flamegraph.pl
//! - Simplified SVG generation
//!
//! ## Usage
//! ```bash
//! reed profile:flamegraph > profile.txt
//! flamegraph.pl profile.txt > flame.svg
//! ```

use super::core::ProfileReport;

/// Generates flame graph data from profile report.
///
/// ## Format
/// Collapsed stack format for flame graph tools:
/// ```
/// request;routing 2100
/// request;reedbase_lookup 8300
/// request;reedbase_lookup;cache_check 500
/// request;reedbase_lookup;csv_read 7800
/// request;template_render 32400
/// ```
///
/// ## Arguments
/// - `report`: Profile report to convert
///
/// ## Returns
/// Collapsed stack format as string
///
/// ## Example
/// ```rust
/// let data = generate_flamegraph_data(&report);
/// println!("{}", data);
/// ```
pub fn generate_flamegraph_data(report: &ProfileReport) -> String {
    let mut output = String::new();
    let mut stack = Vec::new();

    for span in &report.spans {
        // Build stack path (adjust to current depth)
        while stack.len() > span.depth {
            stack.pop();
        }
        stack.push(span.name.clone());

        // Generate line: stack path and sample count
        let path = stack.join(";");
        let samples = (span.duration.as_micros() / 100) as u64; // Convert μs to samples

        output.push_str(&format!("{} {}\n", path, samples));
    }

    output
}

/// Generates simplified SVG flame graph.
///
/// ## Arguments
/// - `report`: Profile report
/// - `width`: SVG width in pixels
/// - `height`: SVG height in pixels
///
/// ## Returns
/// SVG markup as string
///
/// ## Note
/// This is a simplified SVG for demonstration.
/// For production use, integrate with proper flame graph libraries.
pub fn generate_svg(report: &ProfileReport, width: u32, height: u32) -> String {
    let mut svg = String::new();

    svg.push_str(&format!(
        r#"<svg width="{}" height="{}" xmlns="http://www.w3.org/2000/svg">
  <style>
    text {{ font-family: monospace; font-size: 12px; }}
    .title {{ font-size: 16px; font-weight: bold; }}
  </style>
  <text class="title" x="10" y="25">Flame Graph: {}</text>
  <text x="10" y="50">Total: {:.1}ms</text>
"#,
        width,
        height,
        report.name,
        report.total_duration.as_secs_f64() * 1000.0
    ));

    // Draw simple bars for each span
    let mut y = 80;
    for span in &report.spans {
        let indent = span.depth * 20 + 10;
        let duration_ms = span.duration.as_secs_f64() * 1000.0;
        let bar_width = (duration_ms / report.total_duration.as_secs_f64() / 1000.0 * 400.0) as u32;

        let color = if span.depth % 2 == 0 {
            "6b9bd1"
        } else {
            "4a7ba7"
        };

        svg.push_str(&format!(
            "  <rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"20\" fill=\"#{}\" />\n  <text x=\"{}\" y=\"{}\">{}: {:.1}ms</text>\n",
            indent,
            y,
            bar_width.max(50),
            color,
            indent + 5,
            y + 15,
            span.name,
            duration_ms
        ));

        y += 30;
    }

    svg.push_str("</svg>");
    svg
}
