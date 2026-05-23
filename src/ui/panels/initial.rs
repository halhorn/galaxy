use bevy_egui::egui;

use crate::model::constants::{
    ACTIVE_COUNT_MAX, ACTIVE_COUNT_MIN, DISK_MASS_LIMIT_MAX, DISK_MASS_LIMIT_MIN, DISK_R_MAX,
    DISK_R_MIN, N_STARS_MAX, N_STARS_MIN, STAR_MASS_MAX, STAR_MASS_MIN, V_PERTURBATION_MAX,
};

use crate::ui::apply::UiPendingActions;
use crate::ui::draft::ControlPanelDraft;

const SECTION_HEADING_SIZE: f32 = 13.0;
const SECTION_SPACING: f32 = 12.0;
const APPLY_BUTTON_TOP_PADDING: f32 = 16.0;
const APPLY_BUTTON_HEIGHT: f32 = 36.0;

fn section_heading(ui: &mut egui::Ui, text: &str) {
    ui.label(
        egui::RichText::new(text)
            .strong()
            .size(SECTION_HEADING_SIZE),
    );
}

pub fn initial_panel(
    ui: &mut egui::Ui,
    draft: &mut ControlPanelDraft,
    pending: &mut UiPendingActions,
) {
    let initial = &mut draft.initial;

    section_heading(ui, "Seed");
    ui.horizontal(|ui| {
        let mut seed_i64 = initial.seed as i64;
        if ui
            .add(
                egui::DragValue::new(&mut seed_i64)
                    .speed(1)
                    .range(i64::MIN..=i64::MAX),
            )
            .changed()
        {
            initial.seed = seed_i64 as u64;
        }
        if ui.button("Random").clicked() {
            initial.seed = initial.seed.wrapping_mul(6364136223846793005).wrapping_add(1);
        }
    });

    ui.add_space(SECTION_SPACING);
    section_heading(ui, "Central stars");
    ui.add(
        egui::Slider::new(&mut initial.n_stars, N_STARS_MIN..=N_STARS_MAX).text("Count"),
    );
    if initial.n_stars > initial.active_count {
        initial.active_count = initial.n_stars;
    }
    ui.add(
        egui::Slider::new(&mut initial.star_mass, STAR_MASS_MIN..=STAR_MASS_MAX)
            .logarithmic(true)
            .text("Mass (M☉)"),
    );

    ui.add_space(SECTION_SPACING);
    section_heading(ui, "Disk");
    ui.add(
        egui::Slider::new(&mut initial.active_count, ACTIVE_COUNT_MIN..=ACTIVE_COUNT_MAX)
            .logarithmic(true)
            .text("Stars"),
    );
    ui.add(
        egui::Slider::new(
            &mut initial.disk_mass_min,
            DISK_MASS_LIMIT_MIN..=DISK_MASS_LIMIT_MAX,
        )
        .logarithmic(true)
        .text("Mass min (M☉)"),
    );
    ui.add(
        egui::Slider::new(
            &mut initial.disk_mass_max,
            DISK_MASS_LIMIT_MIN..=DISK_MASS_LIMIT_MAX,
        )
        .logarithmic(true)
        .text("Mass max (M☉)"),
    );
    if initial.disk_mass_max <= initial.disk_mass_min {
        initial.disk_mass_max = initial.disk_mass_min + 0.001;
    }
    ui.add(
        egui::Slider::new(&mut initial.disk_r_min, DISK_R_MIN..=DISK_R_MAX)
            .logarithmic(true)
            .text("Inner radius (AU)"),
    );
    ui.add(
        egui::Slider::new(&mut initial.disk_r_max, DISK_R_MIN..=DISK_R_MAX)
            .logarithmic(true)
            .text("Outer radius (AU)"),
    );
    if initial.disk_r_max <= initial.disk_r_min {
        initial.disk_r_max = initial.disk_r_min + 0.1;
    }
    ui.add(
        egui::Slider::new(&mut initial.initial_v_perturbation, 0.0..=V_PERTURBATION_MAX)
            .text("Velocity perturbation"),
    );

    ui.add_space(APPLY_BUTTON_TOP_PADDING);
    let apply = egui::Button::new(egui::RichText::new("Apply & Restart").size(16.0));
    if ui
        .add_sized([ui.available_width(), APPLY_BUTTON_HEIGHT], apply)
        .clicked()
    {
        *initial = initial.clone().clamped();
        pending.restart = true;
    }
}
