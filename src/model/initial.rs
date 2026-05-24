use std::f32::consts::PI;

use super::body::BodyArrays;
use super::constants::{
    ACTIVE_COUNT, ACTIVE_COUNT_MAX, ACTIVE_COUNT_MIN, BODY_COUNT, DISK_ELEVATION_DEG,
    DISK_ELEVATION_DEG_MAX, DISK_MASS_LIMIT_MAX,
    DISK_MASS_LIMIT_MIN, DISK_MASS_MAX, DISK_MASS_MIN, DISK_R_INNER, DISK_R_MAX, DISK_R_MIN,
    DISK_R_OUTER, MIN_MASS, N_STARS,
    N_STARS_MAX, N_STARS_MIN, SEED, SEED_MAX, STAR_MASS, STAR_MASS_MAX, STAR_MASS_MIN, V_PERTURBATION,
    V_PERTURBATION_MAX,
};
use super::force::{pair_acceleration, ForceLaw};
use super::physics::PhysicsSettings;
use super::rng::SimpleRng;

/// Initial-condition parameters for galaxy-style disk + central stars.
#[derive(Debug, Clone, PartialEq)]
pub struct InitialConditions {
    pub seed: u64,
    pub n_stars: u32,
    pub star_mass: f32,
    pub star_orbit_radius: f32,
    pub disk_mass_min: f32,
    pub disk_mass_max: f32,
    pub disk_r_min: f32,
    pub disk_r_max: f32,
    pub disk_elevation_deg: f32,
    pub initial_v_perturbation: f32,
    pub active_count: u32,
}

impl Default for InitialConditions {
    fn default() -> Self {
        Self {
            seed: SEED,
            n_stars: N_STARS,
            star_mass: STAR_MASS,
            star_orbit_radius: 3.0,
            disk_mass_min: DISK_MASS_MIN,
            disk_mass_max: DISK_MASS_MAX,
            disk_r_min: DISK_R_INNER,
            disk_r_max: DISK_R_OUTER,
            disk_elevation_deg: DISK_ELEVATION_DEG,
            initial_v_perturbation: V_PERTURBATION,
            active_count: ACTIVE_COUNT,
        }
    }
}

impl InitialConditions {
    pub fn clamped(self) -> Self {
        let n_stars = self.n_stars.clamp(N_STARS_MIN, N_STARS_MAX);
        let active_count = self
            .active_count
            .clamp(ACTIVE_COUNT_MIN.max(n_stars), ACTIVE_COUNT_MAX);
        let disk_r_min = self.disk_r_min.clamp(DISK_R_MIN, DISK_R_MAX);
        let disk_r_max = self.disk_r_max.clamp(disk_r_min + 0.1, DISK_R_MAX);
        let disk_mass_min = self
            .disk_mass_min
            .clamp(DISK_MASS_LIMIT_MIN, DISK_MASS_LIMIT_MAX);
        let disk_mass_max = self
            .disk_mass_max
            .clamp(disk_mass_min + MIN_MASS, DISK_MASS_LIMIT_MAX);
        Self {
            seed: self.seed.min(SEED_MAX),
            n_stars,
            star_mass: self.star_mass.clamp(STAR_MASS_MIN, STAR_MASS_MAX),
            star_orbit_radius: self.star_orbit_radius.max(0.1),
            disk_mass_min,
            disk_mass_max,
            disk_r_min,
            disk_r_max,
            disk_elevation_deg: self
                .disk_elevation_deg
                .clamp(0.0, DISK_ELEVATION_DEG_MAX),
            initial_v_perturbation: self
                .initial_v_perturbation
                .clamp(0.0, V_PERTURBATION_MAX),
            active_count,
        }
    }
}

