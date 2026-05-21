use std::f32::consts::PI;

use super::body::BodyArrays;
use super::constants::BODY_COUNT;
use super::force::ForceLaw;
use super::physics::PhysicsSettings;
use super::rng::SimpleRng;

/// Initial-condition parameters for galaxy-style disk + central stars.
#[derive(Debug, Clone, PartialEq)]
pub struct InitialConditions {
    pub seed: u64,
    pub n_stars: u32,
    pub star_mass: f32,
    pub star_orbit_radius: f32,
    pub disk_radius_min: f32,
    pub disk_radius_max: f32,
    pub disk_r_min: f32,
    pub disk_r_max: f32,
    pub disk_height: f32,
    pub initial_v_perturbation: f32,
    pub active_count: u32,
}

impl Default for InitialConditions {
    fn default() -> Self {
        Self {
            seed: 0x6a8e_bc2f,
            n_stars: 2,
            star_mass: 1.0,
            star_orbit_radius: 10.0,
            disk_radius_min: 0.14,
            disk_radius_max: 0.36,
            disk_r_min: 5.0,
            disk_r_max: 60.0,
            disk_height: 0.5,
            initial_v_perturbation: 0.02,
            active_count: BODY_COUNT as u32,
        }
    }
}

pub fn generate_initial_state(
    ic: &InitialConditions,
    physics: &PhysicsSettings,
    force: &ForceLaw,
) -> BodyArrays {
    let mut rng = SimpleRng::new(ic.seed);
    let g = physics.g;
    let n_stars = ic.n_stars as usize;
    let active = ic.active_count as usize;

    let mut bodies = BodyArrays::with_capacity(ic.active_count);
    let mut index = 0usize;

    let chord_sum: f32 = (1..n_stars)
        .map(|k| 1.0 / (2.0 * (PI * k as f32 / n_stars as f32).sin()))
        .sum();
    let v_star = (g * ic.star_mass * chord_sum / ic.star_orbit_radius).sqrt();

    for i in 0..n_stars {
        let angle = i as f32 * 2.0 * PI / n_stars as f32;
        let position = [
            ic.star_orbit_radius * angle.cos(),
            0.0,
            ic.star_orbit_radius * angle.sin(),
        ];
        let velocity = [-angle.sin(), 0.0, angle.cos()];
        let velocity = scale3(velocity, v_star);
        bodies.positions[index] = [position[0], position[1], position[2], 0.0];
        bodies.velocities[index] = [velocity[0], velocity[1], velocity[2], 0.0];
        bodies.masses[index] = ic.star_mass;
        index += 1;
    }

    let central_mass = ic.star_mass * n_stars as f32;
    let n_disk = active.saturating_sub(index);

    struct DiskSeed {
        index: usize,
        r: f32,
        theta: f32,
    }

    let mut disk_seeds = Vec::with_capacity(n_disk);

    for _ in 0..n_disk {
        let u: f32 = rng.range(0.0, 1.0);
        let r = (ic.disk_r_min * ic.disk_r_min
            + u * (ic.disk_r_max * ic.disk_r_max - ic.disk_r_min * ic.disk_r_min))
            .sqrt();
        let theta: f32 = rng.range(0.0, 2.0 * PI);
        let height: f32 = rng.range(-ic.disk_height, ic.disk_height);

        let position = [r * theta.cos(), height, r * theta.sin()];
        bodies.positions[index] = [position[0], position[1], position[2], 0.0];
        let radius = rng.range(ic.disk_radius_min, ic.disk_radius_max);
        bodies.masses[index] = (radius / 0.5_f32).powi(3);
        disk_seeds.push(DiskSeed { index, r, theta });
        index += 1;
    }

    disk_seeds.sort_by(|a, b| a.r.partial_cmp(&b.r).unwrap_or(std::cmp::Ordering::Equal));

    let mut enclosed_mass = central_mass;
    for seed in disk_seeds {
        let r = seed.r.max(0.01);
        let v_circ = (g * enclosed_mass / r).sqrt();
        enclosed_mass += bodies.masses[seed.index];

        let vr = v_circ * rng.range(-ic.initial_v_perturbation, ic.initial_v_perturbation);
        let vt = v_circ * (1.0 + rng.range(-ic.initial_v_perturbation, ic.initial_v_perturbation));
        let vy = v_circ
            * rng.range(-ic.initial_v_perturbation, ic.initial_v_perturbation)
            * 0.1;
        let tangent = [-seed.theta.sin(), 0.0, seed.theta.cos()];
        let radial = [seed.theta.cos(), 0.0, seed.theta.sin()];
        let vel = add3(
            add3(scale3(tangent, vt), scale3(radial, vr)),
            [0.0, vy, 0.0],
        );
        bodies.velocities[seed.index] = [vel[0], vel[1], vel[2], 0.0];
    }

    debug_assert_eq!(index, active);
    bodies.accelerations = force.compute_accelerations(&bodies, physics);
    bodies
}

#[inline]
fn scale3(v: [f32; 3], s: f32) -> [f32; 3] {
    [v[0] * s, v[1] * s, v[2] * s]
}

#[inline]
fn add3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}
