#![no_std]
#![no_main]
use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::peripherals::*;
use embassy_stm32::time::Hertz;
use embassy_stm32::{Config, bind_interrupts, can};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    FDCAN2_IT0 => can::IT0InterruptHandler<FDCAN2>;
    FDCAN2_IT1 => can::IT1InterruptHandler<FDCAN2>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    ////////////////////////////////
    // Initialize Peripherals
    ////////////////////////////////
    let mut config = Config::default();
    {
        use embassy_stm32::rcc::*;
        // TODO: Configure Clock
        config.rcc.hse = Some(Hse {
            freq: Hertz(24_000_000),
            mode: HseMode::Oscillator,
        });
        config.rcc.pll = Some(Pll {
            source: PllSource::HSE,
            prediv: PllPreDiv::DIV6,
            mul: PllMul::MUL85,
            divp: None,
            divq: Some(PllQDiv::DIV8), // 42.5 Mhz for fdcan.
            divr: Some(PllRDiv::DIV2), // Main system clock at 170 MHz
        });
        config.rcc.mux.fdcansel = mux::Fdcansel::PLL1_Q;
        config.rcc.sys = Sysclk::PLL1_R;
    }
    let peripherals = embassy_stm32::init(config);

    let mut can =
        can::CanConfigurator::new(peripherals.FDCAN2, peripherals.PB5, peripherals.PB6, Irqs);

    can.properties().set_extended_filter(
        can::filter::ExtendedFilterSlot::_0,
        can::filter::ExtendedFilter::accept_all_into_fifo1(),
    );

    // 250k bps
    can.set_bitrate(250_000);

    let use_fd = false;

    // 1M bps
    if use_fd {
        can.set_fd_data_bitrate(1_000_000, false);
    }

    info!("Configured");

    let mut can = can.start(can::OperatingMode::NormalOperationMode);

    let mut i = 0;
    let mut last_read_ts = embassy_time::Instant::now();

    // Use the FD API's even if we don't get FD packets.
    loop {
        let frame = can::frame::FdFrame::new_extended(0x123456F, &[i; 16]).unwrap();
        info!("Writing frame using FD API");
        _ = can.write_fd(&frame).await;

        match can.read_fd().await {
            Ok(envelope) => {
                let (ts, rx_frame) = (envelope.ts, envelope.frame);
                let delta = (ts - last_read_ts).as_millis();
                last_read_ts = ts;
                info!(
                    "Rx: {} {:02x} --- using FD API {}ms",
                    rx_frame.header().len(),
                    rx_frame.data()[0..rx_frame.header().len() as usize],
                    delta,
                )
            }
            Err(_err) => error!("Error in frame"),
        }

        Timer::after_millis(250).await;

        i += 1;
        if i > 4 {
            break;
        }
    }
}
