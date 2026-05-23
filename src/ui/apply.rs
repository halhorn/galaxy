use bevy::prelude::*;
use bevy_egui::EguiPrimaryContextPass;

use crate::simulation::{
    restart_simulation, PlaybackMode, PlaybackState, SimulationCommand, SimulationConfig,
    SimulationRestartSet, SimulationSettings, SimViewportSystems,
};
use crate::url::AppliedUrlState;

use super::draft::ControlPanelDraft;

/// Flags set during egui layout, processed after the frame.
#[derive(Resource, Default)]
pub struct UiPendingActions {
    pub restart: bool,
    pub reset_all: bool,
}

pub fn process_pending_actions(
    mut pending: ResMut<UiPendingActions>,
    mut draft: ResMut<ControlPanelDraft>,
    mut settings: ResMut<SimulationSettings>,
    mut config: ResMut<SimulationConfig>,
    mut playback: ResMut<PlaybackState>,
    mut commands: MessageWriter<SimulationCommand>,
) {
    if pending.reset_all {
        pending.reset_all = false;
        let state = AppliedUrlState::default().clamped();
        state.apply_to_resources(&mut settings, &mut config);
        draft.initial = settings.initial.clone();
        playback.mode = PlaybackMode::Running;
        pending.restart = true;
    }

    if !pending.restart {
        return;
    }
    pending.restart = false;

    settings.initial = draft.initial.clone().clamped();
    draft.initial = settings.initial.clone();
    commands.write(SimulationCommand::Restart);
}

pub struct UiApplyPlugin;

impl Plugin for UiApplyPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ControlPanelDraft>()
            .init_resource::<UiPendingActions>()
            .add_systems(
                EguiPrimaryContextPass,
                (
                    process_pending_actions.in_set(SimulationRestartSet),
                    restart_simulation.in_set(SimulationRestartSet),
                )
                    .chain()
                    .after(SimViewportSystems::Layout),
            );
    }
}
