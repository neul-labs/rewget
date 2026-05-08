//! Wget engine selection and execution

use crate::{Error, Result};
use std::path::PathBuf;

/// Supported wget engines
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Engine {
    #[default]
    Wget,
    Wget2,
}

impl Engine {
    /// Parse engine from string
    pub fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "wget" => Ok(Engine::Wget),
            "wget2" => Ok(Engine::Wget2),
            _ => Err(Error::InvalidArgument(format!(
                "Unknown engine '{}'. Valid options: wget, wget2",
                s
            ))),
        }
    }

    /// Get the binary name for this engine
    pub fn binary_name(&self) -> &'static str {
        match self {
            Engine::Wget => "wget",
            Engine::Wget2 => "wget2",
        }
    }

    /// Find the engine binary path
    ///
    /// Search order:
    /// 1. Bundled engine in rewget's directory (wget_engine / wget2_engine)
    /// 2. System PATH
    pub fn find_binary(&self) -> Result<PathBuf> {
        let bundled_name = match self {
            Engine::Wget => "wget_engine",
            Engine::Wget2 => "wget2_engine",
        };

        // First, try to find bundled engine next to rewget binary
        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(exe_dir) = exe_path.parent() {
                let bundled_path = exe_dir.join(bundled_name);
                if bundled_path.exists() {
                    return Ok(bundled_path);
                }
            }
        }

        // Fall back to system PATH
        let system_name = self.binary_name();
        which::which(system_name)
            .map_err(|_| Error::EngineNotFound(format!(
                "Could not find '{}' or '{}' in PATH. Please install {} or place {} next to rewget.",
                bundled_name, system_name, system_name, bundled_name
            )))
    }
}

impl std::fmt::Display for Engine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Engine::Wget => write!(f, "wget"),
            Engine::Wget2 => write!(f, "wget2"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_from_str() {
        assert_eq!(Engine::from_str("wget").unwrap(), Engine::Wget);
        assert_eq!(Engine::from_str("wget2").unwrap(), Engine::Wget2);
        assert_eq!(Engine::from_str("WGET").unwrap(), Engine::Wget);
        assert!(Engine::from_str("curl").is_err());
    }

    #[test]
    fn test_engine_binary_name() {
        assert_eq!(Engine::Wget.binary_name(), "wget");
        assert_eq!(Engine::Wget2.binary_name(), "wget2");
    }
}
