use bevy::prelude::*;
use bevy::camera::Viewport;

/// Viewport width below which the control panel moves to the bottom (phone / narrow window).
pub const MOBILE_BREAKPOINT_PX: f32 = 768.0;
pub const MOBILE_PANEL_HEIGHT: f32 = 280.0;
pub const DESKTOP_PANEL_WIDTH: f32 = 260.0;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum SimViewportSystems {
    Layout,
    Apply,
}

/// Logical screen rect (top-left origin) reserved for the 3D simulation view.
#[derive(Resource, Clone, Copy, Debug)]
pub struct SimulationViewportRect {
    pub logical: Rect,
}

impl Default for SimulationViewportRect {
    fn default() -> Self {
        Self {
            logical: Rect::from_corners(Vec2::ZERO, Vec2::ONE),
        }
    }
}

pub fn fallback_logical_rect(window: &Window) -> Rect {
    if window.width() < MOBILE_BREAKPOINT_PX {
        Rect {
            min: Vec2::ZERO,
            max: Vec2::new(window.width(), (window.height() - MOBILE_PANEL_HEIGHT).max(1.0)),
        }
    } else {
        Rect {
            min: Vec2::new(DESKTOP_PANEL_WIDTH, 0.0),
            max: Vec2::new(window.width(), window.height()),
        }
    }
}

pub fn logical_rect_to_camera_viewport(rect: Rect, window: &Window) -> Viewport {
    let scale = window.resolution.scale_factor();

    Viewport {
        physical_position: UVec2::new(
            (rect.min.x * scale).round() as u32,
            (rect.min.y * scale).round() as u32,
        ),
        physical_size: UVec2::new(
            (rect.width().max(1.0) * scale).round() as u32,
            (rect.height().max(1.0) * scale).round() as u32,
        ),
        depth: 0.0..1.0,
    }
}
