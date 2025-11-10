//! Module for the Screen

use display_interface_spi::SPIInterface;
use embassy_stm32::gpio::Output;
use embassy_stm32::spi::Spi;
use embedded_graphics::{
    Drawable,
    geometry::Dimensions,
    mono_font::{MonoTextStyle, iso_8859_14::FONT_10X20},
    pixelcolor::Rgb666,
    prelude::{Point, RgbColor},
    text::{Alignment, Text},
};
use embedded_hal_bus::spi::ExclusiveDevice;
use ili9488_rs::{Ili9488, Rgb666Mode};

/// Typedef for ILI9488 driver
type Ili9488Display = Ili9488<
    SPIInterface<
        ExclusiveDevice<
            Spi<'static, embassy_stm32::mode::Async>,
            Output<'static>,
            embedded_hal_bus::spi::NoDelay,
        >,
        Output<'static>,
    >,
    Output<'static>,
    Rgb666Mode,
>;

/// Responsible for rendering data to the display
#[embassy_executor::task]
pub async fn display_task(mut display: Ili9488Display) {
    let text_style = MonoTextStyle::new(&FONT_10X20, Rgb666::BLACK);

    display
        .clear_screen_fast(ili9488_rs::Rgb111::WHITE)
        .unwrap();

    Text::with_alignment(
        "ILI9488 Inilialized...",
        display.bounding_box().center() + Point::new(20, 20),
        text_style,
        Alignment::Center,
    )
    .draw(&mut display)
    .unwrap();
}
