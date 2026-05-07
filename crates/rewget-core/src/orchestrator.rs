//! Fetch orchestrator state machine
//!
//! Models the entire CLI-side fallback lifecycle as an explicit state machine,
//! replacing the deeply nested `if/else` chain in `exec.rs`.

use crate::{BlockReason, Config, DetectionResult, DomainCache, FetchStage};

/// The current state of a fetch operation.
#[derive(Debug, Clone)]
pub enum FetchState {
    /// Waiting to start.
    Idle,

    /// Running a stage (wget, impersonate, or preflight).
    RunningStage {
        stage: FetchStage,
    },

    /// Stage completed; analyzing detection result.
    Detecting {
        stage: FetchStage,
        result: StageOutput,
    },

    /// Skipping lower stages because the cache says this one works.
    CachedSkip {
        stage: FetchStage,
    },

    /// Fetch succeeded at the given stage.
    Success {
        stage: FetchStage,
    },

    /// Fetch was blocked at the given stage.
    Blocked {
        stage: FetchStage,
        reason: BlockReason,
    },

    /// All stages exhausted; request is still blocked.
    Exhausted {
        last_reason: Option<BlockReason>,
    },

    /// A terminal error occurred.
    Error {
        error: String,
    },

    /// Fetch completed without success (e.g. wget exit code != 0 but not blocked).
    Failed {
        exit_code: i32,
    },
}

/// Output produced by executing a single stage.
#[derive(Debug, Clone)]
pub struct StageOutput {
    pub exit_code: i32,
    pub detection: DetectionResult,
    pub body: Option<Vec<u8>>,
}

/// An action the driver should take.
#[derive(Debug)]
pub enum FetchAction {
    /// Run plain wget.
    RunWget {
        stage: FetchStage,
    },

    /// Run Stage 2 via daemon.
    RunImpersonate {
        stage: FetchStage,
        url: String,
        output: Option<std::path::PathBuf>,
    },

    /// Run Stage 3 via daemon.
    RunPreflight {
        stage: FetchStage,
        url: String,
        output: Option<std::path::PathBuf>,
    },

    /// Cache hit; skip directly to this stage.
    CacheHit {
        stage: FetchStage,
    },

    /// Terminal success.
    Complete {
        stage: FetchStage,
    },

    /// Terminal failure (all stages exhausted).
    GiveUp {
        last_reason: Option<BlockReason>,
    },

    /// Terminal error.
    Fatal {
        error: String,
    },

    /// Non-blocked failure (propagate wget exit code).
    Propagate {
        exit_code: i32,
    },
}

/// Orchestrates the fallback pipeline.
///
/// Usage:
/// ```ignore
/// let mut orch = FetchOrchestrator::new(config, cache, domain);
/// while let Some(action) = orch.next_action() {
///     match action {
///         FetchAction::RunWget { .. } => { /* run wget */ orch.report_stage1(result); }
///         FetchAction::RunImpersonate { .. } => { /* ask daemon */ orch.report_stage2(result); }
///         ...
///     }
/// }
/// ```
#[derive(Debug)]
pub struct FetchOrchestrator {
    pub state: FetchState,
    pub config: Config,
    pub cache: DomainCache,
    pub domain: Option<String>,
}

impl FetchOrchestrator {
    /// Create a new orchestrator.
    pub fn new(config: Config, cache: DomainCache, domain: Option<String>) -> Self {
        Self {
            state: FetchState::Idle,
            config,
            cache,
            domain,
        }
    }

