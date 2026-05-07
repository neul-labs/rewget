//! Browser profile management for rewget
//!
//! Profiles define browser fingerprints (TLS, HTTP/2, headers) for impersonation.
//! Supports remote profile updates with Ed25519 signature verification.

use crate::Result;
use base64::Engine;
use ed25519_dalek::{Signature, VerifyingKey, Verifier};
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Default profile update URL
pub const DEFAULT_PROFILE_URL: &str = "https://rewget.dev/profiles/v1/index.json";

/// Ed25519 public key for profile verification (base64 encoded)
/// This key is used to verify that profiles come from a trusted source.
pub const PROFILE_PUBLIC_KEY: &str = "MCowBQYDK2VwAyEAZXhhbXBsZS1wdWJsaWMta2V5LWZvci1yd2dldC0=";

/// Profile update result
#[derive(Debug)]
pub struct ProfileUpdateResult {
    pub updated: Vec<String>,
    pub added: Vec<String>,
    pub unchanged: usize,
    pub total: usize,
}

/// Browser profile for impersonation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    /// Profile name (e.g., "chrome131", "firefox133")
    pub name: String,

    /// Human-readable description
    pub description: String,

    /// Browser and version this profile emulates
    pub browser: BrowserInfo,

    /// TLS fingerprint settings
    pub tls: TlsSettings,

    /// HTTP/2 fingerprint settings
    pub http2: Http2Settings,

    /// Default headers to send
    pub headers: HashMap<String, String>,

    /// Profile version (for updates)
    pub version: u32,

    /// Timestamp when profile was created/updated
    #[serde(default)]
    pub updated_at: Option<String>,
}

/// Browser identification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserInfo {
    /// Browser name (Chrome, Firefox, Safari, Edge)
    pub name: String,

    /// Browser version
    pub version: String,

    /// Platform (Windows, macOS, Linux)
    pub platform: String,

    /// User-Agent string
    pub user_agent: String,
}

/// TLS fingerprint settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsSettings {
    /// TLS versions to advertise
    pub versions: Vec<String>,

    /// Cipher suites in order
    pub cipher_suites: Vec<String>,

    /// TLS extensions
    pub extensions: Vec<String>,

    /// Elliptic curves
    pub curves: Vec<String>,

    /// Signature algorithms
    pub signature_algorithms: Vec<String>,

    /// ALPN protocols
    pub alpn: Vec<String>,

    /// Use GREASE values
    #[serde(default = "default_true")]
    pub grease: bool,
}

/// HTTP/2 fingerprint settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Http2Settings {
    /// SETTINGS frame values
    pub settings: HashMap<String, u32>,

    /// Window update value
    pub window_update: u32,

    /// Pseudo-header order
    pub pseudo_header_order: Vec<String>,

    /// Priority frames
    #[serde(default)]
    pub priority: Option<PrioritySettings>,
}

/// HTTP/2 priority settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrioritySettings {
    pub weight: u8,
    pub exclusive: bool,
    pub depends_on: u32,
}

fn default_true() -> bool {
    true
}

/// Profile collection
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProfileCollection {
    /// Profile format version
    pub format_version: u32,

    /// Available profiles
    pub profiles: Vec<Profile>,

    /// Ed25519 signature (base64)
    #[serde(default)]
    pub signature: Option<String>,
}

