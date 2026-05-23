use bevy_egui::egui;

use crate::model::constants::{
    FORCE_COEFFICIENT_MAX, FORCE_COEFFICIENT_MIN, FORCE_EXPONENT_MAX, FORCE_EXPONENT_MIN, G,
};
use crate::model::force::{ForceLaw, ForceTerm, MAX_FORCE_TERMS};
use crate::model::PhysicsSettings;
use crate::simulation::SimulationSettings;
use crate::ui::apply::UiPendingActions;
use crate::ui::draft::ControlPanelDraft;

const SECTION_HEADING_SIZE: f32 = 13.0;
const SECTION_SPACING: f32 = 12.0;
const APPLY_BUTTON_TOP_PADDING: f32 = 16.0;
const APPLY_BUTTON_HEIGHT: f32 = 36.0;
const DISPLAY_EXPONENT_MIN: i32 = FORCE_EXPONENT_MIN + 1;
const DISPLAY_EXPONENT_MAX: i32 = FORCE_EXPONENT_MAX + 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ForcePreset {
    Newtonian,
    GravityPlusRepulsion,
    Repulsive,
}

impl ForcePreset {
    const ALL: [Self; 3] = [
        Self::Newtonian,
        Self::GravityPlusRepulsion,
        Self::Repulsive,
    ];

    fn label(self) -> &'static str {
        match self {
            Self::Newtonian => "Newtonian (−d^-2)",
            Self::GravityPlusRepulsion => "Gravity + repulsion (−d^-2 +d^-1)",
            Self::Repulsive => "Repulsive demo (+d^0)",
        }
    }

    fn apply(self, g: f32) -> ForceLaw {
        match self {
            Self::Newtonian => ForceLaw::newtonian(g),
            Self::GravityPlusRepulsion => ForceLaw::preset_gravity_plus_repulsion(g),
            Self::Repulsive => ForceLaw::preset_repulsive(),
        }
    }
}

fn section_heading(ui: &mut egui::Ui, text: &str) {
    ui.label(
        egui::RichText::new(text)
            .strong()
            .size(SECTION_HEADING_SIZE),
    );
}

fn show_term_row(ui: &mut egui::Ui, index: usize, term: &mut ForceTerm, removable: bool) -> bool {
    let mut remove = false;

    ui.horizontal(|ui| {
        let sign_label = if term.sign >= 0 { "−" } else { "+" };
        if ui.button(sign_label).clicked() {
            term.sign = -term.sign;
        }

        ui.add_sized(
            [120.0, 20.0],
            egui::Slider::new(
                &mut term.coefficient,
                FORCE_COEFFICIENT_MIN..=FORCE_COEFFICIENT_MAX,
            )
            .logarithmic(true)
            .show_value(true),
        );

        ui.label(" * d ^ ");

        let mut display_exponent = term.exponent + 1;
        egui::ComboBox::from_id_salt(index)
            .width(52.0)
            .selected_text(display_exponent.to_string())
            .show_ui(ui, |ui| {
                for exp in DISPLAY_EXPONENT_MIN..=DISPLAY_EXPONENT_MAX {
                    ui.selectable_value(&mut display_exponent, exp, exp.to_string());
                }
            });
        term.exponent = display_exponent - 1;

        if removable && ui.button("Remove").clicked() {
            remove = true;
        }
    });

    remove
}

pub fn force_panel(
    ui: &mut egui::Ui,
    draft: &mut ControlPanelDraft,
    settings: &SimulationSettings,
    pending: &mut UiPendingActions,
) {
    let force = &mut draft.force;
    let physics = &settings.physics;

    section_heading(ui, "Force law");
    ui.label(format!("Preview: {}", force.display_string()));

    ui.add_space(SECTION_SPACING);
    section_heading(ui, "Presets");
    ui.horizontal_wrapped(|ui| {
        for preset in ForcePreset::ALL {
            if ui.button(preset.label()).clicked() {
                *force = preset.apply(physics.g);
            }
        }
    });

    egui::CollapsingHeader::new("Force expression")
        .default_open(true)
        .show(ui, |ui| {
            section_heading(ui, "Terms");

            let term_count = force.term_count as usize;
            let mut remove_at = None;
            for index in 0..term_count {
                let term = &mut force.terms[index];
                if show_term_row(ui, index, term, term_count > 1) {
                    remove_at = Some(index);
                }
            }

            if let Some(index) = remove_at {
                force.terms.copy_within(index + 1..term_count, index);
                force.terms[term_count - 1] = ForceTerm {
                    sign: 0,
                    exponent: 0,
                    coefficient: 0.0,
                };
                force.term_count -= 1;
            }

            ui.horizontal(|ui| {
                if force.term_count < MAX_FORCE_TERMS as u8 && ui.button("Add term").clicked() {
                    let slot = force.term_count as usize;
                    force.terms[slot] = ForceTerm {
                        sign: 1,
                        exponent: -3,
                        coefficient: G,
                    };
                    force.term_count += 1;
                }
            });
        });

    show_warnings(ui, force, physics);

    ui.add_space(APPLY_BUTTON_TOP_PADDING);
    let valid = force.is_valid();
    ui.add_enabled_ui(valid, |ui| {
        let apply = egui::Button::new(egui::RichText::new("Apply & Restart").size(16.0));
        if ui
            .add_sized([ui.available_width(), APPLY_BUTTON_HEIGHT], apply)
            .clicked()
        {
            *force = force.clone().clamped();
            pending.restart = true;
        }
    });

    if !valid {
        ui.colored_label(
            egui::Color32::from_rgb(220, 120, 80),
            "At least one valid term is required.",
        );
    }
}

fn show_warnings(ui: &mut egui::Ui, force: &ForceLaw, physics: &PhysicsSettings) {
    let mut warnings = Vec::new();
    if force.needs_softening_warning() && physics.softening <= 0.0 {
        warnings.push("Terms with N ≤ −1 need softening > 0 to avoid singularities.");
    }
    if force.has_repulsive_terms() {
        warnings.push("Repulsive terms can destabilize the simulation quickly.");
    }

    if warnings.is_empty() {
        return;
    }

    ui.add_space(SECTION_SPACING);
    section_heading(ui, "Warnings");
    for warning in warnings {
        ui.colored_label(egui::Color32::from_rgb(220, 180, 80), warning);
    }
}
