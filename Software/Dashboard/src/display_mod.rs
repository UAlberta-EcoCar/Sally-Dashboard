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

use defmt::{info, trace};
use display_interface_spi::SPIInterface;
use embassy_stm32::gpio::Output;
use embassy_stm32::spi::Spi;
use embassy_time::{Instant, Timer};
use embedded_graphics::{
    pixelcolor::Rgb666,
    prelude::{Point, RgbColor},
};
use embedded_hal_bus::spi::ExclusiveDevice;
use ili9488_rs::{Ili9488, Rgb666Mode};

use crate::eco_can::RelayState;
use crate::{
    can_mod::RELAY_STATE,
    mode::{
        charging::render_charging_gui, init_charging::init_render_charging_gui,
        init_running::init_render_running_gui, running::render_running_gui,
        standby::render_standby_gui, startup::render_startup_gui,
    },
};

/// Type Alias for ILI9488 driver, the current display driver
pub type DisplayDevice = Ili9488<
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

pub const DISPLAY_WIDTH: u32 = 480;
pub const DISPLAY_HEIGHT: u32 = 320;
pub const CENTER_POINT: Point = Point::new(DISPLAY_WIDTH as i32 / 2, DISPLAY_HEIGHT as i32 / 2);

/// Responsible for rendering data to the display
#[embassy_executor::task]
pub async fn display_task(mut display: DisplayDevice) {
    info!("Time taken to do a full screen clear:");
    let start = Instant::now().as_millis();
    display.clear_screen(Rgb666::WHITE).unwrap();
    let end = Instant::now().as_millis();
    info!("Full Screen Clear: {} ms", end - start);

    let mut prev_relay_state: Option<RelayState> = None;

    loop {
        let relay_state_lock = RELAY_STATE.lock().await;
        let relay_state = relay_state_lock.clone();
        drop(relay_state_lock);

        // Inialized display screen if switching relay state
        if prev_relay_state.is_none() || *prev_relay_state.as_ref().unwrap() != relay_state {
            match relay_state {
                RelayState::RELAY_STRTP => render_startup_gui(&mut display),
                RelayState::RELAY_CHRGE => init_render_charging_gui(&mut display),
                RelayState::RELAY_STBY => render_standby_gui(&mut display, true).await,
                RelayState::RELAY_RUN => init_render_running_gui(&mut display),
            }
        }

        // Update previous relay state
        prev_relay_state = Some(relay_state.clone());

        // Update display with current relay state
        match relay_state {
            RelayState::RELAY_STRTP => (),
            RelayState::RELAY_CHRGE => render_charging_gui(&mut display),
            RelayState::RELAY_STBY => render_standby_gui(&mut display, false).await,
            RelayState::RELAY_RUN => render_running_gui(&mut display),
        }

        trace!("Display Health check");
        Timer::after_millis(2000).await;
    }
}
