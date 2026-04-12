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

/// Bundle for spawning a simulation body.
#[derive(Bundle)]
pub struct SimulationBodyBundle {
    pub body: SimulationBody,
    pub mass: Mass,
    pub velocity: Velocity,
    pub acceleration: Acceleration,
    pub transform: Transform,
}

impl SimulationBodyBundle {
    pub fn new(mass: f32, position: Vec3, velocity: Vec3) -> Self {
        Self {
            body: SimulationBody,
            mass: Mass(mass),
            velocity: Velocity(velocity),
            acceleration: Acceleration::default(),
            transform: Transform::from_translation(position),
        }
    }
}
