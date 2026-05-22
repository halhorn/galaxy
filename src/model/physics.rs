use super::constants::{
    merge_inv_cell_size, G, G_MAX, G_MIN, MERGE_RADIUS_FACTOR, MERGE_RADIUS_FACTOR_MAX,
    MERGE_RADIUS_FACTOR_MIN, SOFTENING, SOFTENING_MAX, SOFTENING_MIN,
};

/// Runtime physics parameters (defaults match legacy compile-time constants).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PhysicsSettings {
    pub g: f32,
    pub softening: f32,
    pub merge_radius_factor: f32,
}

impl Default for PhysicsSettings {
    fn default() -> Self {
        Self {
            g: G,
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
        merge_inv_cell_size(self.merge_radius_factor)
    }

    pub fn clamped(self) -> Self {
        Self {
            g: self.g.clamp(G_MIN, G_MAX),
            softening: self.softening.clamp(SOFTENING_MIN, SOFTENING_MAX),
            merge_radius_factor: self
                .merge_radius_factor
                .clamp(MERGE_RADIUS_FACTOR_MIN, MERGE_RADIUS_FACTOR_MAX),
        }
    }
}
