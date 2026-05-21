mod commands;
mod config;
pub mod gpu;
mod playback;
mod restart;
mod settings;
pub mod shaders;
mod upload;

pub use commands::SimulationSpawned;
pub use config::SimulationConfig;
pub use gpu::SimulationGpuBuffers;
pub use playback::{PlaybackMode, PlaybackState};
pub use settings::SimulationSettings;

use bevy::prelude::*;

use gpu::SimulationGpuPlugin;
use playback::tick_sim_time;
use restart::spawn_initial_simulation;
use shaders::register_simulation_shaders;

pub struct SimulationPlugin;

impl Plugin for SimulationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SimulationConfig>()
            .init_resource::<SimulationSettings>()
            .init_resource::<PlaybackState>()
            .add_message::<SimulationSpawned>()
            .add_plugins(SimulationGpuPlugin)
            .add_systems(Startup, (register_simulation_shaders, spawn_initial_simulation))
            .add_systems(Update, tick_sim_time);
    }
}
