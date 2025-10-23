// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! ReedBase Key Specification v2 (RBKS v2) validation.
//!
//! Enforces structured key format for reliable index-based queries:
//! `<namespace>.<hierarchy>.<type>[<modifier,modifier,...>]`
//!
//! ## Key Structure Rules
//!
//! 1. **Lowercase only**: `page.header.title` ✅, `Page.Header.Title` ❌
//! 2. **Dots for hierarchy**: `page.header.title` ✅, `page-header-title` ❌
//! 3. **Angle brackets for modifiers**: `<de,prod>` ✅, `@de#prod` ❌
//! 4. **Modifiers comma-separated**: `<de,prod,christmas>` ✅, `<de prod>` ❌
//! 5. **Modifiers order-independent**: `<de,prod>` = `<prod,de>` ✅
//! 6. **No empty segments**: `page.title` ✅, `page..title` ❌
//! 7. **No leading/trailing dots**: `page.title` ✅, `.page.title` ❌
//! 8. **Depth 2-8 levels**: `page.title` (2) ✅, `a.b.c.d.e.f.g.h.i` (9) ❌
//! 9. **No empty modifiers**: `<de>` ✅, `<>` ❌
//!
//! ## Modifier Categories
//!
//! - **Language** (ISO 639-1): `de`, `en`, `fr`, etc. (max 1)
//! - **Environment**: `dev`, `prod`, `staging`, `test` (max 1)
//! - **Season**: `christmas`, `easter`, `summer`, `winter` (max 1)
//! - **Variant**: `mobile`, `desktop`, `tablet` (max 1)
//! - **Custom**: Any other lowercase identifier (multiple allowed)
//!
//! ## Example Usage
//!
//! ```rust
//! use reedbase::schema::rbks::{validate_key, parse_key, normalize_key};
//!
//! // Validate a key
//! validate_key("page.header.title<de,prod>")?;
//!
//! // Parse key with modifiers
//! let parsed = parse_key("page.header.title<de,prod,christmas>")?;
//! assert_eq!(parsed.base, "page.header.title");
//! assert_eq!(parsed.modifiers.language, Some("de".to_string()));
//!
//! // Normalize malformed key
//! let normalized = normalize_key("Page.Header..Title<PROD,DE,prod>")?;
//! assert_eq!(normalized, "page.header.title<de,prod>");
//! ```

use crate::error::{ReedError, ReedResult};
use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::HashSet;

/// RBKS v2 validation regex pattern.
///
/// Matches: `<namespace>.<hierarchy>[<modifier,modifier>]`
/// - 2-8 segments separated by dots
/// - Each segment starts with letter, followed by alphanumeric
/// - Optional modifiers in angle brackets `<mod1,mod2>`
pub static RBKS_V2_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^[a-z][a-z0-9]*(\.[a-z][a-z0-9]*){1,7}(<[a-z][a-z0-9]+(,[a-z][a-z0-9]+)*>)?$")
        .unwrap()
});

/// Known ISO 639-1 language codes (2-letter).
pub static KNOWN_LANGUAGES: &[&str] = &[
    "de", "en", "fr", "es", "it", "pt", "nl", "pl", "ru", "ja", "zh", "ar", "ko", "hi", "tr", "sv",
    "da", "no", "fi", "cs", "sk", "hu", "ro", "el", "he", "id", "th", "vi", "uk", "bg",
];

/// Known environment identifiers.
pub static KNOWN_ENVIRONMENTS: &[&str] = &["dev", "prod", "staging", "test"];

/// Known season identifiers.
pub static KNOWN_SEASONS: &[&str] = &["christmas", "easter", "summer", "winter"];

/// Known variant identifiers.
pub static KNOWN_VARIANTS: &[&str] = &["mobile", "desktop", "tablet"];

/// Parsed key with base and modifiers.
///
/// ## Fields
/// - `base`: Base key without modifiers (e.g., `page.header.title`)
/// - `modifiers`: Classified modifiers
/// - `raw`: Original raw key
#[derive(Debug, Clone, PartialEq)]
pub struct ParsedKey {
    pub base: String,
    pub modifiers: Modifiers,
    pub raw: String,
}

