use bevy::prelude::*;

use crate::model::body::visual_radius;
use crate::model::constants::{BODY_COUNT, MIN_MASS};
use crate::simulation::SimulationConfig;
use crate::view::SimulationCamera;
use crate::view::selection::pick::SelectedBody;
use crate::view::selection::snapshot::SimulationCpuSnapshot;

const MARKER_COLOR: Color = Color::srgba(1.0, 0.22, 0.18, 0.95);
const RING_SCALE: f32 = 1.35;
const CROSS_GAP_SCALE: f32 = 1.05;
/// Crosshair arm length beyond the ring, in units of star radius.
const CROSS_ARM_SCALE: f32 = 4.05;

pub fn draw_selection_marker(
    selected: Res<SelectedBody>,
    snapshot: Res<SimulationCpuSnapshot>,
    config: Res<SimulationConfig>,
    camera: Query<&GlobalTransform, With<SimulationCamera>>,
    mut gizmos: Gizmos,
) {
    let Some(index) = selected.0 else {
        return;
    };
    if !snapshot.ready || index >= BODY_COUNT {
        return;
    }
    let mass = snapshot.masses[index];
    if mass <= MIN_MASS {
        return;
    }
    let center = snapshot.positions[index];
    let Ok(camera_transform) = camera.single() else {
        return;
    };

    let star_r =
        visual_radius(mass, config.star_visual_scale, config.min_star_visual_scale);
    let ring_r = star_r * RING_SCALE;
    let to_camera = (camera_transform.translation() - center).normalize_or_zero();
    if to_camera.length_squared() < 1e-8 {
        return;
    }

    let circle_rotation = Quat::from_rotation_arc(Vec3::Z, to_camera);
    gizmos
        .circle(
            Isometry3d::new(center, circle_rotation),
            ring_r,
            MARKER_COLOR,
        )
        .resolution(48);

    let right = camera_transform.right().as_vec3();
    let up = camera_transform.up().as_vec3();

    let inner = ring_r * CROSS_GAP_SCALE;
    let outer = ring_r + star_r * CROSS_ARM_SCALE;
    for dir in [right, -right, up, -up] {
        gizmos.line(center + dir * inner, center + dir * outer, MARKER_COLOR);
    }
}
