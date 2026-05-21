use super::constants::{merge_inv_cell_size, G, MERGE_RADIUS_FACTOR, SOFTENING};

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
}
