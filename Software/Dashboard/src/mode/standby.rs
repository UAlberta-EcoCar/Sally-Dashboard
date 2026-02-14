use crate::display_mod::{CENTER_POINT, DisplayDevice};
use eg_seven_segment::SevenSegmentStyleBuilder;
use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, mutex::Mutex};
use embedded_graphics::mono_font::iso_8859_1::FONT_9X15;
use embedded_graphics::mono_font::{MonoFont, MonoTextStyle};
use embedded_graphics::text::renderer::CharacterStyle;
use embedded_graphics::{
    Drawable,
    pixelcolor::Rgb666,
    prelude::*,
    text::{Alignment, Text},
};
const MAX_ROWS_PER_COLUMN: i32 = 16;

pub static CURRENT_ROW: Mutex<ThreadModeRawMutex, i32> = Mutex::new(0);

async fn render_can_value(
    field: &str,
    value: u32,
    render_field_name: bool,
    display: &mut DisplayDevice,
) {
    let mut str_buffer = itoa::Buffer::new();
    let value = str_buffer.format(value);

    const CAN_FONT: MonoFont<'static> = FONT_9X15;
    const FONT_WIDTH: u32 = CAN_FONT.character_size.width;
    const FONT_HEIGHT: u32 = CAN_FONT.character_size.height;

    let number_style = SevenSegmentStyleBuilder::new()
        .digit_size(Size::new(FONT_WIDTH, FONT_HEIGHT))
        .digit_spacing(2)
        .segment_width(1)
        .segment_color(Rgb666::WHITE)
        .inactive_segment_color(Rgb666::BLACK)
        .build();
    let mut clear_text_style = number_style.clone();
    clear_text_style.set_text_color(Some(Rgb666::BLACK));

    let mut row = CURRENT_ROW.lock().await;
    let col = if *row >= MAX_ROWS_PER_COLUMN { 1 } else { 0 };

    let text_pos = Point::new(
        FONT_WIDTH as i32 * 12 + col * CENTER_POINT.x,
        20 + (FONT_HEIGHT as i32 + 4) * (*row - col * MAX_ROWS_PER_COLUMN),
    );
    let number_pos = text_pos + Point::new(13 * FONT_WIDTH as i32, 0);

    // Clear previous value
    let clear_number =
        Text::with_alignment("8888888888", number_pos, clear_text_style, Alignment::Right);
    clear_number.draw(display).unwrap();
    // Render Field Value
    let number = Text::with_alignment(value, number_pos, number_style, Alignment::Right);
    number.draw(display).unwrap();

    // Render Field Name
    if render_field_name {
        let text_style = MonoTextStyle::new(&CAN_FONT, Rgb666::WHITE);

        // render field name
        let text = Text::with_alignment(field, text_pos, text_style, Alignment::Right);
        text.draw(display).unwrap();

        // render colon
        let text = Text::with_alignment(":", text_pos, text_style, Alignment::Left);
        text.draw(display).unwrap();
    }
    // Increment Row number by one
    *row += 1;
}

/// Renders the display in Standby Mode
///
/// `render_field_name` - If true then render the field name of each canbus value
pub async fn render_standby_gui(display: &mut DisplayDevice, render_field_name: bool) {
    let mock_value = 0;

    // RELAY_STATE
    render_can_value("relay_state", mock_value, render_field_name, display).await;

    // FET_DATA
    render_can_value("fet_config", mock_value, render_field_name, display).await;
    render_can_value("input_volt", mock_value, render_field_name, display).await;
    render_can_value("cap_volt", mock_value, render_field_name, display).await;
    render_can_value("cap_curr", mock_value, render_field_name, display).await;
    render_can_value("res_curr", mock_value, render_field_name, display).await;
    render_can_value("out_curr", mock_value, render_field_name, display).await;

    // FCC_PACK1_DATA
    render_can_value("fc_press", mock_value, render_field_name, display).await;
    render_can_value("fc_temp", mock_value, render_field_name, display).await;

    // FCC_PACK2_DATA
    render_can_value("fan_rpm1", mock_value, render_field_name, display).await;
    render_can_value("fan_rpm2", mock_value, render_field_name, display).await;

    // // FCC_PACK3_DATA
    // render_can_value("bme_temp", mock_value, render_field_name,display).await;
    // render_can_value("bme_humid", mock_value, render_field_name,display).await;

    // H2_PACK1_DATA
    render_can_value("h2_sense_1", mock_value, render_field_name, display).await;
    render_can_value("h2_sense_2", mock_value, render_field_name, display).await;
    render_can_value("h2_sense_3", mock_value, render_field_name, display).await;
    render_can_value("h2_sense_4", mock_value, render_field_name, display).await;

    // H2_PACK2_DATA
    render_can_value("bme_temp", mock_value, render_field_name, display).await;
    render_can_value("bme_humid", mock_value, render_field_name, display).await;
    render_can_value("imon_7v", mock_value, render_field_name, display).await;
    render_can_value("imon_12v", mock_value, render_field_name, display).await;

    // BOOST_PACK1_DATA
    render_can_value("in_curr", mock_value, render_field_name, display).await;
    render_can_value("in_volt", mock_value, render_field_name, display).await;

    // BOOST_PACK2_DATA
    render_can_value("out_curr", mock_value, render_field_name, display).await;
    render_can_value("out_volt", mock_value, render_field_name, display).await;

    // BOOST_PACK3_DATA
    render_can_value("efficiency", mock_value, render_field_name, display).await;
    render_can_value("joules", mock_value, render_field_name, display).await;

    // REL_FC_PACK
    render_can_value("fc_volt", mock_value, render_field_name, display).await;
    render_can_value("fc_curr", mock_value, render_field_name, display).await;

    // REL_CAP_PACK
    render_can_value("cap_volt", mock_value, render_field_name, display).await;
    render_can_value("cap_curr", mock_value, render_field_name, display).await;

    // REL_MOTOR_PACK
    render_can_value("mtr_volt", mock_value, render_field_name, display).await;
    render_can_value("mtr_curr", mock_value, render_field_name, display).await;

    // Reset Row number after each frame
    let mut row = CURRENT_ROW.lock().await;
    *row = 0;
}
