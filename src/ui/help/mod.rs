//! Setting help popups (text in `assets/help/en.txt`).

use bevy::prelude::*;
use bevy_egui::egui::{self, Frame, Link, Order, RichText, Vec2};
use std::collections::HashMap;

const HELP_LINK_SIZE: f32 = 12.0;
const HELP_LINK_MIN_SIZE: Vec2 = Vec2::new(22.0, 18.0);

const HELP_TEXT: &str = include_str!("../../../assets/help/en.txt");

const POPUP_MAX_WIDTH: f32 = 280.0;
const POPUP_OFFSET: f32 = 8.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HelpId {
    Seed,
    CentralStarsCount,
    CentralStarsMass,
    DiskStars,
    DiskMassMin,
    DiskMassMax,
    DiskInnerRadius,
    DiskOuterRadius,
    DiskElevation,
    VelocityPerturbation,
    Softening,
    MergeRadiusFactor,
    ForcePresets,
    ForceExpression,
    TimeScale,
    StarVisualScale,
    MinStarVisualScale,
}

impl HelpId {
    fn key(self) -> &'static str {
        match self {
            Self::Seed => "seed",
            Self::CentralStarsCount => "central_stars_count",
            Self::CentralStarsMass => "central_stars_mass",
            Self::DiskStars => "disk_stars",
            Self::DiskMassMin => "disk_mass_min",
            Self::DiskMassMax => "disk_mass_max",
            Self::DiskInnerRadius => "disk_inner_radius",
            Self::DiskOuterRadius => "disk_outer_radius",
            Self::DiskElevation => "disk_elevation",
            Self::VelocityPerturbation => "velocity_perturbation",
            Self::Softening => "softening",
            Self::MergeRadiusFactor => "merge_radius_factor",
            Self::ForcePresets => "force_presets",
            Self::ForceExpression => "force_expression",
            Self::TimeScale => "time_scale",
            Self::StarVisualScale => "star_visual_scale",
            Self::MinStarVisualScale => "min_star_visual_scale",
        }
    }
}

#[derive(Debug, Clone)]
struct HelpEntry {
    lines: Vec<String>,
}

#[derive(Resource)]
pub(crate) struct HelpCatalog {
    entries: HashMap<String, HelpEntry>,
}

impl HelpCatalog {
    fn load() -> Self {
        Self {
            entries: parse_help_file(HELP_TEXT),
        }
    }

    fn lookup(&self, id: HelpId) -> Option<&HelpEntry> {
        self.entries.get(id.key())
    }
}

fn parse_help_file(text: &str) -> HashMap<String, HelpEntry> {
    let mut map = HashMap::new();
    let mut current_key: Option<String> = None;
    let mut lines: Vec<String> = Vec::new();

    for line in text.lines() {
        if let Some(key) = line.strip_prefix('@') {
            if let Some(k) = current_key.take() {
                map.insert(k, HelpEntry { lines });
                lines = Vec::new();
            }
            current_key = Some(key.trim().to_string());
        } else if current_key.is_some() && !line.is_empty() {
            lines.push(line.to_string());
        }
    }
    if let Some(k) = current_key {
        map.insert(k, HelpEntry { lines });
    }
    map
}

#[derive(Debug, Clone, Copy)]
struct OpenHelp {
    id: HelpId,
    anchor: egui::Pos2,
}

#[derive(Resource, Default)]
pub struct HelpPopupState {
    open: Option<OpenHelp>,
    popup_rect: Option<egui::Rect>,
    opened_this_frame: bool,
}

impl HelpPopupState {
    fn open(&mut self, id: HelpId, anchor: egui::Pos2) {
        self.open = Some(OpenHelp { id, anchor });
        self.popup_rect = None;
        self.opened_this_frame = true;
    }

    fn close(&mut self) {
        self.open = None;
        self.popup_rect = None;
    }
}

/// Help link placed immediately after a setting label.
pub fn help_link(ui: &mut egui::Ui, id: HelpId, state: &mut HelpPopupState) {
    let response = ui.add_sized(
        HELP_LINK_MIN_SIZE,
        Link::new(RichText::new(" ? ").size(HELP_LINK_SIZE)),
    );
    if response.clicked() {
        state.open(id, response.rect.center());
    }
    response.on_hover_text("Help");
}

/// Slider with label, then `?`.
pub fn slider_row(
    ui: &mut egui::Ui,
    state: &mut HelpPopupState,
    id: HelpId,
    label: &str,
    slider: egui::Slider<'_>,
) {
    ui.horizontal(|ui| {
        ui.add(slider.text(label));
        help_link(ui, id, state);
    });
}

fn popup_position(anchor: egui::Pos2, size: Vec2, screen: egui::Rect) -> egui::Pos2 {
    let mut pos = egui::pos2(anchor.x + POPUP_OFFSET, anchor.y + POPUP_OFFSET);

    if pos.x + size.x > screen.max.x {
        pos.x = (anchor.x - POPUP_OFFSET - size.x).max(screen.min.x);
    }
    if pos.y + size.y > screen.max.y {
        pos.x = screen.center().x - size.x * 0.5;
        pos.y = screen.center().y - size.y * 0.5;
    }
    if pos.y < screen.min.y {
        pos.y = screen.min.y + POPUP_OFFSET;
    }
    pos
}

pub fn show_help_overlay(ctx: &egui::Context, catalog: &HelpCatalog, state: &mut HelpPopupState) {
    let skip_dismiss = state.opened_this_frame;
    state.opened_this_frame = false;

    let Some(open) = state.open else {
        state.popup_rect = None;
        return;
    };

    let entry = catalog.lookup(open.id).cloned().unwrap_or_else(|| HelpEntry {
        lines: vec![
            "Help".to_string(),
            "No description available.".to_string(),
        ],
    });

    let screen = ctx.content_rect();
    let estimated_height = 16.0 + entry.lines.len() as f32 * 18.0 + 16.0;
    let pos = popup_position(
        open.anchor,
        Vec2::new(POPUP_MAX_WIDTH, estimated_height),
        screen,
    );

    let popup_id = egui::Id::new("help_popup");
    let popup_response = egui::Area::new(popup_id)
        .order(Order::Foreground)
        .fixed_pos(pos)
        .interactable(true)
        .show(ctx, |ui| {
            Frame::popup(ui.style()).show(ui, |ui| {
                ui.set_max_width(POPUP_MAX_WIDTH);
                ui.spacing_mut().item_spacing.y = 4.0;
                for (index, line) in entry.lines.iter().enumerate() {
                    if index == 0 {
                        ui.label(RichText::new(line).strong().size(14.0));
                    } else {
                        ui.label(RichText::new(line).size(12.0));
                    }
                }
            });
        });

    state.popup_rect = Some(popup_response.response.rect);

    let popup_rect = state.popup_rect;
    let should_close = ctx.input(|input| {
        input.pointer.any_pressed()
            && input
                .pointer
                .interact_pos()
                .is_some_and(|pointer| !popup_rect.is_some_and(|rect| rect.contains(pointer)))
    });
    if should_close && !skip_dismiss {
        state.close();
    }
}

pub struct HelpPlugin;

impl Plugin for HelpPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(HelpCatalog::load())
            .init_resource::<HelpPopupState>();
    }
}
