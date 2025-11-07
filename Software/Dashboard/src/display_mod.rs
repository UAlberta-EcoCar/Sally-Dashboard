//! Module for the Screen

use display_interface_spi::SPIInterface;
use embassy_stm32::gpio::Output;
use embassy_stm32::spi::Spi;
use embedded_hal_bus::spi::ExclusiveDevice;
use ili9488_rs::{Ili9488, Rgb666Mode};

pub type Ili9488Display = Ili9488<
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
