use super::constants::{BODY_COUNT, MIN_MASS, SUN_RADIUS_AU};

/// CPU-side simulation state (positions/velocities as xyz + padding w).
#[derive(Debug, Clone)]
pub struct BodyArrays {
    pub positions: Vec<[f32; 4]>,
    pub velocities: Vec<[f32; 4]>,
    pub masses: Vec<f32>,
    pub accelerations: Vec<[f32; 4]>,
    pub active_count: u32,
}

impl BodyArrays {
    pub fn with_capacity(active_count: u32) -> Self {
        let n = BODY_COUNT;
        Self {
            positions: vec![[0.0; 4]; n],
            velocities: vec![[0.0; 4]; n],
            masses: vec![0.0; n],
            accelerations: vec![[0.0; 4]; n],
            active_count,
        }
    }
}

#[inline]
pub fn is_active(mass: f32) -> bool {
    mass > MIN_MASS
}

/// Physical stellar radius in AU: `SUN_RADIUS_AU * mass^(1/3)`.
#[inline]
pub fn physical_radius(mass: f32) -> f32 {
    SUN_RADIUS_AU * mass.powf(1.0 / 3.0)
}

/// World-space sphere radius for rendering / picking (matches `bodies.wgsl`).
#[inline]
pub fn visual_radius(mass: f32, star_visual_scale: f32, min_star_visual_scale: f32) -> f32 {
    (physical_radius(mass) * star_visual_scale).max(min_star_visual_scale)
}
