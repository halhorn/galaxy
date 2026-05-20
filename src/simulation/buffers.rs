use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::{
    extract_resource::ExtractResource, render_resource::BufferUsages, storage::ShaderStorageBuffer,
};

use super::constants::{BODY_COUNT, MERGE_BUCKET_COUNT};

/// GPU-resident simulation buffers (handles shared by compute + render).
#[derive(Resource, Clone, ExtractResource)]
pub struct SimulationGpuBuffers {
    pub positions: Handle<ShaderStorageBuffer>,
    pub velocities: Handle<ShaderStorageBuffer>,
    pub masses: Handle<ShaderStorageBuffer>,
    pub accelerations: Handle<ShaderStorageBuffer>,
    pub accelerations_new: Handle<ShaderStorageBuffer>,
    /// Merge pass: spatial hash bucket heads (`u32::MAX` = empty).
    pub merge_bucket_heads: Handle<ShaderStorageBuffer>,
    /// Merge pass: `[0..n)` bucket_next, `[n..2n)` absorbed (0/1).
    pub merge_aux: Handle<ShaderStorageBuffer>,
    /// Merge pass: smallest survivor index `i` per absorbed target `j`.
    pub merge_owner: Handle<ShaderStorageBuffer>,
    /// Merge snapshot: `[0..n)` pos.xyz + mass in w, `[n..2n)` velocity.
    pub merge_scratch: Handle<ShaderStorageBuffer>,
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
        let merge_bucket_heads = buffers.add(storage_buffer(
            vec![u32::MAX; MERGE_BUCKET_COUNT],
            usage,
            asset_usage,
        ));
        let mut merge_aux_data = vec![u32::MAX; BODY_COUNT * 2];
        merge_aux_data[BODY_COUNT..].fill(0);
        let merge_aux = buffers.add(storage_buffer(merge_aux_data, usage, asset_usage));
        let merge_owner = buffers.add(storage_buffer(
            vec![BODY_COUNT as u32; BODY_COUNT],
            usage,
            asset_usage,
        ));
        let merge_scratch = buffers.add(storage_buffer(
            vec![Vec4::ZERO; BODY_COUNT * 2],
            usage,
            asset_usage,
        ));

        Self {
            positions,
            velocities,
            masses,
            accelerations,
            accelerations_new,
            merge_bucket_heads,
            merge_aux,
            merge_owner,
            merge_scratch,
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
