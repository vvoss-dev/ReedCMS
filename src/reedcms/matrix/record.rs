// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Matrix CSV Record Types
//!
//! Provides MatrixRecord and MatrixValue types for structured CSV data.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Matrix CSV value types.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MatrixValue {
    /// Type 1: Simple string value
    Single(String),

    /// Type 2: Comma-separated list
    List(Vec<String>),

    /// Type 3: Value with modifiers (value, [modifiers])
    Modified(String, Vec<String>),

    /// Type 4: List with modifiers [(value, [modifiers])]
    ModifiedList(Vec<(String, Vec<String>)>),
}

impl MatrixValue {
    /// Returns true if this is a Single value.
    pub fn is_single(&self) -> bool {
        matches!(self, MatrixValue::Single(_))
    }

    /// Returns true if this is a List value.
    pub fn is_list(&self) -> bool {
        matches!(self, MatrixValue::List(_))
    }

    /// Returns true if this is a Modified value.
    pub fn is_modified(&self) -> bool {
        matches!(self, MatrixValue::Modified(_, _))
    }

    /// Returns true if this is a ModifiedList value.
    pub fn is_modified_list(&self) -> bool {
        matches!(self, MatrixValue::ModifiedList(_))
    }

    /// Converts to string representation for CSV output.
    pub fn to_csv_string(&self) -> String {
        match self {
            MatrixValue::Single(s) => s.clone(),
            MatrixValue::List(list) => list.join(","),
            MatrixValue::Modified(val, mods) => {
                if mods.is_empty() {
                    val.clone()
                } else {
                    format!("{}[{}]", val, mods.join(","))
                }
            }
            MatrixValue::ModifiedList(items) => items
                .iter()
                .map(|(val, mods)| {
                    if mods.is_empty() {
                        val.clone()
                    } else {
                        format!("{}[{}]", val, mods.join(","))
                    }
                })
                .collect::<Vec<_>>()
                .join(","),
        }
    }
}

/// Matrix CSV record with variable fields.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MatrixRecord {
    /// Field name → value mapping
    pub fields: HashMap<String, MatrixValue>,

    /// Field order (for deterministic CSV output)
    pub field_order: Vec<String>,

    /// Optional description
    pub description: Option<String>,
}

impl MatrixRecord {
    /// Creates a new empty MatrixRecord.
    pub fn new() -> Self {
        Self {
            fields: HashMap::new(),
            field_order: Vec::new(),
            description: None,
        }
    }

    /// Adds a field to the record.
    ///
    /// ## Arguments
    /// - `name`: Field name
    /// - `value`: MatrixValue
    ///
    /// ## Behaviour
    /// - Adds field to HashMap
    /// - Appends to field_order if not already present
    pub fn add_field(&mut self, name: String, value: MatrixValue) {
        if !self.field_order.contains(&name) {
            self.field_order.push(name.clone());
        }
        self.fields.insert(name, value);
    }

    /// Gets a field value by name.
    pub fn get_field(&self, name: &str) -> Option<&MatrixValue> {
        self.fields.get(name)
    }

    /// Sets the description.
    pub fn set_description(&mut self, desc: String) {
        self.description = Some(desc);
    }

    /// Converts to CSV row string.
    ///
    /// ## Output
    /// Pipe-delimited string following field_order
    pub fn to_csv_row(&self) -> String {
        let mut parts: Vec<String> = Vec::new();

        for field_name in &self.field_order {
            if let Some(value) = self.fields.get(field_name) {
                parts.push(value.to_csv_string());
            } else {
                parts.push(String::new());
            }
        }

        if let Some(desc) = &self.description {
            parts.push(desc.clone());
        }

        parts.join("|")
    }
}

impl Default for MatrixRecord {
    fn default() -> Self {
        Self::new()
    }
}
