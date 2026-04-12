use std::collections::HashMap;

use bevy::prelude::*;

use super::components::*;

/// Fraction of combined radii at which bodies merge.
/// 1.0 = surface touch, 0.1 = must overlap to 10% of combined radii.
const MERGE_RADIUS_FACTOR: f32 = 0.1;

/// Spatial grid cell index.
type CellKey = (i32, i32, i32);

fn cell_key(pos: Vec3, inv_cell: f32) -> CellKey {
    (
        (pos.x * inv_cell).floor() as i32,
        (pos.y * inv_cell).floor() as i32,
        (pos.z * inv_cell).floor() as i32,
    )
}

/// System that merges bodies that deeply overlap.
/// Uses spatial hashing to avoid O(N²) pair checks.
pub fn merge_colliding_bodies(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut query: Query<
        (Entity, &mut Transform, &mut Mass, &mut Velocity, &Radius, &mut Acceleration),
        With<SimulationBody>,
    >,
) {
    // Collect snapshot
    let bodies: Vec<_> = query
        .iter()
        .map(|(e, t, m, v, r, _)| (e, t.translation, m.0, v.0, r.0))
        .collect();

    let n = bodies.len();
    if n == 0 {
        return;
    }

    // Cell size = 2 * max_radius * MERGE_RADIUS_FACTOR (max possible merge distance)
    let max_radius = bodies.iter().map(|b| b.4).fold(0.0_f32, f32::max);
    let cell_size = (2.0 * max_radius * MERGE_RADIUS_FACTOR).max(0.01);
    let inv_cell = 1.0 / cell_size;

    // Build spatial grid: cell → list of body indices
    let mut grid: HashMap<CellKey, Vec<usize>> = HashMap::new();
    for (i, b) in bodies.iter().enumerate() {
        grid.entry(cell_key(b.1, inv_cell)).or_default().push(i);
    }

    let mut absorbed = vec![false; n];

    // Check pairs only within same and neighboring cells
    for i in 0..n {
        if absorbed[i] {
            continue;
        }
        let (cx, cy, cz) = cell_key(bodies[i].1, inv_cell);
        for dx in -1..=1 {
            for dy in -1..=1 {
                for dz in -1..=1 {
                    let Some(cell) = grid.get(&(cx + dx, cy + dy, cz + dz)) else {
                        continue;
                    };
                    for &j in cell {
                        if j <= i || absorbed[j] {
                            continue;
                        }
                        let dist = (bodies[i].1 - bodies[j].1).length();
                        let touch_dist = (bodies[i].4 + bodies[j].4) * MERGE_RADIUS_FACTOR;
                        if dist < touch_dist {
                            let m_i = bodies[i].2;
                            let m_j = bodies[j].2;
                            let new_mass = m_i + m_j;
                            let new_vel = (bodies[i].3 * m_i + bodies[j].3 * m_j) / new_mass;
                            let new_pos = (bodies[i].1 * m_i + bodies[j].1 * m_j) / new_mass;

                            if let Ok((_, mut transform, mut mass, mut vel, _, mut acc)) =
                                query.get_mut(bodies[i].0)
                            {
                                mass.0 = new_mass;
                                vel.0 = new_vel;
                                transform.translation = new_pos;
                                acc.0 = Vec3::ZERO;
                            }

                            commands.entity(bodies[j].0).despawn();
                            absorbed[j] = true;
                        }
                    }
                }
            }
        }
    }

    // Update radius, mesh, and color for merged bodies
    for i in 0..n {
        if absorbed[i] {
            continue;
        }
        if let Ok((entity, _, mass, _, _, _)) = query.get(bodies[i].0) {
            if (mass.0 - bodies[i].2).abs() > 1e-6 {
                let new_radius = radius_from_mass(mass.0);
                let new_color = color_from_mass(mass.0);
                commands.entity(entity).insert((
                    Radius(new_radius),
                    Mesh3d(meshes.add(Sphere::new(new_radius))),
                    MeshMaterial3d(materials.add(StandardMaterial {
                        emissive: new_color,
                        ..default()
                    })),
                ));
            }
        }
    }
}
