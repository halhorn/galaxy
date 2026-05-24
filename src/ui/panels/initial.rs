use bevy_egui::egui;

use crate::model::constants::{
    ACTIVE_COUNT_MAX, ACTIVE_COUNT_MIN, DISK_ELEVATION_DEG_MAX, DISK_MASS_LIMIT_MAX,
    DISK_MASS_LIMIT_MIN, DISK_R_MAX, DISK_R_MIN, N_STARS_MAX, N_STARS_MIN, SEED_MAX,
    STAR_MASS_MAX, STAR_MASS_MIN, V_PERTURBATION_MAX,
};

use crate::ui::draft::ControlPanelDraft;
use crate::ui::help::{help_link, slider_row, HelpId, HelpPopupState};

const SECTION_HEADING_SIZE: f32 = 13.0;
const SECTION_SPACING: f32 = 12.0;

fn section_heading(ui: &mut egui::Ui, text: &str) {
    ui.label(
        egui::RichText::new(text)
            .strong()
            .size(SECTION_HEADING_SIZE),
    );
}

pub fn initial_panel(ui: &mut egui::Ui, draft: &mut ControlPanelDraft, help: &mut HelpPopupState) {
    let initial = &mut draft.initial;

    ui.horizontal(|ui| {
        let mut seed_i64 = initial.seed.min(SEED_MAX) as i64;
        if ui
            .add(
                egui::DragValue::new(&mut seed_i64)
                    .speed(1)
                    .range(0..=SEED_MAX as i64),
            )
            .changed()
        {
            initial.seed = seed_i64.clamp(0, SEED_MAX as i64) as u64;
        }
        if ui.button("Random").clicked() {
            initial.seed = initial
                .seed
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1)
                % (SEED_MAX + 1);
        }
        ui.label("Seed");
        help_link(ui, HelpId::Seed, help);
    });

    ui.add_space(SECTION_SPACING);
    section_heading(ui, "Central stars");
    slider_row(
        ui,
        help,
        HelpId::CentralStarsCount,
        "Count",
        egui::Slider::new(&mut initial.n_stars, N_STARS_MIN..=N_STARS_MAX),
    );
    if initial.n_stars > initial.active_count {
        initial.active_count = initial.n_stars;
    }
    slider_row(
        ui,
        help,
        HelpId::CentralStarsMass,
        "Mass (M☉)",
        egui::Slider::new(&mut initial.star_mass, STAR_MASS_MIN..=STAR_MASS_MAX).logarithmic(true),
    );

    ui.add_space(SECTION_SPACING);
    section_heading(ui, "Disk");
    slider_row(
        ui,
        help,
        HelpId::DiskStars,
        "Stars",
        egui::Slider::new(&mut initial.active_count, ACTIVE_COUNT_MIN..=ACTIVE_COUNT_MAX)
            .logarithmic(true),
    );
    slider_row(
        ui,
        help,
        HelpId::DiskMassMin,
        "Mass min (M☉)",
        egui::Slider::new(
            &mut initial.disk_mass_min,
            DISK_MASS_LIMIT_MIN..=DISK_MASS_LIMIT_MAX,
        )
        .logarithmic(true),
    );
    slider_row(
        ui,
        help,
        HelpId::DiskMassMax,
        "Mass max (M☉)",
        egui::Slider::new(
            &mut initial.disk_mass_max,
            DISK_MASS_LIMIT_MIN..=DISK_MASS_LIMIT_MAX,
        )
        .logarithmic(true),
    );
    if initial.disk_mass_max <= initial.disk_mass_min {
        initial.disk_mass_max = initial.disk_mass_min + 0.001;
    }
    slider_row(
        ui,
        help,
        HelpId::DiskInnerRadius,
        "Inner radius (AU)",
        egui::Slider::new(&mut initial.disk_r_min, DISK_R_MIN..=DISK_R_MAX).logarithmic(true),
    );
    slider_row(
        ui,
        help,
        HelpId::DiskOuterRadius,
        "Outer radius (AU)",
        egui::Slider::new(&mut initial.disk_r_max, DISK_R_MIN..=DISK_R_MAX).logarithmic(true),
    );
    if initial.disk_r_max <= initial.disk_r_min {
        initial.disk_r_max = initial.disk_r_min + 0.1;
    }
    slider_row(
        ui,
        help,
        HelpId::DiskElevation,
        "Elevation (°)",
        egui::Slider::new(
            &mut initial.disk_elevation_deg,
            0.0..=DISK_ELEVATION_DEG_MAX,
        ),
    );
    slider_row(
        ui,
        help,
        HelpId::VelocityPerturbation,
        "Velocity perturbation",
        egui::Slider::new(&mut initial.initial_v_perturbation, 0.0..=V_PERTURBATION_MAX),
    );
}