    /// Compute the next action given the current state.
    ///
    /// Returns `None` when the state machine has reached a terminal state
    /// (`Success`, `Exhausted`, `Error`, `Failed`).
    pub fn next_action(&mut self) -> Option<FetchAction> {
        use FetchState::*;

        // Take ownership of the current state so we can mutate self freely.
        let state = std::mem::replace(&mut self.state, FetchState::Idle);

        match state {
            Idle => {
                // Check cache for a known-working stage.
                if let Some(ref d) = self.domain {
                    if !self.config.no_cache {
                        if let Some(cached_stage) = self.cache.get(d) {
                            if cached_stage <= self.config.fallback_stage {
                                self.state = CachedSkip { stage: cached_stage };
                                return Some(FetchAction::CacheHit {
                                    stage: cached_stage,
                                });
                            }
                        }
                    }
                }
                // Always start from the first stage; fallback_stage controls the ceiling.
                let stage = FetchStage::Wget;
                self.state = RunningStage { stage };
                Some(self.action_for_stage(stage))
            }

            CachedSkip { stage } => {
                self.state = Success { stage };
                Some(FetchAction::Complete { stage })
            }

            RunningStage { .. } => {
                // Waiting for the driver to run the stage and call report_*.
                self.state = state; // Restore the state we took.
                None
            }

            Detecting { stage, result } => {
                if result.exit_code == 0 && !result.detection.blocked {
                    // Success — cache it.
                    if let Some(ref d) = self.domain {
                        if !self.config.no_cache {
                            self.cache.set(d, stage);
                        }
                    }
                    self.state = Success { stage };
                    return Some(FetchAction::Complete { stage });
                }

                if result.detection.blocked {
                    self.state = Blocked {
                        stage,
                        reason: result.detection.reason.unwrap_or(BlockReason::StatusCode(
                            result.detection.status_code.unwrap_or(403),
                        )),
                    };
                    // `next_action` will be called again; the Blocked arm handles transition.
                    self.next_action()
                } else {
                    // Non-zero exit but not detected as blocked.
                    self.state = Failed {
                        exit_code: result.exit_code,
                    };
                    Some(FetchAction::Propagate {
                        exit_code: result.exit_code,
                    })
                }
            }

            Blocked { stage, reason } => {
                // Can we try the next stage?
                if let Some(next) = stage.next() {
                    if next <= self.config.fallback_stage {
                        self.state = RunningStage { stage: next };
                        return Some(self.action_for_stage(next));
                    }
                }
                // No more stages.
                self.state = Exhausted {
                    last_reason: Some(reason.clone()),
                };
                Some(FetchAction::GiveUp { last_reason: Some(reason) })
            }

            Success { .. }
            | Exhausted { .. }
            | Error { .. }
            | Failed { .. } => {
                self.state = state; // Restore terminal state.
                None
            }
        }
    }

    /// Report the result of running Stage 1 (wget).
    pub fn report_stage1(&mut self, exit_code: i32, detection: DetectionResult, body: Option<Vec<u8>>) {
        self.state = FetchState::Detecting {
            stage: FetchStage::Wget,
            result: StageOutput {
                exit_code,
                detection,
                body,
            },
        };
    }

    /// Report the result of running Stage 2 (impersonate) via daemon.
    pub fn report_stage2(&mut self, success: bool, blocked: bool, status_code: Option<u16>, reason: Option<String>) {
        let detection = DetectionResult {
            blocked,
            status_code,
            reason: reason.map(BlockReason::BodyPattern),
            exit_code: if success && !blocked { 0 } else { 8 },
        };
        self.state = FetchState::Detecting {
            stage: FetchStage::Impersonate,
            result: StageOutput {
                exit_code: detection.exit_code,
                detection,
                body: None,
            },
        };
    }

    /// Report the result of running Stage 3 (preflight) via daemon.
    pub fn report_stage3(&mut self, success: bool, blocked: bool, status_code: Option<u16>, reason: Option<String>) {
        let detection = DetectionResult {
            blocked,
            status_code,
            reason: reason.map(BlockReason::BodyPattern),
            exit_code: if success && !blocked { 0 } else { 8 },
        };
        self.state = FetchState::Detecting {
            stage: FetchStage::Preflight,
            result: StageOutput {
                exit_code: detection.exit_code,
                detection,
                body: None,
            },
        };
    }

    /// Report a fatal error.
    pub fn report_error(&mut self, error: String) {
        self.state = FetchState::Error { error };
    }

