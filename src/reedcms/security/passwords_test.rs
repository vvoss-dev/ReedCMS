// Copyright 2025 Vivian Voss. Licensed under the Apache Licence, Version 2.0.
// SPDX-Licence-Identifier: Apache-2.0

//! Tests for Password Management

#[cfg(test)]
mod tests {
    use crate::reedcms::security::passwords::{
        hash_password, validate_password_strength, verify_password,
    };

    #[test]
    fn test_hash_password_success() {
        let password = "SecureP@ssw0rd";
        let hash = hash_password(password).expect("Hashing should succeed");

        // Argon2id hash should start with $argon2id$
        assert!(hash.starts_with("$argon2id$"));
        assert!(hash.len() > 50); // Hashes are long
    }

    #[test]
    fn test_hash_password_empty() {
        let result = hash_password("");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(
            err,
            crate::reedcms::reedstream::ReedError::ValidationError { .. }
        ));
    }

    #[test]
    fn test_hash_password_deterministic() {
        // Same password should produce different hashes (due to salt)
        let hash1 = hash_password("TestP@ss123").unwrap();
        let hash2 = hash_password("TestP@ss123").unwrap();

        assert_ne!(hash1, hash2, "Hashes should differ due to random salt");
    }

    #[test]
    fn test_verify_password_correct() {
        let password = "MySecureP@ss";
        let hash = hash_password(password).unwrap();

        let result = verify_password(password, &hash).expect("Verification should succeed");
        assert!(result, "Correct password should verify successfully");
    }

    #[test]
    fn test_verify_password_incorrect() {
        let hash = hash_password("CorrectP@ss").unwrap();

        let result = verify_password("WrongP@ss", &hash).expect("Verification should not error");
        assert!(!result, "Incorrect password should not verify");
    }

    #[test]
    fn test_verify_password_invalid_hash() {
        let result = verify_password("anypass", "invalid_hash_format");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_password_strength_valid() {
        let valid_passwords = vec!["Abcd123!", "MyP@ssw0rd", "Secure#Pass1", "Complex$P4ss"];

        for password in valid_passwords {
            validate_password_strength(password)
                .expect(&format!("Password '{}' should be valid", password));
        }
    }

    #[test]
    fn test_validate_password_strength_too_short() {
        let result = validate_password_strength("Ab1!");
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("at least 8 characters"));
    }

    #[test]
    fn test_validate_password_strength_no_uppercase() {
        let result = validate_password_strength("abcd123!");
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("uppercase"));
    }

    #[test]
    fn test_validate_password_strength_no_lowercase() {
        let result = validate_password_strength("ABCD123!");
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("lowercase"));
    }

    #[test]
    fn test_validate_password_strength_no_digit() {
        let result = validate_password_strength("Abcdefgh!");
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("digit"));
    }

    #[test]
    fn test_validate_password_strength_no_special() {
        let result = validate_password_strength("Abcd1234");
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("special character"));
    }

    #[test]
    fn test_password_hashing_performance() {
        // Argon2 should be intentionally slow (~100ms)
        let start = std::time::Instant::now();
        let _ = hash_password("TestP@ss123").unwrap();
        let duration = start.elapsed();

        // Should take at least 10ms (security feature)
        assert!(
            duration.as_millis() >= 10,
            "Hashing should be slow for security"
        );

        // Performance test - just ensure it completes (no upper bound on slow systems)
        println!("Password hashing took: {}ms", duration.as_millis());
    }

    #[test]
    fn test_verify_password_performance() {
        let hash = hash_password("TestP@ss123").unwrap();

        let start = std::time::Instant::now();
        let _ = verify_password("TestP@ss123", &hash).unwrap();
        let duration = start.elapsed();

        // Verification should be slow for security
        assert!(
            duration.as_millis() >= 10,
            "Verification should be slow for security"
        );

        // Performance test - just ensure it completes (no upper bound on slow systems)
        println!("Password verification took: {}ms", duration.as_millis());
    }

    #[test]
    fn test_argon2_hash_format() {
        let hash = hash_password("TestP@ss123").unwrap();

        // Argon2id PHC format: $argon2id$v=19$m=65536,t=3,p=4$...$...
        let parts: Vec<&str> = hash.split('$').collect();

        assert_eq!(parts[1], "argon2id", "Algorithm should be argon2id");
        assert_eq!(parts[2], "v=19", "Version should be 19");
        assert!(parts[3].contains("m=65536"), "Memory cost should be 65536");
        assert!(parts[3].contains("t=3"), "Time cost should be 3");
        assert!(parts[3].contains("p=4"), "Parallelism should be 4");
    }

    #[test]
    fn test_special_characters_all_valid() {
        let special_chars = "!@#$%^&*()_+-=[]{}|;:,.<>?";

        for ch in special_chars.chars() {
            let password = format!("Test{}Pass1", ch);
            validate_password_strength(&password)
                .expect(&format!("Password with '{}' should be valid", ch));
        }
    }
}
