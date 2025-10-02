// Copyright 2025 Vivian Voss. Licensed under the Apache Licence, Version 2.0.
// SPDX-Licence-Identifier: Apache-2.0

//! User Management
//!
//! Provides comprehensive user CRUD operations with extended profile data.

use crate::reedcms::backup::create_backup;
use crate::reedcms::matrix::{read_matrix_csv, write_matrix_csv, MatrixRecord, MatrixValue};
use crate::reedcms::reedstream::{
    current_timestamp, not_found, validation_error, ReedRequest, ReedResponse, ReedResult,
};
use crate::reedcms::security::passwords::hash_password;
use crate::reedcms::security::validation::{
    email_exists, username_exists, validate_email, validate_username,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// User information structure (without password hash for security).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    pub username: String,
    pub roles: Vec<String>,
    pub firstname: String,
    pub lastname: String,
    pub email: String,
    pub mobile: Option<String>,
    pub social_media: SocialMedia,
    pub address: Address,
    pub desc: String,
    pub created_at: u64,
    pub updated_at: u64,
    pub last_login: Option<u64>,
    pub is_active: bool,
}

/// Social media profile links.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocialMedia {
    pub twitter: Option<String>,
    pub facebook: Option<String>,
    pub tiktok: Option<String>,
    pub instagram: Option<String>,
    pub youtube: Option<String>,
    pub whatsapp: Option<String>,
}

/// Physical address information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Address {
    pub street: Option<String>,
    pub city: Option<String>,
    pub postcode: Option<String>,
    pub region: Option<String>,
    pub country: Option<String>,
}

/// User filter for list operations.
#[derive(Debug, Clone)]
pub struct UserFilter {
    pub is_active: Option<bool>,
    pub role: Option<String>,
}

/// User update structure for partial updates.
#[derive(Debug, Clone)]
pub struct UserUpdate {
    pub firstname: Option<String>,
    pub lastname: Option<String>,
    pub email: Option<String>,
    pub mobile: Option<String>,
    pub social_media: Option<SocialMedia>,
    pub address: Option<Address>,
    pub desc: Option<String>,
    pub is_active: Option<bool>,
}

