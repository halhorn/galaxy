use std::borrow::Cow;

use bevy::{
    prelude::*,
    render::{
        Render, RenderApp, RenderStartup, RenderSystems,
        extract_resource::ExtractResourcePlugin,
        render_asset::RenderAssets,
        render_graph::{self, RenderGraph, RenderLabel},
        render_resource::{
            binding_types::{storage_buffer, storage_buffer_read_only, uniform_buffer},
            *,
        },
        renderer::{RenderContext, RenderDevice, RenderQueue},
        storage::GpuShaderStorageBuffer,
    },
    shader::PipelineCacheError,
};

use super::{
    buffers::SimulationGpuBuffers,
    config::SimulationConfig,
    constants::{
        BODY_COUNT, G, MERGE_BUCKET_COUNT, MERGE_INV_CELL_SIZE, MERGE_RADIUS_FACTOR, MIN_MASS,
        SOFTENING_SQ, dispatch_workgroups,
    },
};

use super::shaders::{GRAVITY_SHADER, INTEGRATE_SHADER, MERGE_SHADER};

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
pub struct SimulationComputeLabel;

#[derive(Resource)]
pub struct SimulationComputePipelines {
    pub gravity_layout: BindGroupLayoutDescriptor,
    pub integrate_layout: BindGroupLayoutDescriptor,
    pub merge_layout: BindGroupLayoutDescriptor,
    pub gravity: CachedComputePipelineId,
    pub position_step: CachedComputePipelineId,
    pub velocity_step: CachedComputePipelineId,
    pub merge_prepare: CachedComputePipelineId,
    pub merge_clear_buckets: CachedComputePipelineId,
    pub merge_init_owner: CachedComputePipelineId,
    pub merge_build_grid: CachedComputePipelineId,
    pub merge_find_owner: CachedComputePipelineId,
    pub merge_apply: CachedComputePipelineId,
}

#[derive(Clone, Copy, ShaderType)]
pub struct GravityParams {
    pub n: u32,
    pub g: f32,
    pub softening_sq: f32,
    pub min_mass: f32,
}

#[derive(Clone, Copy, ShaderType)]
pub struct IntegrateParams {
    pub n: u32,
    pub dt: f32,
    pub min_mass: f32,
    pub _pad: f32,
}

#[derive(Clone, Copy, ShaderType)]
pub struct MergeParams {
    pub n: u32,
    pub merge_radius_factor: f32,
    pub inv_cell_size: f32,
    pub min_mass: f32,
}

pub struct SimulationComputePlugin;

impl Plugin for SimulationComputePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ExtractResourcePlugin::<SimulationGpuBuffers>::default())
            .add_plugins(ExtractResourcePlugin::<SimulationConfig>::default());

        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app
            .add_systems(RenderStartup, init_simulation_compute_pipelines)
            .add_systems(
                Render,
                prepare_simulation_bind_group
                    .in_set(RenderSystems::PrepareBindGroups)
                    .run_if(resource_exists::<SimulationComputePipelines>)
                    .run_if(resource_exists::<SimulationGpuBuffers>)
                    .run_if(resource_exists::<SimulationConfig>),
            );

        let mut render_graph = render_app.world_mut().resource_mut::<RenderGraph>();
        render_graph.add_node(SimulationComputeLabel, SimulationComputeNode::default());
        render_graph.add_node_edge(
            SimulationComputeLabel,
            bevy::render::graph::CameraDriverLabel,
        );
    }
}

