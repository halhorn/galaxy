use bevy::prelude::*;
use bevy::render::extract_resource::ExtractResource;

use crate::model::constants::{
    MIN_STAR_VISUAL_SCALE, MIN_STAR_VISUAL_SCALE_MAX, MIN_STAR_VISUAL_SCALE_MIN,
    STAR_VISUAL_SCALE, STAR_VISUAL_SCALE_MAX, STAR_VISUAL_SCALE_MIN,
};

/// Simulation timing and display parameters (main world → render world).
#[derive(Resource, Clone, ExtractResource)]
pub struct SimulationConfig {
    /// Simulation years per real second.
    pub time_scale: f32,
    /// Multiplier for rendered star sphere radii (physics uses physical `SUN_RADIUS_AU`).
    pub star_visual_scale: f32,
    /// Minimum rendered star radius in AU.
    pub min_star_visual_scale: f32,
    /// Fixed physics timestep in simulation years.
    pub fixed_dt: f32,
}

impl Default for SimulationConfig {
    fn default() -> Self {
        Self {
            time_scale: 1.0,
            star_visual_scale: STAR_VISUAL_SCALE,
            min_star_visual_scale: MIN_STAR_VISUAL_SCALE,
            fixed_dt: 1.0 / 60.0,
        }
    }
}

impl SimulationConfig {
    pub fn dt(&self) -> f32 {
        self.fixed_dt * self.time_scale
    }

    pub fn clamped(self) -> Self {
        Self {
            time_scale: self.time_scale.clamp(0.01, 10.0),
            star_visual_scale: self
                .star_visual_scale
                .clamp(STAR_VISUAL_SCALE_MIN, STAR_VISUAL_SCALE_MAX),
            min_star_visual_scale: self
                .min_star_visual_scale
                .clamp(MIN_STAR_VISUAL_SCALE_MIN, MIN_STAR_VISUAL_SCALE_MAX),
            fixed_dt: self.fixed_dt,
        }
    }
}