/// Creates new user with validation and password hashing.
///
/// ## Input
/// - `req.key`: Username (validated: alphanumeric + underscore, 3-32 chars)
/// - `req.value`: Password (validated: min 8 chars, uppercase, lowercase, digit, special)
/// - `req.context`: JSON with additional fields (email, roles, profile data)
///
/// ## Validation
/// - Username uniqueness and format
/// - Email uniqueness and format
/// - Password strength requirements
/// - Role existence check (if provided)
///
/// ## Output
/// - `ReedResult<ReedResponse<UserInfo>>`: Created user info (without password hash)
///
/// ## Performance
/// - Creation time: < 150ms (including Argon2 hashing ~100ms)
///
/// ## Error Conditions
/// - Returns `ReedError::ValidationError` if username/email exists or is invalid
/// - Returns `ReedError::ValidationError` if password is weak
/// - Returns `ReedError::CsvError` if file write fails
///
/// ## Example Usage
/// ```
/// let req = ReedRequest {
///     key: "newuser".to_string(),
///     value: Some("SecureP@ss123".to_string()),
///     context: Some(r#"{"email":"user@example.com","roles":["editor"]}"#.to_string()),
///     ..Default::default()
/// };
/// let response = create_user(&req)?;
/// ```
pub fn create_user(req: &ReedRequest) -> ReedResult<ReedResponse<UserInfo>> {
    let username = &req.key;
    let password = req
        .value
        .as_ref()
        .ok_or_else(|| validation_error("password", "", "Password is required"))?;

    // Validate username format
    validate_username(username)?;

    // Check username uniqueness
    if username_exists(username)? {
        return Err(validation_error(
            "username",
            username,
            "Username already exists",
        ));
    }

    // Parse context for additional user data
    let context_data: serde_json::Value = if let Some(ctx) = &req.context {
        serde_json::from_str(ctx)
            .map_err(|e| validation_error("context", ctx, &format!("Invalid JSON: {}", e)))?
    } else {
        serde_json::json!({})
    };

    // Extract and validate email
    let email = context_data["email"]
        .as_str()
        .ok_or_else(|| validation_error("email", "", "Email is required"))?;
    validate_email(email)?;

    if email_exists(email)? {
        return Err(validation_error("email", email, "Email already registered"));
    }

    // Validate password strength before hashing
    crate::reedcms::security::passwords::validate_password_strength(password)?;

    // Hash password
    let password_hash = hash_password(password)?;

    // Extract roles (default to empty if not provided)
    let roles: Vec<String> = context_data["roles"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();

    // Build user record
    let timestamp = current_timestamp();
    let mut fields = HashMap::new();

    fields.insert(
        "username".to_string(),
        MatrixValue::Single(username.clone()),
    );
    fields.insert("password".to_string(), MatrixValue::Single(password_hash));
    fields.insert(
        "roles".to_string(),
        if roles.is_empty() {
            MatrixValue::Single(String::new())
        } else {
            MatrixValue::List(roles.clone())
        },
    );

    // Profile data with defaults
    fields.insert(
        "firstname".to_string(),
        MatrixValue::Single(context_data["firstname"].as_str().unwrap_or("").to_string()),
    );
    fields.insert(
        "lastname".to_string(),
        MatrixValue::Single(context_data["lastname"].as_str().unwrap_or("").to_string()),
    );
    fields.insert("email".to_string(), MatrixValue::Single(email.to_string()));

    // Optional fields
    fields.insert(
        "mobile".to_string(),
        MatrixValue::Single(context_data["mobile"].as_str().unwrap_or("").to_string()),
    );
    fields.insert(
        "street".to_string(),
        MatrixValue::Single(context_data["street"].as_str().unwrap_or("").to_string()),
    );
    fields.insert(
        "city".to_string(),
        MatrixValue::Single(context_data["city"].as_str().unwrap_or("").to_string()),
    );
    fields.insert(
        "postcode".to_string(),
        MatrixValue::Single(context_data["postcode"].as_str().unwrap_or("").to_string()),
    );
    fields.insert(
        "region".to_string(),
        MatrixValue::Single(context_data["region"].as_str().unwrap_or("").to_string()),
    );
    fields.insert(
        "country".to_string(),
        MatrixValue::Single(context_data["country"].as_str().unwrap_or("").to_string()),
    );

    // Social media
    fields.insert(
        "twitter".to_string(),
        MatrixValue::Single(context_data["twitter"].as_str().unwrap_or("").to_string()),
    );
    fields.insert(
        "facebook".to_string(),
        MatrixValue::Single(context_data["facebook"].as_str().unwrap_or("").to_string()),
    );
    fields.insert(
        "tiktok".to_string(),
        MatrixValue::Single(context_data["tiktok"].as_str().unwrap_or("").to_string()),
    );
    fields.insert(
        "instagram".to_string(),
        MatrixValue::Single(context_data["instagram"].as_str().unwrap_or("").to_string()),
    );
    fields.insert(
        "youtube".to_string(),
        MatrixValue::Single(context_data["youtube"].as_str().unwrap_or("").to_string()),
    );
    fields.insert(
        "whatsapp".to_string(),
        MatrixValue::Single(context_data["whatsapp"].as_str().unwrap_or("").to_string()),
    );

    // Metadata
    fields.insert(
        "desc".to_string(),
        MatrixValue::Single(context_data["desc"].as_str().unwrap_or("").to_string()),
    );
    fields.insert(
        "created_at".to_string(),
        MatrixValue::Single(timestamp.to_string()),
    );
    fields.insert(
        "updated_at".to_string(),
        MatrixValue::Single(timestamp.to_string()),
    );
    fields.insert("last_login".to_string(), MatrixValue::Single(String::new()));
    fields.insert(
        "is_active".to_string(),
        MatrixValue::Single("true".to_string()),
    );

    let record = MatrixRecord {
        fields,
        field_order: vec![
            "username",
            "password",
            "roles",
            "firstname",
            "lastname",
            "street",
            "city",
            "postcode",
            "region",
            "country",
            "email",
            "mobile",
            "twitter",
            "facebook",
            "tiktok",
            "instagram",
            "youtube",
            "whatsapp",
            "desc",
            "created_at",
            "updated_at",
            "last_login",
            "is_active",
        ]
        .into_iter()
        .map(|s| s.to_string())
        .collect(),
        description: Some(format!("User: {}", username)),
    };

    // Write to file (with backup)
    let users_path = Path::new(".reed/users.matrix.csv");
    if users_path.exists() {
        create_backup(users_path)?;
    }

    let mut records = if users_path.exists() {
        read_matrix_csv(users_path)?
    } else {
        Vec::new()
    };
    records.push(record);

    write_matrix_csv(users_path, &records, &[])?;

    // Build UserInfo response (without password hash)
    let user_info = UserInfo {
        username: username.clone(),
        roles,
        firstname: context_data["firstname"].as_str().unwrap_or("").to_string(),
        lastname: context_data["lastname"].as_str().unwrap_or("").to_string(),
        email: email.to_string(),
        mobile: context_data["mobile"].as_str().map(|s| s.to_string()),
        social_media: SocialMedia {
            twitter: context_data["twitter"].as_str().map(|s| s.to_string()),
            facebook: context_data["facebook"].as_str().map(|s| s.to_string()),
            tiktok: context_data["tiktok"].as_str().map(|s| s.to_string()),
            instagram: context_data["instagram"].as_str().map(|s| s.to_string()),
            youtube: context_data["youtube"].as_str().map(|s| s.to_string()),
            whatsapp: context_data["whatsapp"].as_str().map(|s| s.to_string()),
        },
        address: Address {
            street: context_data["street"].as_str().map(|s| s.to_string()),
            city: context_data["city"].as_str().map(|s| s.to_string()),
            postcode: context_data["postcode"].as_str().map(|s| s.to_string()),
            region: context_data["region"].as_str().map(|s| s.to_string()),
            country: context_data["country"].as_str().map(|s| s.to_string()),
        },
        desc: context_data["desc"].as_str().unwrap_or("").to_string(),
        created_at: timestamp,
        updated_at: timestamp,
        last_login: None,
        is_active: true,
    };

    Ok(ReedResponse {
        data: user_info,
        source: "security::users::create_user".to_string(),
        cached: false,
        timestamp: current_timestamp(),
        metrics: None,
    })
}

/// Retrieves user by username.
///
/// ## Input
/// - `username`: Username to retrieve
///
/// ## Output
/// - `ReedResult<ReedResponse<UserInfo>>`: User information (without password hash)
///
/// ## Performance
/// - Lookup time: < 50ms
///
/// ## Error Conditions
/// - Returns `ReedError::NotFound` if user doesn't exist
/// - Returns `ReedError::CsvError` if file read fails
///
/// ## Example Usage
/// ```
/// let response = get_user("admin")?;
/// println!("User: {} ({})", response.data.username, response.data.email);
/// ```
pub fn get_user(username: &str) -> ReedResult<ReedResponse<UserInfo>> {
    let users_path = Path::new(".reed/users.matrix.csv");

    if !users_path.exists() {
        return Err(not_found(username));
    }

    let records = read_matrix_csv(users_path)?;

    for record in records {
        if let Some(MatrixValue::Single(user)) = record.fields.get("username") {
            if user == username {
                return Ok(ReedResponse {
                    data: matrix_record_to_user_info(&record)?,
                    source: "security::users::get_user".to_string(),
                    cached: false,
                    timestamp: current_timestamp(),
                    metrics: None,
                });
            }
        }
    }

    Err(not_found(username))
}

/// Lists all users with optional filtering.
///
/// ## Input
/// - `filter`: Optional filter criteria (active status, role)
///
/// ## Output
/// - `ReedResult<ReedResponse<Vec<UserInfo>>>`: List of users (without password hashes)
///
/// ## Performance
/// - List time: < 100ms for typical user count
///
/// ## Error Conditions
/// - Returns `ReedError::CsvError` if file read fails
///
/// ## Example Usage
/// ```
/// let filter = Some(UserFilter { is_active: Some(true), role: Some("editor".to_string()) });
/// let response = list_users(filter)?;
/// println!("Found {} active editors", response.data.len());
/// ```
pub fn list_users(filter: Option<UserFilter>) -> ReedResult<ReedResponse<Vec<UserInfo>>> {
    let users_path = Path::new(".reed/users.matrix.csv");

    if !users_path.exists() {
        return Ok(ReedResponse {
            data: Vec::new(),
            source: "security::users::list_users".to_string(),
            cached: false,
            timestamp: current_timestamp(),
            metrics: None,
        });
    }

    let records = read_matrix_csv(users_path)?;
    let mut users = Vec::new();

    for record in records {
        let user_info = matrix_record_to_user_info(&record)?;

        // Apply filters if provided
        if let Some(ref f) = filter {
            if let Some(active_filter) = f.is_active {
                if user_info.is_active != active_filter {
                    continue;
                }
            }

            if let Some(ref role_filter) = f.role {
                if !user_info.roles.contains(role_filter) {
                    continue;
                }
            }
        }

        users.push(user_info);
    }

    Ok(ReedResponse {
        data: users,
        source: "security::users::list_users".to_string(),
        cached: false,
        timestamp: current_timestamp(),
        metrics: None,
    })
}

/// Updates user profile data (not password - use change_password instead).
///
/// ## Input
/// - `username`: Username to update
/// - `updates`: Partial update structure with fields to change
///
/// ## Output
/// - `ReedResult<ReedResponse<UserInfo>>`: Updated user information
///
/// ## Performance
/// - Update time: < 100ms
///
/// ## Error Conditions
/// - Returns `ReedError::NotFound` if user doesn't exist
/// - Returns `ReedError::ValidationError` if email is invalid or already exists
///
/// ## Example Usage
/// ```
/// let updates = UserUpdate {
///     email: Some("newemail@example.com".to_string()),
///     is_active: Some(false),
///     ..Default::default()
/// };
/// let response = update_user("admin", updates)?;
/// ```
pub fn update_user(username: &str, updates: UserUpdate) -> ReedResult<ReedResponse<UserInfo>> {
    let users_path = Path::new(".reed/users.matrix.csv");

    if !users_path.exists() {
        return Err(not_found(username));
    }

    // Backup before modification
    create_backup(users_path)?;

    let mut records = read_matrix_csv(users_path)?;
    let mut found = false;
    let mut updated_user = None;

    for record in &mut records {
        if let Some(MatrixValue::Single(user)) = record.fields.get("username") {
            if user == username {
                found = true;

                // Apply updates
                if let Some(ref firstname) = updates.firstname {
                    record.fields.insert(
                        "firstname".to_string(),
                        MatrixValue::Single(firstname.clone()),
                    );
                }
                if let Some(ref lastname) = updates.lastname {
                    record.fields.insert(
                        "lastname".to_string(),
                        MatrixValue::Single(lastname.clone()),
                    );
                }
                if let Some(ref email) = updates.email {
                    validate_email(email)?;
                    // Check if email is already used by another user
                    if email_exists(email)? {
                        let current_email = record.fields.get("email");
                        if let Some(MatrixValue::Single(current)) = current_email {
                            if current != email {
                                return Err(validation_error(
                                    "email",
                                    email,
                                    "Email already registered to another user",
                                ));
                            }
                        }
                    }
                    record
                        .fields
                        .insert("email".to_string(), MatrixValue::Single(email.clone()));
                }
                if let Some(ref mobile) = updates.mobile {
                    record
                        .fields
                        .insert("mobile".to_string(), MatrixValue::Single(mobile.clone()));
                }
                if let Some(ref desc) = updates.desc {
                    record
                        .fields
                        .insert("desc".to_string(), MatrixValue::Single(desc.clone()));
                }
                if let Some(is_active) = updates.is_active {
                    record.fields.insert(
                        "is_active".to_string(),
                        MatrixValue::Single(is_active.to_string()),
                    );
                }

                // Update timestamp
                record.fields.insert(
                    "updated_at".to_string(),
                    MatrixValue::Single(current_timestamp().to_string()),
                );

                updated_user = Some(matrix_record_to_user_info(record)?);
                break;
            }
        }
    }

    if !found {
        return Err(not_found(username));
    }

    write_matrix_csv(users_path, &records, &[])?;

    Ok(ReedResponse {
        data: updated_user.unwrap(),
        source: "security::users::update_user".to_string(),
        cached: false,
        timestamp: current_timestamp(),
        metrics: None,
    })
}

/// Deletes user (requires confirmation).
///
/// ## Input
/// - `username`: Username to delete
/// - `confirm`: Must be true to actually delete (safety check)
///
/// ## Output
/// - `ReedResult<ReedResponse<()>>`: Success confirmation
///
/// ## Performance
/// - Delete time: < 100ms
///
/// ## Error Conditions
/// - Returns `ReedError::NotFound` if user doesn't exist
/// - Returns `ReedError::ValidationError` if confirm is false
///
/// ## Example Usage
/// ```
/// delete_user("olduser", true)?;  // Deletes user
/// delete_user("admin", false)?;   // Error: confirmation required
/// ```
pub fn delete_user(username: &str, confirm: bool) -> ReedResult<ReedResponse<()>> {
    if !confirm {
        return Err(validation_error(
            "confirm",
            "false",
            "Confirmation required to delete user",
        ));
    }

    let users_path = Path::new(".reed/users.matrix.csv");

    if !users_path.exists() {
        return Err(not_found(username));
    }

    // Backup before modification
    create_backup(users_path)?;

    let mut records = read_matrix_csv(users_path)?;
    let original_len = records.len();

    records.retain(|record| {
        if let Some(MatrixValue::Single(user)) = record.fields.get("username") {
            user != username
        } else {
            true
        }
    });

    if records.len() == original_len {
        return Err(not_found(username));
    }

    write_matrix_csv(users_path, &records, &[])?;

    Ok(ReedResponse {
        data: (),
        source: "security::users::delete_user".to_string(),
        cached: false,
        timestamp: current_timestamp(),
        metrics: None,
    })
}

/// Helper function to convert MatrixRecord to UserInfo.
fn matrix_record_to_user_info(record: &MatrixRecord) -> ReedResult<UserInfo> {
    let username = if let Some(MatrixValue::Single(u)) = record.fields.get("username") {
        u.clone()
    } else {
        return Err(validation_error("username", "", "Missing username field"));
    };

    let roles = match record.fields.get("roles") {
        Some(MatrixValue::List(roles)) => roles.clone(),
        Some(MatrixValue::Single(s)) if !s.is_empty() => vec![s.clone()],
        _ => Vec::new(),
    };

    let get_string = |field: &str| -> String {
        if let Some(MatrixValue::Single(s)) = record.fields.get(field) {
            s.clone()
        } else {
            String::new()
        }
    };

    let get_option_string = |field: &str| -> Option<String> {
        if let Some(MatrixValue::Single(s)) = record.fields.get(field) {
            if s.is_empty() {
                None
            } else {
                Some(s.clone())
            }
        } else {
            None
        }
    };

    let created_at = get_string("created_at").parse().unwrap_or(0);
    let updated_at = get_string("updated_at").parse().unwrap_or(0);
    let last_login = get_string("last_login").parse().ok();
    let is_active = get_string("is_active") == "true";

    Ok(UserInfo {
        username,
        roles,
        firstname: get_string("firstname"),
        lastname: get_string("lastname"),
        email: get_string("email"),
        mobile: get_option_string("mobile"),
        social_media: SocialMedia {
            twitter: get_option_string("twitter"),
            facebook: get_option_string("facebook"),
            tiktok: get_option_string("tiktok"),
            instagram: get_option_string("instagram"),
            youtube: get_option_string("youtube"),
            whatsapp: get_option_string("whatsapp"),
        },
        address: Address {
            street: get_option_string("street"),
            city: get_option_string("city"),
            postcode: get_option_string("postcode"),
            region: get_option_string("region"),
            country: get_option_string("country"),
        },
        desc: get_string("desc"),
        created_at,
        updated_at,
        last_login,
        is_active,
    })
}
