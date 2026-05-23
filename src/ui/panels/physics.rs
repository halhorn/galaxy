use bevy_egui::egui;

use crate::model::constants::{
    G_MAX, G_MIN, MERGE_RADIUS_FACTOR_MAX, MERGE_RADIUS_FACTOR_MIN, SOFTENING_MAX, SOFTENING_MIN,
};
use crate::model::PhysicsSettings;

pub fn physics_panel(ui: &mut egui::Ui, physics: &mut PhysicsSettings) {
    physics_slider_group(ui, physics);
    *physics = physics.clamped();
}

fn physics_slider_group(ui: &mut egui::Ui, physics: &mut PhysicsSettings) {
    ui.add(
        egui::Slider::new(&mut physics.g, G_MIN..=G_MAX)
            .logarithmic(true)
            .text("G"),
    );
    ui.label(format!(
        "Gravitational constant: {:.3} AU³/(M☉·yr²)",
        physics.g
    ));

    ui.add(
        egui::Slider::new(&mut physics.softening, SOFTENING_MIN..=SOFTENING_MAX)
            .logarithmic(true)
            .text("Softening (AU)"),
    );
    ui.label(format!(
        "Plummer softening: {:.4} AU",
        physics.softening,
    ));

    ui.add(
        egui::Slider::new(
            &mut physics.merge_radius_factor,
            MERGE_RADIUS_FACTOR_MIN..=MERGE_RADIUS_FACTOR_MAX,
        )
        .text("Merge radius factor"),
    );
    ui.label(format!(
        "Merge distance: {:.2}× combined radii",
        physics.merge_radius_factor,
    ));
}
