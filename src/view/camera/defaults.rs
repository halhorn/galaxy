use bevy::prelude::*;
use bevy_panorbit_camera::PanOrbitCamera;

pub fn default_simulation_camera_transform() -> Transform {
    Transform::from_xyz(0.0, 80.0, 120.0).looking_at(Vec3::ZERO, Vec3::Y)
}

pub fn default_simulation_pan_orbit() -> PanOrbitCamera {
    PanOrbitCamera {
        zoom_sensitivity: 0.0,
        ..default()
    }
}
