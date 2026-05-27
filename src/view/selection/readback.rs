use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;
use bevy::render::gpu_readback::{Readback, ReadbackComplete};
use bytemuck::pod_read_unaligned;

use crate::model::constants::BODY_COUNT;
use crate::simulation::{SimulationGpuBuffers, SimulationSettings, SimulationSpawned};
use crate::view::sim_viewport::SIMULATION_RENDER_LAYER;

use super::pick::{ReadbackMasses, ReadbackPositions};
use super::snapshot::SimulationCpuSnapshot;

/// Mass readback can lag pick by a frame; positions must stay in sync with the GPU render.
const MASS_READBACK_INTERVAL_FRAMES: u32 = 2;

#[derive(Resource, Default)]
pub(crate) struct ReadbackThrottle {
    mass_update_frame: u32,
    positions_entity: Option<Entity>,
    masses_entity: Option<Entity>,
    tracked_active_count: u32,
}

pub fn configure_selection_gizmos(mut config_store: ResMut<GizmoConfigStore>) {
    let (config, _) = config_store.config_mut::<DefaultGizmoConfigGroup>();
    config.depth_bias = -0.95;
    config.line.width = 3.0;
    // Only the 3D simulation camera uses this layer; avoid drawing on the egui Camera2d.
    config.render_layers = RenderLayers::layer(SIMULATION_RENDER_LAYER);
}

pub fn setup_readback(mut commands: Commands) {
    commands.init_resource::<ReadbackThrottle>();
}

pub fn sync_readback_entities(
    mut commands: Commands,
    gpu: Res<SimulationGpuBuffers>,
    settings: Res<SimulationSettings>,
    mut throttle: ResMut<ReadbackThrottle>,
) {
    let active_count = settings.active_count();
    let positions_bytes = active_count as u64 * 16;
    let masses_bytes = active_count as u64 * 4;

    if throttle.tracked_active_count != active_count {
        remove_readback_entities(&mut commands, &mut throttle);
        throttle.tracked_active_count = active_count;
        throttle.mass_update_frame = 0;
    }

    if throttle.positions_entity.is_none() {
        let entity = commands
            .spawn((
                ReadbackPositions,
                Readback::buffer_range(gpu.positions.clone(), 0, positions_bytes),
            ))
            .observe(on_positions_readback)
            .id();
        throttle.positions_entity = Some(entity);
    }

    if throttle.masses_entity.is_none() {
        let entity = commands
            .spawn((
                ReadbackMasses,
                Readback::buffer_range(gpu.masses.clone(), 0, masses_bytes),
            ))
            .observe(on_masses_readback)
            .id();
        throttle.masses_entity = Some(entity);
    }
}

fn remove_readback_entities(commands: &mut Commands, throttle: &mut ReadbackThrottle) {
    if let Some(entity) = throttle.positions_entity.take() {
        commands.entity(entity).despawn();
    }
    if let Some(entity) = throttle.masses_entity.take() {
        commands.entity(entity).despawn();
    }
}

pub fn on_spawned_seed_snapshot(
    mut events: MessageReader<SimulationSpawned>,
    mut snapshot: ResMut<SimulationCpuSnapshot>,
    mut selected: ResMut<super::pick::SelectedBody>,
    settings: Res<SimulationSettings>,
) {
    let active_count = settings.active_count() as usize;
    for event in events.read() {
        snapshot.positions = event.positions[..active_count.min(event.positions.len())].to_vec();
        snapshot.masses = event.masses[..active_count.min(event.masses.len())].to_vec();
        snapshot.ready = !event.pending_readback;
        if event.pending_readback {
            selected.0 = None;
        }
    }
}

fn on_positions_readback(
    trigger: On<ReadbackComplete>,
    mut snapshot: ResMut<SimulationCpuSnapshot>,
    settings: Res<SimulationSettings>,
) {
    let active_count = settings.active_count() as usize;
    snapshot.positions = parse_positions_readback(&trigger.event().data, active_count);
    if snapshot.positions.len() == active_count && snapshot.masses.len() == active_count {
        snapshot.ready = true;
    }
}

fn on_masses_readback(
    trigger: On<ReadbackComplete>,
    mut snapshot: ResMut<SimulationCpuSnapshot>,
    mut throttle: ResMut<ReadbackThrottle>,
    settings: Res<SimulationSettings>,
) {
    throttle.mass_update_frame = throttle.mass_update_frame.wrapping_add(1);
    if throttle.mass_update_frame % MASS_READBACK_INTERVAL_FRAMES != 0 {
        return;
    }

    let active_count = settings.active_count() as usize;
    snapshot.masses = parse_masses_readback(&trigger.event().data, active_count);
    if snapshot.positions.len() == active_count && snapshot.masses.len() == active_count {
        snapshot.ready = true;
    }
}

/// GPU readback returns a `Vec<u8>` that is not necessarily aligned for `cast_slice`.
fn parse_positions_readback(data: &[u8], active_count: usize) -> Vec<Vec3> {
    data.chunks_exact(16)
        .take(active_count.min(BODY_COUNT))
        .map(|chunk| pod_read_unaligned::<Vec4>(chunk).truncate())
        .collect()
}

fn parse_masses_readback(data: &[u8], active_count: usize) -> Vec<f32> {
    data.chunks_exact(4)
        .take(active_count.min(BODY_COUNT))
        .map(pod_read_unaligned::<f32>)
        .collect()
}
