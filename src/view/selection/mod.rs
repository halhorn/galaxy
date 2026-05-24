mod marker;
mod pick;
mod readback;
mod snapshot;

pub use snapshot::SimulationCpuSnapshot;

use bevy::prelude::*;

use marker::draw_selection_marker;
use pick::click_pick_body;
use readback::{on_spawned_seed_snapshot, setup_readback, sync_readback_entities};

pub struct SelectionPlugin;

impl Plugin for SelectionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<pick::SelectedBody>()
            .init_resource::<pick::ClickPickerState>()
            .init_resource::<SimulationCpuSnapshot>()
            .add_systems(Startup, readback::configure_selection_gizmos)
            .add_systems(PostStartup, setup_readback)
            .add_systems(Update, on_spawned_seed_snapshot)
            .add_systems(Update, sync_readback_entities)
            .add_systems(Update, (click_pick_body, draw_selection_marker));
    }
}
