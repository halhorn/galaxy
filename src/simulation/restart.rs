use bevy::prelude::*;
use bevy::render::storage::ShaderStorageBuffer;

use crate::model::{generate_initial_state, BodyArrays};

use super::commands::SimulationSpawned;
use super::gpu::SimulationGpuBuffers;
use super::settings::SimulationSettings;

/// Convert model arrays to Bevy `Vec4` for GPU upload.
pub fn body_arrays_to_vec4(bodies: &BodyArrays) -> (Vec<Vec4>, Vec<Vec4>, Vec<f32>, Vec<Vec4>) {
    let positions = bodies
        .positions
        .iter()
        .map(|p| Vec4::new(p[0], p[1], p[2], p[3]))
        .collect();
    let velocities = bodies
        .velocities
        .iter()
        .map(|v| Vec4::new(v[0], v[1], v[2], v[3]))
        .collect();
    let masses = bodies.masses.clone();
    let accelerations = bodies
        .accelerations
        .iter()
        .map(|a| Vec4::new(a[0], a[1], a[2], a[3]))
        .collect();
    (positions, velocities, masses, accelerations)
}

/// One-shot startup: generate state and create GPU buffers.
pub fn spawn_initial_simulation(
    mut commands: Commands,
    settings: Res<SimulationSettings>,
    mut buffers: ResMut<Assets<ShaderStorageBuffer>>,
    mut spawned: MessageWriter<SimulationSpawned>,
) {
    let bodies = generate_initial_state(&settings.initial, &settings.physics, &settings.force);
    install_simulation_state(&mut commands, &mut buffers, &bodies, &mut spawned);
}

fn install_simulation_state(
    commands: &mut Commands,
    buffers: &mut Assets<ShaderStorageBuffer>,
    bodies: &BodyArrays,
    spawned: &mut MessageWriter<SimulationSpawned>,
) {
    let (positions, velocities, masses, accelerations) = body_arrays_to_vec4(bodies);

    let gpu = SimulationGpuBuffers::new(buffers, positions, velocities, masses, accelerations);
    commands.insert_resource(gpu);

    spawned.write(SimulationSpawned {
        positions: bodies
            .positions
            .iter()
            .map(|p| Vec3::new(p[0], p[1], p[2]))
            .collect(),
        masses: bodies.masses.clone(),
    });
}
