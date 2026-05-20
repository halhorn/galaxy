//! Web 向け Bevy アプリの組み立て。

use bevy::prelude::*;
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};

/// ネイティブ・WASM 共通の `App` を組み立てて実行する。
pub fn run() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Galaxy — Gravity Simulator".to_string(),
                canvas: Some("#galaxy-canvas".into()),
                fit_canvas_to_parent: true,
                prevent_default_event_handling: true,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(PanOrbitCameraPlugin)
        .insert_resource(ClearColor(Color::BLACK))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 80.0, 120.0).looking_at(Vec3::ZERO, Vec3::Y),
        PanOrbitCamera::default(),
    ));

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
