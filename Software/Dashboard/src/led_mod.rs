//! Module for the LED lights
//!
//! Responsible for handling the WS2812B LED lights on the dashboard.
//!
//! WS2812B Datasheet: [https://cdn-shop.adafruit.com/datasheets/WS2812B.pdf](https://cdn-shop.adafruit.com/datasheets/WS2812B.pdf)

// use defmt::info;
use defmt::trace;
use embassy_stm32::Peri;
use embassy_stm32::peripherals::{DMA2_CH1, TIM2};
use embassy_stm32::timer::simple_pwm::SimplePwm;
use embassy_time::Timer;
use rgb_led_pwm_dma_maker::{LedDataComposition, LedDmaBuffer, RGB, calc_dma_buffer_length};

use crate::can_mod::RELAY_STATE;
use crate::eco_can::RelayState;

// There are 5 LED's on the board
const LED_COUNT: usize = 5;

/// Updates the LED lights on the dashboard
#[embassy_executor::task]
pub async fn led_task(mut led_in: SimplePwm<'static, TIM2>, mut led_dma: Peri<'static, DMA2_CH1>) {
    // RESET_LENGTH = reset_period / data_transfer_time = 50us / 1.25us = 40
    const RESET_LENGTH: usize = 40;
    // Calculate the dma buffer's length at compile time
    // Uses RGB888 formatting
    const DMA_BUFFER_LEN: usize = calc_dma_buffer_length(8 * 3, LED_COUNT, RESET_LENGTH);
    // t1h = T1H / data_transfer_time * max_duty_cycle = 0.8us / 1.25us * 200 =
    let t1h: u16 = 128;
    // t1h = T0H / data_transfer_time * max_duty_cycle = 0.4us / 1.25us * 200 =
    let t0h: u16 = 64;

    let mut led_array: [RGB; LED_COUNT] = [
        RGB::new(3, 0, 0),
        RGB::new(0, 3, 0),
        RGB::new(0, 0, 3),
        RGB::new(0, 3, 3),
        RGB::new(3, 3, 0),
    ];
    let mut dma_buffer = LedDmaBuffer::<DMA_BUFFER_LEN>::new(t1h, t0h, LedDataComposition::GRB);

    loop {
        let relay_state = RELAY_STATE.lock().await;

        // Inialized display screen if switching relay state
        match *relay_state {
            RelayState::RELAY_STRTP => led_startup(&mut led_array),
            RelayState::RELAY_CHRGE => led_charging(&mut led_array),
            RelayState::RELAY_STBY => led_standby(&mut led_array),
            RelayState::RELAY_RUN => led_running(&mut led_array),
        }
        let _ = dma_buffer.set_dma_buffer(&led_array, None);
        // Output pwm waveform to set LEDs
        led_in
            .waveform::<embassy_stm32::timer::Ch1>(led_dma.reborrow(), dma_buffer.get_dma_buffer())
            .await;
        trace!("LED Health check");
        Timer::after_millis(600).await;
    }
}

fn led_startup(led_array: &mut [RGB; LED_COUNT]) {
    *led_array = [
        RGB::new(3, 0, 0),
        RGB::new(0, 3, 0),
        RGB::new(0, 0, 3),
        RGB::new(0, 3, 3),
        RGB::new(3, 3, 0),
    ];
}
fn led_charging(led_array: &mut [RGB; LED_COUNT]) {
    *led_array = [
        RGB::new(0, 3, 0),
        RGB::new(0, 3, 0),
        RGB::new(0, 3, 0),
        RGB::new(0, 3, 0),
        RGB::new(0, 3, 0),
    ];
}
fn led_standby(led_array: &mut [RGB; LED_COUNT]) {
    *led_array = [
        RGB::new(3, 0, 0),
        RGB::new(0, 3, 0),
        RGB::new(0, 0, 3),
        RGB::new(0, 3, 3),
        RGB::new(3, 3, 0),
    ];
}
fn led_running(led_array: &mut [RGB; LED_COUNT]) {
    led_array.rotate_left(1);
}
