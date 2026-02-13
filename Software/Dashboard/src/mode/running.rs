use eg_seven_segment::SevenSegmentStyleBuilder;
use embedded_graphics::mono_font::iso_8859_13::FONT_10X20;
use embedded_graphics::prelude::Transform;
use embedded_graphics::prelude::WebColors;
use embedded_graphics::primitives::PrimitiveStyle;
use embedded_graphics::primitives::{
    Circle, PrimitiveStyleBuilder, Rectangle, StrokeAlignment, StyledDrawable,
};

use embedded_graphics::text::renderer::CharacterStyle;
use embedded_graphics::{
    Drawable,
    geometry::AnchorX,
    mono_font::MonoTextStyle,
    pixelcolor::Rgb666,
    prelude::{Point, RgbColor, Size},
    text::{Alignment, Text},
};

use crate::display_mod::{CENTER_POINT, DISPLAY_HEIGHT, DISPLAY_WIDTH, DisplayDevice};

const SPEED_FONT_WIDTH: u32 = 27;
const SPEED_FONT_HEIGHT: u32 = 63;

const EFF_FONT_WIDTH: u32 = 15;
const EFF_FONT_HEIGHT: u32 = 25;
const EFF_POS: Point = Point::new(50, DISPLAY_HEIGHT as i32 - 50);

const BATT_WIDTH: u32 = 16;
const BATT_HEIGHT: u32 = 40;
const BATT_POS: Point = Point::new(DISPLAY_WIDTH as i32 - 40, DISPLAY_HEIGHT as i32 - 60);

fn greater_than_10(val: u32) -> bool {
    val >= 10
}

fn render_speed_widgets(display: &mut DisplayDevice, speed: u32, prev_speed: u32) {
    const DIGIT_SPACING: u32 = 4;
    let speed_style = SevenSegmentStyleBuilder::new()
        .digit_size(Size::new(SPEED_FONT_WIDTH, SPEED_FONT_HEIGHT))
        .digit_spacing(DIGIT_SPACING)
        .segment_width(6)
        .segment_color(Rgb666::RED)
        .inactive_segment_color(Rgb666::BLACK)
        .build();
    let mut clear_style = speed_style.clone();
    clear_style.set_text_color(Some(Rgb666::BLACK));

    const SPEED_POS: Point = Point::new(
        CENTER_POINT.x + SPEED_FONT_WIDTH as i32,
        CENTER_POINT.y + SPEED_FONT_HEIGHT as i32 / 2,
    );
    const CLEAR_TEXT_POS: Point = Point::new(
        CENTER_POINT.x - (DIGIT_SPACING as i32),
        CENTER_POINT.y + SPEED_FONT_HEIGHT as i32 / 2,
    );

    let mut str_buffer = itoa::Buffer::new();
    let speed_str = str_buffer.format(speed);

    // Clear dead digits
    if greater_than_10(prev_speed) && !greater_than_10(speed) {
        Text::with_alignment("8", CLEAR_TEXT_POS, clear_style, Alignment::Right)
            .draw(display)
            .unwrap();
    }
    // Render Speed
    Text::with_alignment(speed_str, SPEED_POS, speed_style, Alignment::Right)
        .draw(display)
        .unwrap();
}

fn render_tach_widgets(display: &mut DisplayDevice, rpm: u32, _prev_rpm: u32) {
    // Define Styles
    let tach_line_width = 3;

    // The number of tach lines per 1000rpm
    let tach_lines = 5;
    // Maximum RPM Represented is 5000rpm
    let max_tach_lines = tach_lines * 5;

    let tach_empty_style = PrimitiveStyle::with_fill(Rgb666::CSS_SILVER);

    let tach_line_style = PrimitiveStyle::with_fill(Rgb666::RED);
    let tach_line = Rectangle::new(
        CENTER_POINT.x_axis() - Point::new(max_tach_lines * tach_line_width * 2, -15),
        Size::new(tach_line_width as u32, 55),
    );

    let tach_divider_style = PrimitiveStyle::with_fill(Rgb666::CSS_DEEP_PINK);
    let tach_divider_line = tach_line.resized_width(tach_line_width as u32 + 2, AnchorX::Left);

    // Render Tachometer
    // Determines the distance between tachometer bars
    let tach_spacer = 4;
    // Maximum RPM Represented is 5000rpm
    let display_rpm = ((rpm as f32 / 5000f32) * max_tach_lines as f32) as i32;
    for i in 0..=display_rpm {
        let (bar, bar_style) = if (i % tach_lines) == 0 {
            (tach_divider_line, tach_divider_style)
        } else {
            (tach_line, tach_line_style)
        };
        bar.translate(Point::new(i * tach_line_width as i32 * tach_spacer, 0))
            .draw_styled(&bar_style, display)
            .unwrap();
    }
    for i in (display_rpm + 1)..=max_tach_lines {
        let tach_line = if (i % tach_lines) == 0 {
            tach_divider_line
        } else {
            tach_line
        };
        tach_line
            .translate(Point::new(i * tach_line_width as i32 * tach_spacer, 0))
            .draw_styled(&tach_empty_style, display)
            .unwrap();
    }
}

