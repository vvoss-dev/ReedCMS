// Copyright 2025 Vivian Voss. Licensed under the Apache Licence, Version 2.0.
// SPDX-Licence-Identifier: Apache-2.0

//! Response Builder Tests

#[cfg(test)]
mod tests {
    // Note: Full integration tests require routing, templates, and ReedBase to be initialised
    // These tests focus on unit-testable components

    use super::super::builder::build_response;
    use actix_web::test::TestRequest;

    #[test]
    fn test_build_response_returns_response() {
        // Basic smoke test - should not panic
        // Will likely return 404 since routing isn't initialised
        let req = TestRequest::default()
            .uri("/test")
            .to_http_request();

        let runtime = tokio::runtime::Runtime::new().unwrap();
        let result = runtime.block_on(async {
            build_response(req).await
        });

        // Should return Ok with some response (likely 404)
        assert!(result.is_ok());
    }

    #[test]
    fn test_build_response_with_user_agent() {
        let req = TestRequest::default()
            .uri("/test")
            .insert_header(("User-Agent", "Mozilla/5.0 (iPhone)"))
            .to_http_request();

        let runtime = tokio::runtime::Runtime::new().unwrap();
        let result = runtime.block_on(async {
            build_response(req).await
        });

        assert!(result.is_ok());
    }
}