impl ParsedKey {
    /// Get the depth of the key (number of segments).
    ///
    /// ## Example
    /// ```rust
    /// let parsed = parse_key("page.header.title")?;
    /// assert_eq!(parsed.depth(), 3);
    /// ```
    pub fn depth(&self) -> usize {
        self.base.split('.').count()
    }

    /// Get the namespace (first segment).
    ///
    /// ## Example
    /// ```rust
    /// let parsed = parse_key("page.header.title")?;
    /// assert_eq!(parsed.namespace(), "page");
    /// ```
    pub fn namespace(&self) -> &str {
        self.base.split('.').next().unwrap_or("")
    }

    /// Reconstruct the canonical key.
    ///
    /// ## Example
    /// ```rust
    /// let parsed = parse_key("page.title<prod,de>")?;
    /// assert_eq!(parsed.to_canonical(), "page.title<de,prod>");
    /// ```
    pub fn to_canonical(&self) -> String {
        if self.modifiers.is_empty() {
            self.base.clone()
        } else {
            format!("{}<{}>", self.base, self.modifiers.to_string())
        }
    }
}

/// Parsed modifiers from `<lang,env,...>` syntax.
///
/// ## Categories
/// - **language**: ISO 639-1 code (max 1)
/// - **environment**: dev/prod/staging/test (max 1)
/// - **season**: christmas/easter/summer/winter (max 1)
/// - **variant**: mobile/desktop/tablet (max 1)
/// - **custom**: Any other identifier (multiple allowed)
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Modifiers {
    pub language: Option<String>,
    pub environment: Option<String>,
    pub season: Option<String>,
    pub variant: Option<String>,
    pub custom: Vec<String>,
}

impl Modifiers {
    /// Check if modifiers are empty.
    pub fn is_empty(&self) -> bool {
        self.language.is_none()
            && self.environment.is_none()
            && self.season.is_none()
            && self.variant.is_none()
            && self.custom.is_empty()
    }

    /// Convert to sorted, comma-separated string.
    ///
    /// ## Example
    /// ```rust
    /// let mods = Modifiers {
    ///     language: Some("de".to_string()),
    ///     environment: Some("prod".to_string()),
    ///     ..Default::default()
    /// };
    /// assert_eq!(mods.to_string(), "de,prod");
    /// ```
    pub fn to_string(&self) -> String {
        let mut parts = Vec::new();

        if let Some(ref lang) = self.language {
            parts.push(lang.clone());
        }
        if let Some(ref env) = self.environment {
            parts.push(env.clone());
        }
        if let Some(ref season) = self.season {
            parts.push(season.clone());
        }
        if let Some(ref variant) = self.variant {
            parts.push(variant.clone());
        }
        parts.extend(self.custom.clone());

        parts.sort();
        parts.join(",")
    }

