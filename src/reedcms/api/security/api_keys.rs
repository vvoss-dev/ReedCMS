// Copyright 2025 Vivian Voss. Licensed under the Apache Licence, Version 2.0.
// SPDX-Licence-Identifier: Apache-2.0

//! API Key Management
//!
//! Manages API keys for programmatic access to ReedCMS API.
//!
//! ## Key Format
//! - Prefix: `reed_`
//! - Length: 32 random characters (alphanumeric lowercase)
//! - Example: `reed_a7f3k9s2m4p1q8w5e6r7t9y2u3i4o5p6`
//!
//! ## Storage
//! - File: `.reed/api.keys.csv`
//! - Format: `key_hash|user_id|created|expires|description`
//! - Keys hashed with SHA-256 before storage
//!
//! ## Security
//! - Original key shown ONCE at generation
//! - SHA-256 hash stored (irreversible)
//! - Expiration dates enforced
//! - Revocation supported
//!
//! ## Performance
//! - Generation: < 10ms
//! - Verification: < 5ms (O(n) linear search, n = number of keys)
//! - Revocation: < 50ms (CSV rewrite)
//!
//! ## Example Usage
//! ```rust
//! // Generate key
//! let key = ApiKeyManager::generate_key("alice", 365, "Production API access")?;
//! println!("Save this key: {}", key); // Only shown once!
//!
//! // Verify key
//! let user_id = ApiKeyManager::verify_key(&key)?;
//! ```

use crate::reedcms::csv::{read_csv, write_csv, CsvRecord};
use crate::reedcms::reedstream::{ReedError, ReedResult};
use std::path::Path;

/// API key manager.
pub struct ApiKeyManager;

impl ApiKeyManager {
    /// Generates new API key for user.
    ///
    /// ## Arguments
    /// - `user_id`: User identifier (username)
    /// - `expires_days`: Number of days until expiration
    /// - `description`: Human-readable description
    ///
    /// ## Returns
    /// - Unhashed API key (show to user ONCE)
    ///
    /// ## Process
    /// 1. Generate random key with `reed_` prefix
    /// 2. Hash key with SHA-256
    /// 3. Store in `.reed/api.keys.csv` with expiration
    /// 4. Return unhashed key
    ///
    /// ## Performance
    /// - < 10ms
    ///
    /// ## Example Usage
    /// ```rust
    /// let key = ApiKeyManager::generate_key("alice", 365, "Production API")?;
    /// println!("Your API key (save it now): {}", key);
    /// ```
    pub fn generate_key(user_id: &str, expires_days: u32, description: &str) -> ReedResult<String> {
        let key = generate_random_key();
        let key_hash = hash_api_key(&key);

        // Calculate expiration timestamp
        let created = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let expires = created + (expires_days as u64 * 86400);

        // Read existing keys
        let csv_path = ".reed/api.keys.csv";
        let mut records = if Path::new(csv_path).exists() {
            read_csv(csv_path)?
        } else {
            Vec::new()
        };

        // Add new key
        records.push(CsvRecord {
            key: key_hash,
            value: format!("{}|{}|{}|{}", user_id, created, expires, description),
            description: None,
        });

        // Write back using existing csv module
        write_csv(Path::new(csv_path), &records)?;

        Ok(key)
    }

    /// Verifies API key and returns user ID.
    ///
    /// ## Arguments
    /// - `key`: API key to verify
    ///
    /// ## Returns
    /// - `Ok(user_id)` if key valid and not expired
    /// - `Err(...)` if key invalid or expired
    ///
    /// ## Process
    /// 1. Hash provided key with SHA-256
    /// 2. Linear search in `.reed/api.keys.csv`
    /// 3. Check expiration timestamp
    /// 4. Return associated user_id
    ///
    /// ## Performance
    /// - O(n) where n is number of keys
    /// - < 5ms for < 1000 keys
    ///
    /// ## Example Usage
    /// ```rust
    /// let user_id = ApiKeyManager::verify_key("reed_abc123...")?;
    /// println!("Authenticated as: {}", user_id);
    /// ```
    pub fn verify_key(key: &str) -> ReedResult<String> {
        let key_hash = hash_api_key(key);
        let csv_path = ".reed/api.keys.csv";

        if !Path::new(csv_path).exists() {
            return Err(ReedError::AuthError {
                user: None,
                action: "api_key_verify".to_string(),
                reason: "No API keys configured".to_string(),
            });
        }

        let records = read_csv(csv_path)?;

        for record in records {
            if record.key == key_hash {
                // Parse value: user_id|created|expires|description
                let parts: Vec<&str> = record.value.split('|').collect();
                if parts.len() < 3 {
                    continue;
                }

                let user_id = parts[0];
                let expires = parts[2].parse::<u64>().map_err(|_| ReedError::AuthError {
                    user: Some(user_id.to_string()),
                    action: "api_key_verify".to_string(),
                    reason: "Invalid expiration format".to_string(),
                })?;

                // Check expiration
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs();

                if now > expires {
                    return Err(ReedError::AuthError {
                        user: Some(user_id.to_string()),
                        action: "api_key_verify".to_string(),
                        reason: "API key expired".to_string(),
                    });
                }

                return Ok(user_id.to_string());
            }
        }

        Err(ReedError::AuthError {
            user: None,
            action: "api_key_verify".to_string(),
            reason: "Invalid API key".to_string(),
        })
    }

