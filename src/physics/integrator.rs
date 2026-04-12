use bevy::prelude::*;

use super::components::*;
use super::force::ForceCalculator;

/// Holds the active force calculator (swappable at runtime).
#[derive(Resource)]
pub struct ActiveForce(pub Box<dyn ForceCalculator>);

/// Physics simulation parameters.
#[derive(Resource)]
pub struct SimulationConfig {
    /// Simulation time per real second (years/s).
    /// At 60 Hz FixedUpdate, each step advances `time_scale / 60` years.
    pub time_scale: f32,
}

impl Default for SimulationConfig {
    fn default() -> Self {
        Self {
            time_scale: 1.0, // 1 year of simulation per real second
        }
    }
}

/// Velocity Verlet integration step.
///
/// Runs in `FixedUpdate` schedule. Uses `Time` (which gives fixed dt in that context)
/// multiplied by `SimulationConfig::time_scale` to get the physics dt in years.
pub fn velocity_verlet_step(
    time: Res<Time>,
    config: Res<SimulationConfig>,
    force: Res<ActiveForce>,
    mut query: Query<(Entity, &mut Transform, &mut Velocity, &mut Acceleration, &Mass), With<SimulationBody>>,
) {
    let dt = time.delta_secs() * config.time_scale;
    if dt <= 0.0 {
        return;
    }

    // Collect current state
    let mut entities = Vec::new();
    let mut positions = Vec::new();
    let mut velocities = Vec::new();
    let mut old_accelerations = Vec::new();
    let mut masses = Vec::new();

    for (entity, transform, vel, acc, mass) in query.iter() {
        entities.push(entity);
        positions.push(transform.translation);
        velocities.push(vel.0);
        old_accelerations.push(acc.0);
        masses.push(mass.0);
    }

    let n = entities.len();
    if n == 0 {
        return;
    }

    // Step 1: x(t+dt) = x(t) + v(t)·dt + 0.5·a(t)·dt²
    for i in 0..n {
        positions[i] += velocities[i] * dt + 0.5 * old_accelerations[i] * dt * dt;
    }

    // Step 2: a(t+dt) = F(x(t+dt)) / m
    let new_accelerations = force.0.calculate_accelerations(&positions, &masses);

    // Step 3: v(t+dt) = v(t) + 0.5·(a(t) + a(t+dt))·dt
    for i in 0..n {
        velocities[i] += 0.5 * (old_accelerations[i] + new_accelerations[i]) * dt;
    }

    // Write back to ECS
    for i in 0..n {
        if let Ok((_, mut transform, mut vel, mut acc, _)) = query.get_mut(entities[i]) {
            transform.translation = positions[i];
            vel.0 = velocities[i];
            acc.0 = new_accelerations[i];
        }
    }
}

