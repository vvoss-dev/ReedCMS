// Copyright 2025 Vivian Voss. Licensed under the Apache Licence, Version 2.0.
// SPDX-Licence-Identifier: Apache-2.0

//! Taxonomy Module
//!
//! Provides hierarchical taxonomy term management and entity tagging.

pub mod entities;
pub mod hierarchy;
pub mod terms;

#[cfg(test)]
mod entities_test;
#[cfg(test)]
mod hierarchy_test;
#[cfg(test)]
mod terms_test;

pub use entities::{
    assign_terms, get_entity_terms, list_entities_by_term, unassign_terms, EntityTerms, EntityType,
};
pub use hierarchy::{
    get_ancestors, get_children, get_depth, get_path, get_tree, has_circular_reference, TermTree,
};
pub use terms::{
    create_term, delete_term, get_term, list_terms, search_terms, update_term, TermInfo, TermUpdate,
};
