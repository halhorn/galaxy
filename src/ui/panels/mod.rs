mod display;
mod initial;
mod physics;

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPrimaryContextPass};

use crate::simulation::{
    PlaybackState, SimulationConfig, SimulationSettings, SimulationViewportRect,
    SimViewportSystems, DESKTOP_PANEL_WIDTH, MOBILE_BREAKPOINT_PX, MOBILE_PANEL_HEIGHT,
};

use display::display_panel;
use initial::initial_panel;
use physics::physics_panel;

use crate::ui::apply::UiPendingActions;
use crate::ui::draft::ControlPanelDraft;

const PANEL_TOP_PADDING: f32 = 10.0 * 2.0 / 3.0;
const TITLE_TAB_SPACING: f32 = 12.0;
const MOBILE_BOTTOM_PADDING: f32 = 16.0;
const ICON_BUTTON_SIZE: egui::Vec2 = egui::Vec2::new(28.0, 28.0);
const RESTART_PLAY_GAP: f32 = 6.0;
const STATS_BUTTON_GAP: f32 = 6.0;

#[derive(Resource, Default)]
struct FpsDisplay {
    fps: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
enum ControlTab {
    #[default]
    Initial,
    Physics,
    Display,
}

impl ControlTab {
    fn label(self, compact: bool) -> &'static str {
        match self {
            Self::Physics => "Physics",
            Self::Display => "Display",
            Self::Initial if compact => "Initial",
            Self::Initial => "Initial Conditions",
        }
    }

    const ALL: [Self; 3] = [Self::Initial, Self::Physics, Self::Display];
}

fn update_fps_display(time: Res<Time>, mut fps: ResMut<FpsDisplay>, mut smoothed: Local<f32>) {
    let dt = time.delta_secs();
    if dt > 0.0 {
        let instant = 1.0 / dt;
        *smoothed = *smoothed * 0.9 + instant * 0.1;
        fps.fps = *smoothed;
    }
}

fn is_mobile_layout(windows: &Query<&Window>) -> bool {
    windows
        .single()
        .is_ok_and(|window| window.width() < MOBILE_BREAKPOINT_PX)
}

fn icon_button(ui: &mut egui::Ui, icon: &str, tooltip: &str) -> egui::Response {
    ui.add(
        egui::Button::new(egui::RichText::new(icon).size(16.0)).min_size(ICON_BUTTON_SIZE),
    )
    .on_hover_text(tooltip)
}

fn playback_controls(
    ui: &mut egui::Ui,
    playback: &mut PlaybackState,
    pending: &mut UiPendingActions,
) {
    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
        if icon_button(ui, "↻", "Restart").clicked() {
            pending.restart = true;
        }
        ui.add_space(RESTART_PLAY_GAP);

        let (icon, tooltip) = if playback.is_running() {
            ("⏹", "Stop")
        } else {
            ("▶", "Play")
        };
        if icon_button(ui, icon, tooltip).clicked() {
            playback.toggle();
        }
    });
}

fn stats_text(sim_time: f32, fps: f32) -> String {
    format!("{sim_time:.2} yr · {fps:.0} FPS")
}

fn panel_header(
    ui: &mut egui::Ui,
    playback: &mut PlaybackState,
    fps: f32,
    pending: &mut UiPendingActions,
) {
    ui.horizontal(|ui| {
        ui.set_min_height(ICON_BUTTON_SIZE.y);
        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
            ui.heading("Gravitium");
        });

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            playback_controls(ui, playback, pending);
            ui.add_space(STATS_BUTTON_GAP);
            ui.label(stats_text(playback.accumulated_sim_time, fps));
        });
    });
}

fn tab_bar(ui: &mut egui::Ui, tab: &mut ControlTab, compact: bool) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = if compact { 6.0 } else { 8.0 };
        for candidate in ControlTab::ALL {
            if ui
                .selectable_label(*tab == candidate, candidate.label(compact))
                .clicked()
            {
                *tab = candidate;
            }
        }
    });
}

fn active_tab_panel(
    ui: &mut egui::Ui,
    tab: ControlTab,
    config: &mut SimulationConfig,
    settings: &mut SimulationSettings,
    draft: &mut ControlPanelDraft,
) {
    match tab {
        ControlTab::Physics => physics_panel(ui, settings),
        ControlTab::Initial => initial_panel(ui, draft),
        ControlTab::Display => {
            display_panel(ui, config);
            *config = config.clone().clamped();
        }
    }
}

fn egui_rect_to_bevy(rect: egui::Rect) -> Rect {
    Rect {
        min: Vec2::new(rect.min.x, rect.min.y),
        max: Vec2::new(rect.max.x, rect.max.y),
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_control_panel(
    mut contexts: EguiContexts,
    windows: Query<&Window>,
    mut playback: ResMut<PlaybackState>,
    mut config: ResMut<SimulationConfig>,
    mut viewport_rect: ResMut<SimulationViewportRect>,
    mut settings: ResMut<SimulationSettings>,
    mut draft: ResMut<ControlPanelDraft>,
    mut pending: ResMut<UiPendingActions>,
    fps: Res<FpsDisplay>,
    mut tab: Local<ControlTab>,
) -> Result {
    let ctx = contexts.ctx_mut()?;
    let mobile = is_mobile_layout(&windows);

    let panel_contents = |ui: &mut egui::Ui| {
        ui.add_space(PANEL_TOP_PADDING);
        panel_header(ui, &mut playback, fps.fps, &mut pending);
        ui.add_space(TITLE_TAB_SPACING);
        tab_bar(ui, &mut tab, mobile);
        ui.separator();

        let scroll_height = ui.available_height();
        egui::ScrollArea::vertical()
            .id_salt(*tab)
            .max_height(scroll_height)
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.set_width(ui.max_rect().width());
                active_tab_panel(
                    ui,
                    *tab,
                    &mut config,
                    &mut settings,
                    &mut draft,
                );
                if mobile {
                    ui.add_space(MOBILE_BOTTOM_PADDING);
                }
            });
    };

    if mobile {
        egui::TopBottomPanel::bottom("control_panel")
            .exact_height(MOBILE_PANEL_HEIGHT)
            .resizable(false)
            .show(ctx, panel_contents);
    } else {
        egui::SidePanel::left("control_panel")
            .default_width(DESKTOP_PANEL_WIDTH)
            .resizable(true)
            .show(ctx, panel_contents);
    }

    if windows.single().is_ok() {
        viewport_rect.logical = egui_rect_to_bevy(ctx.available_rect());
    } else {
        viewport_rect.logical = Rect::from_corners(Vec2::ZERO, Vec2::ONE);
    }

    Ok(())
}

pub struct ControlPanelsPlugin;

impl Plugin for ControlPanelsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FpsDisplay>()
            .add_systems(Update, update_fps_display)
            .add_systems(
                EguiPrimaryContextPass,
                draw_control_panel.in_set(SimViewportSystems::Layout),
            );
    }
}
