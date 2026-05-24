//! Control panel UI (Phase 1+).

mod apply;
mod draft;
mod fonts;
mod keyboard;
mod panels;

use bevy::prelude::*;
use bevy_egui::{EguiGlobalSettings, EguiPlugin, EguiPrimaryContextPass};
use bevy_panorbit_camera::EguiFocusIncludesHover;

use crate::simulation::SimViewportSystems;

use apply::UiApplyPlugin;
use fonts::setup_equation_font;
use keyboard::playback_shortcuts;
use panels::ControlPanelsPlugin;

pub use draft::ControlPanelDraft;

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
                (SimViewportSystems::Layout, SimViewportSystems::CameraViewport).chain(),
            )
            .add_plugins((ControlPanelsPlugin, UiApplyPlugin))
            .add_systems(PostStartup, setup_equation_font)
            .add_systems(Update, playback_shortcuts);
    }
}
