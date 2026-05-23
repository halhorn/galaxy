use bevy::{
    prelude::*,
    reflect::TypePath,
    render::{render_resource::AsBindGroup, storage::ShaderStorageBuffer},
    shader::ShaderRef,
};

use crate::simulation::shaders::BODIES_SHADER;

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct BodiesMaterial {
    #[storage(0, read_only)]
    pub positions: Handle<ShaderStorageBuffer>,
    #[storage(1, read_only)]
    pub masses: Handle<ShaderStorageBuffer>,
    #[storage(2, read_only)]
    pub body_colors: Handle<ShaderStorageBuffer>,
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
