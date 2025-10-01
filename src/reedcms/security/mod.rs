// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Security Module
//!
//! Provides user management, password hashing, and validation services.

pub mod passwords;
pub mod users;
pub mod validation;

pub use passwords::{change_password, hash_password, validate_password_strength, verify_password};
pub use users::{
    create_user, delete_user, get_user, list_users, update_user, Address, SocialMedia, UserInfo,
};
pub use validation::{email_exists, username_exists, validate_email, validate_username};

#[cfg(test)]
mod passwords_test;
#[cfg(test)]
mod users_test;
#[cfg(test)]
mod validation_test;
