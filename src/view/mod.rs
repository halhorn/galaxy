mod bodies;
mod selection;
mod sim_viewport;

pub use bodies::{setup_bodies_render, BodiesMesh, BodiesRenderPlugin};
pub use sim_viewport::SimulationCamera;

use bevy::prelude::*;

use selection::SelectionPlugin;
use sim_viewport::SimulationViewportPlugin;

pub struct ViewPlugin;

impl Plugin for ViewPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((BodiesRenderPlugin, SelectionPlugin, SimulationViewportPlugin));
    }
}
