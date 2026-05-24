use super::body::BodyArrays;
use super::constants::{
    BODY_COUNT, FORCE_COEFFICIENT_MAX, FORCE_COEFFICIENT_MIN, FORCE_EXPONENT_MAX,
    FORCE_EXPONENT_MIN, G, G_MAX, G_MIN, MIN_MASS,
};
use super::physics::PhysicsSettings;

pub const MAX_FORCE_TERMS: usize = 8;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ForceTerm {
    pub sign: i8,
    pub exponent: i32,
    pub coefficient: f32,
}

/// Distance-polynomial force law: acceleration from j onto i includes `sign * c * d^N * m_j` along r.
#[derive(Debug, Clone, PartialEq)]
pub struct ForceLaw {
    pub terms: [ForceTerm; MAX_FORCE_TERMS],
    pub term_count: u8,
}

impl ForceLaw {
    pub fn gravity_coefficient(&self) -> f32 {
        self.terms
            .iter()
            .take(self.term_count as usize)
            .find(|term| term.sign == 1 && term.exponent == -3)
            .map(|term| term.coefficient)
            .unwrap_or(G)
    }

    pub fn set_gravity_coefficient(&mut self, g: f32) {
        let g = g.clamp(G_MIN, G_MAX);
        if let Some(term) = self
            .terms
            .iter_mut()
            .take(self.term_count as usize)
            .find(|term| term.sign == 1 && term.exponent == -3)
        {
            term.coefficient = g;
        }
    }