fn render_efficiency_gui(display: &mut DisplayDevice, efficiency: u8, prev_efficiency: u8) {
    const DIGIT_SPACING: u32 = 2;
    let eff_style = SevenSegmentStyleBuilder::new()
        .digit_size(Size::new(EFF_FONT_WIDTH, EFF_FONT_HEIGHT))
        .digit_spacing(DIGIT_SPACING)
        .segment_width(3)
        .segment_color(Rgb666::GREEN)
        .inactive_segment_color(Rgb666::BLACK)
        .build();
    let mut clear_style = eff_style.clone();
    clear_style.set_text_color(Some(Rgb666::BLACK));

    let mut str_buffer = itoa::Buffer::new();
    let efficiency_str = str_buffer.format(efficiency);

    const EFF_TEXT_POS: Point = Point::new(
        EFF_POS.x + EFF_FONT_WIDTH as i32,
        EFF_POS.y + EFF_FONT_HEIGHT as i32 / 2,
    );
    const CLEAR_TEXT_POS: Point = Point::new(
        EFF_POS.x - (DIGIT_SPACING as i32),
        EFF_POS.y + EFF_FONT_HEIGHT as i32 / 2,
    );

    // Clear Dead Digits
    if prev_efficiency >= 100 && efficiency < 100 {
        Text::with_alignment("88", CLEAR_TEXT_POS, clear_style, Alignment::Right)
            .draw(display)
            .unwrap();
    } else if prev_efficiency >= 10 && efficiency < 10 {
        Text::with_alignment("8", CLEAR_TEXT_POS, clear_style, Alignment::Right)
            .draw(display)
            .unwrap();
    }
    // Render Efficiency
    Text::with_alignment(efficiency_str, EFF_TEXT_POS, eff_style, Alignment::Right)
        .draw(display)
        .unwrap();
}

fn render_battery_gui(display: &mut DisplayDevice, battery_health: u8, prev_battery_health: u8) {
    let mut str_buffer = itoa::Buffer::new();
    let battery_health_str = str_buffer.format(battery_health);

    let clear_style = PrimitiveStyle::with_fill(Rgb666::BLACK);
    let fill_style = PrimitiveStyle::with_fill(Rgb666::GREEN);

    const BATT_FONT_WIDTH: u32 = 10;
    const BATT_FONT_HEIGHT: u32 = 20;

    const DIGIT_SPACING: u32 = 2;
    let batt_text_style = SevenSegmentStyleBuilder::new()
        .digit_size(Size::new(BATT_FONT_WIDTH, BATT_FONT_HEIGHT))
        .digit_spacing(DIGIT_SPACING)
        .segment_width(2)
        .segment_color(Rgb666::WHITE)
        .inactive_segment_color(Rgb666::BLACK)
        .build();
    let mut clear_text_style = batt_text_style.clone();
    clear_text_style.set_text_color(Some(Rgb666::BLACK));

    const BATT_TEXT_POS: Point = Point::new(
        BATT_POS.x - 1 * (BATT_WIDTH / 2 + BATT_FONT_WIDTH) as i32,
        BATT_POS.y + 40,
    );
    const CLEAR_TEXT_POS: Point = Point::new(
        BATT_POS.x - 1 * (BATT_WIDTH / 2 + BATT_FONT_WIDTH * 2) as i32 - DIGIT_SPACING as i32,
        BATT_POS.y + 40,
    );

    let battery_level = ((100 - battery_health) as f32 / 100f32 * BATT_HEIGHT as f32) as u32;

    let batt_outline = Rectangle::new(BATT_POS, Size::new(BATT_WIDTH, battery_level));
    let batt_fill = Rectangle::with_corners(
        BATT_POS + Point::new(0, battery_level as i32),
        BATT_POS + Size::new(BATT_WIDTH - 1, BATT_HEIGHT),
    );

    // Render Battery Rating
    batt_outline.draw_styled(&clear_style, display).unwrap();
    batt_fill.draw_styled(&fill_style, display).unwrap();

    // Clear Dead Digits
    if prev_battery_health >= 100 && battery_health < 100 {
        Text::with_alignment("88", CLEAR_TEXT_POS, clear_text_style, Alignment::Right)
            .draw(display)
            .unwrap();
    } else if prev_battery_health >= 10 && battery_health < 10 {
        Text::with_alignment("8", CLEAR_TEXT_POS, clear_text_style, Alignment::Right)
            .draw(display)
            .unwrap();
    }
    // Render Battery Percentage
    Text::with_alignment(
        battery_health_str,
        BATT_TEXT_POS,
        batt_text_style,
        Alignment::Right,
    )
    .draw(display)
    .unwrap();
}

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

pub fn running_gui(display: &mut DisplayDevice, frame_index: u32, prev_frame_index: u32) {
    ///////////////////////////////
    // Render Graphics
    ///////////////////////////////
    let prev_frac = prev_frame_index as f32 / 100f32;
    let prev_rpm = prev_frac * 5000f32;
    let prev_speed = prev_frac * 40f32;

    let frac = frame_index as f32 / 100f32;
    let rpm = frac * 5000f32;
    let speed = frac * 40f32;
    render_tach_widgets(display, rpm as u32, prev_rpm as u32);
    render_speed_widgets(display, speed as u32, prev_speed as u32);
    render_efficiency_gui(display, frame_index as u8, prev_frame_index as u8);
    render_battery_gui(display, frame_index as u8, prev_frame_index as u8);
}