/// Run Velocity Verlet for `steps` iterations on raw arrays (for testing without ECS).
pub fn integrate_steps(
    force: &dyn ForceCalculator,
    positions: &mut [Vec3],
    velocities: &mut [Vec3],
    accelerations: &mut [Vec3],
    masses: &[f32],
    dt: f32,
    steps: usize,
) {
    let n = positions.len();
    for _ in 0..steps {
        // Step 1: update positions
        for i in 0..n {
            positions[i] += velocities[i] * dt + 0.5 * accelerations[i] * dt * dt;
        }
        // Step 2: new accelerations
        let new_acc = force.calculate_accelerations(positions, masses);
        // Step 3: update velocities
        for i in 0..n {
            velocities[i] += 0.5 * (accelerations[i] + new_acc[i]) * dt;
            accelerations[i] = new_acc[i];
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::physics::force::NewtonianGravity;
    use std::f32::consts::PI;

    /// Helper: total energy of an N-body system.
    fn total_energy(
        positions: &[Vec3],
        velocities: &[Vec3],
        masses: &[f32],
        g: f32,
    ) -> f32 {
        let n = positions.len();
        let mut kinetic = 0.0;
        for i in 0..n {
            kinetic += 0.5 * masses[i] * velocities[i].length_squared();
        }
        let mut potential = 0.0;
        for i in 0..n {
            for j in (i + 1)..n {
                let r = (positions[j] - positions[i]).length();
                if r > 0.0 {
                    potential -= g * masses[i] * masses[j] / r;
                }
            }
        }
        kinetic + potential
    }

    #[test]
    fn circular_orbit_returns_to_start() {
        let g = 4.0 * PI * PI;
        let gravity = NewtonianGravity {
            gravitational_constant: g,
            softening: 0.0,
        };

        // Star at origin, planet at (1, 0, 0) AU
        // Circular orbit velocity: v = sqrt(G·M/r) = 2π AU/yr
        // Period T = 1 year
        let mut positions = vec![Vec3::ZERO, Vec3::new(1.0, 0.0, 0.0)];
        let mut velocities = vec![Vec3::ZERO, Vec3::new(0.0, 0.0, 2.0 * PI)];
        let masses = vec![1.0, 1e-6]; // Star + negligible planet
        let mut accelerations = gravity.calculate_accelerations(&positions, &masses);

        let dt = 1e-4; // 10,000 steps per orbit
        let steps = (1.0 / dt) as usize;

        let initial_pos = positions[1];
        let e0 = total_energy(&positions, &velocities, &masses, g);

        integrate_steps(
            &gravity,
            &mut positions,
            &mut velocities,
            &mut accelerations,
            &masses,
            dt,
            steps,
        );

        // Position should return close to start
        let error = (positions[1] - initial_pos).length();
        assert!(error < 0.01, "Position error after 1 orbit: {error}");

        // Energy should be conserved
        let e1 = total_energy(&positions, &velocities, &masses, g);
        let energy_drift = ((e1 - e0) / e0).abs();
        assert!(energy_drift < 1e-5, "Energy drift: {energy_drift}");
    }

    #[test]
    fn elliptical_orbit_conserves_energy() {
        let g = 4.0 * PI * PI;
        let gravity = NewtonianGravity {
            gravitational_constant: g,
            softening: 0.0,
        };

        // Planet at perihelion (1 AU) with 1.2× circular velocity → elliptical orbit
        let v_circ = 2.0 * PI;
        let mut positions = vec![Vec3::ZERO, Vec3::new(1.0, 0.0, 0.0)];
        let mut velocities = vec![Vec3::ZERO, Vec3::new(0.0, 0.0, v_circ * 1.2)];
        let masses = vec![1.0, 1e-6];
        let mut accelerations = gravity.calculate_accelerations(&positions, &masses);

        let e0 = total_energy(&positions, &velocities, &masses, g);

        let dt = 1e-4;
        let steps = 20_000; // ~2 orbits

        integrate_steps(
            &gravity,
            &mut positions,
            &mut velocities,
            &mut accelerations,
            &masses,
            dt,
            steps,
        );

        let e1 = total_energy(&positions, &velocities, &masses, g);
        let energy_drift = ((e1 - e0) / e0).abs();
        assert!(energy_drift < 1e-4, "Energy drift: {energy_drift}");
    }

    #[test]
    fn momentum_conserved() {
        let g = 4.0 * PI * PI;
        let gravity = NewtonianGravity {
            gravitational_constant: g,
            softening: 0.0,
        };

        let mut positions = vec![
            Vec3::new(-1.0, 0.0, 0.0),
            Vec3::new(1.0, 0.0, 0.0),
        ];
        let mut velocities = vec![
            Vec3::new(0.0, 0.0, -PI),
            Vec3::new(0.0, 0.0, PI),
        ];
        let masses = vec![1.0, 1.0];
        let mut accelerations = gravity.calculate_accelerations(&positions, &masses);

        let p0: Vec3 = velocities
            .iter()
            .zip(masses.iter())
            .map(|(v, m)| *v * *m)
            .sum();

        integrate_steps(
            &gravity,
            &mut positions,
            &mut velocities,
            &mut accelerations,
            &masses,
            1e-4,
            5_000,
        );

        let p1: Vec3 = velocities
            .iter()
            .zip(masses.iter())
            .map(|(v, m)| *v * *m)
            .sum();

        let drift = (p1 - p0).length();
        assert!(drift < 1e-6, "Momentum drift: {drift}");
    }
}
