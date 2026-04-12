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

    spawn_star_system(&mut commands, &mut meshes, &mut materials);
}

fn spawn_star_system(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    let mut rng = rand::rng();
    let g = 4.0 * PI * PI; // G in AU³/(M☉·yr²)

    // --- Configuration ---
    let n_stars: usize = 2;
    let star_mass: f32 = 1.0;
    let star_orbit_radius: f32 = 10.0;
    let n_disk_bodies: usize = 10000;
    let disk_body_mass: f32 = 0.1;
    let disk_r_min: f32 = 5.0;
    let disk_r_max: f32 = 60.0;
    let disk_height: f32 = 0.5;
    let initial_v_perturbation: f32 = 0.05;
    let orbital_speed_factor: f32 = 2.0;

    // --- Stars ---
    let star_radius = radius_from_mass(star_mass);
    let star_mesh = meshes.add(Sphere::new(star_radius));
    let star_material = materials.add(StandardMaterial {
        emissive: color_from_mass(star_mass),
        ..default()
    });

    if n_stars == 1 {
        commands.spawn((
            SimulationBody,
            Mass(star_mass),
            Radius(star_radius),
            Velocity(Vec3::ZERO),
            Acceleration::default(),
            Mesh3d(star_mesh.clone()),
            MeshMaterial3d(star_material.clone()),
            Transform::from_translation(Vec3::ZERO),
        ));
    } else {
        let chord_sum: f32 = (1..n_stars)
            .map(|k| 1.0 / (2.0 * (PI * k as f32 / n_stars as f32).sin()))
            .sum();
        let v_star = (g * star_mass * chord_sum / star_orbit_radius).sqrt();

        for i in 0..n_stars {
            let angle = i as f32 * 2.0 * PI / n_stars as f32;
            let position = Vec3::new(
                star_orbit_radius * angle.cos(),
                0.0,
                star_orbit_radius * angle.sin(),
            );
            let velocity = Vec3::new(-angle.sin(), 0.0, angle.cos()) * v_star;

            commands.spawn((
                SimulationBody,
                Mass(star_mass),
                Radius(star_radius),
                Velocity(velocity),
                Acceleration::default(),
                Mesh3d(star_mesh.clone()),
                MeshMaterial3d(star_material.clone()),
                Transform::from_translation(position),
            ));
        }
    }

    // --- Disk bodies ---
    let total_mass = star_mass * n_stars as f32;
    let n_bodies = n_disk_bodies - n_stars;

    let body_radius = radius_from_mass(disk_body_mass);
    let body_mesh = meshes.add(Sphere::new(body_radius));
    let body_material = materials.add(StandardMaterial {
        emissive: color_from_mass(disk_body_mass),
        ..default()
    });

    for _ in 0..n_bodies {
        let u: f32 = rng.random_range(0.0..1.0);
        let r = (disk_r_min * disk_r_min + u * (disk_r_max * disk_r_max - disk_r_min * disk_r_min)).sqrt();
        let theta: f32 = rng.random_range(0.0..2.0 * PI);
        let height: f32 = rng.random_range(-disk_height..disk_height);

        let position = Vec3::new(r * theta.cos(), height, r * theta.sin());

        let v_mag = (g * total_mass / r).sqrt() * orbital_speed_factor;
        let vr = v_mag * rng.random_range(-initial_v_perturbation..initial_v_perturbation);
        let vt = v_mag * (1.0 + rng.random_range(-initial_v_perturbation..initial_v_perturbation));
        let vy = v_mag * rng.random_range(-initial_v_perturbation..initial_v_perturbation);
        let tangent = Vec3::new(-theta.sin(), 0.0, theta.cos());
        let radial = Vec3::new(theta.cos(), 0.0, theta.sin());
        let velocity = tangent * vt + radial * vr + Vec3::Y * vy;

        commands.spawn((
            SimulationBody,
            Mass(disk_body_mass),
            Radius(body_radius),
            Velocity(velocity),
            Acceleration::default(),
            Mesh3d(body_mesh.clone()),
            MeshMaterial3d(body_material.clone()),
            Transform::from_translation(position),
        ));
    }
}
