use bevy_egui::egui;

use crate::model::constants::{
    ACTIVE_COUNT_MAX, ACTIVE_COUNT_MIN, DISK_R_MAX, DISK_R_MIN, N_STARS_MAX, N_STARS_MIN,
    V_PERTURBATION_MAX,
};

use crate::ui::apply::UiPendingActions;
use crate::ui::draft::ControlPanelDraft;

pub fn initial_panel(
    ui: &mut egui::Ui,
    draft: &mut ControlPanelDraft,
    pending: &mut UiPendingActions,
) {
    let initial = &mut draft.initial;

    ui.label("Seed");
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

    ui.add(
        egui::Slider::new(&mut initial.n_stars, N_STARS_MIN..=N_STARS_MAX).text("Central stars"),
    );
    if initial.n_stars > initial.active_count {
        initial.active_count = initial.n_stars;
    }

    ui.add(
        egui::Slider::new(&mut initial.active_count, ACTIVE_COUNT_MIN..=ACTIVE_COUNT_MAX)
            .logarithmic(true)
            .text("Active bodies"),
    );
    ui.label(format!(
        "Disk particles: {}",
        initial.active_count.saturating_sub(initial.n_stars)
    ));

    ui.add(
        egui::Slider::new(&mut initial.disk_r_min, DISK_R_MIN..=DISK_R_MAX)
            .logarithmic(true)
            .text("Disk inner radius (AU)"),
    );
    ui.add(
        egui::Slider::new(&mut initial.disk_r_max, DISK_R_MIN..=DISK_R_MAX)
            .logarithmic(true)
            .text("Disk outer radius (AU)"),
    );
    if initial.disk_r_max <= initial.disk_r_min {
        initial.disk_r_max = initial.disk_r_min + 0.1;
    }

    ui.add(
        egui::Slider::new(&mut initial.initial_v_perturbation, 0.0..=V_PERTURBATION_MAX)
            .text("Velocity perturbation"),
    );

    ui.separator();
    if ui.button("Apply & Restart").clicked() {
        *initial = initial.clone().clamped();
        pending.restart = true;
    }

    ui.label("Applies initial conditions and restarts immediately from t = 0.");
}
