use bevy::prelude::*;
use bevy_panorbit_camera::PanOrbitCamera;

use crate::simulation::SimulationCommand;
use crate::view::SimulationCamera;

use super::defaults::{default_simulation_camera_transform, default_simulation_pan_orbit};

pub fn reset_simulation_camera_on_restart(
    mut commands: MessageReader<SimulationCommand>,
    mut camera: Query<(&mut Transform, &mut PanOrbitCamera), With<SimulationCamera>>,
) {
    if !commands
        .read()
        .any(|command| matches!(command, SimulationCommand::Restart))
    {
        return;
    }

    let Ok((mut transform, mut pan_orbit)) = camera.single_mut() else {
        return;
    };

    *transform = default_simulation_camera_transform();
    *pan_orbit = default_simulation_pan_orbit();
}
