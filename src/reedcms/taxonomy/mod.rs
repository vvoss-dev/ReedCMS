// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Taxonomy Module
//!
//! Provides hierarchical taxonomy term management and entity tagging.

pub mod entities;
pub mod terms;

pub use entities::{
    assign_terms, get_entity_terms, list_entities_by_term, unassign_terms, EntityTerms,
};
pub use terms::{
    create_term, delete_term, get_term, list_terms, search_terms, update_term, TermInfo,
};