impl ProfileCollection {
    /// Get profiles directory path
    pub fn profiles_dir() -> PathBuf {
        dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from(".local/share"))
            .join("rewget")
            .join("profiles")
    }

    /// Get path to built-in profiles
    pub fn builtin_path() -> PathBuf {
        Self::profiles_dir().join("builtin.json")
    }

    /// Get path to custom profiles
    pub fn custom_path() -> PathBuf {
        Self::profiles_dir().join("custom.json")
    }

    /// Load profiles from disk
    pub fn load() -> Self {
        let builtin = Self::load_file(&Self::builtin_path()).unwrap_or_default();
        let custom = Self::load_file(&Self::custom_path()).unwrap_or_default();

        // Merge custom over builtin
        let mut profiles = builtin.profiles;
        for custom_profile in custom.profiles {
            // Replace or add
            if let Some(pos) = profiles.iter().position(|p| p.name == custom_profile.name) {
                profiles[pos] = custom_profile;
            } else {
                profiles.push(custom_profile);
            }
        }

        Self {
            format_version: 1,
            profiles,
            signature: None,
        }
    }

    /// Load from a specific file
    fn load_file(path: &PathBuf) -> Option<Self> {
        let content = fs::read_to_string(path).ok()?;
        serde_json::from_str(&content).ok()
    }

    /// Save built-in profiles
    pub fn save_builtin(&self) -> Result<()> {
        let path = Self::builtin_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)?;
        fs::write(&path, content)?;
        Ok(())
    }

    /// Get profile by name
    pub fn get(&self, name: &str) -> Option<&Profile> {
        self.profiles.iter().find(|p| p.name == name)
    }

    /// List all profile names
    pub fn list_names(&self) -> Vec<&str> {
        self.profiles.iter().map(|p| p.name.as_str()).collect()
    }

    /// Get default profiles (built-in)
    pub fn default_profiles() -> Self {
        Self {
            format_version: 1,
            profiles: vec![
                chrome131_profile(),
                chrome130_profile(),
                firefox136_profile(),
                firefox133_profile(),
                safari18_profile(),
                edge131_profile(),
            ],
            signature: None,
        }
    }

    /// Initialize profiles directory with defaults if empty
    pub fn init_defaults() -> Result<()> {
        let path = Self::builtin_path();
        if !path.exists() {
            let defaults = Self::default_profiles();
            defaults.save_builtin()?;
        }
        Ok(())
    }

    /// Fetch profiles from a remote URL
    pub fn fetch_remote(url: &str) -> Result<Self> {
        let response = ureq::get(url)
            .timeout(std::time::Duration::from_secs(30))
            .call()
            .map_err(|e| crate::Error::Config(format!("Failed to fetch profiles: {}", e)))?;

        let body = response.into_string()
            .map_err(|e| crate::Error::Config(format!("Failed to read response: {}", e)))?;

        let collection: Self = serde_json::from_str(&body)?;
        Ok(collection)
    }

    /// Verify the signature of this profile collection
    pub fn verify_signature(&self, public_key_b64: &str) -> Result<bool> {
        let signature_b64 = match &self.signature {
            Some(sig) => sig,
            None => return Ok(false), // No signature to verify
        };

        // Decode public key
        let public_key_bytes = base64::engine::general_purpose::STANDARD
            .decode(public_key_b64)
            .map_err(|e| crate::Error::Config(format!("Invalid public key: {}", e)))?;

        // Handle SPKI format (skip header if present) or raw 32-byte key
        let key_bytes: [u8; 32] = if public_key_bytes.len() == 44 {
            // SPKI format: skip 12-byte header
            public_key_bytes[12..44].try_into()
                .map_err(|_| crate::Error::Config("Invalid public key length".to_string()))?
        } else if public_key_bytes.len() == 32 {
            public_key_bytes.try_into()
                .map_err(|_| crate::Error::Config("Invalid public key length".to_string()))?
        } else {
            return Err(crate::Error::Config(format!(
                "Invalid public key length: {} (expected 32 or 44)",
                public_key_bytes.len()
            )));
        };

        let verifying_key = VerifyingKey::from_bytes(&key_bytes)
            .map_err(|e| crate::Error::Config(format!("Invalid public key: {}", e)))?;

        // Decode signature
        let signature_bytes = base64::engine::general_purpose::STANDARD
            .decode(signature_b64)
            .map_err(|e| crate::Error::Config(format!("Invalid signature: {}", e)))?;

        let signature_arr: [u8; 64] = signature_bytes.try_into()
            .map_err(|_| crate::Error::Config("Invalid signature length".to_string()))?;

        let signature = Signature::from_bytes(&signature_arr);

        // Create message to verify (hash of profiles JSON without signature)
        let mut collection_for_hash = self.clone();
        collection_for_hash.signature = None;
        let message = serde_json::to_string(&collection_for_hash)?;

        // Hash the message
        let mut hasher = Sha256::new();
        hasher.update(message.as_bytes());
        let hash = hasher.finalize();

        // Verify
        match verifying_key.verify(&hash, &signature) {
            Ok(()) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    /// Update profiles from a remote source
    ///
    /// Fetches profiles from the URL, verifies the signature, and merges
    /// with existing profiles.
    pub fn update_from_remote(url: Option<&str>, verify: bool) -> Result<ProfileUpdateResult> {
        let url = url.unwrap_or(DEFAULT_PROFILE_URL);

        // Fetch remote profiles
        let remote = Self::fetch_remote(url)?;

        // Verify signature if required
        if verify
            && !remote.verify_signature(PROFILE_PUBLIC_KEY)? {
                return Err(crate::Error::Config(
                    "Profile signature verification failed. Use --rewget-no-verify to skip.".to_string()
                ));
            }

        // Load existing profiles
        let existing = Self::load();

        // Track changes
        let mut updated = Vec::new();
        let mut added = Vec::new();
        let mut unchanged = 0;

        // Merge profiles
        let mut merged_profiles = existing.profiles.clone();

        for remote_profile in &remote.profiles {
            if let Some(pos) = merged_profiles.iter().position(|p| p.name == remote_profile.name) {
                // Existing profile - check if newer
                if remote_profile.version > merged_profiles[pos].version {
                    merged_profiles[pos] = remote_profile.clone();
                    updated.push(remote_profile.name.clone());
                } else {
                    unchanged += 1;
                }
            } else {
                // New profile
                merged_profiles.push(remote_profile.clone());
                added.push(remote_profile.name.clone());
            }
        }

        let total = merged_profiles.len();

        // Save merged profiles
        let merged = Self {
            format_version: remote.format_version.max(existing.format_version),
            profiles: merged_profiles,
            signature: None, // Don't save remote signature locally
        };
        merged.save_builtin()?;

        Ok(ProfileUpdateResult {
            updated,
            added,
            unchanged,
            total,
        })
    }

    /// Compute SHA256 hash of profile collection (for signing)
    pub fn compute_hash(&self) -> String {
        let mut collection_for_hash = self.clone();
        collection_for_hash.signature = None;
        let message = serde_json::to_string(&collection_for_hash).unwrap_or_default();

        let mut hasher = Sha256::new();
        hasher.update(message.as_bytes());
        let hash = hasher.finalize();

        base64::engine::general_purpose::STANDARD.encode(hash)
    }
}

/// Chrome 131 profile
fn chrome131_profile() -> Profile {
    Profile {
        name: "chrome131".to_string(),
        description: "Chrome 131 on Windows 10".to_string(),
        browser: BrowserInfo {
            name: "Chrome".to_string(),
            version: "131.0.0.0".to_string(),
            platform: "Windows".to_string(),
            user_agent: "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36".to_string(),
        },
        tls: TlsSettings {
            versions: vec!["TLS1.2".to_string(), "TLS1.3".to_string()],
            cipher_suites: vec![
                "TLS_AES_128_GCM_SHA256".to_string(),
                "TLS_AES_256_GCM_SHA384".to_string(),
                "TLS_CHACHA20_POLY1305_SHA256".to_string(),
                "TLS_ECDHE_ECDSA_WITH_AES_128_GCM_SHA256".to_string(),
                "TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256".to_string(),
                "TLS_ECDHE_ECDSA_WITH_AES_256_GCM_SHA384".to_string(),
                "TLS_ECDHE_RSA_WITH_AES_256_GCM_SHA384".to_string(),
                "TLS_ECDHE_ECDSA_WITH_CHACHA20_POLY1305_SHA256".to_string(),
                "TLS_ECDHE_RSA_WITH_CHACHA20_POLY1305_SHA256".to_string(),
            ],
            extensions: vec![
                "server_name".to_string(),
                "extended_master_secret".to_string(),
                "renegotiation_info".to_string(),
                "supported_groups".to_string(),
                "ec_point_formats".to_string(),
                "session_ticket".to_string(),
                "application_layer_protocol_negotiation".to_string(),
                "status_request".to_string(),
                "signature_algorithms".to_string(),
                "signed_certificate_timestamp".to_string(),
                "key_share".to_string(),
                "psk_key_exchange_modes".to_string(),
                "supported_versions".to_string(),
                "compress_certificate".to_string(),
                "application_settings".to_string(),
            ],
            curves: vec![
                "X25519".to_string(),
                "P-256".to_string(),
                "P-384".to_string(),
            ],
            signature_algorithms: vec![
                "ecdsa_secp256r1_sha256".to_string(),
                "rsa_pss_rsae_sha256".to_string(),
                "rsa_pkcs1_sha256".to_string(),
                "ecdsa_secp384r1_sha384".to_string(),
                "rsa_pss_rsae_sha384".to_string(),
                "rsa_pkcs1_sha384".to_string(),
                "rsa_pss_rsae_sha512".to_string(),
                "rsa_pkcs1_sha512".to_string(),
            ],
            alpn: vec!["h2".to_string(), "http/1.1".to_string()],
            grease: true,
        },
        http2: Http2Settings {
            settings: [
                ("HEADER_TABLE_SIZE".to_string(), 65536),
                ("ENABLE_PUSH".to_string(), 0),
                ("MAX_CONCURRENT_STREAMS".to_string(), 1000),
                ("INITIAL_WINDOW_SIZE".to_string(), 6291456),
                ("MAX_HEADER_LIST_SIZE".to_string(), 262144),
            ].into_iter().collect(),
            window_update: 15663105,
            pseudo_header_order: vec![
                ":method".to_string(),
                ":authority".to_string(),
                ":scheme".to_string(),
                ":path".to_string(),
            ],
            priority: Some(PrioritySettings {
                weight: 255,
                exclusive: true,
                depends_on: 0,
            }),
        },
        headers: [
            ("sec-ch-ua".to_string(), r#""Google Chrome";v="131", "Chromium";v="131", "Not_A Brand";v="24""#.to_string()),
            ("sec-ch-ua-mobile".to_string(), "?0".to_string()),
            ("sec-ch-ua-platform".to_string(), r#""Windows""#.to_string()),
            ("upgrade-insecure-requests".to_string(), "1".to_string()),
            ("accept".to_string(), "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8".to_string()),
            ("sec-fetch-site".to_string(), "none".to_string()),
            ("sec-fetch-mode".to_string(), "navigate".to_string()),
            ("sec-fetch-user".to_string(), "?1".to_string()),
            ("sec-fetch-dest".to_string(), "document".to_string()),
            ("accept-encoding".to_string(), "gzip, deflate, br, zstd".to_string()),
            ("accept-language".to_string(), "en-US,en;q=0.9".to_string()),
        ].into_iter().collect(),
        version: 1,
        updated_at: Some("2024-12-01".to_string()),
    }
}

/// Chrome 130 profile
fn chrome130_profile() -> Profile {
    let mut profile = chrome131_profile();
    profile.name = "chrome130".to_string();
    profile.description = "Chrome 130 on Windows 10".to_string();
    profile.browser.version = "130.0.0.0".to_string();
    profile.browser.user_agent = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0.0.0 Safari/537.36".to_string();
    profile.headers.insert("sec-ch-ua".to_string(), r#""Google Chrome";v="130", "Chromium";v="130", "Not_A Brand";v="24""#.to_string());
    profile
}

/// Firefox 136 profile
fn firefox136_profile() -> Profile {
    Profile {
        name: "firefox136".to_string(),
        description: "Firefox 136 on Windows 10".to_string(),
        browser: BrowserInfo {
            name: "Firefox".to_string(),
            version: "136.0".to_string(),
            platform: "Windows".to_string(),
            user_agent: "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:136.0) Gecko/20100101 Firefox/136.0".to_string(),
        },
        tls: TlsSettings {
            versions: vec!["TLS1.2".to_string(), "TLS1.3".to_string()],
            cipher_suites: vec![
                "TLS_AES_128_GCM_SHA256".to_string(),
                "TLS_CHACHA20_POLY1305_SHA256".to_string(),
                "TLS_AES_256_GCM_SHA384".to_string(),
                "TLS_ECDHE_ECDSA_WITH_AES_128_GCM_SHA256".to_string(),
                "TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256".to_string(),
                "TLS_ECDHE_ECDSA_WITH_CHACHA20_POLY1305_SHA256".to_string(),
                "TLS_ECDHE_RSA_WITH_CHACHA20_POLY1305_SHA256".to_string(),
                "TLS_ECDHE_ECDSA_WITH_AES_256_GCM_SHA384".to_string(),
                "TLS_ECDHE_RSA_WITH_AES_256_GCM_SHA384".to_string(),
            ],
            extensions: vec![
                "server_name".to_string(),
                "extended_master_secret".to_string(),
                "renegotiation_info".to_string(),
                "supported_groups".to_string(),
                "ec_point_formats".to_string(),
                "session_ticket".to_string(),
                "application_layer_protocol_negotiation".to_string(),
                "status_request".to_string(),
                "delegated_credentials".to_string(),
                "key_share".to_string(),
                "supported_versions".to_string(),
                "signature_algorithms".to_string(),
                "psk_key_exchange_modes".to_string(),
                "record_size_limit".to_string(),
            ],
            curves: vec![
                "X25519".to_string(),
                "P-256".to_string(),
                "P-384".to_string(),
                "P-521".to_string(),
            ],
            signature_algorithms: vec![
                "ecdsa_secp256r1_sha256".to_string(),
                "ecdsa_secp384r1_sha384".to_string(),
                "ecdsa_secp521r1_sha512".to_string(),
                "rsa_pss_rsae_sha256".to_string(),
                "rsa_pss_rsae_sha384".to_string(),
                "rsa_pss_rsae_sha512".to_string(),
                "rsa_pkcs1_sha256".to_string(),
                "rsa_pkcs1_sha384".to_string(),
                "rsa_pkcs1_sha512".to_string(),
            ],
            alpn: vec!["h2".to_string(), "http/1.1".to_string()],
            grease: false,
        },
        http2: Http2Settings {
            settings: [
                ("HEADER_TABLE_SIZE".to_string(), 65536),
                ("INITIAL_WINDOW_SIZE".to_string(), 131072),
                ("MAX_FRAME_SIZE".to_string(), 16384),
            ].into_iter().collect(),
            window_update: 12517377,
            pseudo_header_order: vec![
                ":method".to_string(),
                ":path".to_string(),
                ":authority".to_string(),
                ":scheme".to_string(),
            ],
            priority: None,
        },
        headers: [
            ("accept".to_string(), "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,*/*;q=0.8".to_string()),
            ("accept-language".to_string(), "en-US,en;q=0.5".to_string()),
            ("accept-encoding".to_string(), "gzip, deflate, br, zstd".to_string()),
            ("upgrade-insecure-requests".to_string(), "1".to_string()),
            ("sec-fetch-dest".to_string(), "document".to_string()),
            ("sec-fetch-mode".to_string(), "navigate".to_string()),
            ("sec-fetch-site".to_string(), "none".to_string()),
            ("sec-fetch-user".to_string(), "?1".to_string()),
            ("priority".to_string(), "u=1".to_string()),
        ].into_iter().collect(),
        version: 1,
        updated_at: Some("2024-12-01".to_string()),
    }
}

/// Firefox 133 profile
fn firefox133_profile() -> Profile {
    let mut profile = firefox136_profile();
    profile.name = "firefox133".to_string();
    profile.description = "Firefox 133 on Windows 10".to_string();
    profile.browser.version = "133.0".to_string();
    profile.browser.user_agent = "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:133.0) Gecko/20100101 Firefox/133.0".to_string();
    profile
}

/// Safari 18 profile
fn safari18_profile() -> Profile {
    Profile {
        name: "safari18".to_string(),
        description: "Safari 18 on macOS".to_string(),
        browser: BrowserInfo {
            name: "Safari".to_string(),
            version: "18.0".to_string(),
            platform: "macOS".to_string(),
            user_agent: "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/18.0 Safari/605.1.15".to_string(),
        },
        tls: TlsSettings {
            versions: vec!["TLS1.2".to_string(), "TLS1.3".to_string()],
            cipher_suites: vec![
                "TLS_AES_128_GCM_SHA256".to_string(),
                "TLS_AES_256_GCM_SHA384".to_string(),
                "TLS_CHACHA20_POLY1305_SHA256".to_string(),
                "TLS_ECDHE_ECDSA_WITH_AES_256_GCM_SHA384".to_string(),
                "TLS_ECDHE_ECDSA_WITH_AES_128_GCM_SHA256".to_string(),
                "TLS_ECDHE_ECDSA_WITH_CHACHA20_POLY1305_SHA256".to_string(),
                "TLS_ECDHE_RSA_WITH_AES_256_GCM_SHA384".to_string(),
                "TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256".to_string(),
                "TLS_ECDHE_RSA_WITH_CHACHA20_POLY1305_SHA256".to_string(),
            ],
            extensions: vec![
                "server_name".to_string(),
                "extended_master_secret".to_string(),
                "renegotiation_info".to_string(),
                "supported_groups".to_string(),
                "ec_point_formats".to_string(),
                "application_layer_protocol_negotiation".to_string(),
                "status_request".to_string(),
                "signature_algorithms".to_string(),
                "signed_certificate_timestamp".to_string(),
                "key_share".to_string(),
                "psk_key_exchange_modes".to_string(),
                "supported_versions".to_string(),
            ],
            curves: vec![
                "X25519".to_string(),
                "P-256".to_string(),
                "P-384".to_string(),
                "P-521".to_string(),
            ],
            signature_algorithms: vec![
                "ecdsa_secp256r1_sha256".to_string(),
                "rsa_pss_rsae_sha256".to_string(),
                "rsa_pkcs1_sha256".to_string(),
                "ecdsa_secp384r1_sha384".to_string(),
                "rsa_pss_rsae_sha384".to_string(),
                "rsa_pkcs1_sha384".to_string(),
                "rsa_pss_rsae_sha512".to_string(),
                "rsa_pkcs1_sha512".to_string(),
            ],
            alpn: vec!["h2".to_string(), "http/1.1".to_string()],
            grease: true,
        },
        http2: Http2Settings {
            settings: [
                ("HEADER_TABLE_SIZE".to_string(), 4096),
                ("ENABLE_PUSH".to_string(), 0),
                ("MAX_CONCURRENT_STREAMS".to_string(), 100),
                ("INITIAL_WINDOW_SIZE".to_string(), 2097152),
                ("MAX_FRAME_SIZE".to_string(), 16384),
                ("MAX_HEADER_LIST_SIZE".to_string(), 32768),
            ].into_iter().collect(),
            window_update: 10551295,
            pseudo_header_order: vec![
                ":method".to_string(),
                ":scheme".to_string(),
                ":path".to_string(),
                ":authority".to_string(),
            ],
            priority: None,
        },
        headers: [
            ("accept".to_string(), "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8".to_string()),
            ("accept-language".to_string(), "en-US,en;q=0.9".to_string()),
            ("accept-encoding".to_string(), "gzip, deflate, br".to_string()),
        ].into_iter().collect(),
        version: 1,
        updated_at: Some("2024-12-01".to_string()),
    }
}

/// Edge 131 profile
fn edge131_profile() -> Profile {
    let mut profile = chrome131_profile();
    profile.name = "edge131".to_string();
    profile.description = "Edge 131 on Windows 10".to_string();
    profile.browser.name = "Edge".to_string();
    profile.browser.user_agent = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36 Edg/131.0.0.0".to_string();
    profile.headers.insert("sec-ch-ua".to_string(), r#""Microsoft Edge";v="131", "Chromium";v="131", "Not_A Brand";v="24""#.to_string());
    profile
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chrome131_profile() {
        let profile = chrome131_profile();
        assert_eq!(profile.name, "chrome131");
        assert!(profile.browser.user_agent.contains("Chrome/131"));
        assert!(profile.tls.grease);
    }

    #[test]
    fn test_firefox136_profile() {
        let profile = firefox136_profile();
        assert_eq!(profile.name, "firefox136");
        assert!(profile.browser.user_agent.contains("Firefox/136"));
        assert!(!profile.tls.grease); // Firefox doesn't use GREASE
    }

    #[test]
    fn test_safari18_profile() {
        let profile = safari18_profile();
        assert_eq!(profile.name, "safari18");
        assert!(profile.browser.user_agent.contains("Safari"));
        assert_eq!(profile.browser.platform, "macOS");
    }

    #[test]
    fn test_profile_collection_defaults() {
        let collection = ProfileCollection::default_profiles();
        assert!(!collection.profiles.is_empty());
        assert!(collection.get("chrome131").is_some());
        assert!(collection.get("firefox136").is_some());
    }

    #[test]
    fn test_profile_collection_list() {
        let collection = ProfileCollection::default_profiles();
        let names = collection.list_names();
        assert!(names.contains(&"chrome131"));
        assert!(names.contains(&"firefox136"));
    }

    #[test]
    fn test_profiles_dir() {
        let dir = ProfileCollection::profiles_dir();
        assert!(dir.to_string_lossy().contains("rewget"));
    }
}
