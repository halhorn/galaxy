use bevy::prelude::*;
use bevy_egui::EguiPrimaryContextPass;

use crate::simulation::{
    restart_simulation, SimulationCommand, SimulationRestartSet, SimulationSettings,
    SimViewportSystems,
};

use super::draft::ControlPanelDraft;

/// Flags set during egui layout, processed after the frame.
#[derive(Resource, Default)]
pub struct UiPendingActions {
    pub restart: bool,
}

pub fn process_pending_actions(
    mut pending: ResMut<UiPendingActions>,
    mut draft: ResMut<ControlPanelDraft>,
    mut settings: ResMut<SimulationSettings>,
    mut commands: MessageWriter<SimulationCommand>,
) {
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
