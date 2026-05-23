use super::constants::{
    MERGE_MAX_RADIUS, MERGE_RADIUS_FACTOR, MERGE_RADIUS_FACTOR_MAX, MERGE_RADIUS_FACTOR_MIN,
    SOFTENING, SOFTENING_MAX, SOFTENING_MIN,
};

/// Runtime physics parameters (defaults match legacy compile-time constants).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PhysicsSettings {
    pub softening: f32,
    pub merge_radius_factor: f32,
}

impl Default for PhysicsSettings {
    fn default() -> Self {
        Self {
            softening: SOFTENING,
            merge_radius_factor: MERGE_RADIUS_FACTOR,
        }
    }
}

impl PhysicsSettings {
    pub fn softening_sq(&self) -> f32 {
        self.softening * self.softening
    }

    pub fn merge_inv_cell_size(&self) -> f32 {
        let cell_size = (2.0 * MERGE_MAX_RADIUS * self.merge_radius_factor).max(0.01);
        1.0 / cell_size
    }

    pub fn clamped(self) -> Self {
        Self {
            softening: self.softening.clamp(SOFTENING_MIN, SOFTENING_MAX),
            merge_radius_factor: self
                .merge_radius_factor
                .clamp(MERGE_RADIUS_FACTOR_MIN, MERGE_RADIUS_FACTOR_MAX),
        }
    }
}
