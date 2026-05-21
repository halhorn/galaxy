use bevy::render::render_resource::ShaderType;

use crate::model::constants::MIN_MASS;
use crate::simulation::config::SimulationConfig;
use crate::simulation::settings::SimulationSettings;

#[derive(Clone, Copy, ShaderType)]
pub struct GravityParams {
    pub n: u32,
    pub g: f32,
    pub softening_sq: f32,
    pub min_mass: f32,
}

#[derive(Clone, Copy, ShaderType)]
pub struct IntegrateParams {
    pub n: u32,
    pub dt: f32,
    pub min_mass: f32,
    pub _pad: f32,
}

#[derive(Clone, Copy, ShaderType)]
pub struct MergeParams {
    pub n: u32,
    pub merge_radius_factor: f32,
    pub inv_cell_size: f32,
    pub min_mass: f32,
}

impl GravityParams {
    pub fn from_settings(settings: &SimulationSettings) -> Self {
        Self {
            n: settings.active_count(),
            g: settings.physics.g,
            softening_sq: settings.physics.softening_sq(),
            min_mass: MIN_MASS,
        }
    }
}

impl IntegrateParams {
    pub fn from_settings(settings: &SimulationSettings, config: &SimulationConfig) -> Self {
        Self {
            n: settings.active_count(),
            dt: config.dt(),
            min_mass: MIN_MASS,
            _pad: 0.0,
        }
    }
}

impl MergeParams {
    pub fn from_settings(settings: &SimulationSettings) -> Self {
        Self {
            n: settings.active_count(),
            merge_radius_factor: settings.physics.merge_radius_factor,
            inv_cell_size: settings.physics.merge_inv_cell_size(),
            min_mass: MIN_MASS,
        }
    }
}
