use bevy::prelude::*;

/// Returns the intersection of a camera ray with the plane through `focus`
/// whose normal is the camera forward direction.
///
/// Falls back to `focus` when the ray is parallel to the plane or points away
/// from it (intersection behind the camera).
pub fn zoom_pivot_on_focus_plane(
    ray_origin: Vec3,
    ray_dir: Vec3,
    focus: Vec3,
    plane_normal: Vec3,
) -> Vec3 {
    let denom = ray_dir.dot(plane_normal);
    if denom.abs() < 1e-6 {
        return focus;
    }

    let t = (focus - ray_origin).dot(plane_normal) / denom;
    if t < 0.0 {
        return focus;
    }

    ray_origin + ray_dir * t
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn intersects_plane_at_expected_point() {
        let focus = Vec3::new(0.0, 0.0, 0.0);
        let normal = Vec3::NEG_Z;
        let origin = Vec3::new(0.0, 0.0, 10.0);
        let dir = Vec3::NEG_Z;

        let pivot = zoom_pivot_on_focus_plane(origin, dir, focus, normal);
        assert!(pivot.abs_diff_eq(focus, 1e-5));
    }

    #[test]
    fn offset_cursor_hits_offset_pivot() {
        let focus = Vec3::new(0.0, 0.0, 0.0);
        let normal = Vec3::NEG_Z;
        let origin = Vec3::new(5.0, 0.0, 10.0);
        let dir = Vec3::NEG_Z;

        let pivot = zoom_pivot_on_focus_plane(origin, dir, focus, normal);
        assert!(pivot.abs_diff_eq(Vec3::new(5.0, 0.0, 0.0), 1e-5));
    }

    #[test]
    fn parallel_ray_falls_back_to_focus() {
        let focus = Vec3::new(1.0, 2.0, 3.0);
        let pivot = zoom_pivot_on_focus_plane(
            Vec3::new(0.0, 0.0, 10.0),
            Vec3::X,
            focus,
            Vec3::NEG_Z,
        );
        assert_eq!(pivot, focus);
    }

    #[test]
    fn ray_behind_camera_falls_back_to_focus() {
        let focus = Vec3::new(0.0, 0.0, 0.0);
        let pivot = zoom_pivot_on_focus_plane(
            Vec3::new(0.0, 0.0, 10.0),
            Vec3::Z,
            focus,
            Vec3::NEG_Z,
        );
        assert_eq!(pivot, focus);
    }
}
