use bevy::prelude::*;
use bevy::render::storage::ShaderStorageBuffer;
use std::f32::consts::PI;

use super::buffers::SimulationGpuBuffers;
use super::constants::*;
use super::selection::SimulationCpuSnapshot;

/// Disk + binary star initial conditions (one-shot CPU upload).
pub fn spawn_initial_state(
    mut commands: Commands,
    mut buffers: ResMut<Assets<ShaderStorageBuffer>>,
) {
    let mut rng = SimpleRng::new(0x6a8e_bc2f);
    let g = G;

    let n_stars: usize = 2;
    let star_mass: f32 = 1.0;
    let star_orbit_radius: f32 = 10.0;
    // Visual radius range for disk bodies (shader: radius = 0.5 * mass^(1/3)).
    let disk_radius_min: f32 = 0.14;
    let disk_radius_max: f32 = 0.36;
    let disk_r_min: f32 = 5.0;
    let disk_r_max: f32 = 60.0;
    let disk_height: f32 = 0.5;
    let initial_v_perturbation: f32 = 0.05;
    let orbital_speed_factor: f32 = 2.0;

    let mut positions = vec![Vec4::ZERO; BODY_COUNT];
    let mut velocities = vec![Vec4::ZERO; BODY_COUNT];
    let mut masses = vec![0.0f32; BODY_COUNT];
    let mut accelerations = vec![Vec4::ZERO; BODY_COUNT];

    let mut index = 0usize;

    let chord_sum: f32 = (1..n_stars)
        .map(|k| 1.0 / (2.0 * (PI * k as f32 / n_stars as f32).sin()))
        .sum();
    let v_star = (g * star_mass * chord_sum / star_orbit_radius).sqrt();

    for i in 0..n_stars {
        let angle = i as f32 * 2.0 * PI / n_stars as f32;
        let position = Vec3::new(
            star_orbit_radius * angle.cos(),
            0.0,
            star_orbit_radius * angle.sin(),
        );
        let velocity = Vec3::new(-angle.sin(), 0.0, angle.cos()) * v_star;
        positions[index] = position.extend(0.0);
        velocities[index] = velocity.extend(0.0);
        masses[index] = star_mass;
        index += 1;
    }

    let total_mass = star_mass * n_stars as f32;
    let n_disk = BODY_COUNT - index;

    for _ in 0..n_disk {
        let u: f32 = rng.range(0.0, 1.0);
        let r = (disk_r_min * disk_r_min + u * (disk_r_max * disk_r_max - disk_r_min * disk_r_min))
            .sqrt();
        let theta: f32 = rng.range(0.0, 2.0 * PI);
        let height: f32 = rng.range(-disk_height, disk_height);

        let position = Vec3::new(r * theta.cos(), height, r * theta.sin());

        let v_mag = (g * total_mass / r).sqrt() * orbital_speed_factor;
        let vr = v_mag * rng.range(-initial_v_perturbation, initial_v_perturbation);
        let vt = v_mag * (1.0 + rng.range(-initial_v_perturbation, initial_v_perturbation));
        let vy = v_mag * rng.range(-initial_v_perturbation, initial_v_perturbation);
        let tangent = Vec3::new(-theta.sin(), 0.0, theta.cos());
        let radial = Vec3::new(theta.cos(), 0.0, theta.sin());
        let velocity = tangent * vt + radial * vr + Vec3::Y * vy;

        positions[index] = position.extend(0.0);
        velocities[index] = velocity.extend(0.0);
        let radius = rng.range(disk_radius_min, disk_radius_max);
        masses[index] = (radius / 0.5).powi(3);
        index += 1;
    }

    debug_assert_eq!(index, BODY_COUNT);

    compute_initial_accelerations(&positions, &masses, &mut accelerations);

    let cpu_snapshot = SimulationCpuSnapshot {
        positions: positions.iter().map(|p| p.truncate()).collect(),
        masses: masses.clone(),
        ready: true,
    };

    let gpu = SimulationGpuBuffers::new(&mut buffers, positions, velocities, masses, accelerations);
    commands.insert_resource(cpu_snapshot);
    commands.insert_resource(gpu);
}

/// Deterministic RNG for one-shot initial conditions (no `getrandom` on wasm).
struct SimpleRng(u64);

impl SimpleRng {
    fn new(seed: u64) -> Self {
        Self(seed)
    }

    fn next_u32(&mut self) -> u32 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1);
        (self.0 >> 32) as u32
    }

    fn range(&mut self, min: f32, max: f32) -> f32 {
        let u = (self.next_u32() as f64) / (u32::MAX as f64);
        min + (max - min) * u as f32
    }
}

fn compute_initial_accelerations(positions: &[Vec4], masses: &[f32], accelerations: &mut [Vec4]) {
    let n = BODY_COUNT;
    for i in 0..n {
        let pos_i = positions[i].truncate();
        let mut acc = Vec3::ZERO;
        for j in 0..n {
            if i == j {
                continue;
            }
            let r = positions[j].truncate() - pos_i;
            let dist_sq = r.length_squared() + SOFTENING_SQ;
            let inv_dist3 = G / (dist_sq * dist_sq.sqrt());
            acc += r * inv_dist3 * masses[j];
        }
        accelerations[i] = acc.extend(0.0);
    }
}
