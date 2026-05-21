use bevy::prelude::*;
use bevy::render::extract_resource::ExtractResource;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PlaybackMode {
    #[default]
    Running,
    Paused,
}

/// Playback gate for GPU compute and elapsed simulation time (simulation years).
#[derive(Resource, Debug, Clone, Copy, ExtractResource)]
pub struct PlaybackState {
    pub mode: PlaybackMode,
    pub accumulated_sim_time: f32,
}

impl Default for PlaybackState {
    fn default() -> Self {
        Self {
            mode: PlaybackMode::Running,
            accumulated_sim_time: 0.0,
        }
    }
}

impl PlaybackState {
    pub fn is_running(&self) -> bool {
        matches!(self.mode, PlaybackMode::Running)
    }

    pub fn toggle(&mut self) {
        self.mode = match self.mode {
            PlaybackMode::Running => PlaybackMode::Paused,
            PlaybackMode::Paused => PlaybackMode::Running,
        };
    }
}

pub fn tick_sim_time(
    config: Res<super::config::SimulationConfig>,
    mut playback: ResMut<PlaybackState>,
) {
    if !playback.is_running() {
        return;
    }
    playback.accumulated_sim_time += config.dt();
}
