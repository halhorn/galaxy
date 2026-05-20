use bevy::{
    camera::visibility::NoFrustumCulling,
    mesh::{Indices, MeshVertexAttribute, SphereKind, VertexAttributeValues, VertexFormat},
    prelude::*,
    reflect::TypePath,
    render::{render_resource::AsBindGroup, storage::ShaderStorageBuffer},
    shader::ShaderRef,
};

/// Ico-sphere subdivisions for the unit template (92 verts @ 2 — balance of look vs build cost).
const SPHERE_SUBDIVISIONS: u32 = 2;

use super::shaders::BODIES_SHADER;

use super::{buffers::SimulationGpuBuffers, constants::BODY_COUNT};

/// Per-vertex body index (unit sphere duplicated per body).
pub const ATTRIBUTE_BODY_ID: MeshVertexAttribute =
    MeshVertexAttribute::new("BodyId", 5, VertexFormat::Uint32);

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
    let mesh_handle = meshes.add(build_bodies_mesh());
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
        // Mesh AABB is only the unit sphere at origin; world positions come from GPU storage.
        NoFrustumCulling,
    ));
}

/// One indexed unit sphere per body (single draw call; positions from GPU storage in the shader).
fn build_bodies_mesh() -> Mesh {
    let unit = Sphere::new(0.5)
        .mesh()
        .kind(SphereKind::Ico {
            subdivisions: SPHERE_SUBDIVISIONS,
        })
        .build();
    let base_positions = unit
        .attribute(Mesh::ATTRIBUTE_POSITION)
        .and_then(|values| match values {
            VertexAttributeValues::Float32x3(data) => Some(data.clone()),
            _ => None,
        })
        .expect("sphere mesh has positions");
    let base_normals = unit
        .attribute(Mesh::ATTRIBUTE_NORMAL)
        .and_then(|values| match values {
            VertexAttributeValues::Float32x3(data) => Some(data.clone()),
            _ => None,
        })
        .expect("sphere mesh has normals");
    let base_indices = match unit.indices().expect("sphere mesh is indexed") {
        Indices::U32(indices) => indices.clone(),
        _ => panic!("sphere mesh uses u32 indices"),
    };

    let verts_per_body = base_positions.len();
    let total_verts = BODY_COUNT * verts_per_body;
    let mut positions = Vec::with_capacity(total_verts);
    let mut normals = Vec::with_capacity(total_verts);
    let mut body_ids = Vec::with_capacity(total_verts);
    let mut indices = Vec::with_capacity(BODY_COUNT * base_indices.len());

    for body_id in 0..BODY_COUNT {
        let base = (body_id * verts_per_body) as u32;
        positions.extend(base_positions.iter().copied());
        normals.extend(base_normals.iter().copied());
        body_ids.extend(std::iter::repeat_n(body_id as u32, verts_per_body));
        indices.extend(base_indices.iter().map(|&i| base + i));
    }

    let mut mesh = Mesh::new(
        bevy::mesh::PrimitiveTopology::TriangleList,
        bevy::asset::RenderAssetUsages::default(),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(ATTRIBUTE_BODY_ID, body_ids);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}
