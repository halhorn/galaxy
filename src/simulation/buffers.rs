use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::{
    extract_resource::ExtractResource, render_resource::BufferUsages, storage::ShaderStorageBuffer,
};

use super::constants::BODY_COUNT;

/// GPU-resident simulation buffers (handles shared by compute + render).
#[derive(Resource, Clone, ExtractResource)]
pub struct SimulationGpuBuffers {
    pub positions: Handle<ShaderStorageBuffer>,
    pub velocities: Handle<ShaderStorageBuffer>,
    pub masses: Handle<ShaderStorageBuffer>,
    pub accelerations: Handle<ShaderStorageBuffer>,
    pub accelerations_new: Handle<ShaderStorageBuffer>,
}

impl SimulationGpuBuffers {
    pub fn new(
        buffers: &mut Assets<ShaderStorageBuffer>,
        positions: Vec<Vec4>,
        velocities: Vec<Vec4>,
        masses: Vec<f32>,
        accelerations: Vec<Vec4>,
    ) -> Self {
        debug_assert_eq!(positions.len(), BODY_COUNT);
        let usage = BufferUsages::STORAGE | BufferUsages::COPY_DST;
        let asset_usage = RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD;

        let positions = buffers.add(storage_buffer(positions, usage, asset_usage));
        let velocities = buffers.add(storage_buffer(velocities, usage, asset_usage));
        let masses = buffers.add(storage_buffer(masses, usage, asset_usage));
        let accelerations = buffers.add(storage_buffer(accelerations, usage, asset_usage));
        let accelerations_new = buffers.add(storage_buffer(
            vec![Vec4::ZERO; BODY_COUNT],
            usage,
            asset_usage,
        ));

        Self {
            positions,
            velocities,
            masses,
            accelerations,
            accelerations_new,
        }
    }
}

fn storage_buffer<T: bytemuck::Pod>(
    data: Vec<T>,
    usage: BufferUsages,
    asset_usage: RenderAssetUsages,
) -> ShaderStorageBuffer {
    let mut buffer = ShaderStorageBuffer::new(bytemuck::cast_slice(data.as_slice()), asset_usage);
    buffer.buffer_description.usage = usage;
    buffer
}
