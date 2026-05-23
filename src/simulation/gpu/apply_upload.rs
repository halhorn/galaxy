use bevy::{
    prelude::*,
    render::{
        render_asset::RenderAssets,
        renderer::RenderQueue,
        storage::GpuShaderStorageBuffer,
    },
};

use super::buffers::SimulationGpuBuffers;
use crate::simulation::upload::{PendingSimulationUpload, SimulationUploadPayload};

fn write_gpu<T: bytemuck::Pod>(
    storage: &RenderAssets<GpuShaderStorageBuffer>,
    render_queue: &RenderQueue,
    handle: &Handle<bevy::render::storage::ShaderStorageBuffer>,
    data: &[T],
) {
    let Some(gpu) = storage.get(handle) else {
        return;
    };
    render_queue.write_buffer(&gpu.buffer, 0, bytemuck::cast_slice(data));
}

fn apply_payload(
    payload: &SimulationUploadPayload,
    gpu: &SimulationGpuBuffers,
    storage: &RenderAssets<GpuShaderStorageBuffer>,
    render_queue: &RenderQueue,
) {
    write_gpu(storage, render_queue, &gpu.positions, &payload.positions);
    write_gpu(storage, render_queue, &gpu.velocities, &payload.velocities);
    write_gpu(storage, render_queue, &gpu.masses, &payload.masses);
    write_gpu(storage, render_queue, &gpu.accelerations, &payload.accelerations);
    write_gpu(
        storage,
        render_queue,
        &gpu.accelerations_new,
        &payload.accelerations_new,
    );
    write_gpu(
        storage,
        render_queue,
        &gpu.merge_bucket_heads,
        &payload.merge_bucket_heads,
    );
    write_gpu(storage, render_queue, &gpu.merge_aux, &payload.merge_aux);
    write_gpu(storage, render_queue, &gpu.merge_owner, &payload.merge_owner);
    write_gpu(
        storage,
        render_queue,
        &gpu.merge_scratch,
        &payload.merge_scratch,
    );
}

/// Write queued body state into existing GPU buffers without recreating them.
pub fn apply_pending_simulation_upload(
    mut pending: ResMut<PendingSimulationUpload>,
    gpu: Res<SimulationGpuBuffers>,
    storage: Res<RenderAssets<GpuShaderStorageBuffer>>,
    render_queue: Res<RenderQueue>,
) {
    let Some(payload) = pending.payload.take() else {
        return;
    };
    apply_payload(&payload, &gpu, &storage, &render_queue);
}
