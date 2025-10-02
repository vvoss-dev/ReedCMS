// Copyright 2025 Vivian Voss. Licensed under the Apache Licence, Version 2.0.
// SPDX-Licence-Identifier: Apache-2.0

//! Content Type Negotiation Tests

#[cfg(test)]
mod tests {
    use super::super::content_type::{negotiate_content_type, ContentType};
    use actix_web::test::TestRequest;

    #[test]
    fn test_content_type_mime_types() {
        assert_eq!(ContentType::Html.mime_type(), "text/html; charset=utf-8");
        assert_eq!(
            ContentType::Json.mime_type(),
            "application/json; charset=utf-8"
        );
        assert_eq!(ContentType::Plain.mime_type(), "text/plain; charset=utf-8");
    }

    #[test]
    fn test_negotiate_html_default() {
        let req = TestRequest::default().to_http_request();
        assert_eq!(negotiate_content_type(&req), ContentType::Html);
    }

    #[test]
    fn test_negotiate_json() {
        let req = TestRequest::default()
            .insert_header(("Accept", "application/json"))
            .to_http_request();
        assert_eq!(negotiate_content_type(&req), ContentType::Json);
    }

    #[test]
    fn test_negotiate_plain() {
        let req = TestRequest::default()
            .insert_header(("Accept", "text/plain"))
            .to_http_request();
        assert_eq!(negotiate_content_type(&req), ContentType::Plain);
    }

    #[test]
    fn test_negotiate_html_explicit() {
        let req = TestRequest::default()
            .insert_header(("Accept", "text/html"))
            .to_http_request();
        assert_eq!(negotiate_content_type(&req), ContentType::Html);
    }

    #[test]
    fn test_negotiate_priority_json() {
        // JSON should be detected first even with mixed Accept header
        let req = TestRequest::default()
            .insert_header(("Accept", "text/html, application/json"))
            .to_http_request();
        assert_eq!(negotiate_content_type(&req), ContentType::Json);
    }
}
