#![no_std]
#![no_main]
use dashboard::btn_mod::{btn1_task, btn2_task};
use dashboard::can_mod::can_receive_task;
use dashboard::display_mod::display_task;
use dashboard::led_mod::led_task;
use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::exti::ExtiInput;
use embassy_stm32::gpio::{Level, Output, OutputType, Pull, Speed};
use embassy_stm32::spi::{self, Spi};
use embassy_stm32::time::Hertz;
use embassy_stm32::timer::low_level::CountingMode;
use embassy_stm32::timer::simple_pwm::{PwmPin, SimplePwm};
use embassy_stm32::{Config, bind_interrupts, can, peripherals::*};
use embassy_time::Delay;
use embedded_hal_bus::spi::ExclusiveDevice;
use mipidsi::Builder;
use mipidsi::interface::SpiInterface;
use mipidsi::models::ILI9488Rgb666;
use mipidsi::options::Orientation;
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    FDCAN2_IT0 => can::IT0InterruptHandler<FDCAN2>;
    FDCAN2_IT1 => can::IT1InterruptHandler<FDCAN2>;
});

/// Buffer Size for the CAN TX buffer
pub const TX_BUF_SIZE: usize = 2;
/// Buffer Size for the CAN RX buffer
pub const RX_BUF_SIZE: usize = 20;
// Default baud rate is 1 MHz
const CAN_BAUD_RATE: u32 = 100_000;

