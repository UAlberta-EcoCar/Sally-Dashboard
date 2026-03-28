use core::sync::atomic::AtomicU32;

use super::init_charging::*;
use crate::can_mod::REL_FC_PACK;
use crate::display_mod::{CENTER_POINT, DisplayDevice};
use eg_seven_segment::SevenSegmentStyleBuilder;
use embedded_graphics::primitives::StyledDrawable;
use embedded_graphics::text::renderer::CharacterStyle;
use embedded_graphics::{
    Drawable,
    pixelcolor::Rgb666,
    prelude::*,
    primitives::{Arc, PrimitiveStyle},
    text::{Alignment, Text},
};

static PREV_BATT_VOLTAGE: AtomicU32 = AtomicU32::new(0);

fn render_battery_voltage_gui(
    display: &mut DisplayDevice,
    batt_voltage: u32,
    prev_batt_voltage: u32,
) {
    const DIGIT_SPACING: u32 = 3;
    const VOLTAGE_POS: Point = Point::new(
        CENTER_POINT.x + BATT_FONT_WIDTH as i32,
        CENTER_POINT.y + BATT_FONT_HEIGHT as i32 / 2,
    );
    const CLEAR_TEXT_POS: Point = Point::new(
        CENTER_POINT.x - DIGIT_SPACING as i32,
        CENTER_POINT.y + BATT_FONT_HEIGHT as i32 / 2,
    );

    // Define Styles
    let batt_style = SevenSegmentStyleBuilder::new()
        .digit_size(Size::new(BATT_FONT_WIDTH, BATT_FONT_HEIGHT))
        .digit_spacing(DIGIT_SPACING)
        .segment_width(4)
        .segment_color(Rgb666::WHITE)
        .inactive_segment_color(Rgb666::BLACK)
        .build();
    let mut clear_style = batt_style.clone();
    clear_style.set_text_color(Some(Rgb666::BLACK));

    let mut str_buffer = itoa::Buffer::new();
    let batt_voltage_str = str_buffer.format(batt_voltage);

    // Clear Dead Text
    if prev_batt_voltage >= 10 && batt_voltage < 10 {
        Text::with_alignment("8", CLEAR_TEXT_POS, clear_style, Alignment::Right)
            .draw(display)
            .unwrap();
    }
    // Render Battery Voltage
    Text::with_alignment(batt_voltage_str, VOLTAGE_POS, batt_style, Alignment::Right)
        .draw(display)
        .unwrap();
}

fn render_battery_meter_gui(display: &mut DisplayDevice, battery_percent: f32) {
    let empty_style = PrimitiveStyle::with_stroke(Rgb666::BLACK, 12);
    let fill_style = PrimitiveStyle::with_stroke(Rgb666::GREEN, 12);

    const ANGLE_END: f32 = ANGLE_START + (360.0 - (ANGLE_START - 90.0) * 2.0);
    const MAX_METER_LENGTH: f32 = 360.0 - (ANGLE_START - 90.0) * 2.0;

    let charge_length = MAX_METER_LENGTH * battery_percent;
    let empty_length = ANGLE_END - (charge_length + ANGLE_START);

    Arc::with_center(
        CENTER_POINT,
        ARC_DIAMTER,
        ANGLE_START.deg() + charge_length.deg(),
        empty_length.deg(),
    )
    .draw_styled(&empty_style, display)
    .unwrap();

    Arc::with_center(
        CENTER_POINT,
        ARC_DIAMTER,
        ANGLE_START.deg(),
        charge_length.deg(),
    )
    .draw_styled(&fill_style, display)
    .unwrap();
}

pub async fn render_charging_gui(display: &mut DisplayDevice) {
    let prev_batt_voltage = PREV_BATT_VOLTAGE.load(core::sync::atomic::Ordering::Relaxed);
    let relay_fc_pack = REL_FC_PACK.lock().await;
    let batt_voltage = relay_fc_pack.fc_volt;
    drop(relay_fc_pack);
    let batt_voltage_percent = batt_voltage as f32 / 48.0;

    render_battery_voltage_gui(display, batt_voltage, prev_batt_voltage);
    render_battery_meter_gui(display, batt_voltage_percent);

    PREV_BATT_VOLTAGE.store(batt_voltage, core::sync::atomic::Ordering::Relaxed);
}
