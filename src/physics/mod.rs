pub mod components;
pub mod force;
pub mod integrator;

use bevy::prelude::*;

use components::*;
use force::NewtonianGravity;
use integrator::*;

/// Bevy plugin that runs the N-body gravity simulation.
pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ActiveForce(Box::new(NewtonianGravity::default())))
            .insert_resource(SimulationConfig::default())
            .add_systems(FixedUpdate, velocity_verlet_step);
    }
}

/// Spawn a simulation body with the given mass, position, and velocity.
pub fn spawn_body(commands: &mut Commands, mass: f32, position: Vec3, velocity: Vec3) -> Entity {
    commands
        .spawn((
            SimulationBody,
            Mass(mass),
            Velocity(velocity),
            Acceleration::default(),
            Transform::from_translation(position),
        ))
        .id()
}
