use bevy::prelude::*;

/// Simulation transport commands (Phase 1+ handlers).
#[derive(Message, Debug, Clone)]
#[allow(dead_code)]
pub enum SimulationCommand {
    Restart,
}

/// CPU body data after spawn/restart (view layer seeds readback snapshot).
#[derive(Message, Clone)]
pub struct SimulationSpawned {
    pub positions: Vec<Vec3>,
    pub masses: Vec<f32>,
}
