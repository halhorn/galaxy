use super::constants::{BODY_COUNT, MIN_MASS};

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
