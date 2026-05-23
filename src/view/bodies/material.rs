use bevy::{
    prelude::*,
    reflect::TypePath,
    render::{
        render_resource::{AsBindGroup, ShaderType},
        storage::ShaderStorageBuffer,
    },
    shader::ShaderRef,
};

use crate::model::constants::{MIN_STAR_VISUAL_SCALE, STAR_VISUAL_SCALE, SUN_RADIUS_AU};
use crate::simulation::shaders::BODIES_SHADER;

#[derive(Clone, Copy, Debug, ShaderType)]
pub struct StarsRenderParams {
    pub star_visual_scale: f32,
    pub min_star_visual_scale: f32,
    pub sun_radius_au: f32,
    pub _pad: f32,
}

impl Default for StarsRenderParams {
    fn default() -> Self {
        Self {
            star_visual_scale: STAR_VISUAL_SCALE,
            min_star_visual_scale: MIN_STAR_VISUAL_SCALE,
            sun_radius_au: SUN_RADIUS_AU,
            _pad: 0.0,
        }
    }
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct BodiesMaterial {
    #[storage(0, read_only)]
    pub positions: Handle<ShaderStorageBuffer>,
    #[storage(1, read_only)]
    pub masses: Handle<ShaderStorageBuffer>,
    #[storage(2, read_only)]
    pub body_colors: Handle<ShaderStorageBuffer>,
    #[uniform(3)]
    pub params: StarsRenderParams,
}

impl Material for BodiesMaterial {
    fn vertex_shader() -> ShaderRef {
        ShaderRef::Handle(BODIES_SHADER)
    }

    fn fragment_shader() -> ShaderRef {
        ShaderRef::Handle(BODIES_SHADER)
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Opaque
    }
}

#[derive(Component, Deref)]
pub struct BodiesMaterialHandle(pub Handle<BodiesMaterial>);

pub fn sync_star_render_params(
    config: Res<crate::simulation::SimulationConfig>,
    query: Query<&BodiesMaterialHandle>,
    mut materials: ResMut<Assets<BodiesMaterial>>,
) {
    if !config.is_changed() {
        return;
    }
    for handle in query.iter() {
        if let Some(material) = materials.get_mut(&handle.0) {
            material.params.star_visual_scale = config.star_visual_scale;
            material.params.min_star_visual_scale = config.min_star_visual_scale;
        }
    }
}
