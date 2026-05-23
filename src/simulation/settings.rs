use bevy::prelude::*;
use bevy::render::extract_resource::ExtractResource;

use crate::model::constants::G;
use crate::model::{ForceLaw, InitialConditions, PhysicsSettings};

/// Simulation parameters (Main world → Render world).
#[derive(Resource, Clone, ExtractResource)]
pub struct SimulationSettings {
    pub physics: PhysicsSettings,
    pub initial: InitialConditions,
    pub force: ForceLaw,
}

impl Default for SimulationSettings {
    fn default() -> Self {
        let physics = PhysicsSettings::default();
        Self {
            force: ForceLaw::newtonian(G),
            physics,
            initial: InitialConditions::default(),
        }
    }
}

impl SimulationSettings {
    pub fn active_count(&self) -> u32 {
        self.initial.active_count
    }
}