    pub fn newtonian(g: f32) -> Self {
        let mut terms = empty_terms();
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

    pub fn preset_gravity_plus_repulsion(g: f32) -> Self {
        let mut terms = empty_terms();
        terms[0] = ForceTerm {
            sign: 1,
            exponent: -3,
            coefficient: g,
        };
        terms[1] = ForceTerm {
            sign: -1,
            exponent: -2,
            coefficient: 1.0,
        };
        Self {
            terms,
            term_count: 2,
        }
    }

    pub fn preset_repulsive() -> Self {
        let mut terms = empty_terms();
        terms[0] = ForceTerm {
            sign: -1,
            exponent: -1,
            coefficient: 1.0,
        };
        Self {
            terms,
            term_count: 1,
        }
    }

    pub fn is_valid(&self) -> bool {
        self.term_count > 0
            && self.terms[..self.term_count as usize].iter().all(|term| {
                (term.sign == 1 || term.sign == -1)
                    && term.coefficient > 0.0
                    && term.exponent >= FORCE_EXPONENT_MIN
                    && term.exponent <= FORCE_EXPONENT_MAX
            })
    }

    pub fn clamped(self) -> Self {
        let term_count = self.term_count.min(MAX_FORCE_TERMS as u8);
        let mut terms = empty_terms();
        for (i, term) in self.terms.iter().take(term_count as usize).enumerate() {
            terms[i] = ForceTerm {
                sign: if term.sign >= 0 { 1 } else { -1 },
                exponent: term
                    .exponent
                    .clamp(FORCE_EXPONENT_MIN, FORCE_EXPONENT_MAX),
                coefficient: term
                    .coefficient
                    .clamp(FORCE_COEFFICIENT_MIN, FORCE_COEFFICIENT_MAX),
            };
        }
        Self { terms, term_count }
    }

    pub fn display_string(&self) -> String {
        if self.term_count == 0 {
            return "(empty)".to_string();
        }

        self.terms
            .iter()
            .take(self.term_count as usize)
            .map(format_term)
            .collect::<Vec<_>>()
            .join(" ")
    }

    pub fn needs_softening_warning(&self) -> bool {
        self.terms
            .iter()
            .take(self.term_count as usize)
            .any(|term| term.exponent <= -2)
    }

    pub fn has_repulsive_terms(&self) -> bool {
        self.terms.iter().take(self.term_count as usize).any(|term| {
            (term.sign < 0 && term.coefficient > 0.0)
                || (term.sign > 0 && term.exponent > 0)
        })
    }

    /// Circular-orbit speed at radius `r` around enclosed mass `M`, consistent with this force law.
    pub fn circular_orbit_speed(&self, r: f32, enclosed_mass: f32, softening_sq: f32) -> f32 {
        let r = r.max(0.01);
        let pos_body = [r, 0.0, 0.0];
        let pos_center = [0.0, 0.0, 0.0];
        let acc = pair_acceleration(pos_body, pos_center, enclosed_mass, softening_sq, self);
        let centripetal = -acc[0];
        if centripetal <= 0.0 {
            return 0.0;
        }
        (r * centripetal).sqrt()
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

fn empty_terms() -> [ForceTerm; MAX_FORCE_TERMS] {
    [ForceTerm {
        sign: 0,
        exponent: 0,
        coefficient: 0.0,
    }; MAX_FORCE_TERMS]
}

fn format_term(term: &ForceTerm) -> String {
    // Display convention: exponent is one higher than computation (d^N along r ↔ d^(N-2) force),
    // and sign is inverted (approaching −, receding +).
    let sign_char = if term.sign >= 0 { '-' } else { '+' };
    let coeff = format_coefficient(term.coefficient);
    format!("{sign_char} {coeff} * d^{}", term.exponent + 1)
}

const DISPLAY_SIG_FIGS: i32 = 3;

fn format_coefficient(c: f32) -> String {
    let rounded = round_to_sig_figs(f64::from(c), DISPLAY_SIG_FIGS);
    if rounded == round_to_sig_figs(f64::from(G), DISPLAY_SIG_FIGS) {
        return "G".to_string();
    }
    if rounded == 1.0 {
        return "1".to_string();
    }
    format_f64_sig_figs(rounded, DISPLAY_SIG_FIGS)
}

fn round_to_sig_figs(x: f64, sig: i32) -> f64 {
    if x == 0.0 || !x.is_finite() {
        return x;
    }
    let log10 = x.abs().log10();
    if !log10.is_finite() {
        return x;
    }
    let scale = 10_f64.powf(sig as f64 - 1.0 - log10.floor());
    (x * scale).round() / scale
}

fn format_f64_sig_figs(x: f64, sig: i32) -> String {
    let rounded = round_to_sig_figs(x, sig);
    if rounded == 0.0 {
        return "0".to_string();
    }
    let m_i = rounded.abs().log10().floor() as i32;
    let frac_digits = ((sig - 1) - m_i).clamp(0, 20) as usize;
    let rendered = format!("{rounded:.frac_digits$}");
    trim_trailing_fraction_zeros(&rendered)
}

fn trim_trailing_fraction_zeros(s: &str) -> String {
    if !s.contains('.') {
        return s.to_string();
    }
    let s = s.trim_end_matches('0').trim_end_matches('.');
    if s.is_empty() || s == "-" {
        "0".to_string()
    } else {
        s.to_string()
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

    #[test]
    fn display_string_formats_newtonian() {
        let force = ForceLaw::newtonian(G);
        assert_eq!(force.display_string(), "- G * d^-2");
    }

    #[test]
    fn display_string_uses_three_significant_figures() {
        let mut terms = empty_terms();
        terms[0] = ForceTerm {
            sign: 1,
            exponent: -3,
            coefficient: 12.3456,
        };
        let force = ForceLaw {
            terms,
            term_count: 1,
        };
        assert_eq!(force.display_string(), "- 12.3 * d^-2");

        terms[0].coefficient = 0.00123456;
        let force = ForceLaw {
            terms,
            term_count: 1,
        };
        assert_eq!(force.display_string(), "- 0.00123 * d^-2");
    }

    #[test]
    fn circular_orbit_speed_matches_newtonian() {
        let force = ForceLaw::newtonian(G);
        let r = 10.0;
        let mass = 100.0;
        let v = force.circular_orbit_speed(r, mass, 0.0);
        let expected = (G * mass / r).sqrt();
        assert!((v - expected).abs() < 1e-4);
    }

    #[test]
    fn empty_force_law_is_invalid() {
        let force = ForceLaw {
            terms: empty_terms(),
            term_count: 0,
        };
        assert!(!force.is_valid());
    }
}
