use crate::model::constants::G;
use crate::model::{ForceLaw, InitialConditions, PhysicsSettings};
use crate::simulation::{SimulationConfig, SimulationSettings};

/// Snapshot of simulation settings that are serialized to the URL fragment.
#[derive(Debug, Clone, PartialEq)]
pub struct AppliedUrlState {
    pub physics: PhysicsSettings,
    pub initial: InitialConditions,
    pub force: ForceLaw,
    pub time_scale: f32,
    pub star_visual_scale: f32,
    pub min_star_visual_scale: f32,
}

impl Default for AppliedUrlState {
    fn default() -> Self {
        let physics = PhysicsSettings::default();
        let config = SimulationConfig::default();
        Self {
            force: ForceLaw::newtonian(G),
            physics,
            initial: InitialConditions::default(),
            time_scale: config.time_scale,
            star_visual_scale: config.star_visual_scale,
            min_star_visual_scale: config.min_star_visual_scale,
        }
    }
}

impl AppliedUrlState {
    pub fn from_resources(settings: &SimulationSettings, config: &SimulationConfig) -> Self {
        Self {
            physics: settings.physics,
            initial: settings.initial.clone(),
            force: settings.force.clone(),
            time_scale: config.time_scale,
            star_visual_scale: config.star_visual_scale,
            min_star_visual_scale: config.min_star_visual_scale,
        }
    }

    pub fn apply_to_resources(self, settings: &mut SimulationSettings, config: &mut SimulationConfig) {
        settings.physics = self.physics;
        settings.initial = self.initial;
        settings.force = self.force;
        config.time_scale = self.time_scale;
        config.star_visual_scale = self.star_visual_scale;
        config.min_star_visual_scale = self.min_star_visual_scale;
    }

    pub fn clamped(self) -> Self {
        let config = SimulationConfig {
            time_scale: self.time_scale,
            star_visual_scale: self.star_visual_scale,
            min_star_visual_scale: self.min_star_visual_scale,
            fixed_dt: SimulationConfig::default().fixed_dt,
        }
        .clamped();
        Self {
            physics: self.physics.clamped(),
            initial: self.initial.clamped(),
            force: self.force.clamped(),
            time_scale: config.time_scale,
            star_visual_scale: config.star_visual_scale,
            min_star_visual_scale: config.min_star_visual_scale,
        }
    }
}
