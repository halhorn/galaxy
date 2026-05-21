use bevy_egui::egui;

use crate::simulation::{PlaybackMode, PlaybackState, SimulationConfig};

pub fn playback_panel(
    ui: &mut egui::Ui,
    playback: &mut PlaybackState,
    config: &mut SimulationConfig,
    fps: f32,
) {
    let status = match playback.mode {
        PlaybackMode::Running => "Running",
        PlaybackMode::Paused => "Paused",
    };
    ui.label(format!("Status: {status}"));
    ui.label(format!(
        "Sim time: {:.2} yr",
        playback.accumulated_sim_time
    ));
    ui.label(format!("FPS: {fps:.0}"));

    ui.horizontal(|ui| {
        if playback.is_running() {
            if ui.button("Pause").clicked() {
                playback.mode = PlaybackMode::Paused;
            }
        } else if ui.button("Resume").clicked() {
            playback.mode = PlaybackMode::Running;
        }
    });

    ui.add(
        egui::Slider::new(&mut config.time_scale, 0.25..=4.0)
            .logarithmic(true)
            .text("Time scale"),
    );
    ui.label(format!("Time scale: {:.2}x", config.time_scale));

    ui.horizontal(|ui| {
        ui.label("Presets:");
        if ui.button("0.25x").clicked() {
            config.time_scale = 0.25;
        }
        if ui.button("1x").clicked() {
            config.time_scale = 1.0;
        }
        if ui.button("2x").clicked() {
            config.time_scale = 2.0;
        }
        if ui.button("4x").clicked() {
            config.time_scale = 4.0;
        }
    });
}
