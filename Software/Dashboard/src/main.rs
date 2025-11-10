#![no_std]
#![no_main]
use dashboard::display_mod::display_task;
use defmt::*;
use display_interface_spi::SPIInterface;
use embassy_executor::Spawner;
use embassy_stm32::can::Can;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::peripherals::*;
use embassy_stm32::spi::{self, Spi};
use embassy_stm32::time::Hertz;
use embassy_stm32::{Config, bind_interrupts, can};
use embassy_time::{Delay, Timer};
use embedded_hal_bus::spi::ExclusiveDevice;
use ili9488_rs::{Ili9488, Orientation, Rgb666Mode};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    FDCAN2_IT0 => can::IT0InterruptHandler<FDCAN2>;
    FDCAN2_IT1 => can::IT1InterruptHandler<FDCAN2>;
});

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    /////////////////////////////////////////////////
    // Initialize Peripherals
    /////////////////////////////////////////////////

    ////////////////////////////////
    // Initialize Clock
    ////////////////////////////////
    let mut config = Config::default();
    {
        use embassy_stm32::rcc::*;
        config.rcc.hse = Some(Hse {
            freq: Hertz(8_000_000),
            mode: HseMode::Bypass,
        });
        config.rcc.pll = Some(Pll {
            source: PllSource::HSE,
            prediv: PllPreDiv::DIV1,
            mul: PllMul::MUL40,
            divp: Some(PllPDiv::DIV2), // 160 MHz PLLP
            divq: Some(PllQDiv::DIV4), // 80 MHz PLLQ
            divr: Some(PllRDiv::DIV2), // Main system clock at 160 MHz
        });
        config.rcc.mux.fdcansel = mux::Fdcansel::PLL1_Q;
        config.rcc.sys = Sysclk::PLL1_R;
    }
    let peripherals = embassy_stm32::init(config);

    ////////////////////////////////
    // Initialize CAN
    ////////////////////////////////
    let can_rx = peripherals.PB5;
    let can_tx = peripherals.PB6;
    let mut can = can::CanConfigurator::new(peripherals.FDCAN2, can_rx, can_tx, Irqs);

    can.properties().set_extended_filter(
        can::filter::ExtendedFilterSlot::_0,
        can::filter::ExtendedFilter::accept_all_into_fifo1(),
    );
    // Nominal Baud Rate: 1MHz
    can.set_bitrate(1_000_000);

    // FD CAN Clock Mux: 8MHz
    can.set_fd_data_bitrate(8_000_000, false);
    let can = can.start(can::OperatingMode::NormalOperationMode);
    info!("Configured CAN");

    ////////////////////////////////
    // Initialize SPI
    ////////////////////////////////
    let mut spi_config = spi::Config::default();
    spi_config.frequency = Hertz::mhz(40);
    spi_config.miso_pull = embassy_stm32::gpio::Pull::Up;
    spi_config.gpio_speed = Speed::VeryHigh;

    let spi_sck = peripherals.PA5;
    let spi_miso = peripherals.PA6;
    let spi_mosi = peripherals.PA7;

    let spi = Spi::new(
        peripherals.SPI1,
        spi_sck,
        spi_mosi,
        spi_miso,
        peripherals.DMA1_CH1,
        peripherals.DMA1_CH2,
        spi_config,
    );

    info!("Configured SPI Peripherals");

    ////////////////////////////////
    // Initialize Touch Screen Peripherals
    ////////////////////////////////
    let touch_cs = peripherals.PA9;
    let _touch_irq = peripherals.PA8;

    // CS is Active Low
    let _touch_cs = Output::new(touch_cs, Level::High, Speed::VeryHigh);

    ////////////////////////////////
    // Initialize Screen Peripherals
    ////////////////////////////////
    let lcd_cs = peripherals.PA4;
    let lcd_reset = peripherals.PB0;
    let lcd_bright = peripherals.PA2;
    let lcd_dc = peripherals.PA3;

    let lcd_cs = Output::new(lcd_cs, Level::High, Speed::VeryHigh);
    let lcd_reset = Output::new(lcd_reset, Level::Low, Speed::VeryHigh);
    let _ = Output::new(lcd_bright, Level::High, Speed::VeryHigh);
    let lcd_dc = Output::new(lcd_dc, Level::Low, Speed::VeryHigh);
    let mut delay = Delay;

    // Turn on LCD Display
    let spi_device = ExclusiveDevice::new_no_delay(spi, lcd_cs).unwrap();
    let spi_interface = SPIInterface::new(spi_device, lcd_dc);
    let display = Ili9488::new(
        spi_interface,
        lcd_reset,
        &mut delay,
        Orientation::LandscapeFlipped,
        Rgb666Mode,
    )
    .unwrap();
    info!("Initialized ILI9488 Display");

    ////////////////////////////////
    // Spawn Threads
    ////////////////////////////////
    spawner.spawn(can_task(can)).unwrap();
    spawner.spawn(display_task(display)).unwrap();
}

#[embassy_executor::task]
async fn can_task(mut can: Can<'static>) {
    let mut last_read_ts = embassy_time::Instant::now();

    // Use the FD API's even if we don't get FD packets.
    loop {
        match can.read_fd().await {
            Ok(envelope) => {
                let (ts, rx_frame) = (envelope.ts, envelope.frame);
                let delta = (ts - last_read_ts).as_millis();
                last_read_ts = ts;
                info!(
                    "Rx: {} {:02x} --- using FD API {} ms",
                    rx_frame.header().len(),
                    rx_frame.data()[0..rx_frame.header().len() as usize],
                    delta,
                )
            }
            Err(err) => error!("Error in frame: {}", err),
        }
        Timer::after_millis(1).await;
    }
}
