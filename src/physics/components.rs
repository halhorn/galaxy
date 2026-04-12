use bevy::prelude::*;

/// Body mass in solar masses (M☉).
#[derive(Component, Debug, Clone, Copy)]
pub struct Mass(pub f32);

/// Physical radius in AU (for collision detection and rendering scale).
#[derive(Component, Debug, Clone, Copy)]
pub struct Radius(pub f32);

/// Velocity in AU/yr.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct Velocity(pub Vec3);

/// Acceleration in AU/yr² (stored for Velocity Verlet).
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct Acceleration(pub Vec3);

/// Marker component for bodies participating in the simulation.
#[derive(Component, Debug, Clone, Copy)]
pub struct SimulationBody;

const RADIUS_SCALE: f32 = 0.5;

/// Compute visual/collision radius from mass: r = 0.5 * m^(1/3).
pub fn radius_from_mass(mass: f32) -> f32 {
    RADIUS_SCALE * mass.cbrt()
}

/// Compute emissive color from mass.
/// Small: cool blue, medium: bright cyan/white, large: intense red.
pub fn color_from_mass(mass: f32) -> LinearRgba {
    let t = (mass.log10() + 1.0) / 3.0; // maps 0.1→0, 100→1
    let t = t.clamp(0.0, 1.0);

    let brightness = 1.5 + t * 14.0; // 1.5..15.5
    let r;
    let g;
    let b;
    if t < 0.5 {
        // Blue → Cyan/White (small → medium)
        let s = t * 2.0; // 0..1
        r = (0.1 + 0.9 * s) * brightness;
        g = (0.3 + 0.7 * s) * brightness;
        b = (1.0) * brightness;
    } else {
        // White → Red (medium → large)
        let s = (t - 0.5) * 2.0; // 0..1
        r = 1.0 * brightness;
        g = (1.0 - 0.9 * s) * brightness;
        b = (1.0 - 0.95 * s) * brightness;
    }
    LinearRgba::new(r, g, b, 1.0)
}
