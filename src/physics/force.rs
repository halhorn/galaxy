use bevy::prelude::*;
use std::f32::consts::PI;

/// Trait for computing accelerations on all bodies.
///
/// Implementations receive positions and masses of every body
/// and return one acceleration vector per body.
pub trait ForceCalculator: Send + Sync + 'static {
    fn calculate_accelerations(&self, positions: &[Vec3], masses: &[f32]) -> Vec<Vec3>;
}

/// Newtonian gravitational force: F = G·m₁·m₂ / (r² + ε²).
///
/// Uses a softening length ε to prevent singularity at r → 0.
pub struct NewtonianGravity {
    /// G in AU³/(M☉·yr²). Default: 4π² ≈ 39.478.
    pub gravitational_constant: f32,
    /// Softening length in AU.
    pub softening: f32,
}

impl Default for NewtonianGravity {
    fn default() -> Self {
        Self {
            gravitational_constant: 4.0 * PI * PI,
            softening: 0.01,
        }
    }
}

impl ForceCalculator for NewtonianGravity {
    fn calculate_accelerations(&self, positions: &[Vec3], masses: &[f32]) -> Vec<Vec3> {
        let n = positions.len();
        let mut accelerations = vec![Vec3::ZERO; n];
        let eps_sq = self.softening * self.softening;
        let g = self.gravitational_constant;

        for i in 0..n {
            for j in (i + 1)..n {
                let r = positions[j] - positions[i];
                let dist_sq = r.length_squared() + eps_sq;
                let inv_dist3 = g / (dist_sq * dist_sq.sqrt());

                accelerations[i] += r * inv_dist3 * masses[j];
                accelerations[j] -= r * inv_dist3 * masses[i];
            }
        }

        accelerations
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn two_body_acceleration_magnitude() {
        let gravity = NewtonianGravity {
            gravitational_constant: 4.0 * PI * PI,
            softening: 0.0,
        };
        // Two 1 M☉ bodies separated by 1 AU along x-axis
        let positions = vec![Vec3::ZERO, Vec3::new(1.0, 0.0, 0.0)];
        let masses = vec![1.0, 1.0];
        let accel = gravity.calculate_accelerations(&positions, &masses);

        // Expected: |a| = G·M/r² = 4π²
        let expected = 4.0 * PI * PI;
        assert!((accel[0].x - expected).abs() < 1e-4, "a0.x = {}", accel[0].x);
        assert!((accel[1].x + expected).abs() < 1e-4, "a1.x = {}", accel[1].x);
    }

    #[test]
    fn softening_prevents_divergence() {
        let gravity = NewtonianGravity {
            gravitational_constant: 4.0 * PI * PI,
            softening: 0.1,
        };
        // Two bodies at the same position
        let positions = vec![Vec3::ZERO, Vec3::ZERO];
        let masses = vec![1.0, 1.0];
        let accel = gravity.calculate_accelerations(&positions, &masses);

        // With softening, acceleration should be finite (zero because r=0)
        assert!(accel[0].length().is_finite());
        assert!(accel[1].length().is_finite());
    }

    #[test]
    fn acceleration_is_zero_for_single_body() {
        let gravity = NewtonianGravity::default();
        let positions = vec![Vec3::new(5.0, 3.0, 1.0)];
        let masses = vec![100.0];
        let accel = gravity.calculate_accelerations(&positions, &masses);

        assert_eq!(accel[0], Vec3::ZERO);
    }
}
