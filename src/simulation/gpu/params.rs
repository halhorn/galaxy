use bevy::render::render_resource::ShaderType;

use crate::model::constants::MIN_MASS;
use crate::model::force::MAX_FORCE_TERMS;
use crate::simulation::config::SimulationConfig;
use crate::simulation::settings::SimulationSettings;

#[derive(Clone, Copy, ShaderType)]
pub struct GpuForceTerm {
    pub sign: i32,
    pub exponent: i32,
    pub coefficient: f32,
    pub _pad: u32,
}

#[derive(Clone, Copy, ShaderType)]
pub struct GravityParams {
    pub n: u32,
    pub term_count: u32,
    pub softening_sq: f32,
    pub min_mass: f32,
    pub terms: [GpuForceTerm; MAX_FORCE_TERMS],
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
        let force = settings.force.clone().clamped();
        let mut terms = [GpuForceTerm {
            sign: 0,
            exponent: 0,
            coefficient: 0.0,
            _pad: 0,
        }; MAX_FORCE_TERMS];

        for (i, term) in force
            .terms
            .iter()
            .take(force.term_count as usize)
            .enumerate()
        {
            terms[i] = GpuForceTerm {
                sign: term.sign as i32,
                exponent: term.exponent,
                coefficient: term.coefficient,
                _pad: 0,
            };
        }

        Self {
            n: settings.active_count(),
            term_count: force.term_count as u32,
            softening_sq: settings.physics.softening_sq(),
            min_mass: MIN_MASS,
            terms,
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
