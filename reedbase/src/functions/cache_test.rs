// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Tests for function memoization cache.

#[cfg(test)]
mod tests {
    use crate::functions::cache::{CacheKey, FunctionCache};

    #[test]
    fn test_cache_key_creation() {
        let key = CacheKey::new("test_func", vec!["arg1", "arg2"]);
        assert_eq!(key.function, "test_func");
        assert_eq!(key.args, vec!["arg1", "arg2"]);
    }

    #[test]
    fn test_cache_hit() {
        let cache = FunctionCache::new();

        let key = CacheKey::new("test", vec!["arg1"]);
        cache.insert(key.clone(), "result".to_string());

        let cached = cache.get(&key);
        assert_eq!(cached, Some("result".to_string()));

        let stats = cache.stats();
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 0);
        assert_eq!(stats.inserts, 1);
    }

    #[test]
    fn test_cache_miss() {
        let cache = FunctionCache::new();

        let key = CacheKey::new("test", vec!["arg1"]);
        let cached = cache.get(&key);

        assert_eq!(cached, None);

        let stats = cache.stats();
        assert_eq!(stats.hits, 0);
        assert_eq!(stats.misses, 1);
    }

    #[test]
    fn test_cache_multiple_hits() {
        let cache = FunctionCache::new();

        let key = CacheKey::new("test", vec!["arg1"]);
        cache.insert(key.clone(), "result".to_string());

        // Multiple retrievals
        cache.get(&key);
        cache.get(&key);
        cache.get(&key);

        let stats = cache.stats();
        assert_eq!(stats.hits, 3);
        assert_eq!(stats.misses, 0);
    }

    #[test]
    fn test_cache_different_keys() {
        let cache = FunctionCache::new();

        let key1 = CacheKey::new("func1", vec!["arg1"]);
        let key2 = CacheKey::new("func2", vec!["arg1"]);
        let key3 = CacheKey::new("func1", vec!["arg2"]);

        cache.insert(key1.clone(), "result1".to_string());
        cache.insert(key2.clone(), "result2".to_string());
        cache.insert(key3.clone(), "result3".to_string());

        assert_eq!(cache.get(&key1), Some("result1".to_string()));
        assert_eq!(cache.get(&key2), Some("result2".to_string()));
        assert_eq!(cache.get(&key3), Some("result3".to_string()));

        assert_eq!(cache.entry_count(), 3);
    }

    #[test]
    fn test_cache_clear() {
        let cache = FunctionCache::new();

        let key1 = CacheKey::new("test", vec!["arg1"]);
        let key2 = CacheKey::new("test", vec!["arg2"]);

        cache.insert(key1.clone(), "result1".to_string());
        cache.insert(key2.clone(), "result2".to_string());

        assert_eq!(cache.entry_count(), 2);

        cache.clear();

        assert_eq!(cache.entry_count(), 0);
        assert_eq!(cache.get(&key1), None);
        assert_eq!(cache.get(&key2), None);
    }

    #[test]
    fn test_cache_clear_function() {
        let cache = FunctionCache::new();

        let key1 = CacheKey::new("func1", vec!["arg1"]);
        let key2 = CacheKey::new("func2", vec!["arg1"]);
        let key3 = CacheKey::new("func1", vec!["arg2"]);

        cache.insert(key1.clone(), "result1".to_string());
        cache.insert(key2.clone(), "result2".to_string());
        cache.insert(key3.clone(), "result3".to_string());

        cache.clear_function("func1");

        assert_eq!(cache.get(&key1), None);
        assert_eq!(cache.get(&key2), Some("result2".to_string()));
        assert_eq!(cache.get(&key3), None);
    }

    #[test]
    fn test_cache_invalidate_table() {
        let cache = FunctionCache::new();

        // Aggregation functions typically have table as first arg
        let key1 = CacheKey::new("count", vec!["users"]);
        let key2 = CacheKey::new("sum", vec!["users", "age"]);
        let key3 = CacheKey::new("count", vec!["posts"]);

        cache.insert(key1.clone(), "100".to_string());
        cache.insert(key2.clone(), "3500".to_string());
        cache.insert(key3.clone(), "50".to_string());

        // Invalidate "users" table
        cache.invalidate_table("users");

        assert_eq!(cache.get(&key1), None);
        assert_eq!(cache.get(&key2), None);
        assert_eq!(cache.get(&key3), Some("50".to_string()));
    }

    #[test]
    fn test_cache_stats_hit_rate() {
        let cache = FunctionCache::new();

        let key = CacheKey::new("test", vec!["arg1"]);
        cache.insert(key.clone(), "result".to_string());

        // 8 hits
        for _ in 0..8 {
            cache.get(&key);
        }

        // 2 misses
        let key2 = CacheKey::new("test", vec!["arg2"]);
        cache.get(&key2);
        cache.get(&key2);

        let stats = cache.stats();
        assert_eq!(stats.hits, 8);
        assert_eq!(stats.misses, 2);
        assert_eq!(stats.hit_rate(), 80.0); // 8 / 10 = 80%
    }

    #[test]
    fn test_cache_stats_zero_requests() {
        let cache = FunctionCache::new();
        let stats = cache.stats();

        assert_eq!(stats.hit_rate(), 0.0);
    }

    #[test]
    fn test_cache_memory_usage() {
        let cache = FunctionCache::new();

        // Empty cache
        assert_eq!(cache.memory_usage(), 0);

        // Add entries
        cache.insert(CacheKey::new("test", vec!["arg1"]), "result1".to_string());
        cache.insert(CacheKey::new("test", vec!["arg2"]), "result2".to_string());

        // Should have some memory usage
        assert!(cache.memory_usage() > 0);
    }

    #[test]
    fn test_cache_entry_count() {
        let cache = FunctionCache::new();

        assert_eq!(cache.entry_count(), 0);

        cache.insert(CacheKey::new("test", vec!["arg1"]), "result".to_string());
        assert_eq!(cache.entry_count(), 1);

        cache.insert(CacheKey::new("test", vec!["arg2"]), "result".to_string());
        assert_eq!(cache.entry_count(), 2);

        cache.clear();
        assert_eq!(cache.entry_count(), 0);
    }

    #[test]
    fn test_cache_overwrite_same_key() {
        let cache = FunctionCache::new();

        let key = CacheKey::new("test", vec!["arg1"]);

        cache.insert(key.clone(), "result1".to_string());
        assert_eq!(cache.get(&key), Some("result1".to_string()));

        cache.insert(key.clone(), "result2".to_string());
        assert_eq!(cache.get(&key), Some("result2".to_string()));

        // Should still be 1 entry (overwritten)
        assert_eq!(cache.entry_count(), 1);
    }
}
