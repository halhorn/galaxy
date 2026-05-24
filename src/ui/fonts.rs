//! Custom egui fonts for equation display.

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

pub const EQUATION_FONT: &str = "equation";
const EQUATION_FONT_DATA: &str = "stix_two_text";

/// Registers a serif font suited to mathematical notation.
pub fn setup_equation_font(mut contexts: EguiContexts) -> Result {
    let ctx = contexts.ctx_mut()?;
    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert(
        EQUATION_FONT_DATA.to_owned(),
        std::sync::Arc::new(egui::FontData::from_static(include_bytes!(
            "../../assets/fonts/STIXTwoText-Regular.ttf"
        ))),
    );
    fonts.families.insert(
        egui::FontFamily::Name(EQUATION_FONT.into()),
        vec![EQUATION_FONT_DATA.to_owned()],
    );
    ctx.set_fonts(fonts);
    Ok(())
}