fn init_simulation_compute_pipelines(
    mut commands: Commands,
    pipeline_cache: Res<PipelineCache>,
) {
    let gravity_layout = BindGroupLayoutDescriptor::new(
        "gravity_layout",
        &BindGroupLayoutEntries::sequential(
            ShaderStages::COMPUTE,
            (
                storage_buffer_read_only::<Vec<Vec4>>(false),
                storage_buffer_read_only::<Vec<f32>>(false),
                storage_buffer::<Vec<Vec4>>(false),
                uniform_buffer::<GravityParams>(false),
            ),
        ),
    );

    let integrate_layout = BindGroupLayoutDescriptor::new(
        "integrate_layout",
        &BindGroupLayoutEntries::sequential(
            ShaderStages::COMPUTE,
            (
                storage_buffer::<Vec<Vec4>>(false),
                storage_buffer::<Vec<Vec4>>(false),
                storage_buffer::<Vec<Vec4>>(false),
                storage_buffer_read_only::<Vec<Vec4>>(false),
                storage_buffer_read_only::<Vec<f32>>(false),
                uniform_buffer::<IntegrateParams>(false),
            ),
        ),
    );

    let merge_layout = BindGroupLayoutDescriptor::new(
        "merge_layout",
        &BindGroupLayoutEntries::sequential(
            ShaderStages::COMPUTE,
            (
                storage_buffer::<Vec<Vec4>>(false),
                storage_buffer::<Vec<Vec4>>(false),
                storage_buffer::<Vec<f32>>(false),
                storage_buffer::<Vec<Vec4>>(false),
                storage_buffer::<Vec<Vec4>>(false),
                storage_buffer::<Vec<u32>>(false),
                storage_buffer::<Vec<u32>>(false),
                storage_buffer::<Vec<u32>>(false),
                uniform_buffer::<MergeParams>(false),
            ),
        ),
    );

    let gravity = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
        label: Some("gravity".into()),
        layout: vec![gravity_layout.clone()],
        shader: GRAVITY_SHADER.clone(),
        entry_point: Some(Cow::from("main")),
        ..default()
    });

    let position_step = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
        label: Some("position_step".into()),
        layout: vec![integrate_layout.clone()],
        shader: INTEGRATE_SHADER.clone(),
        entry_point: Some(Cow::from("position_step")),
        ..default()
    });

    let velocity_step = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
        label: Some("velocity_step".into()),
        layout: vec![integrate_layout.clone()],
        shader: INTEGRATE_SHADER,
        entry_point: Some(Cow::from("velocity_step")),
        ..default()
    });

    let merge_prepare = queue_merge_pipeline(&pipeline_cache, &merge_layout, "prepare");
    let merge_clear_buckets =
        queue_merge_pipeline(&pipeline_cache, &merge_layout, "clear_buckets");
    let merge_init_owner = queue_merge_pipeline(&pipeline_cache, &merge_layout, "init_owner");
    let merge_build_grid = queue_merge_pipeline(&pipeline_cache, &merge_layout, "build_grid");
    let merge_find_owner = queue_merge_pipeline(&pipeline_cache, &merge_layout, "find_owner");
    let merge_apply = queue_merge_pipeline(&pipeline_cache, &merge_layout, "apply_merge");

    commands.insert_resource(SimulationComputePipelines {
        gravity_layout,
        integrate_layout,
        merge_layout,
        gravity,
        position_step,
        velocity_step,
        merge_prepare,
        merge_clear_buckets,
        merge_init_owner,
        merge_build_grid,
        merge_find_owner,
        merge_apply,
    });
}

fn queue_merge_pipeline(
    pipeline_cache: &PipelineCache,
    layout: &BindGroupLayoutDescriptor,
    entry: &'static str,
) -> CachedComputePipelineId {
    pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
        label: Some(entry.into()),
        layout: vec![layout.clone()],
        shader: MERGE_SHADER.clone(),
        entry_point: Some(Cow::from(entry)),
        ..default()
    })
}

