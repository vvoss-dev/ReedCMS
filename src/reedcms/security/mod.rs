// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Security Module
//!
//! Provides user management, password hashing, and validation services.

pub mod inheritance;
pub mod passwords;
pub mod permissions;
pub mod roles;
pub mod users;
pub mod validation;

pub use inheritance::{
    has_circular_inheritance, merge_inherited_permissions, resolve_inheritance,
    resolve_role_permissions,
};
pub use passwords::{change_password, hash_password, validate_password_strength, verify_password};
pub use permissions::{
    format_permission, format_permissions, parse_permission, parse_permissions,
    validate_permission_syntax, Permission,
};
pub use roles::{
    create_role, delete_role, get_role, list_roles, update_role, RoleInfo, RoleUpdate,
};
pub use users::{
    create_user, delete_user, get_user, list_users, update_user, Address, SocialMedia, UserInfo,
};
pub use validation::{email_exists, username_exists, validate_email, validate_username};

#[cfg(test)]
mod inheritance_test;
#[cfg(test)]
mod passwords_test;
#[cfg(test)]
mod permissions_test;
#[cfg(test)]
mod roles_test;
#[cfg(test)]
mod users_test;
#[cfg(test)]
mod validation_test;
