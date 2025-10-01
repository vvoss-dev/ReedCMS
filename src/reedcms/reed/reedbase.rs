// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! ReedBase Dispatcher
//!
//! Intelligent data coordinator with O(1) HashMap caches and persistence rights.

use crate::reedcms::reedbase::{get as get_service, init as init_service, set as set_service};
use crate::reedcms::reedstream::{ReedRequest, ReedResponse, ReedResult};
use std::collections::HashMap;
use std::sync::RwLock;

/// ReedBase Dispatcher with thread-safe caches.
pub struct ReedBase {
    /// Text content cache (key → value)
    text_cache: RwLock<HashMap<String, String>>,

    /// Route cache (key → value)
    route_cache: RwLock<HashMap<String, String>>,

    /// Metadata cache (key → value)
    meta_cache: RwLock<HashMap<String, String>>,

    /// Path to text CSV file
    text_path: String,

    /// Path to route CSV file
    route_path: String,

    /// Path to meta CSV file
    meta_path: String,
}

impl ReedBase {
    /// Creates a new ReedBase dispatcher.
    ///
    /// ## Input
    /// - `text_path`: Path to text.csv file
    /// - `route_path`: Path to routes.csv file
    /// - `meta_path`: Path to meta.csv file
    ///
    /// ## Output
    /// - `ReedBase`: New instance with empty caches
    ///
    /// ## Example Usage
    /// ```
    /// let reedbase = ReedBase::new(
    ///     ".reed/text.csv",
    ///     ".reed/routes.csv",
    ///     ".reed/meta.csv",
    /// );
    /// ```
    pub fn new(text_path: &str, route_path: &str, meta_path: &str) -> Self {
        Self {
            text_cache: RwLock::new(HashMap::new()),
            route_cache: RwLock::new(HashMap::new()),
            meta_cache: RwLock::new(HashMap::new()),
            text_path: text_path.to_string(),
            route_path: route_path.to_string(),
            meta_path: meta_path.to_string(),
        }
    }

    /// Initialises all caches from CSV files.
    ///
    /// ## Output
    /// - `ReedResult<()>`: Success or error
    ///
    /// ## Performance
    /// - O(n) where n is total number of records
    /// - < 30ms for < 3000 total records
    ///
    /// ## Example Usage
    /// ```
    /// reedbase.init()?;
    /// ```
    pub fn init(&self) -> ReedResult<()> {
        // Init text cache
        let text_request = ReedRequest {
            key: String::new(),
            language: None,
            environment: None,
            context: Some("text".to_string()),
            value: Some(self.text_path.clone()),
            description: None,
        };
        let text_response = init_service(text_request)?;
        *self.text_cache.write().unwrap() = text_response.data;

        // Init route cache
        let route_request = ReedRequest {
            key: String::new(),
            language: None,
            environment: None,
            context: Some("route".to_string()),
            value: Some(self.route_path.clone()),
            description: None,
        };
        let route_response = init_service(route_request)?;
        *self.route_cache.write().unwrap() = route_response.data;

        // Init meta cache
        let meta_request = ReedRequest {
            key: String::new(),
            language: None,
            environment: None,
            context: Some("meta".to_string()),
            value: Some(self.meta_path.clone()),
            description: None,
        };
        let meta_response = init_service(meta_request)?;
        *self.meta_cache.write().unwrap() = meta_response.data;

        Ok(())
    }

    /// Retrieves a value by key from the appropriate cache.
    ///
    /// ## Input
    /// - `request`: ReedRequest with key and context (text/route/meta)
    ///
    /// ## Output
    /// - `ReedResult<ReedResponse<String>>`: Value or error
    ///
    /// ## Performance
    /// - O(1) HashMap lookup
    /// - < 100μs typical
    ///
    /// ## Example Usage
    /// ```
    /// let request = ReedRequest {
    ///     key: "page.title@en".to_string(),
    ///     language: None,
    ///     environment: Some("dev".to_string()),
    ///     context: Some("text".to_string()),
    ///     value: None,
    ///     description: None,
    /// };
    /// let response = reedbase.get(request)?;
    /// ```
    pub fn get(&self, request: ReedRequest) -> ReedResult<ReedResponse<String>> {
        let context = request.context.as_deref().unwrap_or("text");

        match context {
            "text" => {
                let cache = self.text_cache.read().unwrap();
                get_service(request, &cache)
            }
            "route" => {
                let cache = self.route_cache.read().unwrap();
                get_service(request, &cache)
            }
            "meta" => {
                let cache = self.meta_cache.read().unwrap();
                get_service(request, &cache)
            }
            _ => {
                // Default to text cache
                let cache = self.text_cache.read().unwrap();
                get_service(request, &cache)
            }
        }
    }

    /// Updates a key-value pair in the appropriate cache and persists to CSV.
    ///
    /// ## Input
    /// - `request`: ReedRequest with key, value, and context
    ///
    /// ## Output
    /// - `ReedResult<ReedResponse<String>>`: Confirmation
    ///
    /// ## Performance
    /// - O(1) cache update
    /// - O(n) CSV write where n is cache size
    /// - < 10ms for < 1000 records
    ///
    /// ## Example Usage
    /// ```
    /// let request = ReedRequest {
    ///     key: "page.title@en".to_string(),
    ///     language: None,
    ///     environment: None,
    ///     context: Some("text".to_string()),
    ///     value: Some("New Title".to_string()),
    ///     description: Some("Homepage title".to_string()),
    /// };
    /// reedbase.set(request)?;
    /// ```
    pub fn set(&self, request: ReedRequest) -> ReedResult<ReedResponse<String>> {
        let context = request.context.as_deref().unwrap_or("text");

        match context {
            "text" => {
                let mut cache = self.text_cache.write().unwrap();
                set_service(request, &mut cache, &self.text_path)
            }
            "route" => {
                let mut cache = self.route_cache.write().unwrap();
                set_service(request, &mut cache, &self.route_path)
            }
            "meta" => {
                let mut cache = self.meta_cache.write().unwrap();
                set_service(request, &mut cache, &self.meta_path)
            }
            _ => {
                // Default to text cache
                let mut cache = self.text_cache.write().unwrap();
                set_service(request, &mut cache, &self.text_path)
            }
        }
    }
}