#[allow(clippy::too_many_arguments)]
fn prepare_simulation_bind_group(
    mut commands: Commands,
    pipelines: Res<SimulationComputePipelines>,
    gpu_buffers: Res<SimulationGpuBuffers>,
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

    let gravity_params = GravityParams {
        n: BODY_COUNT as u32,
        g: G,
        softening_sq: SOFTENING_SQ,
        min_mass: MIN_MASS,
    };
    let mut gravity_uniform = UniformBuffer::from(gravity_params);
    gravity_uniform.write_buffer(&render_device, &render_queue);

    let integrate_params = IntegrateParams {
        n: BODY_COUNT as u32,
        dt: config.dt(),
        min_mass: MIN_MASS,
        _pad: 0.0,
    };
    let mut integrate_uniform = UniformBuffer::from(integrate_params);
    integrate_uniform.write_buffer(&render_device, &render_queue);

    let merge_params = MergeParams {
        n: BODY_COUNT as u32,
        merge_radius_factor: MERGE_RADIUS_FACTOR,
        inv_cell_size: MERGE_INV_CELL_SIZE,
        min_mass: MIN_MASS,
    };
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

#[derive(Resource)]
pub struct SimulationComputeBindGroups {
    pub gravity: BindGroup,
    pub integrate: BindGroup,
    pub merge: BindGroup,
}

#[derive(Default)]
struct SimulationComputeNode {
    ready: bool,
}

impl render_graph::Node for SimulationComputeNode {
    fn update(&mut self, world: &mut World) {
        if self.ready {
            return;
        }
        let pipelines = world.resource::<SimulationComputePipelines>();
        let cache = world.resource::<PipelineCache>();
        for id in [
            pipelines.gravity,
            pipelines.position_step,
            pipelines.velocity_step,
            pipelines.merge_prepare,
            pipelines.merge_clear_buckets,
            pipelines.merge_init_owner,
            pipelines.merge_build_grid,
            pipelines.merge_find_owner,
            pipelines.merge_apply,
        ] {
            match cache.get_compute_pipeline_state(id) {
                CachedPipelineState::Ok(_) => {}
                CachedPipelineState::Err(PipelineCacheError::ShaderNotLoaded(_)) => return,
                CachedPipelineState::Err(err) => {
                    bevy::log::error!("simulation compute pipeline: {err}");
                    return;
                }
                _ => return,
            }
        }
        self.ready = true;
    }

    fn run(
        &self,
        _graph: &mut render_graph::RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), render_graph::NodeRunError> {
        if !self.ready {
            return Ok(());
        }
        let Some(bind_groups) = world.get_resource::<SimulationComputeBindGroups>() else {
            return Ok(());
        };

        let pipelines = world.resource::<SimulationComputePipelines>();
        let cache = world.resource::<PipelineCache>();
        let workgroups = dispatch_workgroups();

        let gravity = cache.get_compute_pipeline(pipelines.gravity).unwrap();
        let position_step = cache.get_compute_pipeline(pipelines.position_step).unwrap();
        let velocity_step = cache.get_compute_pipeline(pipelines.velocity_step).unwrap();

        let mut pass = render_context
            .command_encoder()
            .begin_compute_pass(&ComputePassDescriptor::default());

        pass.set_bind_group(0, &bind_groups.integrate, &[]);
        pass.set_pipeline(position_step);
        pass.dispatch_workgroups(workgroups, 1, 1);

        pass.set_bind_group(0, &bind_groups.gravity, &[]);
        pass.set_pipeline(gravity);
        pass.dispatch_workgroups(workgroups, 1, 1);

        pass.set_bind_group(0, &bind_groups.integrate, &[]);
        pass.set_pipeline(velocity_step);
        pass.dispatch_workgroups(workgroups, 1, 1);

        let bucket_workgroups = (MERGE_BUCKET_COUNT as u32).div_ceil(256);
        let merge_prepare = cache.get_compute_pipeline(pipelines.merge_prepare).unwrap();
        let merge_clear_buckets = cache
            .get_compute_pipeline(pipelines.merge_clear_buckets)
            .unwrap();
        let merge_init_owner = cache.get_compute_pipeline(pipelines.merge_init_owner).unwrap();
        let merge_build_grid = cache.get_compute_pipeline(pipelines.merge_build_grid).unwrap();
        let merge_find_owner = cache.get_compute_pipeline(pipelines.merge_find_owner).unwrap();
        let merge_apply = cache.get_compute_pipeline(pipelines.merge_apply).unwrap();

        pass.set_bind_group(0, &bind_groups.merge, &[]);
        pass.set_pipeline(merge_prepare);
        pass.dispatch_workgroups(workgroups, 1, 1);
        pass.set_pipeline(merge_clear_buckets);
        pass.dispatch_workgroups(bucket_workgroups, 1, 1);
        pass.set_pipeline(merge_init_owner);
        pass.dispatch_workgroups(workgroups, 1, 1);
        pass.set_pipeline(merge_build_grid);
        pass.dispatch_workgroups(workgroups, 1, 1);
        pass.set_pipeline(merge_find_owner);
        pass.dispatch_workgroups(workgroups, 1, 1);
        pass.set_pipeline(merge_apply);
        pass.dispatch_workgroups(workgroups, 1, 1);

        Ok(())
    }
}
