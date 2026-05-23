use bevy::prelude::*;

use crate::simulation::{
    logical_rect_to_camera_viewport, SimulationViewportRect, SimViewportSystems,
};

/// Render layer for the 3D simulation (bodies, gizmos, pan-orbit camera).
pub const SIMULATION_RENDER_LAYER: usize = 0;
/// Render layer for the full-window egui overlay camera (no simulation gizmos).
pub const UI_RENDER_LAYER: usize = 1;

/// Marks the 3D camera that renders the simulation (not the egui overlay camera).
#[derive(Component)]
pub struct SimulationCamera;

pub fn update_simulation_camera_viewport(
    viewport_rect: Res<SimulationViewportRect>,
    windows: Query<&Window>,
    mut cameras: Query<&mut Camera, With<SimulationCamera>>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let Ok(mut camera) = cameras.single_mut() else {
        return;
    };

    camera.viewport = Some(logical_rect_to_camera_viewport(
        viewport_rect.logical,
        window,
    ));
}

pub struct SimulationViewportPlugin;

impl Plugin for SimulationViewportPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            bevy_egui::EguiPrimaryContextPass,
            update_simulation_camera_viewport.in_set(SimViewportSystems::CameraViewport),
        );
    }
}
