//! Control panel UI (Phase 1+).

mod keyboard;
mod panels;

use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use bevy_panorbit_camera::EguiFocusIncludesHover;

use keyboard::playback_shortcuts;
use panels::ControlPanelsPlugin;

pub struct ControlUiPlugin;

impl Plugin for ControlUiPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(EguiFocusIncludesHover(true))
            .add_plugins(EguiPlugin::default())
            .add_plugins(ControlPanelsPlugin)
            .add_systems(Update, playback_shortcuts);
    }
}
