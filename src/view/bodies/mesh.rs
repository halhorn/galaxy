use bevy::{
    mesh::{Indices, MeshVertexAttribute, SphereKind, VertexAttributeValues, VertexFormat},
    prelude::*,
};

use crate::model::constants::BODY_COUNT;

/// Ico-sphere subdivisions for the unit template (92 verts @ 2 — balance of look vs build cost).
const SPHERE_SUBDIVISIONS: u32 = 2;

/// Per-vertex body index (unit sphere duplicated per body).
pub const ATTRIBUTE_BODY_ID: MeshVertexAttribute =
    MeshVertexAttribute::new("BodyId", 5, VertexFormat::Uint32);

/// Marker for the single draw entity that renders all bodies.
#[derive(Component)]
pub struct BodiesMesh;

/// One indexed unit sphere per active body slot (single draw call; positions from GPU storage in the shader).
pub fn build_bodies_mesh(active_count: usize) -> Mesh {
    let body_count = active_count.min(BODY_COUNT);
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
    let total_verts = body_count * verts_per_body;
    let mut positions = Vec::with_capacity(total_verts);
    let mut normals = Vec::with_capacity(total_verts);
    let mut body_ids = Vec::with_capacity(total_verts);
    let mut indices = Vec::with_capacity(body_count * base_indices.len());

    for body_id in 0..body_count {
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
