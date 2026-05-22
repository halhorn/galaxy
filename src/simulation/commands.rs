use bevy::prelude::*;

/// CPU body data after spawn/restart (view layer seeds readback snapshot).
#[derive(Message, Clone)]
pub struct SimulationSpawned {
    pub positions: Vec<Vec3>,
    pub masses: Vec<f32>,
}
