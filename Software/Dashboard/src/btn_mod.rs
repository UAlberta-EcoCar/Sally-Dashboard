//! Module for Handling Buttons
//! Uses external interrupts to handle button input.
//!
//! Note that the documentation and examples for `embassy-stm32` version "0.4.0" does
//! not match the actual source code for the `exti` module. The `exti` module
//! is actually the same as version "0.3.0".
//!
//! Note that **Non-Blocking** delays are used to handle signal bouncing.
//!
use defmt::info;
use embassy_stm32::exti::ExtiInput;
use embassy_time::Timer;

/// A delay to handle signal bounce. Default at 50ms.
pub const BOUNCE_DELAY: u64 = 50;

#[embassy_executor::task]
pub async fn btn1_task(mut btn1: ExtiInput<'static>) {
    let mut i = 0;
    loop {
        btn1.wait_for_falling_edge().await;
        info!("Btn 1 Pressed!");
        Timer::after_millis(BOUNCE_DELAY).await;

        i += 1;
        btn1.wait_for_high().await;
        Timer::after_millis(BOUNCE_DELAY).await;
        info!("Btn 1 Released {} times!", i);
    }
}

#[embassy_executor::task]
pub async fn btn2_task(mut btn2: ExtiInput<'static>) {
    let mut i = 0;
    loop {
        btn2.wait_for_falling_edge().await;
        info!("Btn 2 Pressed!");
        Timer::after_millis(BOUNCE_DELAY).await;

        i += 1;
        btn2.wait_for_high().await;
        Timer::after_millis(BOUNCE_DELAY).await;
        info!("Btn 2 Released {} times!", i);
    }
}