pub fn generate_initial_state(
    ic: &InitialConditions,
    physics: &PhysicsSettings,
    force: &ForceLaw,
) -> BodyArrays {
    let ic = ic.clone().clamped();
    let mut rng = SimpleRng::new(ic.seed);
    let softening_sq = physics.softening_sq();
    let n_stars = ic.n_stars as usize;
    let active = ic.active_count as usize;

    let mut bodies = BodyArrays::with_capacity(ic.active_count);
    let mut index = place_central_stars(&ic, force, softening_sq, &mut bodies);

    let central_mass = ic.star_mass * n_stars as f32;
    let n_disk = active.saturating_sub(index);
    let disk_r_min = effective_disk_r_min(&ic, n_stars);

    struct DiskSeed {
        index: usize,
        r: f32,
    }

    let mut disk_seeds = Vec::with_capacity(n_disk);

    for _ in 0..n_disk {
        let u: f32 = rng.range(0.0, 1.0);
        let r = (disk_r_min * disk_r_min
            + u * (ic.disk_r_max * ic.disk_r_max - disk_r_min * disk_r_min))
            .sqrt();
        let theta: f32 = rng.range(0.0, 2.0 * PI);
        let phi_deg = rng.range(-ic.disk_elevation_deg, ic.disk_elevation_deg);
        let phi_rad = phi_deg.to_radians();
        let r_horiz = r * phi_rad.cos();

        let position = [
            r_horiz * theta.cos(),
            r * phi_rad.sin(),
            r_horiz * theta.sin(),
        ];
        bodies.positions[index] = [position[0], position[1], position[2], 0.0];
        bodies.masses[index] = sample_disk_mass(&mut rng, &ic);
        let r_3d = (position[0] * position[0]
            + position[1] * position[1]
            + position[2] * position[2])
            .sqrt()
            .max(0.01);
        disk_seeds.push(DiskSeed { index, r: r_3d });
        index += 1;
    }

    disk_seeds.sort_by(|a, b| a.r.partial_cmp(&b.r).unwrap_or(std::cmp::Ordering::Equal));

    let mut enclosed_mass = central_mass;
    for seed in disk_seeds {
        let r = seed.r;
        let v_circ = force.circular_orbit_speed(r, enclosed_mass, softening_sq);
        enclosed_mass += bodies.masses[seed.index];

        let pos = [
            bodies.positions[seed.index][0],
            bodies.positions[seed.index][1],
            bodies.positions[seed.index][2],
        ];
        let base_vel = circular_velocity_about_y(pos, v_circ);
        let perturb_mag =
            v_circ * rng.range(-ic.initial_v_perturbation, ic.initial_v_perturbation);
        let vel = add3(base_vel, scale3(random_unit_vector(&mut rng), perturb_mag));
        bodies.velocities[seed.index] = [vel[0], vel[1], vel[2], 0.0];
    }

    debug_assert_eq!(index, active);

    for slot in active..BODY_COUNT {
        bodies.positions[slot] = [0.0; 4];
        bodies.velocities[slot] = [0.0; 4];
        bodies.masses[slot] = 0.0;
    }

    bodies.accelerations = force.compute_accelerations(&bodies, physics);
    bodies
}

fn sample_disk_mass(rng: &mut SimpleRng, ic: &InitialConditions) -> f32 {
    rng.range(ic.disk_mass_min, ic.disk_mass_max)
        .max(MIN_MASS)
}

/// Place one or more massive stars at the galaxy center.
/// `n_stars == 1` → single bulge star at the origin; `n >= 2` → equal-mass ring binary/multiple.
fn place_central_stars(
    ic: &InitialConditions,
    force: &ForceLaw,
    softening_sq: f32,
    bodies: &mut BodyArrays,
) -> usize {
    let n_stars = ic.n_stars as usize;
    if n_stars == 0 {
        return 0;
    }

    if n_stars == 1 {
        bodies.positions[0] = [0.0, 0.0, 0.0, 0.0];
        bodies.velocities[0] = [0.0, 0.0, 0.0, 0.0];
        bodies.masses[0] = ic.star_mass;
        return 1;
    }

    let v_star = central_ring_orbit_speed(
        n_stars,
        ic.star_mass,
        ic.star_orbit_radius,
        force,
        softening_sq,
    );

    for i in 0..n_stars {
        let angle = i as f32 * 2.0 * PI / n_stars as f32;
        let position = [
            ic.star_orbit_radius * angle.cos(),
            0.0,
            ic.star_orbit_radius * angle.sin(),
        ];
        let tangent = [-angle.sin(), 0.0, angle.cos()];
        let velocity = scale3(tangent, v_star);
        bodies.positions[i] = [position[0], position[1], position[2], 0.0];
        bodies.velocities[i] = [velocity[0], velocity[1], velocity[2], 0.0];
        bodies.masses[i] = ic.star_mass;
    }

    n_stars
}

/// Orbit speed for equal-mass stars on a regular polygon, from pairwise forces under `force`.
fn central_ring_orbit_speed(
    n_stars: usize,
    star_mass: f32,
    orbit_radius: f32,
    force: &ForceLaw,
    softening_sq: f32,
) -> f32 {
    let orbit_radius = orbit_radius.max(0.01);
    let pos_i = [orbit_radius, 0.0, 0.0];
    let mut acc = [0.0f32; 3];
    for k in 1..n_stars {
        let angle = k as f32 * 2.0 * PI / n_stars as f32;
        let pos_j = [
            orbit_radius * angle.cos(),
            0.0,
            orbit_radius * angle.sin(),
        ];
        acc = add3(
            acc,
            pair_acceleration(pos_i, pos_j, star_mass, softening_sq, force),
        );
    }
    let centripetal = -acc[0];
    if centripetal <= 0.0 {
        return 0.0;
    }
    (orbit_radius * centripetal).sqrt()
}