    /// Generate fallback chain for modifier resolution.
    ///
    /// ## Priority (highest to lowest)
    /// 1. Exact match with all modifiers
    /// 2. Without season
    /// 3. Without environment
    /// 4. Language only
    /// 5. Environment + season
    /// 6. Environment only
    /// 7. Season only
    /// 8. Base key (no modifiers)
    ///
    /// ## Performance
    /// - Max 8 lookups (power set of 3 modifier types)
    /// - Early exit on first match
    ///
    /// ## Example
    /// ```rust
    /// let mods = Modifiers {
    ///     language: Some("de".to_string()),
    ///     environment: Some("prod".to_string()),
    ///     season: Some("christmas".to_string()),
    ///     ..Default::default()
    /// };
    /// let chain = mods.fallback_chain();
    /// // Returns: ["de,prod,christmas", "de,prod", "de,christmas", "de", "prod,christmas", "prod", "christmas", ""]
    /// ```
    pub fn fallback_chain(&self) -> Vec<String> {
        let mut chain = Vec::new();

        // 1. Exact match (all modifiers)
        if !self.is_empty() {
            chain.push(self.to_string());
        }

        // 2. Without season
        if self.season.is_some() {
            let mut without_season = self.clone();
            without_season.season = None;
            if !without_season.is_empty() {
                chain.push(without_season.to_string());
            }
        }

        // 3. Without environment
        if self.environment.is_some() {
            let mut without_env = self.clone();
            without_env.environment = None;
            if !without_env.is_empty() {
                chain.push(without_env.to_string());
            }
        }

        // 4. Language only
        if self.language.is_some() {
            let lang_only = Modifiers {
                language: self.language.clone(),
                ..Default::default()
            };
            chain.push(lang_only.to_string());
        }

        // 5. Environment + season
        if self.environment.is_some() && self.season.is_some() {
            let env_season = Modifiers {
                environment: self.environment.clone(),
                season: self.season.clone(),
                ..Default::default()
            };
            chain.push(env_season.to_string());
        }

        // 6. Environment only
        if self.environment.is_some() {
            let env_only = Modifiers {
                environment: self.environment.clone(),
                ..Default::default()
            };
            chain.push(env_only.to_string());
        }

        // 7. Season only
        if self.season.is_some() {
            let season_only = Modifiers {
                season: self.season.clone(),
                ..Default::default()
            };
            chain.push(season_only.to_string());
        }

        // 8. Base key (empty modifiers)
        chain.push(String::new());

        // Deduplicate
        let mut seen = HashSet::new();
        chain.retain(|s| seen.insert(s.clone()));

        chain
    }
}

/// Validate key structure according to RBKS v2.
///
/// ## Input
/// - `key`: Key to validate (e.g., `page.title<de,prod>`)
///
/// ## Output
/// - `Ok(())`: Key is valid
/// - `Err(ReedError)`: Validation failed with reason
///
/// ## Performance
/// - < 20μs typical (regex + basic checks)
///
/// ## Error Conditions
/// - `InvalidCsv`: Key format invalid (uppercase, wrong separators, etc.)
///
/// ## Example Usage
/// ```rust
/// validate_key("page.title<de,prod>")?;  // ✅ Valid
/// validate_key("Page.Title<DE>")?;       // ❌ Error: Must be lowercase
/// ```
pub fn validate_key(key: &str) -> ReedResult<()> {
    // Check regex pattern
    if !RBKS_V2_PATTERN.is_match(key) {
        // Provide helpful error messages
        if key.chars().any(|c| c.is_uppercase()) {
            return Err(ReedError::InvalidCsv {
                reason: format!("Key must be lowercase: '{}'", key),
                line: 0,
            });
        }

        if key.contains('-') {
            return Err(ReedError::InvalidCsv {
                reason: format!("Use dots (.) for hierarchy, not hyphens: '{}'", key),
                line: 0,
            });
        }

        if key.contains("..") {
            return Err(ReedError::InvalidCsv {
                reason: format!("Empty segments not allowed: '{}'", key),
                line: 0,
            });
        }

        if key.starts_with('.') || key.ends_with('.') {
            return Err(ReedError::InvalidCsv {
                reason: format!("Leading/trailing dots not allowed: '{}'", key),
                line: 0,
            });
        }

        if key.contains("<>") {
            return Err(ReedError::InvalidCsv {
                reason: format!("Empty modifiers <> not allowed: '{}'", key),
                line: 0,
            });
        }

        if key.contains(' ') {
            return Err(ReedError::InvalidCsv {
                reason: format!("Spaces not allowed in keys: '{}'", key),
                line: 0,
            });
        }

        // Count depth
        let base = key.split('<').next().unwrap_or(key);
        let depth = base.split('.').count();
        if depth < 2 {
            return Err(ReedError::InvalidCsv {
                reason: format!("Key must have at least 2 segments: '{}'", key),
                line: 0,
            });
        }
        if depth > 8 {
            return Err(ReedError::InvalidCsv {
                reason: format!("Key depth exceeds maximum of 8: '{}'", key),
                line: 0,
            });
        }

        return Err(ReedError::InvalidCsv {
            reason: format!("Invalid key format: '{}'", key),
            line: 0,
        });
    }

    // Additional validation: parse modifiers if present
    if key.contains('<') {
        let parsed = parse_key(key)?;
        classify_modifiers(&parsed)?;
    }

    Ok(())
}

