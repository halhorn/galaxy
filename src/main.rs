use bevy::post_process::bloom::Bloom;
use bevy::prelude::*;
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use galaxy::physics::components::*;
use galaxy::physics::PhysicsPlugin;
use rand::RngExt;
use std::f32::consts::PI;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Galaxy — Gravity Simulator".to_string(),
                resolution: (1280, 720).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(PhysicsPlugin)
        .add_plugins(PanOrbitCameraPlugin)
        .insert_resource(ClearColor(Color::BLACK))
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Camera with orbit controls + bloom for emissive glow
    // Bloom requires(Hdr) automatically in Bevy 0.18
    commands.spawn((
        Camera3d::default(),
        Bloom::default(),
        Transform::from_xyz(0.0, 80.0, 120.0).looking_at(Vec3::ZERO, Vec3::Y),
        PanOrbitCamera::default(),
    ));

    spawn_triple_star_system(&mut commands, &mut meshes, &mut materials);
}

fn spawn_triple_star_system(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    let mut rng = rand::rng();
    let g = 4.0 * PI * PI; // G in AU³/(M☉·yr²)

    // --- Triple star: equilateral triangle in stable circular orbit ---
    let star_mass: f32 = 100.0;
    let orbit_radius: f32 = 10.0; // Each star at 10 AU from center

    // Circular orbit velocity for 3 equal masses in Lagrange configuration:
    // v = sqrt(G·M / (√3·R))
    let v_star = (g * star_mass / (3.0_f32.sqrt() * orbit_radius)).sqrt();

    let star_mesh = meshes.add(Sphere::new(1.5));
    let star_colors = [
        LinearRgba::new(10.0, 8.0, 2.0, 1.0),  // A: golden
        LinearRgba::new(3.0, 5.0, 12.0, 1.0),   // B: blue-white
        LinearRgba::new(12.0, 3.0, 2.0, 1.0),   // C: red-orange
    ];

    for i in 0..3 {
        let angle = i as f32 * 2.0 * PI / 3.0;
        let position = Vec3::new(
            orbit_radius * angle.cos(),
            0.0,
            orbit_radius * angle.sin(),
        );
        // Tangential velocity (counter-clockwise)
        let velocity = Vec3::new(-angle.sin(), 0.0, angle.cos()) * v_star;

        commands.spawn((
            SimulationBody,
            Mass(star_mass),
            Velocity(velocity),
            Acceleration::default(),
            Mesh3d(star_mesh.clone()),
            MeshMaterial3d(materials.add(StandardMaterial {
                emissive: star_colors[i],
                ..default()
            })),
            Transform::from_translation(position),
        ));
    }

    // --- Disk bodies orbiting the triple system ---
    let total_mass = star_mass * 3.0;
    let body_mass: f32 = 0.1;
    let n_bodies = 9997;
    let r_min: f32 = 20.0;
    let r_max: f32 = 60.0;

    let body_mesh = meshes.add(Sphere::new(0.3));
    let body_material = materials.add(StandardMaterial {
        emissive: LinearRgba::new(1.5, 2.0, 4.0, 1.0),
        ..default()
    });

    for _ in 0..n_bodies {
        let u: f32 = rng.random_range(0.0..1.0);
        let r = (r_min * r_min + u * (r_max * r_max - r_min * r_min)).sqrt();
        let theta: f32 = rng.random_range(0.0..2.0 * PI);
        let height: f32 = rng.random_range(-0.5..0.5);

        let position = Vec3::new(r * theta.cos(), height, r * theta.sin());

        let v_mag = (g * total_mass / r).sqrt();
        let velocity = Vec3::new(-theta.sin(), 0.0, theta.cos()) * v_mag;

        commands.spawn((
            SimulationBody,
            Mass(body_mass),
            Velocity(velocity),
            Acceleration::default(),
            Mesh3d(body_mesh.clone()),
            MeshMaterial3d(body_material.clone()),
            Transform::from_translation(position),
        ));
    }
}
