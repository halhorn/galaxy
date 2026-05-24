//! Shared icon buttons for the control panel (playback, help, etc.).

use bevy_egui::egui::{Button, RichText, Response, Ui, Vec2};

pub const HEADER_ICON_BUTTON_SIZE: Vec2 = Vec2::new(28.0, 28.0);

const ICON_FONT_RATIO: f32 = 16.0 / 28.0;

pub fn icon_button(ui: &mut Ui, icon: &str, tooltip: &str, min_size: Vec2) -> Response {
    let font_size = min_size.y * ICON_FONT_RATIO;
    ui.add(
        Button::new(RichText::new(icon).size(font_size)).min_size(min_size),
    )
    .on_hover_text(tooltip)
}
