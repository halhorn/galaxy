use bevy::prelude::*;

/// Simulation control messages from UI / keyboard.
#[derive(Message, Clone, Copy)]
pub enum SimulationCommand {
    Restart,
}

/// CPU body data after spawn/restart (view layer seeds readback snapshot).
#[derive(Message, Clone)]
pub struct SimulationSpawned {
    pub positions: Vec<Vec3>,
    pub masses: Vec<f32>,
    /// When true, selection waits for GPU readback before becoming ready again.
    pub pending_readback: bool,
}
