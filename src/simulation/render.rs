use bevy::{
    prelude::*,
    reflect::TypePath,
    render::{render_resource::AsBindGroup, storage::ShaderStorageBuffer},
    shader::ShaderRef,
};

use super::shaders::BODIES_SHADER;

use super::{buffers::SimulationGpuBuffers, constants::BODY_COUNT};

/// Marker for the single draw entity that renders all bodies.
#[derive(Component)]
pub struct BodiesMesh;

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct BodiesMaterial {
    #[storage(0, read_only)]
    pub positions: Handle<ShaderStorageBuffer>,
    #[storage(1, read_only)]
    pub masses: Handle<ShaderStorageBuffer>,
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

pub struct BodiesRenderPlugin;

impl Plugin for BodiesRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<BodiesMaterial>::default());
    }
}

pub fn setup_bodies_render(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<BodiesMaterial>>,
    gpu_buffers: Res<SimulationGpuBuffers>,
) {
    let mesh_handle = meshes.add(build_points_mesh());
    let material_handle = materials.add(BodiesMaterial {
        positions: gpu_buffers.positions.clone(),
        masses: gpu_buffers.masses.clone(),
    });
    commands.spawn((
        BodiesMesh,
        Mesh3d(mesh_handle),
        MeshMaterial3d(material_handle),
        Transform::IDENTITY,
        Visibility::default(),
    ));
}

/// Point per body; vertex shader uses `@builtin(vertex_index)` as body id.
fn build_points_mesh() -> Mesh {
    let mut mesh = Mesh::new(
        bevy::mesh::PrimitiveTopology::PointList,
        bevy::asset::RenderAssetUsages::default(),
    );
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_POSITION,
        vec![Vec3::ZERO; BODY_COUNT],
    );
    mesh
}
