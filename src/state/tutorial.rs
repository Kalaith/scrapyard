//! Data-driven tutorial system

use serde::Deserialize;

/// Configuration for a single tutorial step (loaded from JSON)
#[derive(Debug, Clone, Deserialize)]
pub struct TutorialStepConfig {
    pub id: String,
    pub message: String,
    pub target_room: Option<usize>,
    pub show_highlight: bool,
}

/// Tutorial configuration (loaded from JSON)
#[derive(Debug, Clone, Deserialize)]
pub struct TutorialConfig {
    pub steps: Vec<TutorialStepConfig>,
}

impl TutorialConfig {
    /// Load tutorial config from embedded JSON
    pub fn load() -> Self {
        serde_json::from_str(include_str!("../../assets/tutorial.json"))
            .unwrap_or_else(|e| {
                eprintln!("Warning: Failed to load tutorial.json: {}. Using empty tutorial.", e);
                Self { steps: Vec::new() }
            })
    }
}

/// Runtime tutorial state
#[derive(Debug, Clone)]
pub struct TutorialState {
    pub current_index: usize,
    pub completed: bool,
}

impl TutorialState {
    pub fn new() -> Self {
        Self {
            current_index: 0,
            completed: false,
        }
    }

    /// Get current step from config
    pub fn current_step<'a>(&self, config: &'a TutorialConfig) -> Option<&'a TutorialStepConfig> {
        if self.completed {
            None
        } else {
            config.steps.get(self.current_index)
        }
    }

    /// Check if currently on the first step (welcome)
    pub fn is_welcome(&self) -> bool {
        self.current_index == 0 && !self.completed
    }

    /// Check if tutorial is complete
    pub fn is_complete(&self) -> bool {
        self.completed
    }

    /// Get target room for current step
    pub fn target_room(&self, config: &TutorialConfig) -> Option<usize> {
        self.current_step(config).and_then(|s| s.target_room)
    }

    /// Advance to next step
    pub fn advance(&mut self, config: &TutorialConfig) {
        if self.current_index + 1 < config.steps.len() {
            self.current_index += 1;
        } else {
            self.completed = true;
        }
    }

    /// Check if should highlight room
    pub fn should_highlight(&self, config: &TutorialConfig, room_id: usize) -> bool {
        if let Some(step) = self.current_step(config) {
            step.show_highlight && step.target_room == Some(room_id)
        } else {
            false
        }
    }
}

// Legacy enum for backwards compatibility during transition
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TutorialStep {
    Welcome,
    RepairReactor,
    RepairShields,
    RepairWeapon,
    RepairEngine,
    Complete,
}

impl TutorialStep {
    /// Convert from index-based state to enum (for legacy code compatibility)
    pub fn from_state(state: &TutorialState) -> Self {
        if state.completed {
            TutorialStep::Complete
        } else {
            match state.current_index {
                0 => TutorialStep::Welcome,
                1 => TutorialStep::RepairReactor,
                2 => TutorialStep::RepairShields,
                3 => TutorialStep::RepairWeapon,
                4 => TutorialStep::RepairEngine,
                _ => TutorialStep::Complete,
            }
        }
    }
}
