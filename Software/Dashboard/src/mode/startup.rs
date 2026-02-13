use embedded_graphics::{
    pixelcolor::Rgb666,
    prelude::{Point, RgbColor, Size, WebColors},
    primitives::{PrimitiveStyle, Rectangle, StyledDrawable},
};

use crate::display_mod::{DISPLAY_HEIGHT, DISPLAY_WIDTH, DisplayDevice};

fn linear_gradient(
    start_color: Rgb666,
    end_color: Rgb666,
    index: u32,
    gradient_width: u32,
) -> Rgb666 {
    let t = index as f32 / gradient_width as f32;
    let interpolate_color =
        |start: u8, end: u8| (start as f32 + (t * (end as f32 - start as f32))) as u8;

    Rgb666::new(
        interpolate_color(start_color.r(), end_color.r()),
        interpolate_color(start_color.g(), end_color.g()),
        interpolate_color(start_color.b(), end_color.b()),
    )
}

fn render_linear_gradient(
    display: &mut DisplayDevice,
    start_color: Rgb666,
    end_color: Rgb666,
    start_column: usize,
    gradient_width: u32,
) {
    for i in 0..gradient_width {
        let column_color = linear_gradient(start_color, end_color, i, gradient_width);
        let column_rect = Rectangle::new(
            Point::new(i as i32 + start_column as i32, 0),
            Size::new(1, DISPLAY_HEIGHT),
        );
        let column_style = PrimitiveStyle::with_fill(column_color);

        column_rect.draw_styled(&column_style, display).unwrap();
    }
}

pub fn startup_gui(display: &mut DisplayDevice) {
    let colors = [
        Rgb666::RED,
        Rgb666::CSS_ORANGE,
        Rgb666::CSS_YELLOW,
        Rgb666::GREEN,
        Rgb666::CYAN,
        Rgb666::BLUE,
        Rgb666::CSS_INDIGO,
        Rgb666::CSS_PURPLE,
        Rgb666::CSS_VIOLET,
    ];
    let gradient_width = DISPLAY_WIDTH / (colors.len() as u32 - 1);
    for column in 0..(colors.len() - 1) {
        render_linear_gradient(
            display,
            colors[column],
            colors[column + 1],
            column * gradient_width as usize,
            gradient_width,
        );
    }
}