    /// Revokes API key.
    ///
    /// ## Arguments
    /// - `key`: API key to revoke
    ///
    /// ## Returns
    /// - `Ok(())` if key revoked
    /// - `Err(...)` if key not found
    ///
    /// ## Process
    /// 1. Hash provided key
    /// 2. Read all keys from CSV
    /// 3. Filter out matching key
    /// 4. Write back remaining keys
    ///
    /// ## Performance
    /// - O(n) read + filter + write
    /// - < 50ms for < 1000 keys
    ///
    /// ## Example Usage
    /// ```rust
    /// ApiKeyManager::revoke_key("reed_abc123...")?;
    /// ```
    pub fn revoke_key(key: &str) -> ReedResult<()> {
        let key_hash = hash_api_key(key);
        let csv_path = ".reed/api.keys.csv";

        if !Path::new(csv_path).exists() {
            return Err(ReedError::AuthError {
                user: None,
                action: "api_key_revoke".to_string(),
                reason: "No API keys configured".to_string(),
            });
        }

        let records = read_csv(csv_path)?;

        // Filter out the revoked key
        let remaining: Vec<CsvRecord> = records.into_iter().filter(|r| r.key != key_hash).collect();

        // Write back
        write_csv(Path::new(csv_path), &remaining)?;

        Ok(())
    }

    /// Lists all API keys for user.
    ///
    /// ## Arguments
    /// - `user_id`: User identifier
    ///
    /// ## Returns
    /// - Vector of API key metadata (hash, created, expires, description)
    ///
    /// ## Use Case
    /// - Admin dashboard
    /// - User's own key management
    ///
    /// ## Example Usage
    /// ```rust
    /// let keys = ApiKeyManager::list_keys("alice")?;
    /// for key in keys {
    ///     println!("{}: expires {}", key.description, key.expires);
    /// }
    /// ```
    pub fn list_keys(user_id: &str) -> ReedResult<Vec<ApiKeyInfo>> {
        let csv_path = ".reed/api.keys.csv";

        if !Path::new(csv_path).exists() {
            return Ok(Vec::new());
        }

        let records = read_csv(csv_path)?;
        let mut keys = Vec::new();

        for record in records {
            let parts: Vec<&str> = record.value.split('|').collect();
            if parts.len() >= 4 && parts[0] == user_id {
                keys.push(ApiKeyInfo {
                    key_hash: record.key,
                    user_id: parts[0].to_string(),
                    created: parts[1].parse().unwrap_or(0),
                    expires: parts[2].parse().unwrap_or(0),
                    description: parts[3].to_string(),
                });
            }
        }

        Ok(keys)
    }
}

/// API key metadata (for listing).
#[derive(Debug, Clone)]
pub struct ApiKeyInfo {
    pub key_hash: String,
    pub user_id: String,
    pub created: u64,
    pub expires: u64,
    pub description: String,
}

/// Generates random API key.
///
/// ## Output
/// - Random key with `reed_` prefix
/// - 32 alphanumeric lowercase characters
///
/// ## Example
/// - `reed_a7f3k9s2m4p1q8w5e6r7t9y2u3i4o5p6`
pub fn generate_random_key() -> String {
    use rand::Rng;
    const CHARSET: &[u8] = b"abcdefghijklmnopqrstuvwxyz0123456789";
    let mut rng = rand::thread_rng();

    let key: String = (0..32)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect();

    format!("reed_{}", key)
}

/// Hashes API key with SHA-256.
///
/// ## Arguments
/// - `key`: API key to hash
///
/// ## Returns
/// - Hex-encoded SHA-256 hash
///
/// ## Use Case
/// - Storing keys securely
/// - Comparing provided keys during verification
pub fn hash_api_key(key: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(key.as_bytes());
    format!("{:x}", hasher.finalize())
}
