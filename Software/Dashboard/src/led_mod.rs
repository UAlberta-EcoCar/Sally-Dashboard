//! Module for the LED lights
//!
//! Responsible for handling the WS2812B LED lights on the dashboard.
//!
//! WS2812B Datasheet: [https://cdn-shop.adafruit.com/datasheets/WS2812B.pdf](https://cdn-shop.adafruit.com/datasheets/WS2812B.pdf)

use embassy_stm32::Peri;
use embassy_stm32::peripherals::{DMA2_CH1, TIM2};
use embassy_stm32::timer::simple_pwm::SimplePwm;
use embassy_time::Timer;
use rgb_led_pwm_dma_maker::{LedDataComposition, LedDmaBuffer, RGB, calc_dma_buffer_length};

/// Updates the LED lights on the dashboard
#[embassy_executor::task]
pub async fn led_task(mut led_in: SimplePwm<'static, TIM2>, mut led_dma: Peri<'static, DMA2_CH1>) {
    // There are 5 LED's on the board
    const LED_COUNT: usize = 5;
    // RESET_LENGTH = reset_period / data_transfer_time = 50us / 1.25us = 40
    const RESET_LENGTH: usize = 40;
    // Calculate the dma buffer's length at compile time
    const DMA_BUFFER_LEN: usize = calc_dma_buffer_length(8 * 3, LED_COUNT, RESET_LENGTH);
    // t1h = T1H / data_transfer_time * max_duty_cycle = 0.8us / 1.25us * 50 = 32
    let t1h: u16 = 32;
    // t1h = T0H / data_transfer_time * max_duty_cycle = 0.4us / 1.25us * 50 = 16
    let t0h: u16 = 16;

    let led_array: [RGB; LED_COUNT] = [
        RGB::new(1, 0, 0),
        RGB::new(0, 1, 0),
        RGB::new(0, 0, 1),
        RGB::new(0, 1, 1),
        RGB::new(1, 1, 0),
    ];
    let mut dma_buffer = LedDmaBuffer::<DMA_BUFFER_LEN>::new(t1h, t0h, LedDataComposition::GRB);

    loop {
        let _ = dma_buffer.set_dma_buffer(&led_array, None);
        led_in
            .waveform::<embassy_stm32::timer::Ch1>(led_dma.reborrow(), dma_buffer.get_dma_buffer())
            .await;
        Timer::after_millis(200).await;
    }
}
