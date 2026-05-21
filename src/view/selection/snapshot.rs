use bevy::prelude::*;

/// CPU mirror of GPU positions/masses (updated via readback after spawn).
#[derive(Resource, Default)]
pub struct SimulationCpuSnapshot {
    pub positions: Vec<Vec3>,
    pub masses: Vec<f32>,
    pub ready: bool,
}
