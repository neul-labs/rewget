//! Domain stage cache for rewget
//!
//! Remembers which stage worked for each domain to skip failed stages
//! on subsequent requests. Cache entries expire after 7 days.

use crate::{FetchStage, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

/// Default cache expiry in seconds (7 days)
const CACHE_EXPIRY_SECS: u64 = 7 * 24 * 60 * 60;

/// Domain stage cache
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct DomainCache {
    /// Map of domain -> cache entry
    entries: HashMap<String, CacheEntry>,
}

/// A cached stage result for a domain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    /// The stage that worked
    pub stage: FetchStage,

    /// Unix timestamp when this entry was created
    pub timestamp: u64,
}

impl DomainCache {
    /// Load cache from disk, or return empty cache if not found
    pub fn load() -> Self {
        let path = Self::cache_path();
        if !path.exists() {
            return Self::default();
        }

        match fs::read_to_string(&path) {
            Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    /// Save cache to disk
    pub fn save(&self) -> Result<()> {
        let path = Self::cache_path();

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = serde_json::to_string_pretty(self)?;
        fs::write(&path, content)?;
        Ok(())
    }

    /// Get the cached stage for a domain, if not expired
    pub fn get(&self, domain: &str) -> Option<FetchStage> {
        let entry = self.entries.get(domain)?;

        // Check expiry
        let now = current_timestamp();
        if now.saturating_sub(entry.timestamp) > CACHE_EXPIRY_SECS {
            return None;
        }

        Some(entry.stage)
    }

    /// Set the working stage for a domain
    pub fn set(&mut self, domain: &str, stage: FetchStage) {
        self.entries.insert(
            domain.to_string(),
            CacheEntry {
                stage,
                timestamp: current_timestamp(),
            },
        );
    }

    /// Remove expired entries from the cache
    pub fn prune_expired(&mut self) {
        let now = current_timestamp();
        self.entries
            .retain(|_, entry| now.saturating_sub(entry.timestamp) <= CACHE_EXPIRY_SECS);
    }

    /// Clear all cache entries
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// Get the number of entries in the cache
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if cache is empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Get cache file path
    fn cache_path() -> PathBuf {
        dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from(".cache"))
            .join("rewget")
            .join("domain-cache.json")
    }

    /// Get cache directory path (for clearing)
    pub fn cache_dir() -> PathBuf {
        dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from(".cache"))
            .join("rewget")
    }
}

/// Extract domain from a URL
pub fn extract_domain(url: &str) -> Option<String> {
    // Handle URLs with or without scheme
    let url = if url.contains("://") {
        url.to_string()
    } else {
        format!("https://{}", url)
    };

    // Parse the URL to extract host
    let without_scheme = url
        .split("://")
        .nth(1)
        .unwrap_or(&url);

    // Get the host part (before any path, query, or port)
    let host = without_scheme
        .split('/')
        .next()?
        .split(':')
        .next()?
        .split('?')
        .next()?;

    if host.is_empty() {
        return None;
    }

    Some(host.to_lowercase())
}

/// Get current Unix timestamp
fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_domain_with_scheme() {
        assert_eq!(
            extract_domain("https://example.com/path"),
            Some("example.com".to_string())
        );
    }

    #[test]
    fn test_extract_domain_without_scheme() {
        assert_eq!(
            extract_domain("example.com/path"),
            Some("example.com".to_string())
        );
    }

    #[test]
    fn test_extract_domain_with_port() {
        assert_eq!(
            extract_domain("https://example.com:8080/path"),
            Some("example.com".to_string())
        );
    }

    #[test]
    fn test_extract_domain_subdomain() {
        assert_eq!(
            extract_domain("https://www.example.com/path"),
            Some("www.example.com".to_string())
        );
    }

    #[test]
    fn test_extract_domain_case_insensitive() {
        assert_eq!(
            extract_domain("https://EXAMPLE.COM/path"),
            Some("example.com".to_string())
        );
    }

    #[test]
    fn test_cache_set_get() {
        let mut cache = DomainCache::default();
        cache.set("example.com", FetchStage::Impersonate);

        assert_eq!(cache.get("example.com"), Some(FetchStage::Impersonate));
        assert_eq!(cache.get("other.com"), None);
    }

    #[test]
    fn test_cache_clear() {
        let mut cache = DomainCache::default();
        cache.set("example.com", FetchStage::Impersonate);
        cache.set("other.com", FetchStage::Preflight);

        assert_eq!(cache.len(), 2);
        cache.clear();
        assert!(cache.is_empty());
    }

    #[test]
    fn test_cache_expired_entry() {
        let mut cache = DomainCache::default();

        // Insert an entry with an old timestamp
        cache.entries.insert(
            "old.com".to_string(),
            CacheEntry {
                stage: FetchStage::Impersonate,
                timestamp: 0, // Very old
            },
        );

        // Should return None due to expiry
        assert_eq!(cache.get("old.com"), None);
    }

    #[test]
    fn test_cache_prune_expired() {
        let mut cache = DomainCache::default();

        // Add a fresh entry
        cache.set("fresh.com", FetchStage::Impersonate);

        // Add an expired entry
        cache.entries.insert(
            "old.com".to_string(),
            CacheEntry {
                stage: FetchStage::Impersonate,
                timestamp: 0,
            },
        );

        assert_eq!(cache.len(), 2);
        cache.prune_expired();
        assert_eq!(cache.len(), 1);
        assert!(cache.get("fresh.com").is_some());
    }
}
