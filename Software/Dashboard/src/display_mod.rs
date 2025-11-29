//! Module for the Screen
//!
//! The [embedded_graphics](https://docs.rs/embedded-graphics/latest/embedded_graphics/) library
//! is used to render 2D graphics to the screen. Examples for how to use the library can be
//! found [here](https://docs.rs/embedded-graphics/latest/embedded_graphics/#shapes-and-text).
//!
//! The STM32G491KE has 512 Kbytes of Flash memory, and 112 Kbytes of SRAM. Because of the
//! low memory constraints, a framebuffer cannot be used.
//!
//! # How rendering works
//! The ILIxxxx IC drivers operate using commands and data. The command list can be found
//!  [here](https://www.displayfuture.com/Display/datasheet/controller/ILI9488.pdf#pages=140)
//!
//! What happens is the following:
//!
//! - A drawing window is prepared (with the 2 opposite corner coordinates), using three commands.
//!     - The [column address set](https://www.displayfuture.com/Display/datasheet/controller/ILI9488.pdf#pages=175)
//!     command.
//!     - The [page address set](https://www.displayfuture.com/Display/datasheet/controller/ILI9488.pdf#pages=177)
//!     command.
//!     - The [memory write](https://www.displayfuture.com/Display/datasheet/controller/ILI9488.pdf#pages=179)
//!     command begins the transmission of pixel data to the area defined by the column/page address set commands.
//! - The starting point for drawint is the top left corner of this window
//! - Every set of bytes received is intepreted as a pixel value in the current display format (Rgb666, Rgb565, etc.).
//! How pixels are formatted into bytes depends on the display format and interface type. More information can be
//! found in the [Display Data Format](https://www.displayfuture.com/Display/datasheet/controller/ILI9488.pdf#pages=119)
//! section of the ILI9488 datasheet.
//! - As soon as a pixel is received, an internal counter is incremented,
//!   and the next word will fill the next pixel (the adjacent on the right, or
//!   the first of the next row if the row ended)
//!
//! # Optimization Strategies
//! 1. The hardware is optimized for drawing rectangles. So prefer rendering rectangles over other shapes.
//! 1. If a text/gui element's state does not change between render frames, do not redraw it.
//! 1. Numbers that are rendered on each frame (e.g speed, temperature) should use the seven-segment display font.
//!  The reason for this is because the seven-segment font is rendered using multiple horizontal/veritcal lines
//! (rectangles), [source](https://github.com/embedded-graphics/eg-seven-segment/blob/master/src/segment.rs#L39).

use defmt::info;
use display_interface_spi::SPIInterface;
use embassy_stm32::gpio::Output;
use embassy_stm32::spi::Spi;
use embassy_time::{Instant, Timer};
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

/// Type Alias for ILI9488 driver
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
pub async fn display_task(mut display: Ili9488Display, mut lcd_bright: Output<'static>) {
    let text_style = MonoTextStyle::new(&FONT_10X20, Rgb666::BLACK);
    lcd_bright.set_high();

    info!("Time taken to do a full screen clear:");
    let start = Instant::now().as_millis();
    display.clear_screen(Rgb666::WHITE).unwrap();
    let end = Instant::now().as_millis();
    info!("(rgb 6-6-6) fast version: {} ms", end - start);

    loop {
        // display.clear_screen(Rgb666::WHITE).unwrap();
        Text::with_alignment(
            "ILI9488 Inilialized...",
            display.bounding_box().center() + Point::new(20, 20),
            text_style,
            Alignment::Center,
        )
        .draw(&mut display)
        .unwrap();
        // info!("Display Health check");
        Timer::after_millis(1000).await;
    }
}
