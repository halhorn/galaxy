use bevy::{
    prelude::*,
    render::{
        render_asset::RenderAssets,
        render_resource::*,
        renderer::{RenderDevice, RenderQueue},
        storage::GpuShaderStorageBuffer,
    },
};

use crate::simulation::config::SimulationConfig;
use crate::simulation::settings::SimulationSettings;

use super::buffers::SimulationGpuBuffers;
use super::params::{GravityParams, IntegrateParams, MergeParams};
use super::pipelines::SimulationComputePipelines;

#[derive(Resource)]
pub struct SimulationComputeBindGroups {
    pub gravity: BindGroup,
    pub integrate: BindGroup,
    pub merge: BindGroup,
}

#[allow(clippy::too_many_arguments)]
pub fn prepare_simulation_bind_groups(
    mut commands: Commands,
    pipelines: Res<SimulationComputePipelines>,
    gpu_buffers: Res<SimulationGpuBuffers>,
    settings: Res<SimulationSettings>,
    config: Res<SimulationConfig>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    pipeline_cache: Res<PipelineCache>,
    storage: Res<RenderAssets<GpuShaderStorageBuffer>>,
) {
    let Some(positions) = storage.get(&gpu_buffers.positions) else {
        return;
    };
    let Some(velocities) = storage.get(&gpu_buffers.velocities) else {
        return;
    };
    let Some(masses) = storage.get(&gpu_buffers.masses) else {
        return;
    };
    let Some(accelerations) = storage.get(&gpu_buffers.accelerations) else {
        return;
    };
    let Some(accelerations_new) = storage.get(&gpu_buffers.accelerations_new) else {
        return;
    };
    let Some(merge_bucket_heads) = storage.get(&gpu_buffers.merge_bucket_heads) else {
        return;
    };
    let Some(merge_aux) = storage.get(&gpu_buffers.merge_aux) else {
        return;
    };
    let Some(merge_owner) = storage.get(&gpu_buffers.merge_owner) else {
        return;
    };
    let Some(merge_scratch) = storage.get(&gpu_buffers.merge_scratch) else {
        return;
    };

    let gravity_params = GravityParams::from_settings(&settings);
    let mut gravity_uniform = UniformBuffer::from(gravity_params);
    gravity_uniform.write_buffer(&render_device, &render_queue);

    let integrate_params = IntegrateParams::from_settings(&settings, &config);
    let mut integrate_uniform = UniformBuffer::from(integrate_params);
    integrate_uniform.write_buffer(&render_device, &render_queue);

    let merge_params = MergeParams::from_settings(&settings);
    let mut merge_uniform = UniformBuffer::from(merge_params);
    merge_uniform.write_buffer(&render_device, &render_queue);

    let gravity_bind_group = render_device.create_bind_group(
        None,
        &pipeline_cache.get_bind_group_layout(&pipelines.gravity_layout),
        &BindGroupEntries::sequential((
            positions.buffer.as_entire_buffer_binding(),
            masses.buffer.as_entire_buffer_binding(),
            accelerations_new.buffer.as_entire_buffer_binding(),
            &gravity_uniform,
        )),
    );

    let integrate_bind_group = render_device.create_bind_group(
        None,
        &pipeline_cache.get_bind_group_layout(&pipelines.integrate_layout),
        &BindGroupEntries::sequential((
            positions.buffer.as_entire_buffer_binding(),
            velocities.buffer.as_entire_buffer_binding(),
            accelerations.buffer.as_entire_buffer_binding(),
            accelerations_new.buffer.as_entire_buffer_binding(),
            masses.buffer.as_entire_buffer_binding(),
            &integrate_uniform,
        )),
    );

    let merge_bind_group = render_device.create_bind_group(
        None,
        &pipeline_cache.get_bind_group_layout(&pipelines.merge_layout),
        &BindGroupEntries::sequential((
            positions.buffer.as_entire_buffer_binding(),
            velocities.buffer.as_entire_buffer_binding(),
            masses.buffer.as_entire_buffer_binding(),
            accelerations.buffer.as_entire_buffer_binding(),
            merge_scratch.buffer.as_entire_buffer_binding(),
            merge_bucket_heads.buffer.as_entire_buffer_binding(),
            merge_aux.buffer.as_entire_buffer_binding(),
            merge_owner.buffer.as_entire_buffer_binding(),
            &merge_uniform,
        )),
    );

    commands.insert_resource(SimulationComputeBindGroups {
        gravity: gravity_bind_group,
        integrate: integrate_bind_group,
        merge: merge_bind_group,
    });
}
