use bevy::{
    ecs::system::SystemParam,
    input::mouse::MouseButton,
    input::touch::Touches,
    prelude::*,
};
use bevy_panorbit_camera::EguiWantsFocus;

use crate::model::body::visual_radius;
use crate::model::constants::{BODY_COUNT, MIN_MASS};
use crate::simulation::SimulationConfig;
use crate::view::SimulationCamera;
use crate::view::selection::snapshot::SimulationCpuSnapshot;

/// Index of the currently selected body, if any.
#[derive(Resource, Default, Debug, Clone, Copy)]
pub struct SelectedBody(pub Option<usize>);

#[derive(Component)]
pub struct ReadbackPositions;

#[derive(Component)]
pub struct ReadbackMasses;

#[derive(Resource, Default)]
pub struct ClickPickerState {
    press_cursor: Option<Vec2>,
}

/// Max cursor movement (logical px) to count as a click rather than orbit drag.
const CLICK_DRAG_THRESHOLD_PX: f32 = 16.0;

#[derive(SystemParam)]
pub struct ClickPickInput<'w, 's> {
    mouse: Res<'w, ButtonInput<MouseButton>>,
    touches: Res<'w, Touches>,
    windows: Query<'w, 's, &'static Window>,
    egui_wants_focus: Res<'w, EguiWantsFocus>,
    config: Res<'w, SimulationConfig>,
    picker: ResMut<'w, ClickPickerState>,
    selected: ResMut<'w, SelectedBody>,
    snapshot: Res<'w, SimulationCpuSnapshot>,
    camera: Query<'w, 's, (&'static Camera, &'static GlobalTransform), With<SimulationCamera>>,
}

pub fn click_pick_body(mut input: ClickPickInput<'_, '_>) {
    if input.egui_wants_focus.prev || input.egui_wants_focus.curr {
        return;
    }

    let Ok(window) = input.windows.single() else {
        return;
    };

    if input.mouse.just_pressed(MouseButton::Left) {
        if let Some(cursor) = window.cursor_position() {
            input.picker.press_cursor = Some(cursor);
        }
        return;
    }

    let touch_pick = input
        .touches
        .iter_just_released()
        .find(|touch| touch.distance().length() <= CLICK_DRAG_THRESHOLD_PX)
        .map(|touch| touch.position());

    let click_pick = input
        .mouse
        .just_released(MouseButton::Left)
        .then(|| {
            let cursor = window.cursor_position()?;
            let press = input.picker.press_cursor.take()?;
            (press.distance(cursor) <= CLICK_DRAG_THRESHOLD_PX).then_some(cursor)
        })
        .flatten();

    let middle_pick = input
        .mouse
        .just_pressed(MouseButton::Middle)
        .then(|| window.cursor_position())
        .flatten();

    let Some(cursor) = touch_pick.or(middle_pick).or(click_pick) else {
        return;
    };

    pick_body_at_cursor(
        cursor,
        &input.snapshot,
        &input.camera,
        input.config.star_visual_scale,
        input.config.min_star_visual_scale,
        &mut input.selected,
    );
}

fn pick_body_at_cursor(
    cursor: Vec2,
    snapshot: &SimulationCpuSnapshot,
    camera: &Query<(&Camera, &GlobalTransform), With<SimulationCamera>>,
    star_visual_scale: f32,
    min_star_visual_scale: f32,
    selected: &mut SelectedBody,
) {
    if !snapshot.ready || snapshot.masses.len() < BODY_COUNT || snapshot.positions.len() < BODY_COUNT {
        return;
    }

    let Ok((camera, camera_transform)) = camera.single() else {
        return;
    };
    let Ok(ray) = camera.viewport_to_world(camera_transform, cursor) else {
        return;
    };

    let mut best: Option<(usize, f32)> = None;
    for i in 0..BODY_COUNT {
        let mass = snapshot.masses[i];
        if mass <= MIN_MASS {
            continue;
        }
        let center = snapshot.positions[i];
        let radius = visual_radius(mass, star_visual_scale, min_star_visual_scale);
        let Some(t) = ray_sphere_hit(ray.origin, *ray.direction, center, radius) else {
            continue;
        };
        if best.is_none_or(|(_, best_t)| t < best_t) {
            best = Some((i, t));
        }
    }

    selected.0 = best.map(|(i, _)| i);
}

fn ray_sphere_hit(origin: Vec3, dir: Vec3, center: Vec3, radius: f32) -> Option<f32> {
    let oc = origin - center;
    let b = oc.dot(dir);
    let c = oc.dot(oc) - radius * radius;
    let disc = b * b - c;
    if disc < 0.0 {
        return None;
    }
    let sqrt_disc = disc.sqrt();
    let t0 = -b - sqrt_disc;
    if t0 >= 0.0 {
        return Some(t0);
    }
    let t1 = -b + sqrt_disc;
    (t1 >= 0.0).then_some(t1)
}
