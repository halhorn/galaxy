//! Web 向け Bevy アプリの組み立て。

use bevy::prelude::*;
use bevy::render::settings::{RenderCreation, WgpuSettings, WgpuSettingsPriority};
use bevy::render::RenderPlugin;
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};

use crate::simulation::{render::BodiesMesh, SimulationPlugin};

/// ネイティブ・WASM 共通の `App` を組み立てて実行する。
pub fn run() {
    App::new()
        .add_plugins(DefaultPlugins.set(RenderPlugin {
            render_creation: RenderCreation::Automatic(WgpuSettings {
                // Safari / iOS WebGPU は Functionality 優先だとパイプラインが落ちることがある。
                priority: WgpuSettingsPriority::Compatibility,
                ..default()
            }),
            ..default()
        }).set(WindowPlugin {
            primary_window: Some(Window {
                title: "Galaxy — Gravity Simulator".to_string(),
                canvas: Some("#galaxy-canvas".into()),
                fit_canvas_to_parent: true,
                prevent_default_event_handling: true,
                ..default()
            }),
            ..default()
        }))
        .add_plugins((PanOrbitCameraPlugin, SimulationPlugin))
        .insert_resource(ClearColor(Color::BLACK))
        .add_systems(Startup, setup_camera)
        .add_systems(Update, hide_loading_when_ready)
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 80.0, 120.0).looking_at(Vec3::ZERO, Vec3::Y),
        PanOrbitCamera::default(),
    ));
}

/// シミュレーションの描画エンティティが揃ってからローディング UI を消す。
fn hide_loading_when_ready(bodies: Query<(), With<BodiesMesh>>, mut done: Local<bool>) {
    if *done || bodies.is_empty() {
        return;
    }
    *done = true;
    #[cfg(target_arch = "wasm32")]
    hide_web_loading_overlay();
}

/// `index.html` のローディングオーバーレイを非表示にする。
#[cfg(target_arch = "wasm32")]
fn hide_web_loading_overlay() {
    let Some(win) = web_sys::window() else {
        return;
    };
    let Some(doc) = win.document() else {
        return;
    };
    let Some(el) = doc.get_element_by_id("galaxy-loading") else {
        return;
    };
    let _ = el.set_attribute("hidden", "");
    let _ = el.set_attribute("aria-busy", "false");
}
