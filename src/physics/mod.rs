pub mod components;
pub mod force;
pub mod gpu_force;
pub mod integrator;

use bevy::prelude::*;

use force::NewtonianGravity;
use gpu_force::GpuForceCalculator;
use integrator::*;

/// Bevy plugin that runs the N-body gravity simulation.
/// Attempts GPU compute first, falls back to CPU if unavailable.
pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        let force_calculator: Box<dyn force::ForceCalculator> =
            match GpuForceCalculator::try_default() {
                Some(gpu) => {
                    info!("Using GPU force calculator");
                    Box::new(gpu)
                }
                None => {
                    warn!("GPU unavailable, falling back to CPU force calculator");
                    Box::new(NewtonianGravity::default())
                }
            };

        app.insert_resource(ActiveForce(force_calculator))
            .insert_resource(SimulationConfig::default())
            .add_systems(FixedUpdate, velocity_verlet_step);
    }
}
