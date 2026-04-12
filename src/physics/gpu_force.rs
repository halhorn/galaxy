use bevy::prelude::*;
use bytemuck::{Pod, Zeroable};
use std::f32::consts::PI;
use wgpu::util::DeviceExt;

use super::force::ForceCalculator;

const MAX_BODIES: usize = 65536;
const WORKGROUP_SIZE: u32 = 256;
const SHADER_SOURCE: &str = include_str!("shaders/gravity.wgsl");

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct GpuParams {
    n: u32,
    g: f32,
    softening_sq: f32,
    _pad: u32, // 16-byte alignment for uniform
}

/// GPU-accelerated Newtonian gravity using wgpu compute shaders.
pub struct GpuForceCalculator {
    device: wgpu::Device,
    queue: wgpu::Queue,
    pipeline: wgpu::ComputePipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    gravitational_constant: f32,
    softening: f32,
}

impl GpuForceCalculator {
    /// Attempt to create a GPU force calculator. Returns None if GPU is unavailable.
    pub fn try_new(gravitational_constant: f32, softening: f32) -> Option<Self> {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..default()
        });

        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            ..default()
        }))
        .ok()?;

        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: Some("galaxy-compute"),
                ..default()
            },
        ))
        .ok()?;

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("gravity-shader"),
            source: wgpu::ShaderSource::Wgsl(SHADER_SOURCE.into()),
        });

        let bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("gravity-bind-group-layout"),
                entries: &[
                    // positions: storage read
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    // masses: storage read
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    // accelerations: storage read_write
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    // params: uniform
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("gravity-pipeline-layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("gravity-pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: Some("main"),
            compilation_options: default(),
            cache: None,
        });

        info!("GPU force calculator initialized: {}", adapter.get_info().name);

        Some(Self {
            device,
            queue,
            pipeline,
            bind_group_layout,
            gravitational_constant,
            softening,
        })
    }

    /// Create with default Newtonian gravity parameters (G=4π², ε=0.01).
    pub fn try_default() -> Option<Self> {
        Self::try_new(4.0 * PI * PI, 0.01)
    }
}

impl ForceCalculator for GpuForceCalculator {
    fn calculate_accelerations(&self, positions: &[Vec3], masses: &[f32]) -> Vec<Vec3> {
        let n = positions.len();
        assert!(n <= MAX_BODIES, "Too many bodies for GPU: {n} > {MAX_BODIES}");

        if n == 0 {
            return vec![];
        }

        // Pack Vec3 → [f32; 4] (vec4 with w=0)
        let positions_packed: Vec<[f32; 4]> =
            positions.iter().map(|p| [p.x, p.y, p.z, 0.0]).collect();

        let params = GpuParams {
            n: n as u32,
            g: self.gravitational_constant,
            softening_sq: self.softening * self.softening,
            _pad: 0,
        };

        // Create buffers
        let pos_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("positions"),
                contents: bytemuck::cast_slice(&positions_packed),
                usage: wgpu::BufferUsages::STORAGE,
            });

        let mass_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("masses"),
                contents: bytemuck::cast_slice(masses),
                usage: wgpu::BufferUsages::STORAGE,
            });

        let accel_size = (n * 16) as u64; // n × vec4<f32>
        let accel_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("accelerations"),
            size: accel_size,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        let staging_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("staging"),
            size: accel_size,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let params_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("params"),
                contents: bytemuck::bytes_of(&params),
                usage: wgpu::BufferUsages::UNIFORM,
            });

        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("gravity-bind-group"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: pos_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: mass_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: accel_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: params_buffer.as_entire_binding(),
                },
            ],
        });

        // Dispatch compute
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("gravity-encoder"),
            });

        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("gravity-pass"),
                timestamp_writes: None,
            });
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            let workgroups = (n as u32).div_ceil(WORKGROUP_SIZE);
            pass.dispatch_workgroups(workgroups, 1, 1);
        }

        encoder.copy_buffer_to_buffer(&accel_buffer, 0, &staging_buffer, 0, accel_size);
        self.queue.submit(Some(encoder.finish()));

        // Read back results
        let slice = staging_buffer.slice(..);
        slice.map_async(wgpu::MapMode::Read, |_| {});
        let _ = self.device.poll(wgpu::PollType::Wait {
            submission_index: None,
            timeout: None,
        });

        let data = slice.get_mapped_range();
        let result_packed: &[[f32; 4]] = bytemuck::cast_slice(&data);

        let accelerations: Vec<Vec3> = result_packed
            .iter()
            .map(|a| Vec3::new(a[0], a[1], a[2]))
            .collect();

        drop(data);
        staging_buffer.unmap();

        accelerations
    }
}
