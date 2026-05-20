use bevy::prelude::*;
use bevy::render::extract_resource::ExtractResource;

/// Simulation timing and physics parameters (main world → render world).
#[derive(Resource, Clone, ExtractResource)]
pub struct SimulationConfig {
    /// Simulation years per real second.
    pub time_scale: f32,
    /// Fixed physics timestep in simulation years.
    pub fixed_dt: f32,
}

impl Default for SimulationConfig {
    fn default() -> Self {
        Self {
            time_scale: 1.0,
            fixed_dt: 1.0 / 60.0,
        }
    }
}

impl SimulationConfig {
    pub fn dt(&self) -> f32 {
        self.fixed_dt * self.time_scale
    }
}
