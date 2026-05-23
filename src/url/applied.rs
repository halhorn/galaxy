use crate::model::constants::G;
use crate::model::{ForceLaw, InitialConditions, PhysicsSettings};
use crate::simulation::{PlaybackMode, PlaybackState, SimulationConfig, SimulationSettings};

/// Snapshot of simulation settings that are serialized to the URL fragment.
#[derive(Debug, Clone, PartialEq)]
pub struct AppliedUrlState {
    pub physics: PhysicsSettings,
    pub initial: InitialConditions,
    pub force: ForceLaw,
    pub time_scale: f32,
    pub paused: bool,
}

impl Default for AppliedUrlState {
    fn default() -> Self {
        let physics = PhysicsSettings::default();
        Self {
            force: ForceLaw::newtonian(G),
            physics,
            initial: InitialConditions::default(),
            time_scale: SimulationConfig::default().time_scale,
            paused: false,
        }
    }
}

impl AppliedUrlState {
    pub fn from_resources(
        settings: &SimulationSettings,
        config: &SimulationConfig,
        playback: &PlaybackState,
    ) -> Self {
        Self {
            physics: settings.physics,
            initial: settings.initial.clone(),
            force: settings.force.clone(),
            time_scale: config.time_scale,
            paused: matches!(playback.mode, PlaybackMode::Paused),
        }
    }

    pub fn apply_to_resources(
        self,
        settings: &mut SimulationSettings,
        config: &mut SimulationConfig,
        playback: &mut PlaybackState,
    ) {
        settings.physics = self.physics;
        settings.initial = self.initial;
        settings.force = self.force;
        config.time_scale = self.time_scale;
        playback.mode = if self.paused {
            PlaybackMode::Paused
        } else {
            PlaybackMode::Running
        };
    }

    pub fn clamped(self) -> Self {
        Self {
            physics: self.physics.clamped(),
            initial: self.initial.clamped(),
            force: self.force.clamped(),
            time_scale: self.time_scale.clamp(0.25, 4.0),
            paused: self.paused,
        }
    }
}
