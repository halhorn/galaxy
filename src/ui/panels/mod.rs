mod force;
mod initial;
mod playback;
mod physics;

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPrimaryContextPass};

use crate::simulation::{PlaybackState, SimulationConfig};

use force::force_panel;
use initial::initial_panel;
use playback::playback_panel;
use physics::physics_panel;

#[derive(Resource, Default)]
struct FpsDisplay {
    fps: f32,
}

fn update_fps_display(time: Res<Time>, mut fps: ResMut<FpsDisplay>, mut smoothed: Local<f32>) {
    let dt = time.delta_secs();
    if dt > 0.0 {
        let instant = 1.0 / dt;
        *smoothed = *smoothed * 0.9 + instant * 0.1;
        fps.fps = *smoothed;
    }
}

fn draw_control_panel(
    mut contexts: EguiContexts,
    mut playback: ResMut<PlaybackState>,
    mut config: ResMut<SimulationConfig>,
    fps: Res<FpsDisplay>,
) -> Result {
    egui::SidePanel::left("control_panel")
        .default_width(240.0)
        .resizable(true)
        .show(contexts.ctx_mut()?, |ui| {
            ui.heading("Gravitium");

            egui::CollapsingHeader::new("Playback")
                .default_open(true)
                .show(ui, |ui| {
                    playback_panel(ui, &mut playback, &mut config, fps.fps);
                });

            egui::CollapsingHeader::new("Physics")
                .default_open(false)
                .show(ui, |ui| {
                    physics_panel(ui);
                });

            egui::CollapsingHeader::new("Initial Conditions")
                .default_open(false)
                .show(ui, |ui| {
                    initial_panel(ui);
                });

            egui::CollapsingHeader::new("Force Law")
                .default_open(false)
                .show(ui, |ui| {
                    force_panel(ui);
                });
        });

    Ok(())
}

pub struct ControlPanelsPlugin;

impl Plugin for ControlPanelsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FpsDisplay>()
            .add_systems(Update, update_fps_display)
            .add_systems(EguiPrimaryContextPass, draw_control_panel);
    }
}
