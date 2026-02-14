use crate::display_mod::{CENTER_POINT, DisplayDevice};
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::mono_font::iso_8859_13::FONT_10X20;
use embedded_graphics::prelude::WebColors;
use embedded_graphics::primitives::StyledDrawable;
use embedded_graphics::{
    Drawable,
    pixelcolor::Rgb666,
    prelude::*,
    primitives::{Arc, PrimitiveStyle},
    text::{Alignment, Text},
};

pub const ANGLE_START: f32 = 130f32;
pub const ARC_DIAMTER: u32 = 160;
pub const BORDER_WIDTH: u32 = 2;
pub const BATT_FONT_WIDTH: u32 = 20;
pub const BATT_FONT_HEIGHT: u32 = 35;

pub fn init_render_charging_gui(display: &mut DisplayDevice) {
    // Render loading bar border
    let border_style = PrimitiveStyle::with_stroke(Rgb666::CSS_DARK_GRAY, 12 + BORDER_WIDTH * 2);
    Arc::with_center(
        CENTER_POINT,
        ARC_DIAMTER,
        (ANGLE_START - BORDER_WIDTH as f32).deg(),
        (360.0 - (ANGLE_START - 90.0 - BORDER_WIDTH as f32) * 2.0).deg(),
    )
    .draw_styled(&border_style, display)
    .unwrap();

    // Render Speed Unit
    let batt_unit_style = MonoTextStyle::new(&FONT_10X20, Rgb666::WHITE);

    Text::with_alignment(
        "V",
        CENTER_POINT
            + Point::new(
                BATT_FONT_WIDTH as i32 + FONT_10X20.character_size.width as i32 + 5,
                BATT_FONT_HEIGHT as i32 / 2,
            ),
        batt_unit_style,
        Alignment::Right,
    )
    .draw(display)
    .unwrap();
}