/// Median 3D radius of an area-uniform disk annulus from initial-condition sliders.
/// Used as a fallback reference radius when live body positions are not available yet.
pub fn nominal_disk_median_radius(ic: &InitialConditions) -> f32 {
    let ic = ic.clone().clamped();
    let r_min = effective_disk_r_min(&ic, ic.n_stars as usize);
    crate::model::force_coefficient::nominal_disk_median_radius_from_annulus(
        r_min,
        ic.disk_r_max,
    )
}

/// Keep the disk outside the central star system so bulge stars stay visible.
fn effective_disk_r_min(ic: &InitialConditions, n_stars: usize) -> f32 {
    match n_stars {
        0 => ic.disk_r_min,
        1 => ic.disk_r_min.max(1.0),
        _ => ic.disk_r_min.max(ic.star_orbit_radius * 1.5),
    }
}

/// Tangential velocity for a circle centered at the origin, rotating about the y-axis.
fn circular_velocity_about_y(pos: [f32; 3], speed: f32) -> [f32; 3] {
    let tangent = cross3([0.0, 1.0, 0.0], pos);
    let r_horiz_sq = tangent[0] * tangent[0] + tangent[2] * tangent[2];
    if r_horiz_sq > 1e-8 {
        scale3(tangent, speed * r_horiz_sq.sqrt().recip())
    } else {
        // On the y-axis; orbit in the x direction.
        [speed, 0.0, 0.0]
    }
}

#[inline]
fn cross3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

/// Uniform random unit vector (Marsaglia method).
fn random_unit_vector(rng: &mut SimpleRng) -> [f32; 3] {
    loop {
        let x = rng.range(-1.0, 1.0);
        let y = rng.range(-1.0, 1.0);
        let z = rng.range(-1.0, 1.0);
        let len_sq = x * x + y * y + z * z;
        if len_sq > 1e-8 && len_sq <= 1.0 {
            let inv_len = len_sq.sqrt().recip();
            return [x * inv_len, y * inv_len, z * inv_len];
        }
    }
}

#[inline]
fn scale3(v: [f32; 3], s: f32) -> [f32; 3] {
    [v[0] * s, v[1] * s, v[2] * s]
}