const SPI_BUFFER: usize = 512;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    /////////////////////////////////////////////////
    // Initialize Peripherals
    /////////////////////////////////////////////////

    ////////////////////////////////
    // Initialize Clock
    ////////////////////////////////
    let mut config = Config::default();
    // NOTE: Changing clock speed to anything not 170 MHz will make the LED colors wrong.
    // Will effect the PWM frequency and duty cycle.
    {
        // 170 MHz
        use embassy_stm32::rcc::*;
        // Use external 8 MHz crystal osscillator
        config.rcc.hse = Some(Hse {
            freq: Hertz(8_000_000),
            mode: HseMode::Bypass,
        });
        config.rcc.pll = Some(Pll {
            source: PllSource::HSE,
            prediv: PllPreDiv::DIV2,
            mul: PllMul::MUL85,
            divp: Some(PllPDiv::DIV2), // 170 MHz PLLP
            divq: Some(PllQDiv::DIV2), // 170 MHz PLLQ
            divr: Some(PllRDiv::DIV2), // Main system clock at 170 MHz
        });
        config.rcc.mux.fdcansel = mux::Fdcansel::HSE;
        config.rcc.sys = Sysclk::PLL1_R;
    }

    let peripherals = embassy_stm32::init(config);

    let can_rx = peripherals.PB5;
    let can_tx = peripherals.PB6;
    let can_stby = peripherals.PB7;
    let can_peripheral = peripherals.FDCAN2;

    let btn1_pin = peripherals.PB3;
    let btn2_pin = peripherals.PB4;

    let led_pwm = peripherals.PA0;
    let led_timer = peripherals.TIM2;

    let spi_sck = peripherals.PA5;
    let spi_miso = peripherals.PA6;
    let spi_mosi = peripherals.PA7;
    let spi_peripheral = peripherals.SPI1;
    let spi_tx_dma = peripherals.DMA1_CH1;
    let spi_rx_dma = peripherals.DMA1_CH2;

    let touch_cs = peripherals.PA9;
    let _touch_irq = peripherals.PA8;
    let lcd_cs = peripherals.PA4;
    let lcd_reset = peripherals.PB0;
    let lcd_bright = peripherals.PA2;
    let lcd_dc = peripherals.PA3;

    ////////////////////////////////
    // Initialize CAN
    ////////////////////////////////
    static _TX_BUF: StaticCell<can::TxFdBuf<TX_BUF_SIZE>> = StaticCell::new();
    static _RX_BUF: StaticCell<can::RxFdBuf<RX_BUF_SIZE>> = StaticCell::new();

    let mut can = can::CanConfigurator::new(can_peripheral, can_rx, can_tx, Irqs);
    let can_stby = Output::new(can_stby, Level::Low, Speed::Low);
    // Because the destructor resets the gpio pin's state, use mem::forget to drop the variable
    core::mem::forget(can_stby);

    can.properties().set_extended_filter(
        can::filter::ExtendedFilterSlot::_0,
        can::filter::ExtendedFilter::accept_all_into_fifo1(),
    );
    // Nominal Baud Rate: 1M bits/s
    can.set_bitrate(CAN_BAUD_RATE); // for prototyping

    let can = can.start(can::OperatingMode::NormalOperationMode);

    // let can = can.buffered_fd(
    //     TX_BUF.init(can::TxFdBuf::new()),
    //     RX_BUF.init(can::RxFdBuf::new()),
    // );
    info!("Configured CAN");

    ////////////////////////////////
    // Initialize External Interrupt Buttons
    ////////////////////////////////
    let btn1 = ExtiInput::new(btn1_pin, peripherals.EXTI3, Pull::Up);
    let btn2 = ExtiInput::new(btn2_pin, peripherals.EXTI4, Pull::Up);

    ////////////////////////////////
    // Initialize LED Lights
    ////////////////////////////////
    let led_in = PwmPin::new(led_pwm, OutputType::PushPull);
    let led_dma = peripherals.DMA2_CH1;

    // PWM_FREQ = 1 / data_transfer_time = 1 / 1.25us = 800kHz
    const PWM_FREQ: Hertz = Hertz::khz(800);

    // Obtain a PWM handler, configure the Timer and Frequency
    // The prescaler and ARR are automatically set
    // Given this system frequency and pwm frequency the max duty cycle will be 50
    let mut led_in = SimplePwm::new(
        led_timer,
        Some(led_in),
        None,
        None,
        None,
        PWM_FREQ,
        CountingMode::EdgeAlignedUp,
    );
    // Enable channel 1
    led_in.ch1().enable();
    info!("Configured LED Peripherals");

    ////////////////////////////////
    // Initialize SPI
    ////////////////////////////////
    let mut spi_config = spi::Config::default();
    // 40 MHz is the maximum frequency the ILI9488 can handle
    spi_config.frequency = Hertz::mhz(40);
    spi_config.miso_pull = embassy_stm32::gpio::Pull::Up;
    spi_config.gpio_speed = Speed::VeryHigh;

    let spi = Spi::new(
        spi_peripheral,
        spi_sck,
        spi_mosi,
        spi_miso,
        spi_tx_dma,
        spi_rx_dma,
        spi_config,
    );

    info!("Configured SPI Peripherals");

    ////////////////////////////////
    // Initialize Touch Screen Peripherals
    ////////////////////////////////

    // CS is Active Low
    let _touch_cs = Output::new(touch_cs, Level::High, Speed::VeryHigh);

    ////////////////////////////////
    // Initialize Screen Peripherals
    ////////////////////////////////

    let lcd_cs = Output::new(lcd_cs, Level::High, Speed::VeryHigh);
    let lcd_reset = Output::new(lcd_reset, Level::Low, Speed::VeryHigh);
    // Turn the LCD's backlight on indefinetly
    // Because the destructor resets the gpio pin's state, use mem::forget to drop the variable
    let _lcd_bright = Output::new(lcd_bright, Level::High, Speed::Medium);
    core::mem::forget(_lcd_bright);
    let lcd_dc = Output::new(lcd_dc, Level::Low, Speed::VeryHigh);
    let mut delay = Delay;

    // Turn on LCD Display
    static DISPLAY_BUFFER: StaticCell<[u8; SPI_BUFFER]> = StaticCell::new();
    let spi_buffer = DISPLAY_BUFFER.init([0u8; SPI_BUFFER]);
    let spi_device = ExclusiveDevice::new_no_delay(spi, lcd_cs).unwrap();
    let spi_interface = SpiInterface::new(spi_device, lcd_dc, spi_buffer);

    let display = Builder::new(ILI9488Rgb666, spi_interface)
        .reset_pin(lcd_reset)
        .color_order(mipidsi::options::ColorOrder::Bgr)
        .orientation(
            Orientation::new()
                .rotate(mipidsi::options::Rotation::Deg270)
                .flip_vertical(),
        )
        .init(&mut delay)
        .unwrap();

    info!("Configured ILI9488 Display");

    ////////////////////////////////3
    // Spawn Tasks
    ////////////////////////////////
    info!("Spawning Tasks");
    spawner.spawn(can_receive_task(can)).unwrap();
    spawner.spawn(led_task(led_in, led_dma)).unwrap();
    spawner.spawn(display_task(display)).unwrap();
    spawner.spawn(btn1_task(btn1)).unwrap();
    spawner.spawn(btn2_task(btn2)).unwrap();
}
