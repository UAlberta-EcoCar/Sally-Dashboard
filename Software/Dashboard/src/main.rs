#![no_std]
#![no_main]
use dashboard::btn_mod::{btn1_task, btn2_task};
use dashboard::can_mod::{RX_BUF_SIZE, TX_BUF_SIZE, can_receive_task};
use dashboard::display_mod::display_task;
use dashboard::led_mod::led_task;
use defmt::*;
use display_interface_spi::SPIInterface;
use embassy_executor::Spawner;
use embassy_stm32::can::filter::Mask32;
use embassy_stm32::can::{
    Can, Fifo, Frame, Id, Rx0InterruptHandler, Rx1InterruptHandler, SceInterruptHandler,
    TxInterruptHandler,
};
use embassy_stm32::exti::ExtiInput;
use embassy_stm32::gpio::{Level, Output, OutputType, Pull, Speed};
use embassy_stm32::spi::{self, Spi};
use embassy_stm32::time::Hertz;
use embassy_stm32::timer::low_level::CountingMode;
use embassy_stm32::timer::simple_pwm::{PwmPin, SimplePwm};
use embassy_stm32::{Config, bind_interrupts, can, peripherals::*};
use embassy_time::Delay;
use embedded_hal_bus::spi::ExclusiveDevice;
use ili9488_rs::{Ili9488, Orientation, Rgb666Mode};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    CAN1_RX0 => Rx0InterruptHandler<CAN1>;
    CAN1_RX1 => Rx1InterruptHandler<CAN1>;
    CAN1_SCE => SceInterruptHandler<CAN1>;
    CAN1_TX => TxInterruptHandler<CAN1>;
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
        config.rcc.pll = Some(Pll {
            source: PllSource::HSI,
            prediv: PllPreDiv::DIV1,
            mul: PllMul::MUL10,
            divp: None,
            divq: None,
            divr: Some(PllRDiv::DIV2), // Main system clock at 80 MHz
        });
        config.rcc.hsi = true;
        config.rcc.sys = Sysclk::PLL1_R;
    }
    let peripherals = embassy_stm32::init(config);

    ////////////////////////////////
    // Initialize CAN
    ////////////////////////////////
    // static _TX_BUF: StaticCell<can::TxFdBuf<TX_BUF_SIZE>> = StaticCell::new();
    // static _RX_BUF: StaticCell<can::RxFdBuf<RX_BUF_SIZE>> = StaticCell::new();
    let can_rx = peripherals.PA11;
    let can_tx = peripherals.PA12;
    let can_stby = peripherals.PB1;

    let mut can = Can::new(peripherals.CAN1, can_rx, can_tx, Irqs);
    let _can_stby = Output::new(can_stby, Level::Low, Speed::Low);
    // Because the destructor resets the gpio pin's state, use mem::forget to drop the variable
    core::mem::forget(_can_stby);

    can.modify_filters()
        .enable_bank(0, can::Fifo::Fifo0, Mask32::accept_all());

    can.modify_config()
        .set_loopback(false)
        // .set_loopback(true) // Receivenew_extended own frames
        .set_silent(false)
        .set_bitrate(100_000);
    can.enable().await;

    info!("Configured CAN");

    ////////////////////////////////
    // Initialize External Interrupt Buttons
    ////////////////////////////////
    // let btn1 = ExtiInput::new(peripherals.PA8, peripherals.EXTI8, Pull::Up);
    // let btn2 = ExtiInput::new(peripherals.PA9, peripherals.EXTI9, Pull::Up);

    ////////////////////////////////
    // Initialize LED Lights
    ////////////////////////////////
    let led_in = PwmPin::new(peripherals.PA0, OutputType::PushPull);
    let led_dma = peripherals.DMA2_CH1;

    // PWM_FREQ = 1 / data_transfer_time = 1 / 1.25us = 800kHz
    const PWM_FREQ: Hertz = Hertz::khz(800);

    // Obtain a PWM handler, configure the Timer and Frequency
    // The prescaler and ARR are automatically set
    // Given this system frequency and pwm frequency the max duty cycle will be 50
    let mut led_in = SimplePwm::new(
        peripherals.TIM2,
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
    spi_config.frequency = Hertz::mhz(40);
    spi_config.miso_pull = embassy_stm32::gpio::Pull::Up;
    spi_config.gpio_speed = Speed::VeryHigh;

    let spi_sck = peripherals.PA1;
    let spi_miso = peripherals.PA6;
    let spi_mosi = peripherals.PA7;

    let spi = Spi::new(
        peripherals.SPI1,
        spi_sck,
        spi_mosi,
        spi_miso,
        peripherals.DMA1_CH3,
        peripherals.DMA2_CH3,
        spi_config,
    );

    info!("Configured SPI Peripherals");

    ////////////////////////////////
    // Initialize Touch Screen Peripherals
    ////////////////////////////////
    // let touch_cs = peripherals.PB0;
    // let _touch_irq = peripherals.PB1;

    // CS is Active Low
    // let _touch_cs = Output::new(touch_cs, Level::High, Speed::VeryHigh);

    ////////////////////////////////
    // Initialize Screen Peripherals
    ////////////////////////////////
    let lcd_cs = peripherals.PA5;
    let lcd_reset = peripherals.PA3;
    let lcd_bright = peripherals.PA4;
    let lcd_dc = peripherals.PA2;

    let lcd_cs = Output::new(lcd_cs, Level::High, Speed::VeryHigh);
    let lcd_reset = Output::new(lcd_reset, Level::Low, Speed::VeryHigh);
    // Turn the LCD's backlight on indefinetly
    // Because the destructor resets the gpio pin's state, use mem::forget to drop the variable
    let _lcd_bright = Output::new(lcd_bright, Level::High, Speed::Medium);
    core::mem::forget(_lcd_bright);
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
    info!("Configured ILI9488 Display");

    ////////////////////////////////3
    // Spawn Tasks
    ////////////////////////////////
    info!("Spawning Tasks");
    spawner.spawn(can_receive_task(can)).unwrap();
    // spawner.spawn(led_task(led_in, led_dma)).unwrap();
    spawner.spawn(display_task(display)).unwrap();
    //     spawner.spawn(btn1_task(btn1)).unwrap();
    //     spawner.spawn(btn2_task(btn2)).unwrap();
}
