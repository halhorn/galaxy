use bevy::prelude::*;
use bevy::render::gpu_readback::{Readback, ReadbackComplete};
use bytemuck::pod_read_unaligned;

use crate::model::constants::BODY_COUNT;
use crate::simulation::SimulationGpuBuffers;
use crate::simulation::SimulationSpawned;

use super::pick::{ReadbackMasses, ReadbackPositions};
use super::snapshot::SimulationCpuSnapshot;

pub fn configure_selection_gizmos(mut config_store: ResMut<GizmoConfigStore>) {
    let (config, _) = config_store.config_mut::<DefaultGizmoConfigGroup>();
    config.depth_bias = -0.95;
    config.line.width = 3.0;
}

pub fn setup_readback(mut commands: Commands, gpu: Res<SimulationGpuBuffers>) {
    commands
        .spawn((ReadbackPositions, Readback::buffer(gpu.positions.clone())))
        .observe(on_positions_readback);
    commands
        .spawn((ReadbackMasses, Readback::buffer(gpu.masses.clone())))
        .observe(on_masses_readback);
}

pub fn on_spawned_seed_snapshot(
    mut events: MessageReader<SimulationSpawned>,
    mut snapshot: ResMut<SimulationCpuSnapshot>,
) {
    for event in events.read() {
        snapshot.positions = event.positions.clone();
        snapshot.masses = event.masses.clone();
        snapshot.ready = true;
    }
}

fn on_positions_readback(
    trigger: On<ReadbackComplete>,
    mut snapshot: ResMut<SimulationCpuSnapshot>,
) {
    snapshot.positions = parse_positions_readback(&trigger.event().data);
    if snapshot.positions.len() == BODY_COUNT && snapshot.masses.len() == BODY_COUNT {
        snapshot.ready = true;
    }
}

fn on_masses_readback(trigger: On<ReadbackComplete>, mut snapshot: ResMut<SimulationCpuSnapshot>) {
    snapshot.masses = parse_masses_readback(&trigger.event().data);
    if snapshot.positions.len() == BODY_COUNT && snapshot.masses.len() == BODY_COUNT {
        snapshot.ready = true;
    }
}

/// GPU readback returns a `Vec<u8>` that is not necessarily aligned for `cast_slice`.
fn parse_positions_readback(data: &[u8]) -> Vec<Vec3> {
    data.chunks_exact(16)
        .take(BODY_COUNT)
        .map(|chunk| pod_read_unaligned::<Vec4>(chunk).truncate())
        .collect()
}

fn parse_masses_readback(data: &[u8]) -> Vec<f32> {
    data.chunks_exact(4)
        .take(BODY_COUNT)
        .map(pod_read_unaligned::<f32>)
        .collect()
}
