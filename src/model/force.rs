use super::body::BodyArrays;
use super::constants::{BODY_COUNT, MIN_MASS};
use super::physics::PhysicsSettings;

pub const MAX_FORCE_TERMS: usize = 8;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ForceTerm {
    pub sign: i8,
    pub exponent: i32,
    pub coefficient: f32,
}

/// Distance-polynomial force law: acceleration from j onto i includes `sign * c * d^N * m_j` along r̂.
#[derive(Debug, Clone, PartialEq)]
pub struct ForceLaw {
    pub terms: [ForceTerm; MAX_FORCE_TERMS],
    pub term_count: u8,
}

impl ForceLaw {
    pub fn newtonian(g: f32) -> Self {
        let mut terms = [ForceTerm {
            sign: 0,
            exponent: 0,
            coefficient: 0.0,
        }; MAX_FORCE_TERMS];
        terms[0] = ForceTerm {
            sign: 1,
            exponent: -3,
            coefficient: g,
        };
        Self {
            terms,
            term_count: 1,
        }
    }

    pub fn compute_accelerations(
        &self,
        bodies: &BodyArrays,
        physics: &PhysicsSettings,
    ) -> Vec<[f32; 4]> {
        let n = bodies.active_count as usize;
        let softening_sq = physics.softening_sq();
        let mut accelerations = vec![[0.0; 4]; BODY_COUNT];

        for (i, acc) in accelerations.iter_mut().enumerate().take(n) {
            let pos_i = truncate3(bodies.positions[i]);
            let mut sum = [0.0f32; 3];
            for j in 0..n {
                if i == j || bodies.masses[j] <= MIN_MASS {
                    continue;
                }
                let pos_j = truncate3(bodies.positions[j]);
                sum = add3(
                    sum,
                    pair_acceleration(
                        pos_i,
                        pos_j,
                        bodies.masses[j],
                        softening_sq,
                        self,
                    ),
                );
            }
            *acc = [sum[0], sum[1], sum[2], 0.0];
        }

        accelerations
    }
}

/// Acceleration contribution on `pos_i` from body j at `pos_j`.
pub fn pair_acceleration(
    pos_i: [f32; 3],
    pos_j: [f32; 3],
    mass_j: f32,
    softening_sq: f32,
    force: &ForceLaw,
) -> [f32; 3] {
    let r = sub3(pos_j, pos_i);
    let dist_sq = dot3(r, r) + softening_sq;
    let d = dist_sq.sqrt();
    let mut acc = [0.0f32; 3];

    for term in force.terms.iter().take(force.term_count as usize) {
        if term.coefficient == 0.0 {
            continue;
        }
        let sign = term.sign as f32;
        let scalar = sign * term.coefficient * mass_j * d.powi(term.exponent);
        acc = add3(acc, scale3(r, scalar));
    }

    acc
}

#[inline]
fn truncate3(v: [f32; 4]) -> [f32; 3] {
    [v[0], v[1], v[2]]
}

#[inline]
fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

#[inline]
fn add3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

#[inline]
fn scale3(v: [f32; 3], s: f32) -> [f32; 3] {
    [v[0] * s, v[1] * s, v[2] * s]
}

#[inline]
fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::constants::G;

    #[test]
    fn newtonian_pair_matches_inverse_square() {
        let force = ForceLaw::newtonian(G);
        let softening_sq = 0.0;
        let pos_i = [0.0, 0.0, 0.0];
        let pos_j = [2.0, 0.0, 0.0];
        let mass_j = 1.0;
        let acc = pair_acceleration(pos_i, pos_j, mass_j, softening_sq, &force);
        let expected = G * mass_j / 4.0;
        assert!((acc[0] - expected).abs() < 1e-5);
        assert!(acc[1].abs() < 1e-6);
        assert!(acc[2].abs() < 1e-6);
    }
}
