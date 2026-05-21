use bevy::prelude::*;

/// Playback gate for GPU compute (Phase 1 fills in pause/resume).
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Default)]
#[allow(dead_code)]
pub enum PlaybackState {
    #[default]
    Running,
    Paused,
}

/// Elapsed simulation time in years (Phase 1).
#[derive(Resource, Debug, Clone, Copy, Default)]
#[allow(dead_code)]
pub struct SimulationClock {
    pub accumulated_sim_time: f32,
}
