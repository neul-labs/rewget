//! Stage definitions for the rewget fallback pipeline
//!
//! Replaces raw `u8` stage identifiers with a type-safe enum.

use serde::{Deserialize, Serialize};

/// A stage in the fetch fallback pipeline.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default,
)]
pub enum FetchStage {
    /// Stage 1: Plain wget / wget2
    #[serde(rename = "wget")]
    #[default]
    Wget,

    /// Stage 2: Browser TLS/HTTP2 impersonation via rquest
    #[serde(rename = "impersonate")]
    Impersonate,

    /// Stage 3: Headless Chromium JS preflight
    #[serde(rename = "preflight")]
    Preflight,
}

impl FetchStage {
    /// All stages in ascending order.
    pub const ALL: &[FetchStage] = &[
        FetchStage::Wget,
        FetchStage::Impersonate,
        FetchStage::Preflight,
    ];

    /// Return the next stage, if any.
    pub fn next(self) -> Option<Self> {
        match self {
            FetchStage::Wget => Some(FetchStage::Impersonate),
            FetchStage::Impersonate => Some(FetchStage::Preflight),
            FetchStage::Preflight => None,
        }
    }

    /// Return the numeric representation (1, 2, 3).
    pub fn number(self) -> u8 {
        match self {
            FetchStage::Wget => 1,
            FetchStage::Impersonate => 2,
            FetchStage::Preflight => 3,
        }
    }

    /// Iterate from this stage through the remaining stages (inclusive).
    pub fn iter_from(self) -> impl Iterator<Item = Self> {
        let mut started = false;
        Self::ALL.iter().copied().filter(move |&s| {
            if s == self {
                started = true;
            }
            started
        })
    }
}

impl std::fmt::Display for FetchStage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FetchStage::Wget => write!(f, "wget"),
            FetchStage::Impersonate => write!(f, "impersonate"),
            FetchStage::Preflight => write!(f, "preflight"),
        }
    }
}

impl TryFrom<u8> for FetchStage {
    type Error = &'static str;

    fn try_from(value: u8) -> std::result::Result<Self, Self::Error> {
        match value {
            1 => Ok(FetchStage::Wget),
            2 => Ok(FetchStage::Impersonate),
            3 => Ok(FetchStage::Preflight),
            _ => Err("invalid stage number"),
        }
    }
}

impl From<FetchStage> for u8 {
    fn from(stage: FetchStage) -> Self {
        stage.number()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stage_next() {
        assert_eq!(FetchStage::Wget.next(), Some(FetchStage::Impersonate));
        assert_eq!(FetchStage::Impersonate.next(), Some(FetchStage::Preflight));
        assert_eq!(FetchStage::Preflight.next(), None);
    }

    #[test]
    fn test_stage_number() {
        assert_eq!(FetchStage::Wget.number(), 1);
        assert_eq!(FetchStage::Impersonate.number(), 2);
        assert_eq!(FetchStage::Preflight.number(), 3);
    }

    #[test]
    fn test_stage_iter_from() {
        let stages: Vec<_> = FetchStage::Impersonate.iter_from().collect();
        assert_eq!(stages, vec![FetchStage::Impersonate, FetchStage::Preflight]);
    }

    #[test]
    fn test_try_from_u8() {
        assert_eq!(FetchStage::try_from(1).unwrap(), FetchStage::Wget);
        assert_eq!(FetchStage::try_from(2).unwrap(), FetchStage::Impersonate);
        assert_eq!(FetchStage::try_from(3).unwrap(), FetchStage::Preflight);
        assert!(FetchStage::try_from(0).is_err());
        assert!(FetchStage::try_from(4).is_err());
    }

    #[test]
    fn test_display() {
        assert_eq!(FetchStage::Wget.to_string(), "wget");
        assert_eq!(FetchStage::Impersonate.to_string(), "impersonate");
        assert_eq!(FetchStage::Preflight.to_string(), "preflight");
    }

    #[test]
    fn test_serde_roundtrip() {
        let stage = FetchStage::Impersonate;
        let json = serde_json::to_string(&stage).unwrap();
        assert_eq!(json, "\"impersonate\"");
        let decoded: FetchStage = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded, stage);
    }
}
