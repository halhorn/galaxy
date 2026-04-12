use bevy::prelude::*;

/// Body mass in solar masses (M☉).
#[derive(Component, Debug, Clone, Copy)]
pub struct Mass(pub f32);

/// Velocity in AU/yr.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct Velocity(pub Vec3);

/// Acceleration in AU/yr² (stored for Velocity Verlet).
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct Acceleration(pub Vec3);

/// Marker component for bodies participating in the simulation.
#[derive(Component, Debug, Clone, Copy)]
pub struct SimulationBody;
