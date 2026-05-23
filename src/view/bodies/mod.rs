mod material;
mod mesh;
mod setup;

pub use mesh::BodiesMesh;
pub use setup::setup_bodies_render;

use bevy::prelude::*;

pub struct BodiesRenderPlugin;

impl Plugin for BodiesRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<material::BodiesMaterial>::default())
            .add_systems(Update, material::sync_star_render_params);
    }
}