#[inline]
fn add3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::constants::{ACTIVE_COUNT, ACTIVE_COUNT_MAX, BODY_COUNT, G};
    use crate::model::PhysicsSettings;

    #[test]
    fn clamped_enforces_active_count_bounds() {
        let ic = InitialConditions {
            n_stars: 4,
            active_count: 1,
            ..InitialConditions::default()
        };
        let clamped = ic.clamped();
        assert_eq!(clamped.n_stars, 4);
        assert_eq!(clamped.active_count, 4);
    }

    #[test]
    fn default_active_count_is_active_count_constant() {
        assert_eq!(InitialConditions::default().active_count, ACTIVE_COUNT);
    }

    #[test]
    fn clamped_caps_active_count_at_max() {
        let ic = InitialConditions {
            active_count: ACTIVE_COUNT_MAX + 1,
            ..InitialConditions::default()
        };
        assert_eq!(ic.clamped().active_count, ACTIVE_COUNT_MAX);
    }

    #[test]
    fn generate_at_active_count_max_does_not_panic() {
        let ic = InitialConditions {
            n_stars: 1,
            active_count: ACTIVE_COUNT_MAX,
            ..InitialConditions::default()
        };
        let physics = PhysicsSettings::default();
        let force = ForceLaw::newtonian(G);
        let bodies = generate_initial_state(&ic, &physics, &force);
        assert_eq!(bodies.active_count, ACTIVE_COUNT_MAX);
        assert_eq!(bodies.masses.len(), BODY_COUNT);
        assert!(bodies.masses[ACTIVE_COUNT_MAX as usize - 1] > MIN_MASS);
        assert_eq!(bodies.masses[0], ic.star_mass);
    }

    #[test]
    fn same_seed_produces_same_disk_layout() {
        let ic = InitialConditions {
            seed: 42,
            n_stars: 2,
            active_count: 128,
            ..InitialConditions::default()
        };
        let physics = PhysicsSettings::default();
        let force = ForceLaw::newtonian(G);
        let a = generate_initial_state(&ic, &physics, &force);
        let b = generate_initial_state(&ic, &physics, &force);
        assert_eq!(a.positions, b.positions);
        assert_eq!(a.velocities, b.velocities);
        assert_eq!(a.masses, b.masses);
    }

    #[test]
    fn single_central_star_sits_at_origin() {
        let ic = InitialConditions {
            n_stars: 1,
            active_count: 32,
            ..InitialConditions::default()
        };
        let physics = PhysicsSettings::default();
        let force = ForceLaw::newtonian(G);
        let bodies = generate_initial_state(&ic, &physics, &force);
        assert_eq!(bodies.masses[0], ic.star_mass);
        assert_eq!(bodies.positions[0][0], 0.0);
        assert_eq!(bodies.positions[0][1], 0.0);
        assert_eq!(bodies.positions[0][2], 0.0);
        for slot in 1..ic.active_count as usize {
            assert!(bodies.masses[slot] < ic.star_mass);
        }
    }

    #[test]
    fn multiple_central_stars_use_first_slots() {
        let ic = InitialConditions {
            n_stars: 4,
            active_count: 64,
            ..InitialConditions::default()
        };
        let physics = PhysicsSettings::default();
        let force = ForceLaw::newtonian(G);
        let bodies = generate_initial_state(&ic, &physics, &force);
        for slot in 0..4 {
            assert_eq!(bodies.masses[slot], ic.star_mass);
        }
        for slot in 4..ic.active_count as usize {
            assert!(bodies.masses[slot] < ic.star_mass);
        }
    }

    #[test]
    fn disk_starts_outside_central_ring() {
        let ic = InitialConditions {
            n_stars: 2,
            disk_r_min: 1.0,
            active_count: 32,
            ..InitialConditions::default()
        };
        let physics = PhysicsSettings::default();
        let force = ForceLaw::newtonian(G);
        let bodies = generate_initial_state(&ic, &physics, &force);
        let min_disk_r = (0..ic.active_count as usize)
            .filter(|&i| i >= 2)
            .map(|i| {
                let p = bodies.positions[i];
                (p[0] * p[0] + p[2] * p[2]).sqrt()
            })
            .fold(f32::INFINITY, f32::min);
        assert!(min_disk_r >= ic.star_orbit_radius * 1.5 - 0.01);
    }

    #[test]
    fn disk_masses_are_uniform_in_range() {
        let ic = InitialConditions {
            n_stars: 0,
            disk_mass_min: 0.05,
            disk_mass_max: 0.15,
            active_count: 256,
            ..InitialConditions::default()
        };
        let physics = PhysicsSettings::default();
        let force = ForceLaw::newtonian(G);
        let bodies = generate_initial_state(&ic, &physics, &force);
        for slot in 0..ic.active_count as usize {
            assert!(bodies.masses[slot] >= 0.05 - 1e-6);
            assert!(bodies.masses[slot] <= 0.15 + 1e-6);
        }
    }

    #[test]
    fn disk_uses_force_law_for_circular_speed() {
        let ic = InitialConditions {
            n_stars: 1,
            active_count: 64,
            seed: 7,
            ..InitialConditions::default()
        };
        let physics = PhysicsSettings::default();
        let newton = ForceLaw::newtonian(G);
        let repulsive = ForceLaw::preset_gravity_plus_repulsion(G);
        let newton_bodies = generate_initial_state(&ic, &physics, &newton);
        let repulsive_bodies = generate_initial_state(&ic, &physics, &repulsive);
        let newton_speeds: f32 = (1..ic.active_count as usize)
            .map(|i| {
                let v = newton_bodies.velocities[i];
                (v[0] * v[0] + v[2] * v[2]).sqrt()
            })
            .sum();
        let repulsive_speeds: f32 = (1..ic.active_count as usize)
            .map(|i| {
                let v = repulsive_bodies.velocities[i];
                (v[0] * v[0] + v[2] * v[2]).sqrt()
            })
            .sum();
        assert!(repulsive_speeds < newton_speeds);
    }

    #[test]
    fn inactive_slots_are_zeroed() {
        let ic = InitialConditions {
            active_count: 16,
            ..InitialConditions::default()
        };
        let physics = PhysicsSettings::default();
        let force = ForceLaw::newtonian(G);
        let bodies = generate_initial_state(&ic, &physics, &force);
        for slot in ic.active_count as usize..BODY_COUNT {
            assert_eq!(bodies.masses[slot], 0.0);
            assert_eq!(bodies.positions[slot], [0.0; 4]);
            assert_eq!(bodies.velocities[slot], [0.0; 4]);
        }
    }
}
