use crate::can_mod::{
    BOOST_PACK1_DATA, BOOST_PACK2_DATA, BOOST_PACK3_DATA, FCC_PACK1_DATA, FCC_PACK2_DATA, FET_DATA,
    H2_PACK1_DATA, H2_PACK2_DATA, REL_CAP_PACK, REL_FC_PACK, RELAY_MOTOR_PACK, RELAY_STATE,
};
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
    // RELAY_STATE
    let relay_state = RELAY_STATE.lock().await;
    let relay_state_val = (*relay_state).clone() as u32;
    render_can_value("relay_state", relay_state_val, render_field_name, display).await;
    drop(relay_state);

    // FET_DATA
    let fet_data = FET_DATA.lock().await;
    render_can_value(
        "fet_config",
        fet_data.fet_config,
        render_field_name,
        display,
    )
    .await;
    render_can_value(
        "input_volt",
        fet_data.input_volt,
        render_field_name,
        display,
    )
    .await;
    render_can_value("cap_volt", fet_data.cap_volt, render_field_name, display).await;
    render_can_value("cap_curr", fet_data.cap_curr, render_field_name, display).await;
    render_can_value("res_curr", fet_data.res_curr, render_field_name, display).await;
    render_can_value("out_curr", fet_data.out_curr, render_field_name, display).await;
    drop(fet_data);

    // FCC_PACK1_DATA
    let fcc_pack1_data = FCC_PACK1_DATA.lock().await;
    render_can_value(
        "fc_press",
        fcc_pack1_data.fc_press,
        render_field_name,
        display,
    )
    .await;
    render_can_value(
        "fc_temp",
        fcc_pack1_data.fc_temp as u32,
        render_field_name,
        display,
    )
    .await;
    drop(fcc_pack1_data);

    // FCC_PACK2_DATA
    let fcc_pack2 = FCC_PACK2_DATA.lock().await;
    render_can_value("fan_rpm1", fcc_pack2.fan_rpm1, render_field_name, display).await;
    render_can_value("fan_rpm2", fcc_pack2.fan_rpm2, render_field_name, display).await;
    drop(fcc_pack2);

    // FCC_PACK3_DATA
    // Values are already displayed from other packets
    // let fcc_pack3 = FCC_PACK3_DATA.lock().await;
    // render_can_value("bme_temp", fcc_pack3.bme_temp, render_field_name, display).await;
    // render_can_value("bme_humid", fcc_pack3.bme_humid, render_field_name, display).await;
    // drop(fcc_pack3);

    // H2_PACK1_DATA
    let h2_pack1 = H2_PACK1_DATA.lock().await;
    render_can_value(
        "h2_sense_1",
        h2_pack1.h2_sense_1 as u32,
        render_field_name,
        display,
    )
    .await;
    render_can_value(
        "h2_sense_2",
        h2_pack1.h2_sense_2 as u32,
        render_field_name,
        display,
    )
    .await;
    render_can_value(
        "h2_sense_3",
        h2_pack1.h2_sense_3 as u32,
        render_field_name,
        display,
    )
    .await;
    render_can_value(
        "h2_sense_4",
        h2_pack1.h2_sense_4 as u32,
        render_field_name,
        display,
    )
    .await;
    drop(h2_pack1);

    // H2_PACK2_DATA
    let h2_pack2 = H2_PACK2_DATA.lock().await;
    render_can_value(
        "bme_temp",
        h2_pack2.bme_temp as u32,
        render_field_name,
        display,
    )
    .await;
    render_can_value(
        "bme_humid",
        h2_pack2.bme_humid as u32,
        render_field_name,
        display,
    )
    .await;
    render_can_value(
        "imon_7v",
        h2_pack2.imon_7v as u32,
        render_field_name,
        display,
    )
    .await;
    render_can_value(
        "imon_12v",
        h2_pack2.imon_12v as u32,
        render_field_name,
        display,
    )
    .await;
    drop(h2_pack2);

    // BOOST_PACK1_DATA
    let boost1 = BOOST_PACK1_DATA.lock().await;
    render_can_value("in_curr", boost1.in_curr, render_field_name, display).await;
    render_can_value("in_volt", boost1.in_volt, render_field_name, display).await;
    drop(boost1);

    // BOOST_PACK2_DATA
    let boost2 = BOOST_PACK2_DATA.lock().await;
    render_can_value("out_curr", boost2.out_curr, render_field_name, display).await;
    render_can_value("out_volt", boost2.out_volt, render_field_name, display).await;
    drop(boost2);

    // BOOST_PACK3_DATA
    let boost3 = BOOST_PACK3_DATA.lock().await;
    render_can_value("efficiency", boost3.efficiency, render_field_name, display).await;
    render_can_value("joules", boost3.joules, render_field_name, display).await;
    drop(boost3);

    // REL_FC_PACK
    let rel_fc = REL_FC_PACK.lock().await;
    render_can_value("fc_volt", rel_fc.fc_volt, render_field_name, display).await;
    render_can_value("fc_curr", rel_fc.fc_curr, render_field_name, display).await;
    drop(rel_fc);

    // REL_CAP_PACK
    let rel_cap = REL_CAP_PACK.lock().await;
    render_can_value("cap_volt", rel_cap.cap_volt, render_field_name, display).await;
    render_can_value(
        "cap_curr",
        rel_cap.cap_curr as u32,
        render_field_name,
        display,
    )
    .await;
    drop(rel_cap);

    // REL_MOTOR_PACK
    let rel_mtr = RELAY_MOTOR_PACK.lock().await;
    render_can_value("mtr_volt", rel_mtr.mtr_volt, render_field_name, display).await;
    render_can_value("mtr_curr", rel_mtr.mtr_curr, render_field_name, display).await;
    drop(rel_mtr);

    // Reset Row number after each frame
    let mut row = CURRENT_ROW.lock().await;
    *row = 0;
}