/// Parse key into base and modifiers.
///
/// ## Input
/// - `key`: Key to parse (e.g., `page.title<de,prod>`)
///
/// ## Output
/// - `Ok(ParsedKey)`: Parsed key with base and raw modifiers
/// - `Err(ReedError)`: Parse failed
///
/// ## Performance
/// - < 15μs typical (string split + allocation)
///
/// ## Example Usage
/// ```rust
/// let parsed = parse_key("page.title<de,prod>")?;
/// assert_eq!(parsed.base, "page.title");
/// assert_eq!(parsed.modifiers.language, Some("de".to_string()));
/// ```
pub fn parse_key(key: &str) -> ReedResult<ParsedKey> {
    // Split into base and modifiers
    if let Some(idx) = key.find('<') {
        let base = &key[..idx];
        let mods_str = &key[idx + 1..key.len() - 1]; // Remove < and >

        // Parse modifiers
        let raw_mods: Vec<String> = mods_str.split(',').map(|s| s.trim().to_string()).collect();

        // Classify modifiers
        let modifiers = classify_modifiers_from_vec(&raw_mods)?;

        Ok(ParsedKey {
            base: base.to_string(),
            modifiers,
            raw: key.to_string(),
        })
    } else {
        // No modifiers
        Ok(ParsedKey {
            base: key.to_string(),
            modifiers: Modifiers::default(),
            raw: key.to_string(),
        })
    }
}

/// Classify modifiers from ParsedKey into categories.
///
/// ## Input
/// - `parsed`: Parsed key with modifiers
///
/// ## Output
/// - `Ok(())`: Classification successful
/// - `Err(ReedError)`: Multiple modifiers of same category
///
/// ## Example Usage
/// ```rust
/// let parsed = parse_key("page.title<de,en>")?;
/// classify_modifiers(&parsed)?;  // ❌ Error: Multiple languages
/// ```
fn classify_modifiers(parsed: &ParsedKey) -> ReedResult<()> {
    let mods = &parsed.modifiers;

    // Check for multiple languages
    if mods.language.is_some() {
        let lang_count = vec![&mods.language].iter().filter(|o| o.is_some()).count();
        if lang_count > 1 {
            return Err(ReedError::InvalidCsv {
                reason: format!("Multiple languages not allowed: '{}'", parsed.raw),
                line: 0,
            });
        }
    }

    // Check for multiple environments
    if mods.environment.is_some() {
        let env_count = vec![&mods.environment]
            .iter()
            .filter(|o| o.is_some())
            .count();
        if env_count > 1 {
            return Err(ReedError::InvalidCsv {
                reason: format!("Multiple environments not allowed: '{}'", parsed.raw),
                line: 0,
            });
        }
    }

    // Check for multiple seasons
    if mods.season.is_some() {
        let season_count = vec![&mods.season].iter().filter(|o| o.is_some()).count();
        if season_count > 1 {
            return Err(ReedError::InvalidCsv {
                reason: format!("Multiple seasons not allowed: '{}'", parsed.raw),
                line: 0,
            });
        }
    }

    // Check for multiple variants
    if mods.variant.is_some() {
        let variant_count = vec![&mods.variant].iter().filter(|o| o.is_some()).count();
        if variant_count > 1 {
            return Err(ReedError::InvalidCsv {
                reason: format!("Multiple variants not allowed: '{}'", parsed.raw),
                line: 0,
            });
        }
    }

    Ok(())
}

