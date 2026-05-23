use bevy::prelude::*;

use crate::model::InitialConditions;

/// Draft values for restart-only panels (initial conditions).
#[derive(Resource, Debug, Clone, PartialEq)]
pub struct ControlPanelDraft {
    pub initial: InitialConditions,
}

impl Default for ControlPanelDraft {
    fn default() -> Self {
        Self {
            initial: InitialConditions::default(),
        }
    }
}