    fn action_for_stage(&self,
        stage: FetchStage,
    ) -> FetchAction {
        match stage {
            FetchStage::Wget => FetchAction::RunWget { stage },
            FetchStage::Impersonate => {
                // Extract URL and output from config/wget_args.
                // The driver (`exec.rs`) will provide these.
                FetchAction::RunImpersonate {
                    stage,
                    url: String::new(),
                    output: None,
                }
            }
            FetchStage::Preflight => FetchAction::RunPreflight {
                stage,
                url: String::new(),
                output: None,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> Config {
        Config {
            fallback_stage: FetchStage::Preflight,
            ..Config::default()
        }
    }

    #[test]
    fn test_idle_to_wget() {
        let mut orch = FetchOrchestrator::new(test_config(), DomainCache::default(), None);
        let action = orch.next_action().unwrap();
        assert!(matches!(action, FetchAction::RunWget { stage: FetchStage::Wget }));
    }

    #[test]
    fn test_wget_success() {
        let mut orch = FetchOrchestrator::new(test_config(), DomainCache::default(), None);
        let _ = orch.next_action();
        orch.report_stage1(0, DetectionResult {
            blocked: false,
            status_code: Some(200),
            reason: None,
            exit_code: 0,
        }, None);
        let action = orch.next_action().unwrap();
        assert!(matches!(action, FetchAction::Complete { stage: FetchStage::Wget }));
    }

    #[test]
    fn test_wget_blocked_falls_back() {
        let mut orch = FetchOrchestrator::new(test_config(), DomainCache::default(), None);
        let _ = orch.next_action();
        orch.report_stage1(8, DetectionResult {
            blocked: true,
            status_code: Some(403),
            reason: Some(BlockReason::StatusCode(403)),
            exit_code: 8,
        }, None);
        let action = orch.next_action().unwrap();
        assert!(matches!(action, FetchAction::RunImpersonate { stage: FetchStage::Impersonate, .. }));
    }

    #[test]
    fn test_all_stages_exhausted() {
        let mut orch = FetchOrchestrator::new(test_config(), DomainCache::default(), None);
        let _ = orch.next_action(); // Idle -> Wget
        orch.report_stage1(8, DetectionResult {
            blocked: true,
            status_code: Some(403),
            reason: Some(BlockReason::StatusCode(403)),
            exit_code: 8,
        }, None);
        let _ = orch.next_action(); // Blocked -> Impersonate
        orch.report_stage2(false, true, Some(403), Some("Blocked".to_string()));
        let _ = orch.next_action(); // Blocked -> Preflight
        orch.report_stage3(false, true, Some(403), Some("Blocked".to_string()));
        let action = orch.next_action().unwrap();
        assert!(matches!(action, FetchAction::GiveUp { .. }));
    }

    #[test]
    fn test_cache_hit_skips() {
        let mut cache = DomainCache::default();
        cache.set("example.com", FetchStage::Impersonate);

        let mut orch = FetchOrchestrator::new(
            test_config(),
            cache,
            Some("example.com".to_string()),
        );
        let action = orch.next_action().unwrap();
        assert!(matches!(action, FetchAction::CacheHit { stage: FetchStage::Impersonate }));
    }

    #[test]
    fn test_fallback_stage_limits_stages() {
        let mut config = test_config();
        config.fallback_stage = FetchStage::Impersonate; // Don't try preflight

        let mut orch = FetchOrchestrator::new(config, DomainCache::default(), None);
        let _ = orch.next_action(); // Wget
        orch.report_stage1(8, DetectionResult {
            blocked: true,
            status_code: Some(403),
            reason: Some(BlockReason::StatusCode(403)),
            exit_code: 8,
        }, None);
        let _ = orch.next_action(); // Impersonate
        orch.report_stage2(false, true, Some(403), Some("Blocked".to_string()));
        let action = orch.next_action().unwrap();
        assert!(matches!(action, FetchAction::GiveUp { .. }));
    }
}
