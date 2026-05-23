use bevy::prelude::*;

use crate::model::{ForceLaw, InitialConditions};

/// Draft values for restart-only panels (initial conditions, force law).
#[derive(Resource, Debug, Clone, PartialEq)]
pub struct ControlPanelDraft {
    pub initial: InitialConditions,
    pub force: ForceLaw,
}

impl Default for ControlPanelDraft {
    fn default() -> Self {
        let physics = crate::model::PhysicsSettings::default();
        Self {
            initial: InitialConditions::default(),
            force: ForceLaw::newtonian(physics.g),
        }
    }
}
