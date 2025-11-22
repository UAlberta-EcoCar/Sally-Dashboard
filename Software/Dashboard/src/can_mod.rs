//! Module for CAN communication
//!
//! Handles the reception and transmission of CAN messages
//!
//! This requires a lot of global mutable data. See this
//! [article](https://blog.theembeddedrustacean.com/sharing-data-among-tasks-in-rust-embassy-synchronization-primitives#heading-the-list-of-primitives)
//! on sharing data in Embassy.
//!
//! <div class="warning">
//! Rust has non-lexical lifetimes, however anytime a mutex lock is acquired it should
//! be dropped as soon as it is not needed. Mutex locks can last to the end of a function's
//! lifetime and create deadlocks which are hard to debug.
//! </div>

use bincode::{
    Decode, Encode,
    error::{DecodeError, EncodeError},
};
use defmt::*;
use embassy_stm32::can::{BufferedCanFd, frame::FdFrame};
use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, mutex::Mutex};
use embassy_time::Timer;
use embedded_can::Id;

use crate::eco_can::{
    ECOCAN_H2Pack1_t, ECOCAN_H2Pack2_t, FDCAN_BOOSTPack1_t, FDCAN_BOOSTPack2_t, FDCAN_BOOSTPack3_t,
    FDCAN_FccPack1_t, FDCAN_FccPack2_t, FDCAN_FccPack3_t, FDCAN_FetPack_t, FDCAN_RelPackCap_t,
    FDCAN_RelPackFc_t, FDCAN_RelPackMtr_t, FDCANPack, RelayState,
};

/// Buffer Size for the CAN TX buffer
pub const TX_BUF_SIZE: usize = 1;
/// Buffer Size for the CAN RX buffer
pub const RX_BUF_SIZE: usize = 16;

pub static RELAY_STATE: Mutex<ThreadModeRawMutex, RelayState> = Mutex::new(RelayState::RELAY_STBY);

pub static FET_DATA: Mutex<ThreadModeRawMutex, FDCAN_FetPack_t> = Mutex::new(FDCAN_FetPack_t {
    fet_config: 0,
    input_volt: 0,
    cap_volt: 0,
    cap_curr: 0,
    res_curr: 0,
    out_curr: 0,
});

pub static FCC_PACK1_DATA: Mutex<ThreadModeRawMutex, FDCAN_FccPack1_t> =
    Mutex::new(FDCAN_FccPack1_t {
        fc_press: 0,
        fc_temp: 0,
    });
pub static FCC_PACK2_DATA: Mutex<ThreadModeRawMutex, FDCAN_FccPack2_t> =
    Mutex::new(FDCAN_FccPack2_t {
        fan_rpm1: 0,
        fan_rpm2: 0,
    });
pub static FCC_PACK3_DATA: Mutex<ThreadModeRawMutex, FDCAN_FccPack3_t> =
    Mutex::new(FDCAN_FccPack3_t {
        bme_temp: 0,
        bme_humid: 0,
    });

pub static H2_PACK1_DATA: Mutex<ThreadModeRawMutex, ECOCAN_H2Pack1_t> =
    Mutex::new(ECOCAN_H2Pack1_t {
        h2_sense_1: 0,
        h2_sense_2: 0,
        h2_sense_3: 0,
        h2_sense_4: 0,
    });
pub static H2_PACK2_DATA: Mutex<ThreadModeRawMutex, ECOCAN_H2Pack2_t> =
    Mutex::new(ECOCAN_H2Pack2_t {
        bme_temp: 0,
        bme_humid: 0,
        imon_7v: 0,
        imon_12v: 0,
    });

pub static BOOST_PACK1_DATA: Mutex<ThreadModeRawMutex, FDCAN_BOOSTPack1_t> =
    Mutex::new(FDCAN_BOOSTPack1_t {
        in_curr: 0,
        in_volt: 0,
    });
pub static BOOST_PACK2_DATA: Mutex<ThreadModeRawMutex, FDCAN_BOOSTPack2_t> =
    Mutex::new(FDCAN_BOOSTPack2_t {
        out_curr: 0,
        out_volt: 0,
    });
pub static BOOST_PACK3_DATA: Mutex<ThreadModeRawMutex, FDCAN_BOOSTPack3_t> =
    Mutex::new(FDCAN_BOOSTPack3_t {
        efficiency: 0,
        joules: 0,
    });

/// Fuel Cell Reading
pub static REL_FC_PACK: Mutex<ThreadModeRawMutex, FDCAN_RelPackFc_t> =
    Mutex::new(FDCAN_RelPackFc_t {
        fc_volt: 0,
        fc_curr: 0,
    });
pub static REL_CAP_PACK: Mutex<ThreadModeRawMutex, FDCAN_RelPackCap_t> =
    Mutex::new(FDCAN_RelPackCap_t {
        cap_volt: 0,
        cap_curr: 0,
    });
pub static RELAY_MOTOR_PACK: Mutex<ThreadModeRawMutex, FDCAN_RelPackMtr_t> =
    Mutex::new(FDCAN_RelPackMtr_t {
        mtr_volt: 0,
        mtr_curr: 0,
    });

