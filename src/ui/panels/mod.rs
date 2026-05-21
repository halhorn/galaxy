mod force;
mod initial;
mod playback;
mod physics;

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPrimaryContextPass};

use crate::simulation::{
    PlaybackState, SimulationConfig, SimulationViewportRect, SimViewportSystems,
    DESKTOP_PANEL_WIDTH, MOBILE_BREAKPOINT_PX, MOBILE_PANEL_HEIGHT,
};

use force::force_panel;
use initial::initial_panel;
use playback::playback_panel;
use physics::physics_panel;

const TITLE_TAB_SPACING: f32 = 10.0;
const MOBILE_BOTTOM_PADDING: f32 = 16.0;

#[derive(Resource, Default)]
struct FpsDisplay {
    fps: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum ControlTab {
    #[default]
    Playback,
    Physics,
    Initial,
    Force,
}

impl ControlTab {
    fn label(self, compact: bool) -> &'static str {
        match self {
            Self::Playback => "Playback",
            Self::Physics => "Physics",
            Self::Initial if compact => "Initial",
            Self::Initial => "Initial Conditions",
            Self::Force => "Force Law",
        }
    }

    const ALL: [Self; 4] = [Self::Playback, Self::Physics, Self::Initial, Self::Force];
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
    playback: &mut PlaybackState,
    config: &mut SimulationConfig,
    fps: f32,
) {
    match tab {
        ControlTab::Playback => playback_panel(ui, playback, config, fps),
        ControlTab::Physics => physics_panel(ui),
        ControlTab::Initial => initial_panel(ui),
        ControlTab::Force => force_panel(ui),
    }
}

fn egui_rect_to_bevy(rect: egui::Rect) -> Rect {
    Rect {
        min: Vec2::new(rect.min.x, rect.min.y),
        max: Vec2::new(rect.max.x, rect.max.y),
    }
}

fn draw_control_panel(
    mut contexts: EguiContexts,
    windows: Query<&Window>,
    mut playback: ResMut<PlaybackState>,
    mut config: ResMut<SimulationConfig>,
    mut viewport_rect: ResMut<SimulationViewportRect>,
    fps: Res<FpsDisplay>,
    mut tab: Local<ControlTab>,
) -> Result {
    let ctx = contexts.ctx_mut()?;
    let mobile = is_mobile_layout(&windows);

    let panel_contents = |ui: &mut egui::Ui| {
        ui.heading("Gravitium");
        ui.add_space(TITLE_TAB_SPACING);
        tab_bar(ui, &mut tab, mobile);
        ui.separator();
        active_tab_panel(ui, *tab, &mut playback, &mut config, fps.fps);
        if mobile {
            ui.add_space(MOBILE_BOTTOM_PADDING);
        }
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
