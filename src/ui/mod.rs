//! Control panel UI (Phase 1+).

mod keyboard;
mod panels;

use bevy::prelude::*;
use bevy_egui::{EguiGlobalSettings, EguiPlugin, EguiPrimaryContextPass};
use bevy_panorbit_camera::EguiFocusIncludesHover;

use crate::simulation::SimViewportSystems;

use keyboard::playback_shortcuts;
use panels::ControlPanelsPlugin;

pub struct ControlUiPlugin;

impl Plugin for ControlUiPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(EguiGlobalSettings {
            auto_create_primary_context: false,
            ..default()
        })
            .insert_resource(EguiFocusIncludesHover(true))
            .add_plugins(EguiPlugin::default())
            .configure_sets(
                EguiPrimaryContextPass,
                (SimViewportSystems::Layout, SimViewportSystems::Apply).chain(),
            )
            .add_plugins(ControlPanelsPlugin)
            .add_systems(Update, playback_shortcuts);
    }
}
