//! Click/tap-to-select a body and draw a camera-facing target reticle.

use bevy::{
    input::mouse::MouseButton,
    input::touch::Touches,
    prelude::*,
    render::gpu_readback::{Readback, ReadbackComplete},
};
use bytemuck::pod_read_unaligned;

use super::{
    buffers::SimulationGpuBuffers,
    constants::{BODY_COUNT, MIN_MASS},
};

/// Index of the currently selected body, if any.
#[derive(Resource, Default, Debug, Clone, Copy)]
pub struct SelectedBody(pub Option<usize>);

/// CPU mirror of GPU positions/masses (updated via [`Readback`]).
#[derive(Resource, Default)]
pub struct SimulationCpuSnapshot {
    pub positions: Vec<Vec3>,
    pub masses: Vec<f32>,
    pub ready: bool,
}

#[derive(Component)]
struct ReadbackPositions;

#[derive(Component)]
struct ReadbackMasses;

#[derive(Resource, Default)]
struct ClickPickerState {
    press_cursor: Option<Vec2>,
}

/// Max cursor movement (logical px) to count as a click rather than orbit drag.
const CLICK_DRAG_THRESHOLD_PX: f32 = 6.0;

const MARKER_COLOR: Color = Color::srgba(1.0, 0.22, 0.18, 0.95);
const RING_SCALE: f32 = 1.35;
const CROSS_GAP_SCALE: f32 = 1.05;
/// Crosshair arm length beyond the ring, in units of body radius.
const CROSS_ARM_SCALE: f32 = 4.05;
/// Minimum world-space pick/test radius so small disk bodies remain clickable.
const MIN_PICK_RADIUS: f32 = 0.35;

pub struct SelectionPlugin;

impl Plugin for SelectionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SelectedBody>()
            .init_resource::<ClickPickerState>()
            .add_systems(Startup, configure_selection_gizmos)
            .add_systems(PostStartup, setup_readback)
            .add_systems(Update, (click_pick_body, draw_selection_marker));
    }
}

fn configure_selection_gizmos(mut config_store: ResMut<GizmoConfigStore>) {
    let (config, _) = config_store.config_mut::<DefaultGizmoConfigGroup>();
    config.depth_bias = -0.95;
    config.line.width = 3.0;
}

fn setup_readback(mut commands: Commands, gpu: Res<SimulationGpuBuffers>) {
    commands
        .spawn((ReadbackPositions, Readback::buffer(gpu.positions.clone())))
        .observe(on_positions_readback);
    commands
        .spawn((ReadbackMasses, Readback::buffer(gpu.masses.clone())))
        .observe(on_masses_readback);
}

fn on_positions_readback(
    trigger: On<ReadbackComplete>,
    mut snapshot: ResMut<SimulationCpuSnapshot>,
) {
    snapshot.positions = parse_positions_readback(&trigger.event().data);
    if snapshot.positions.len() == BODY_COUNT && snapshot.masses.len() == BODY_COUNT {
        snapshot.ready = true;
    }
}

fn on_masses_readback(trigger: On<ReadbackComplete>, mut snapshot: ResMut<SimulationCpuSnapshot>) {
    snapshot.masses = parse_masses_readback(&trigger.event().data);
    if snapshot.positions.len() == BODY_COUNT && snapshot.masses.len() == BODY_COUNT {
        snapshot.ready = true;
    }
}

/// GPU readback returns a `Vec<u8>` that is not necessarily aligned for `cast_slice`.
fn parse_positions_readback(data: &[u8]) -> Vec<Vec3> {
    data.chunks_exact(16)
        .take(BODY_COUNT)
        .map(|chunk| pod_read_unaligned::<Vec4>(chunk).truncate())
        .collect()
}

fn parse_masses_readback(data: &[u8]) -> Vec<f32> {
    data.chunks_exact(4)
        .take(BODY_COUNT)
        .map(|chunk| pod_read_unaligned::<f32>(chunk))
        .collect()
}

fn click_pick_body(
    mouse: Res<ButtonInput<MouseButton>>,
    touches: Res<Touches>,
    windows: Query<&Window>,
    mut picker: ResMut<ClickPickerState>,
    mut selected: ResMut<SelectedBody>,
    snapshot: Res<SimulationCpuSnapshot>,
    camera: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
) {
    let Ok(window) = windows.single() else {
        return;
    };

    if mouse.just_pressed(MouseButton::Left) {
        if let Some(cursor) = window.cursor_position() {
            picker.press_cursor = Some(cursor);
        }
        return;
    }

    // Mobile Safari has no cursor; use touch release with minimal movement as a tap.
    let touch_pick = touches
        .iter_just_released()
        .find(|touch| touch.distance().length() <= CLICK_DRAG_THRESHOLD_PX)
        .map(|touch| touch.position());

    let click_pick = mouse.just_released(MouseButton::Left).then(|| {
        let cursor = window.cursor_position()?;
        let press = picker.press_cursor.take()?;
        (press.distance(cursor) <= CLICK_DRAG_THRESHOLD_PX).then_some(cursor)
    }).flatten();

    // Middle click always selects (avoids fighting orbit drag).
    let middle_pick = mouse
        .just_pressed(MouseButton::Middle)
        .then(|| window.cursor_position())
        .flatten();

    let Some(cursor) = touch_pick.or(middle_pick).or(click_pick) else {
        return;
    };

    pick_body_at_cursor(
        cursor,
        &snapshot,
        &camera,
        &mut selected,
    );
}

fn pick_body_at_cursor(
    cursor: Vec2,
    snapshot: &SimulationCpuSnapshot,
    camera: &Query<(&Camera, &GlobalTransform), With<Camera3d>>,
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
        let radius = visual_radius(mass).max(MIN_PICK_RADIUS);
        let Some(t) = ray_sphere_hit(ray.origin, *ray.direction, center, radius) else {
            continue;
        };
        if best.is_none_or(|(_, best_t)| t < best_t) {
            best = Some((i, t));
        }
    }

    selected.0 = best.map(|(i, _)| i);
}

fn draw_selection_marker(
    selected: Res<SelectedBody>,
    snapshot: Res<SimulationCpuSnapshot>,
    camera: Query<&GlobalTransform, With<Camera3d>>,
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

    let body_r = visual_radius(mass).max(MIN_PICK_RADIUS);
    let ring_r = body_r * RING_SCALE;
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
    let outer = ring_r + body_r * CROSS_ARM_SCALE;
    for dir in [right, -right, up, -up] {
        gizmos.line(center + dir * inner, center + dir * outer, MARKER_COLOR);
    }
}

/// Matches `bodies.wgsl`: `radius = 0.5 * mass^(1/3)`, mesh scale `* 2`.
fn visual_radius(mass: f32) -> f32 {
    0.5 * mass.powf(1.0 / 3.0) * 2.0
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
