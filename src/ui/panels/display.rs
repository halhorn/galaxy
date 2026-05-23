use bevy_egui::egui;

use crate::model::constants::{
    MIN_STAR_VISUAL_SCALE_MAX, MIN_STAR_VISUAL_SCALE_MIN, STAR_VISUAL_SCALE_MAX,
    STAR_VISUAL_SCALE_MIN,
};
use crate::simulation::SimulationConfig;

pub fn display_panel(ui: &mut egui::Ui, config: &mut SimulationConfig) {
    ui.add(
        egui::Slider::new(&mut config.time_scale, 0.25..=4.0)
            .logarithmic(true)
            .text("Time scale"),
    );

    ui.add(
        egui::Slider::new(
            &mut config.star_visual_scale,
            STAR_VISUAL_SCALE_MIN..=STAR_VISUAL_SCALE_MAX,
        )
        .logarithmic(true)
        .text("Star visual scale"),
    );

    ui.add(
        egui::Slider::new(
            &mut config.min_star_visual_scale,
            MIN_STAR_VISUAL_SCALE_MIN..=MIN_STAR_VISUAL_SCALE_MAX,
        )
        .logarithmic(true)
        .text("Min star visual scale"),
    );
}
