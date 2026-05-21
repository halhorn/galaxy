use bevy::prelude::*;
use bevy_panorbit_camera::EguiWantsFocus;

use crate::simulation::{PlaybackState, SimulationConfig};

const TIME_SCALE_PRESETS: [(KeyCode, f32); 4] = [
    (KeyCode::Digit1, 0.25),
    (KeyCode::Digit2, 1.0),
    (KeyCode::Digit3, 2.0),
    (KeyCode::Digit4, 4.0),
];

pub fn playback_shortcuts(
    keyboard: Res<ButtonInput<KeyCode>>,
    egui_wants_focus: Res<EguiWantsFocus>,
    mut playback: ResMut<PlaybackState>,
    mut config: ResMut<SimulationConfig>,
) {
    if egui_wants_focus.prev || egui_wants_focus.curr {
        return;
    }

    if keyboard.just_pressed(KeyCode::Space) {
        playback.toggle();
    }

    for (key, scale) in TIME_SCALE_PRESETS {
        if keyboard.just_pressed(key) {
            config.time_scale = scale;
        }
    }
}