/// Classify raw modifiers into categories.
///
/// ## Input
/// - `raw`: Raw modifier strings (e.g., `["de", "prod", "christmas"]`)
///
/// ## Output
/// - `Ok(Modifiers)`: Classified modifiers
/// - `Err(ReedError)`: Multiple modifiers of same category
///
/// ## Rules
/// - Language: 2-letter ISO 639-1 codes (max 1)
/// - Environment: dev/prod/staging/test (max 1)
/// - Season: christmas/easter/summer/winter (max 1)
/// - Variant: mobile/desktop/tablet (max 1)
/// - Custom: anything else (multiple allowed)
///
/// ## Performance
/// - < 10μs typical (category matching)
///
/// ## Example Usage
/// ```rust
/// let mods = classify_modifiers_from_vec(&vec!["de".to_string(), "prod".to_string()])?;
/// assert_eq!(mods.language, Some("de".to_string()));
/// assert_eq!(mods.environment, Some("prod".to_string()));
/// ```
fn classify_modifiers_from_vec(raw: &[String]) -> ReedResult<Modifiers> {
    let mut modifiers = Modifiers::default();

    for modifier in raw {
        let m = modifier.as_str();

        // Check language
        if KNOWN_LANGUAGES.contains(&m) {
            if modifiers.language.is_some() {
                return Err(ReedError::InvalidCsv {
                    reason: format!("Multiple languages not allowed: {:?}", raw),
                    line: 0,
                });
            }
            modifiers.language = Some(m.to_string());
        }
        // Check environment
        else if KNOWN_ENVIRONMENTS.contains(&m) {
            if modifiers.environment.is_some() {
                return Err(ReedError::InvalidCsv {
                    reason: format!("Multiple environments not allowed: {:?}", raw),
                    line: 0,
                });
            }
            modifiers.environment = Some(m.to_string());
        }
        // Check season
        else if KNOWN_SEASONS.contains(&m) {
            if modifiers.season.is_some() {
                return Err(ReedError::InvalidCsv {
                    reason: format!("Multiple seasons not allowed: {:?}", raw),
                    line: 0,
                });
            }
            modifiers.season = Some(m.to_string());
        }
        // Check variant
        else if KNOWN_VARIANTS.contains(&m) {
            if modifiers.variant.is_some() {
                return Err(ReedError::InvalidCsv {
                    reason: format!("Multiple variants not allowed: {:?}", raw),
                    line: 0,
                });
            }
            modifiers.variant = Some(m.to_string());
        }
        // Custom modifier
        else {
            modifiers.custom.push(m.to_string());
        }
    }

    Ok(modifiers)
}

/// Normalize key to canonical format.
///
/// ## Input
/// - `raw`: Raw key (possibly malformed)
///
/// ## Output
/// - `Ok(String)`: Normalized canonical key
/// - `Err(ReedError)`: Normalization failed
///
/// ## Operations
/// - Convert to lowercase
/// - Sort modifiers alphabetically
/// - Remove duplicate modifiers
/// - Trim whitespace
/// - Remove duplicate dots
/// - Remove leading/trailing dots
///
/// ## Performance
/// - O(n + m log m) where n = key length, m = modifiers count
/// - < 15μs typical
///
/// ## Example Usage
/// ```rust
/// let normalized = normalize_key("Page.Header..Title<PROD,DE,prod>")?;
/// assert_eq!(normalized, "page.header.title<de,prod>");
/// ```
pub fn normalize_key(raw: &str) -> ReedResult<String> {
    // Convert to lowercase
    let mut normalized = raw.to_lowercase().trim().to_string();

    // Remove duplicate dots
    while normalized.contains("..") {
        normalized = normalized.replace("..", ".");
    }

    // Remove leading/trailing dots
    normalized = normalized.trim_matches('.').to_string();

    // Split into base and modifiers
    if let Some(idx) = normalized.find('<') {
        let base = &normalized[..idx];
        let mods_str = &normalized[idx + 1..normalized.len() - 1];

        // Parse and deduplicate modifiers
        let mut mods: Vec<String> = mods_str.split(',').map(|s| s.trim().to_string()).collect();

        // Remove empty modifiers
        mods.retain(|m| !m.is_empty());

        // Deduplicate
        let mut seen = HashSet::new();
        mods.retain(|m| seen.insert(m.clone()));

        // Sort
        mods.sort();

        // Reconstruct
        if mods.is_empty() {
            normalized = base.to_string();
        } else {
            normalized = format!("{}<{}>", base, mods.join(","));
        }
    }

    // Validate normalized key
    validate_key(&normalized)?;

    Ok(normalized)
}
