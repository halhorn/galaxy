use bevy::{
    ecs::system::SystemParam,
    input::mouse::MouseButton,
    input::touch::Touches,
    prelude::*,
};

use crate::model::body::visual_radius;
use crate::model::constants::MIN_MASS;
use crate::simulation::{
    pick_ray_in_rect, point_in_simulation_viewport, world_to_viewport_in_rect, SimulationConfig,
    SimulationSettings, SimulationViewportRect,
};
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
/// Minimum world-space pick/test radius so small disk bodies remain clickable.
const MIN_PICK_RADIUS: f32 = 0.35;
/// Minimum hit target radius in screen pixels.
const MIN_PICK_PX: f32 = 6.0;

#[derive(SystemParam)]
pub struct ClickPickInput<'w, 's> {
    mouse: Res<'w, ButtonInput<MouseButton>>,
    touches: Res<'w, Touches>,
    windows: Query<'w, 's, &'static Window>,
    viewport_rect: Res<'w, SimulationViewportRect>,
    config: Res<'w, SimulationConfig>,
    settings: Res<'w, SimulationSettings>,
    picker: ResMut<'w, ClickPickerState>,
    selected: ResMut<'w, SelectedBody>,
    snapshot: Res<'w, SimulationCpuSnapshot>,
    camera: Query<'w, 's, (&'static Camera, &'static GlobalTransform), With<SimulationCamera>>,
}

pub fn click_pick_body(mut input: ClickPickInput<'_, '_>) {
    let Ok(window) = input.windows.single() else {
        return;
    };

    if input.mouse.just_pressed(MouseButton::Left) {
        if let Some(cursor) = window.cursor_position() {
            input.picker.press_cursor = Some(cursor);
        }
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
        &input.viewport_rect,
        &input.snapshot,
        input.settings.active_count() as usize,
        &input.camera,
        input.config.star_visual_scale,
        input.config.min_star_visual_scale,
        &mut input.selected,
    );
}

fn pick_body_at_cursor(
    cursor: Vec2,
    viewport_rect: &SimulationViewportRect,
    snapshot: &SimulationCpuSnapshot,
    active_count: usize,
    camera: &Query<(&Camera, &GlobalTransform), With<SimulationCamera>>,
    star_visual_scale: f32,
    min_star_visual_scale: f32,
    selected: &mut SelectedBody,
) {
    if !snapshot.ready
        || snapshot.masses.len() < active_count
        || snapshot.positions.len() < active_count
    {
        return;
    }

    let Ok((camera, camera_transform)) = camera.single() else {
        return;
    };

    if !point_in_simulation_viewport(cursor, viewport_rect, camera) {
        return;
    }

    let viewport = viewport_rect.logical;
    let Some(pick_ray) = pick_ray_in_rect(camera, camera_transform, cursor, viewport) else {
        return;
    };

    let camera_pos = camera_transform.translation();
    let ray_dir = pick_ray.direction.as_vec3();
    let camera_right = camera_transform.right().as_vec3();

    // Among bodies under the cursor on screen, prefer smallest angle to the pick ray.
    let mut best: Option<(usize, f32, f32)> = None;

    for i in 0..active_count {
        let mass = snapshot.masses[i];
        if mass <= MIN_MASS {
            continue;
        }
        let center = snapshot.positions[i];
        let Some(screen) = world_to_viewport_in_rect(camera, camera_transform, center, viewport)
        else {
            continue;
        };

        let world_radius = visual_radius(mass, star_visual_scale, min_star_visual_scale)
            .max(MIN_PICK_RADIUS);
        let pick_radius_px = screen_pick_radius(
            camera,
            camera_transform,
            center,
            world_radius,
            camera_right,
            viewport,
        );
        let dist_sq = screen.distance_squared(cursor);
        if dist_sq > pick_radius_px * pick_radius_px {
            continue;
        }

        let to_center = center - camera_pos;
        let dist_len_sq = to_center.length_squared();
        if dist_len_sq < 1e-12 {
            continue;
        }
        let cos_align = ray_dir.dot(to_center / dist_len_sq.sqrt());

        match best {
            None => best = Some((i, cos_align, dist_sq)),
            Some((_, best_cos, _))
                if cos_align > best_cos + f32::EPSILON =>
            {
                best = Some((i, cos_align, dist_sq));
            }
            Some((_, best_cos, best_dist))
                if (cos_align - best_cos).abs() <= f32::EPSILON && dist_sq < best_dist =>
            {
                best = Some((i, cos_align, dist_sq));
            }
            _ => {}
        }
    }

    selected.0 = best.map(|(i, _, _)| i);
}

fn screen_pick_radius(
    camera: &Camera,
    camera_transform: &GlobalTransform,
    center: Vec3,
    world_radius: f32,
    camera_right: Vec3,
    viewport: Rect,
) -> f32 {
    let center_screen =
        world_to_viewport_in_rect(camera, camera_transform, center, viewport).unwrap_or(Vec2::ZERO);
    let edge_screen = world_to_viewport_in_rect(
        camera,
        camera_transform,
        center + camera_right * world_radius,
        viewport,
    )
    .unwrap_or(center_screen);
    center_screen.distance(edge_screen).max(MIN_PICK_PX)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::simulation::world_to_viewport_in_rect;

    #[test]
    fn ndc_roundtrip_within_viewport_rect() {
        let viewport = Rect {
            min: Vec2::new(300.0, 0.0),
            max: Vec2::new(900.0, 600.0),
        };
        let cursor = Vec2::new(600.0, 300.0);
        let rel = (cursor - viewport.min) / viewport.size();
        let mut ndc = rel * 2.0 - Vec2::ONE;
        ndc.y = -ndc.y;

        let mut ndc_back = ndc;
        ndc_back.y = -ndc_back.y;
        let roundtrip = (ndc_back + Vec2::ONE) / 2.0 * viewport.size() + viewport.min;
        assert!(roundtrip.distance(cursor) < 1e-4);
    }

    #[test]
    fn world_to_viewport_in_rect_matches_manual_center_for_identity_camera() {
        let mut camera = Camera::default();
        camera.computed.clip_from_view = Mat4::orthographic_rh(-1.0, 1.0, -1.0, 1.0, 0.1, 1000.0);
        let transform = GlobalTransform::from(Transform::from_xyz(0.0, 0.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y));
        let viewport = Rect::from_corners(Vec2::ZERO, Vec2::new(600.0, 400.0));

        let screen = world_to_viewport_in_rect(&camera, &transform, Vec3::ZERO, viewport)
            .expect("origin should project");
        assert!((screen - viewport.center()).length() < 1.0);
    }

    #[test]
    fn prefers_smaller_angle_over_closer_depth() {
        let ray_dir = Vec3::NEG_Z;
        let near = Vec3::new(0.0, 0.0, 10.0);
        let far = Vec3::new(0.05, 0.0, 100.0);

        let cos_near = ray_dir.dot((near - Vec3::ZERO).normalize());
        let cos_far = ray_dir.dot((far - Vec3::ZERO).normalize());

        assert!(cos_far > cos_near);
    }
}
