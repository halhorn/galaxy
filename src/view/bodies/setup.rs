use bevy::{
    camera::visibility::NoFrustumCulling,
    prelude::*,
};

use crate::simulation::SimulationGpuBuffers;

use super::material::BodiesMaterial;
use super::mesh::{build_bodies_mesh, BodiesMesh};

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
        body_colors: gpu_buffers.body_colors.clone(),
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
