use std::f32::consts::PI;

/// Fixed number of simulated bodies (not ECS entities).
pub const BODY_COUNT: usize = 10_000;

pub const WORKGROUP_SIZE: u32 = 256;

/// Gravitational constant in AU³/(M☉·yr²).
pub const G: f32 = 4.0 * PI * PI;

/// Plummer softening length (AU).
pub const SOFTENING: f32 = 0.01;

pub const SOFTENING_SQ: f32 = SOFTENING * SOFTENING;

pub fn dispatch_workgroups() -> u32 {
    (BODY_COUNT as u32).div_ceil(WORKGROUP_SIZE)
}
