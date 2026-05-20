mod buffers;
mod compute;
mod config;
mod constants;
mod init;
pub mod render;
mod shaders;

pub use config::SimulationConfig;

use bevy::prelude::*;

use compute::SimulationComputePlugin;
use init::spawn_initial_state;
use render::{setup_bodies_render, BodiesRenderPlugin};
use shaders::register_simulation_shaders;

pub struct SimulationPlugin;

impl Plugin for SimulationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SimulationConfig>()
            .add_plugins(BodiesRenderPlugin)
            .add_plugins(SimulationComputePlugin)
            .add_systems(Startup, (register_simulation_shaders, spawn_initial_state))
            .add_systems(PostStartup, setup_bodies_render);
    }
}
