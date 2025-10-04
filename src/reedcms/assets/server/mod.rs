// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Static asset server module with compression and caching.

pub mod compression;
pub mod precompress;
pub mod routes;
pub mod static_server;

// TODO: Create test files for REED-08-03
// #[cfg(test)]
// mod compression_test;
// #[cfg(test)]
// mod precompress_test;
// #[cfg(test)]
// mod static_server_test;

// Re-export main public API
pub use compression::{
    compress_brotli, compress_gzip, compress_with_method, get_compression_method, CompressionMethod,
};
pub use precompress::{
    clean_precompressed_assets, discover_compressible_assets, precompress_all_assets,
    precompress_asset,
};
pub use routes::{
    configure_static_routes, serve_css, serve_font, serve_image, serve_js, serve_source_map,
};
pub use static_server::{
    detect_mime_type, generate_etag, get_cache_control, serve_static_asset, validate_path,
};
