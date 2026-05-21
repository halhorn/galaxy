use std::f32::consts::PI;

/// Fixed number of simulated bodies (not ECS entities).
pub const BODY_COUNT: usize = 20_000;

pub const WORKGROUP_SIZE: u32 = 256;

/// Gravitational constant in AU³/(M☉·yr²).
pub const G: f32 = 4.0 * PI * PI;

/// Plummer softening length (AU).
pub const SOFTENING: f32 = 0.01;

pub const SOFTENING_SQ: f32 = SOFTENING * SOFTENING;

/// Bodies with `mass <= MIN_MASS` are inactive (merged away).
pub const MIN_MASS: f32 = 1e-8;

/// Fraction of combined radii at which bodies merge (legacy `merger.rs`).
pub const MERGE_RADIUS_FACTOR: f32 = 0.15;

/// Spatial hash buckets for the merge pass.
pub const MERGE_BUCKET_COUNT: usize = 16_384;

/// Conservative max body radius (AU) for merge grid; large enough for merged stars.
pub const MERGE_MAX_RADIUS: f32 = 2.0;

pub const MERGE_CELL_SIZE: f32 = (2.0 * MERGE_MAX_RADIUS * MERGE_RADIUS_FACTOR).max(0.01);

pub const MERGE_INV_CELL_SIZE: f32 = 1.0 / MERGE_CELL_SIZE;

pub fn dispatch_workgroups() -> u32 {
    (BODY_COUNT as u32).div_ceil(WORKGROUP_SIZE)
}
