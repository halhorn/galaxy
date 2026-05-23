use bevy::{
    prelude::*,
    render::{
        render_graph::{self, RenderLabel},
        render_resource::*,
    },
    shader::PipelineCacheError,
};

use crate::model::constants::{BODY_COUNT, MERGE_BUCKET_COUNT, WORKGROUP_SIZE};

use crate::simulation::playback::PlaybackState;

use super::bind_groups::SimulationComputeBindGroups;
use super::pipelines::SimulationComputePipelines;

fn dispatch_workgroups() -> u32 {
    (BODY_COUNT as u32).div_ceil(WORKGROUP_SIZE)
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
pub struct SimulationComputeLabel;

#[derive(Default)]
pub struct SimulationComputeNode {
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
            pipelines.colors,
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
        render_context: &mut bevy::render::renderer::RenderContext,
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

        let mut pass = render_context
            .command_encoder()
            .begin_compute_pass(&ComputePassDescriptor::default());

        let playback = world.resource::<PlaybackState>();
        if playback.is_running() {
            let gravity = cache.get_compute_pipeline(pipelines.gravity).unwrap();
            let position_step = cache.get_compute_pipeline(pipelines.position_step).unwrap();
            let velocity_step = cache.get_compute_pipeline(pipelines.velocity_step).unwrap();

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
        }

        // O(n) color pass — runs while paused so body colors stay in sync with mass/flash state.
        let colors = cache.get_compute_pipeline(pipelines.colors).unwrap();
        pass.set_bind_group(0, &bind_groups.colors, &[]);
        pass.set_pipeline(colors);
        pass.dispatch_workgroups(workgroups, 1, 1);

        Ok(())
    }
}
