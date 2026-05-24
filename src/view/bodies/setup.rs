use bevy::{
    camera::visibility::NoFrustumCulling,
    prelude::*,
};

use crate::simulation::{SimulationGpuBuffers, SimulationSettings, SimulationSpawned};

use super::material::{BodiesMaterial, BodiesMaterialHandle};
use super::mesh::{build_bodies_mesh, BodiesMesh};

pub fn setup_bodies_render(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<BodiesMaterial>>,
    gpu_buffers: Res<SimulationGpuBuffers>,
    settings: Res<SimulationSettings>,
) {
    spawn_bodies_entity(
        &mut commands,
        &mut meshes,
        &mut materials,
        &gpu_buffers,
        settings.active_count() as usize,
    );
}

pub fn rebuild_bodies_mesh_on_spawn(
    mut events: MessageReader<SimulationSpawned>,
    settings: Res<SimulationSettings>,
    gpu_buffers: Res<SimulationGpuBuffers>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<BodiesMaterial>>,
    existing: Query<Entity, With<BodiesMesh>>,
) {
    if !events.read().any(|event| event.pending_readback) {
        return;
    }

    for entity in existing.iter() {
        commands.entity(entity).despawn();
    }

    spawn_bodies_entity(
        &mut commands,
        &mut meshes,
        &mut materials,
        &gpu_buffers,
        settings.active_count() as usize,
    );
}

fn spawn_bodies_entity(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<BodiesMaterial>,
    gpu_buffers: &SimulationGpuBuffers,
    active_count: usize,
) {
    let mesh_handle = meshes.add(build_bodies_mesh(active_count));
    let material_handle = materials.add(BodiesMaterial {
        positions: gpu_buffers.positions.clone(),
        masses: gpu_buffers.masses.clone(),
        body_colors: gpu_buffers.body_colors.clone(),
        params: Default::default(),
    });
    commands.spawn((
        BodiesMesh,
        Mesh3d(mesh_handle),
        MeshMaterial3d(material_handle.clone()),
        BodiesMaterialHandle(material_handle),
        Transform::IDENTITY,
        Visibility::default(),
        // Mesh AABB is only the unit sphere at origin; world positions come from GPU storage.
        NoFrustumCulling,
    ));
}
