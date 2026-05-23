use std::f32::consts::PI;

/// Fixed number of simulated bodies (not ECS entities).
pub const BODY_COUNT: usize = 10_000;

pub const WORKGROUP_SIZE: u32 = 256;

/// Bodies with `mass <= MIN_MASS` are inactive (merged away).
pub const MIN_MASS: f32 = 1e-8;

/// Gravitational constant in AU³/(M☉·yr²).
pub const G_MIN: f32 = 1.0;
pub const G: f32 = 4.0 * PI * PI;
pub const G_MAX: f32 = 100.0;

/// Plummer softening length (AU).
pub const SOFTENING_MIN: f32 = 0.001;
pub const SOFTENING: f32 = 0.01;
pub const SOFTENING_MAX: f32 = 0.1;

/// Fraction of combined radii at which bodies merge (legacy `merger.rs`).
pub const MERGE_RADIUS_FACTOR_MIN: f32 = 0.00;
pub const MERGE_RADIUS_FACTOR: f32 = 0.00;
pub const MERGE_RADIUS_FACTOR_MAX: f32 = 1.0;

/// Spatial hash buckets for the merge pass.
pub const MERGE_BUCKET_COUNT: usize = 16_384;

/// Conservative max body radius (AU) for merge grid; large enough for merged stars.
pub const MERGE_MAX_RADIUS: f32 = 2.0;

pub fn merge_inv_cell_size(merge_radius_factor: f32) -> f32 {
    let cell_size = (2.0 * MERGE_MAX_RADIUS * merge_radius_factor).max(0.01);
    1.0 / cell_size
}

pub fn dispatch_workgroups() -> u32 {
    (BODY_COUNT as u32).div_ceil(WORKGROUP_SIZE)
}
