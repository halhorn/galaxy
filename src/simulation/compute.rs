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
    constants::{BODY_COUNT, G, SOFTENING_SQ, dispatch_workgroups},
};

use super::shaders::{GRAVITY_SHADER, INTEGRATE_SHADER};

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
pub struct SimulationComputeLabel;

#[derive(Resource)]
pub struct SimulationComputePipelines {
    pub gravity_layout: BindGroupLayoutDescriptor,
    pub integrate_layout: BindGroupLayoutDescriptor,
    pub gravity: CachedComputePipelineId,
    pub position_step: CachedComputePipelineId,
    pub velocity_step: CachedComputePipelineId,
}

#[derive(Clone, Copy, ShaderType)]
pub struct GravityParams {
    pub n: u32,
    pub g: f32,
    pub softening_sq: f32,
    pub _pad: f32,
}

#[derive(Clone, Copy, ShaderType)]
pub struct IntegrateParams {
    pub n: u32,
    pub dt: f32,
    pub _pad0: f32,
    pub _pad1: f32,
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
                uniform_buffer::<IntegrateParams>(false),
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

    commands.insert_resource(SimulationComputePipelines {
        gravity_layout,
        integrate_layout,
        gravity,
        position_step,
        velocity_step,
    });
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

    let gravity_params = GravityParams {
        n: BODY_COUNT as u32,
        g: G,
        softening_sq: SOFTENING_SQ,
        _pad: 0.0,
    };
    let mut gravity_uniform = UniformBuffer::from(gravity_params);
    gravity_uniform.write_buffer(&render_device, &render_queue);

    let integrate_params = IntegrateParams {
        n: BODY_COUNT as u32,
        dt: config.dt(),
        _pad0: 0.0,
        _pad1: 0.0,
    };
    let mut integrate_uniform = UniformBuffer::from(integrate_params);
    integrate_uniform.write_buffer(&render_device, &render_queue);

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
            &integrate_uniform,
        )),
    );

    commands.insert_resource(SimulationComputeBindGroups {
        gravity: gravity_bind_group,
        integrate: integrate_bind_group,
    });
}

#[derive(Resource)]
pub struct SimulationComputeBindGroups {
    pub gravity: BindGroup,
    pub integrate: BindGroup,
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

        Ok(())
    }
}
