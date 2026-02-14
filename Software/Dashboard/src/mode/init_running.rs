use embedded_graphics::prelude::WebColors;
use embedded_graphics::primitives::PrimitiveStyle;
use embedded_graphics::primitives::{
    Circle, PrimitiveStyleBuilder, Rectangle, StrokeAlignment, StyledDrawable,
};

use embedded_graphics::mono_font::iso_8859_13::FONT_10X20;
use embedded_graphics::{
    Drawable,
    pixelcolor::Rgb666,
    prelude::{Point, RgbColor, Size},
    text::{Alignment, Text},
};

use crate::display_mod::{CENTER_POINT, DISPLAY_HEIGHT, DISPLAY_WIDTH, DisplayDevice};
use embedded_graphics::mono_font::MonoTextStyle;

pub const SPEED_FONT_WIDTH: u32 = 27;
pub const SPEED_FONT_HEIGHT: u32 = 63;

pub const EFF_FONT_WIDTH: u32 = 15;
pub const EFF_FONT_HEIGHT: u32 = 25;
pub const EFF_POS: Point = Point::new(50, DISPLAY_HEIGHT as i32 - 50);

pub const BATT_WIDTH: u32 = 16;
pub const BATT_HEIGHT: u32 = 40;
pub const BATT_POS: Point = Point::new(DISPLAY_WIDTH as i32 - 40, DISPLAY_HEIGHT as i32 - 60);

fn init_render_speed_gui(display: &mut DisplayDevice) {
    let speed_unit_style = MonoTextStyle::new(&FONT_10X20, Rgb666::RED);
    let speed_circle_style = PrimitiveStyleBuilder::new()
        .stroke_color(Rgb666::CSS_FIRE_BRICK)
        .stroke_width(5)
        .stroke_alignment(StrokeAlignment::Outside)
        .build();

    // Render Speed Circle
    Circle::with_center(CENTER_POINT, 120)
        .draw_styled(&speed_circle_style, display)
        .unwrap();
    // Render Speed Unit
    Text::with_alignment(
        "km/h",
        CENTER_POINT + Point::new(0, SPEED_FONT_HEIGHT as i32 / 2 + 15),
        speed_unit_style,
        Alignment::Center,
    )
    .draw(display)
    .unwrap();
}

fn init_render_efficiency_gui(display: &mut DisplayDevice) {
    let eff_unit_style = MonoTextStyle::new(&FONT_10X20, Rgb666::GREEN);
    let eff_circle_style = PrimitiveStyleBuilder::new()
        .stroke_color(Rgb666::GREEN)
        .stroke_width(4)
        .stroke_alignment(StrokeAlignment::Outside)
        .build();
    // Render Efficiency Circle
    Circle::with_center(EFF_POS, 70)
        .draw_styled(&eff_circle_style, display)
        .unwrap();
    // Render Efficiency %
    Text::with_alignment(
        "%",
        EFF_POS + Point::new(EFF_FONT_WIDTH as i32 + 2, EFF_FONT_HEIGHT as i32 / 2),
        eff_unit_style,
        Alignment::Left,
    )
    .draw(display)
    .unwrap();
}

fn init_render_battery_gui(display: &mut DisplayDevice) {
    let bat_tip_width = 12;
    let bat_tip_height = 8;

    let bat_tip = Rectangle::new(
        BATT_POS + Point::new((BATT_WIDTH as i32 - bat_tip_width) / 2, -bat_tip_height),
        Size::new(bat_tip_width as u32, bat_tip_height as u32),
    );
    let batt_outline = Rectangle::new(BATT_POS, Size::new(BATT_WIDTH, BATT_HEIGHT));

    let outline_style = PrimitiveStyleBuilder::new()
        .stroke_alignment(StrokeAlignment::Outside)
        .stroke_color(Rgb666::WHITE)
        .stroke_width(4)
        .build();
    let tip_style = PrimitiveStyle::with_fill(Rgb666::WHITE);
    let batt_unit_style = MonoTextStyle::new(&FONT_10X20, Rgb666::WHITE);

    // Render Battery Tip
    bat_tip.draw_styled(&tip_style, display).unwrap();
    // Render Battery Border
    batt_outline.draw_styled(&outline_style, display).unwrap();
    // Render Battey %
    Text::with_alignment(
        "%",
        BATT_POS + Point::new(-8, 40),
        batt_unit_style,
        Alignment::Right,
    )
    .draw(display)
    .unwrap();
}
pub fn init_render_running_gui(display: &mut DisplayDevice) {
    init_render_speed_gui(display);
    init_render_efficiency_gui(display);
    init_render_battery_gui(display);
}
