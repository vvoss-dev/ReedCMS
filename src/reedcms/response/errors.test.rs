// Copyright 2025 Vivian Voss. Licensed under the Apache Licence, Version 2.0.
// SPDX-Licence-Identifier: Apache-2.0

//! Error Response Tests

#[cfg(test)]
mod tests {
    use super::super::errors::{build_404_response, build_500_response};
    use crate::reedcms::reedstream::ReedError;
    use actix_web::http::StatusCode;

    #[test]
    fn test_404_response_status() {
        let response = build_404_response();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn test_404_response_content_type() {
        let response = build_404_response();
        let content_type = response
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");

        // Should be either HTML (with template) or plain text (fallback)
        assert!(content_type.contains("text/html") || content_type.contains("text/plain"));
    }

    #[test]
    fn test_500_response_status() {
        let error = ReedError::FileNotFound {
            path: "test.txt".to_string(),
        };
        let response = build_500_response(error);
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn test_500_response_content_type() {
        let error = ReedError::FileNotFound {
            path: "test.txt".to_string(),
        };
        let response = build_500_response(error);
        let content_type = response
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");

        // Should be either HTML (with template) or plain text (fallback)
        assert!(content_type.contains("text/html") || content_type.contains("text/plain"));
    }
}
