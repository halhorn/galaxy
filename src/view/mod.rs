mod bodies;
mod selection;

pub use bodies::{setup_bodies_render, BodiesMesh, BodiesRenderPlugin};

use bevy::prelude::*;

use selection::SelectionPlugin;

pub struct ViewPlugin;

impl Plugin for ViewPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((BodiesRenderPlugin, SelectionPlugin));
    }
}
