use bevy::prelude::*;

use crate::model::InitialConditions;

/// Draft values for initial conditions (applied only via Apply & Restart).
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