/// Responsible for handling the reception of CAN messages
#[embassy_executor::task]
pub async fn can_receive_task(mut can: BufferedCanFd<'static, TX_BUF_SIZE, RX_BUF_SIZE>) {
    let mut last_read_ts = embassy_time::Instant::now();
    let mut tx_data = [0; 64];

    // Use the FD API's even if we don't get FD packets.
    let mut i = 0u32;
    loop {
        let mut pack = RELAY_MOTOR_PACK.lock().await;
        pack.mtr_volt = i;
        drop(pack);
        i += 1;

        let pack = REL_CAP_PACK.lock().await;
        info!("Cap pack cap_volt: {}", pack.cap_volt);
        drop(pack);

        if let Err(_) = encode_can_package(&RELAY_MOTOR_PACK, &mut tx_data).await {
            error!("CAN Encode Error");
        }
        let frame = FdFrame::new_extended(FDCAN_RelPackCap_t::FDCAN_ID, &tx_data).unwrap();
        can.write(frame).await;
        Timer::after_millis(100).await;

        match can.read().await {
            Ok(envelope) => {
                let (ts, rx_frame) = (envelope.ts, envelope.frame);
                let delta = (ts - last_read_ts).as_millis();
                last_read_ts = ts;
                info!(
                    "Rx: {} {:02x} --- using FD API {} ms",
                    rx_frame.header().len(),
                    rx_frame.data()[0..rx_frame.header().len() as usize],
                    delta,
                );
                if let Err(e) = decode_can_frame(&rx_frame).await {
                    match e {
                        DecodeError::Other(error) => {
                            error!("CAN Decode Error: {}", error);
                        }
                        _ => {
                            error!("CAN Decode Error")
                        }
                    }
                }
            }
            Err(err) => error!("Error in frame: {}", err),
        }
        Timer::after_millis(1).await;
    }
}

/// Decodes a CAN frame into its corresponding CAN package
///
/// Returns an error if the frame cannot be decoded.
pub async fn decode_can_frame(frame: &FdFrame) -> Result<(), DecodeError> {
    // Get ID
    let id = match frame.header().id() {
        Id::Standard(id) => u32::from(id.as_raw()),
        Id::Extended(id) => id.as_raw(),
    };
    // Get data of CAN package (up to 64 bytes)
    let data = &frame.data()[..frame.header().len() as usize];

    // Match ID to CAN package, and decode
    match id {
        RelayState::FDCAN_ID => {
            let mut relay_state = RELAY_STATE.lock().await;
            *relay_state = RelayState::try_from(data[0])?;
            Ok(())
        }

        FDCAN_FccPack1_t::FDCAN_ID => decode_can_data(&FCC_PACK1_DATA, data).await,
        FDCAN_FccPack2_t::FDCAN_ID => decode_can_data(&FCC_PACK2_DATA, data).await,
        FDCAN_FccPack3_t::FDCAN_ID => decode_can_data(&FCC_PACK3_DATA, data).await,

        FDCAN_FetPack_t::FDCAN_ID => decode_can_data(&FET_DATA, data).await,

        FDCAN_RelPackMtr_t::FDCAN_ID => decode_can_data(&RELAY_MOTOR_PACK, data).await,
        FDCAN_RelPackCap_t::FDCAN_ID => decode_can_data(&REL_CAP_PACK, data).await,
        FDCAN_RelPackFc_t::FDCAN_ID => decode_can_data(&REL_FC_PACK, data).await,

        ECOCAN_H2Pack1_t::FDCAN_ID => decode_can_data(&H2_PACK1_DATA, data).await,
        ECOCAN_H2Pack2_t::FDCAN_ID => decode_can_data(&H2_PACK2_DATA, data).await,

        FDCAN_BOOSTPack1_t::FDCAN_ID => decode_can_data(&BOOST_PACK1_DATA, data).await,
        FDCAN_BOOSTPack2_t::FDCAN_ID => decode_can_data(&BOOST_PACK2_DATA, data).await,
        FDCAN_BOOSTPack3_t::FDCAN_ID => decode_can_data(&BOOST_PACK3_DATA, data).await,

        _ => {
            info!("Non-Relevant ID: {:016b}", id);
            Ok(())
        }
    }
}

/// Decodes a byte array into a CAN package
async fn decode_can_data<T: Decode<()>>(
    package: &Mutex<ThreadModeRawMutex, T>,
    data: &[u8],
) -> Result<(), DecodeError> {
    let bincode_config = bincode::config::standard()
        .with_big_endian()
        .with_fixed_int_encoding();
    // Decode received package bytes into the desired package struct and update can package
    let mut p = package.lock().await;
    *p = bincode::decode_from_slice(&data, bincode_config)?.0;
    Ok(())
}

/// Encodes a CAN package into a byte array
async fn encode_can_package<T: Encode + Clone>(
    package: &Mutex<ThreadModeRawMutex, T>,
    mut encoded_data: &mut [u8],
) -> Result<usize, EncodeError> {
    let bincode_config = bincode::config::standard()
        .with_big_endian()
        .with_fixed_int_encoding();
    let p = package.lock().await;
    bincode::encode_into_slice(p.clone(), &mut encoded_data, bincode_config)
}
