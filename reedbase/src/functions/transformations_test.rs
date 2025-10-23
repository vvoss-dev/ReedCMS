// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Tests for transformation functions.

#[cfg(test)]
mod tests {
    use crate::functions::cache::get_cache;
    use crate::functions::transformations::*;

    #[test]
    fn test_normalize_email() {
        get_cache().clear();

        let email = normalize_email("  John.Doe@Example.COM  ").unwrap();
        assert_eq!(email, "john.doe@example.com");
    }

    #[test]
    fn test_normalize_email_already_clean() {
        get_cache().clear();

        let email = normalize_email("user@example.com").unwrap();
        assert_eq!(email, "user@example.com");
    }

    #[test]
    fn test_normalize_email_cached() {
        get_cache().clear();

        let email1 = normalize_email("Test@Example.COM").unwrap();
        let email2 = normalize_email("Test@Example.COM").unwrap();

        assert_eq!(email1, email2);

        let stats = get_cache().stats();
        assert!(stats.hits >= 1);
    }

    #[test]
    fn test_trim_spaces() {
        let result = trim("  hello  ").unwrap();
        assert_eq!(result, "hello");
    }

    #[test]
    fn test_trim_tabs_newlines() {
        let result = trim("\t\nhello\t\n").unwrap();
        assert_eq!(result, "hello");
    }

    #[test]
    fn test_trim_no_whitespace() {
        let result = trim("hello").unwrap();
        assert_eq!(result, "hello");
    }

    #[test]
    fn test_capitalize_lowercase() {
        get_cache().clear();

        let result = capitalize("hello").unwrap();
        assert_eq!(result, "Hello");
    }

    #[test]
    fn test_capitalize_already_capitalized() {
        get_cache().clear();

        let result = capitalize("World").unwrap();
        assert_eq!(result, "World");
    }

    #[test]
    fn test_capitalize_empty() {
        get_cache().clear();

        let result = capitalize("").unwrap();
        assert_eq!(result, "");
    }

    #[test]
    fn test_slugify_basic() {
        get_cache().clear();

        let slug = slugify("Hello World!").unwrap();
        assert_eq!(slug, "hello-world");
    }

    #[test]
    fn test_slugify_special_chars() {
        get_cache().clear();

        let slug = slugify("A/B Testing & Analysis").unwrap();
        assert_eq!(slug, "a-b-testing-analysis");
    }

    #[test]
    fn test_slugify_multiple_spaces() {
        get_cache().clear();

        let slug = slugify("  Multiple   Spaces  ").unwrap();
        assert_eq!(slug, "multiple-spaces");
    }

    #[test]
    fn test_slugify_numbers() {
        get_cache().clear();

        let slug = slugify("Test 123").unwrap();
        assert_eq!(slug, "test-123");
    }

    #[test]
    fn test_truncate_no_truncation() {
        get_cache().clear();

        let result = truncate("Hello", "10").unwrap();
        assert_eq!(result, "Hello");
    }

    #[test]
    fn test_truncate_with_ellipsis() {
        get_cache().clear();

        let result = truncate("This is a very long string", "10").unwrap();
        assert_eq!(result, "This is...");
    }

    #[test]
    fn test_truncate_exact_length() {
        get_cache().clear();

        let result = truncate("Hello", "5").unwrap();
        assert_eq!(result, "Hello");
    }

    #[test]
    fn test_replace_single() {
        get_cache().clear();

        let result = replace("hello world", "world", "rust").unwrap();
        assert_eq!(result, "hello rust");
    }

    #[test]
    fn test_replace_multiple() {
        get_cache().clear();

        let result = replace("foo bar foo", "foo", "baz").unwrap();
        assert_eq!(result, "baz bar baz");
    }

    #[test]
    fn test_replace_no_match() {
        get_cache().clear();

        let result = replace("hello world", "xyz", "abc").unwrap();
        assert_eq!(result, "hello world");
    }

    #[test]
    fn test_uppercase() {
        get_cache().clear();

        let result = uppercase("hello").unwrap();
        assert_eq!(result, "HELLO");
    }

    #[test]
    fn test_uppercase_mixed() {
        get_cache().clear();

        let result = uppercase("Hello World").unwrap();
        assert_eq!(result, "HELLO WORLD");
    }

    #[test]
    fn test_lowercase() {
        get_cache().clear();

        let result = lowercase("HELLO").unwrap();
        assert_eq!(result, "hello");
    }

    #[test]
    fn test_lowercase_mixed() {
        get_cache().clear();

        let result = lowercase("Hello World").unwrap();
        assert_eq!(result, "hello world");
    }

    #[test]
    fn test_remove_whitespace() {
        get_cache().clear();

        let result = remove_whitespace("hello world").unwrap();
        assert_eq!(result, "helloworld");
    }

    #[test]
    fn test_remove_whitespace_tabs_newlines() {
        get_cache().clear();

        let result = remove_whitespace("foo\t\nbar").unwrap();
        assert_eq!(result, "foobar");
    }

    #[test]
    fn test_pad_right_needed() {
        get_cache().clear();

        let result = pad_right("hello", "10").unwrap();
        assert_eq!(result, "hello     ");
        assert_eq!(result.len(), 10);
    }

    #[test]
    fn test_pad_right_not_needed() {
        get_cache().clear();

        let result = pad_right("hello", "3").unwrap();
        assert_eq!(result, "hello");
    }

    #[test]
    fn test_pad_right_exact() {
        get_cache().clear();

        let result = pad_right("hello", "5").unwrap();
        assert_eq!(result, "hello");
    }

    #[test]
    fn test_reverse() {
        get_cache().clear();

        let result = reverse("hello").unwrap();
        assert_eq!(result, "olleh");
    }

    #[test]
    fn test_reverse_palindrome() {
        get_cache().clear();

        let result = reverse("racecar").unwrap();
        assert_eq!(result, "racecar");
    }

    #[test]
    fn test_reverse_empty() {
        get_cache().clear();

        let result = reverse("").unwrap();
        assert_eq!(result, "");
    }

    #[test]
    fn test_transformation_cached() {
        get_cache().clear();

        let slug1 = slugify("Hello World").unwrap();
        let slug2 = slugify("Hello World").unwrap();

        assert_eq!(slug1, slug2);

        let stats = get_cache().stats();
        assert!(stats.hits >= 1);
    }
}
