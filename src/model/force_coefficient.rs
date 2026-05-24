//! Pure helpers for choosing force-term coefficients and reference radii.
//!
//! When the force-law exponent changes during a live simulation, positions and
//! velocities stay fixed while acceleration jumps. Rescaling the coefficient at
//! a reference radius keeps radial acceleration roughly continuous at the typical
//! disk scale and reduces stars flying off.

use super::constants::{FORCE_COEFFICIENT_MAX, FORCE_COEFFICIENT_MIN, MIN_MASS, NEW_FORCE_TERM_COEFFICIENT};

/// Default coefficient for a newly added force term (small perturbation).
pub fn default_new_term_coefficient() -> f32 {
    NEW_FORCE_TERM_COEFFICIENT
}

/// Median radius of an area-uniform disk annulus `[r_min, r_max]`.
pub fn nominal_disk_median_radius_from_annulus(r_min: f32, r_max: f32) -> f32 {
    (0.5 * (r_min * r_min + r_max * r_max)).sqrt()
}

/// Median 3D radius of active disk stars, excluding central bulge slots and merged bodies.
pub fn median_disk_radius(
    positions: &[[f32; 3]],
    masses: &[f32],
    central_star_count: usize,
) -> Option<f32> {
    let n = positions.len().min(masses.len());
    let mut radii: Vec<f32> = (central_star_count..n)
        .filter(|&i| masses[i] > MIN_MASS)
        .map(|i| {
            let p = positions[i];
            (p[0] * p[0] + p[1] * p[1] + p[2] * p[2]).sqrt()
        })
        .filter(|r| r.is_finite() && *r > 0.0)
        .collect();

    if radii.is_empty() {
        return None;
    }

    radii.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let mid = radii.len() / 2;
    if radii.len() % 2 == 1 {
        Some(radii[mid])
    } else {
        Some(0.5 * (radii[mid - 1] + radii[mid]))
    }
}

/// Rescale a term coefficient when its distance exponent changes.
///
/// For a single term, radial acceleration scales as `c * r^(N+1)`.
/// Holding acceleration fixed at `r_ref` gives `c_new = c_old * r_ref^(N_old - N_new)`.
pub fn rescale_coefficient_for_exponent_change(
    coefficient: f32,
    old_exponent: i32,
    new_exponent: i32,
    r_ref: f32,
) -> f32 {
    if old_exponent == new_exponent {
        return coefficient;
    }
    let r_ref = r_ref.max(0.01);
    let scaled = coefficient * r_ref.powi(old_exponent - new_exponent);
    scaled.clamp(FORCE_COEFFICIENT_MIN, FORCE_COEFFICIENT_MAX)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::constants::G;

    #[test]
    fn rescale_preserves_newtonian_acceleration_at_r_ref() {
        let r_ref = 10.0;
        let n_new = -2;
        let c_new = rescale_coefficient_for_exponent_change(G, -3, n_new, r_ref);
        let a_old = G * r_ref.powi(-2);
        let a_new = c_new * r_ref.powi(n_new + 1);
        assert!((a_old - a_new).abs() < 1e-4);
    }

    #[test]
    fn rescale_is_unchanged_when_exponent_matches() {
        assert_eq!(
            rescale_coefficient_for_exponent_change(12.0, -3, -3, 5.0),
            12.0
        );
    }

    #[test]
    fn median_disk_radius_ignores_central_stars() {
        let positions = [[0.0, 0.0, 0.0], [4.0, 0.0, 0.0], [10.0, 0.0, 0.0]];
        let masses = [100.0, 1.0, 1.0];
        let median = median_disk_radius(&positions, &masses, 1).unwrap();
        assert!((median - 7.0).abs() < 1e-5);
    }

    #[test]
    fn nominal_annulus_median_matches_area_uniform_disk() {
        let median = nominal_disk_median_radius_from_annulus(0.01, 60.0);
        assert!((median - 42.4264).abs() < 1e-3);
    }
}
