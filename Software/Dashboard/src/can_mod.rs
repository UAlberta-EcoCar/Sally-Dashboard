//! Module for CAN communication
//!
//! Handles the reception and transmission of CAN messages
//!
//! This requires a lot of global mutable data. See this
//! [article](https://blog.theembeddedrustacean.com/sharing-data-among-tasks-in-rust-embassy-synchronization-primitives#heading-the-list-of-primitives)
//! on sharing data in Embassy.
//!
//! <div class="warning">
//! Anytime a mutex lock is acquired it should be dropped as soon as it is not needed
//! to avoid deadlocks. Mutex locks will last to the end of its scope if they are not
//! dropped before then. See <a href="https://stackoverflow.com/questions/57467555/will-the-non-lexical-lifetime-borrow-checker-release-locks-prematurely">here</a>
//! for more information.
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
pub const RX_BUF_SIZE: usize = 20;

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
    let mut tx_data = [0; 64];

    // Use the FD API's even if we don't get FD packets.
    let debug = true;
    if debug {
        for _ in 0..40 {
            let mut pack = RELAY_MOTOR_PACK.lock().await;
            pack.mtr_volt += 1;
            drop(pack);

            match encode_can_package(&RELAY_MOTOR_PACK, &mut tx_data).await {
                Ok(tx_len) => {
                    let frame =
                        FdFrame::new_extended(FDCAN_RelPackCap_t::FDCAN_ID, &tx_data[..tx_len])
                            .unwrap();
                    can.write(frame).await;
                }
                Err(_) => {
                    error!("CAN Encode Error");
                }
            }
        }
    }
    loop {
        // await one frame (blocks until at least one frame arrives)
        match can.read().await {
            Ok(envelope) => {
                // Process the first can frame received
                process_rx_can_frame(&envelope.frame).await;
                // then drain the receive buffer
                drain_rx_can_buffer(&can).await;
            }
            Err(err) => error!("CAN Frame Error: {}", err),
        }
        Timer::after_millis(1).await;
    }
}

/// Process the remaining CAN frames in the RX buffer
async fn drain_rx_can_buffer(can: &BufferedCanFd<'static, TX_BUF_SIZE, RX_BUF_SIZE>) {
    // repeatedly call try_receive() until the buffer is empty
    let reader = can.reader();
    for _ in 0..RX_BUF_SIZE {
        if let Ok(frame) = reader.try_receive() {
            match frame {
                Ok(envelope) => {
                    process_rx_can_frame(&envelope.frame).await;
                }
                Err(err) => error!("CAN Frame Error: {}", err),
            }
        } else {
            return;
        }
    }
}

/// Decodes a CAN frame and handles decode errors
async fn process_rx_can_frame(rx_frame: &FdFrame) {
    if let Err(_) = decode_can_frame(&rx_frame).await {
        error!("CAN Decode Error");
    }
    // Put debug messages after decoding a CAN frame here
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
    let rx_data = &frame.data()[..frame.header().len() as usize];

    // Match ID to CAN package, and decode
    match id {
        RelayState::FDCAN_ID => {
            let mut relay_state = RELAY_STATE.lock().await;
            *relay_state = RelayState::try_from(rx_data[0])?;
            Ok(())
        }

        FDCAN_FccPack1_t::FDCAN_ID => decode_can_data(&FCC_PACK1_DATA, rx_data).await,
        FDCAN_FccPack2_t::FDCAN_ID => decode_can_data(&FCC_PACK2_DATA, rx_data).await,
        FDCAN_FccPack3_t::FDCAN_ID => decode_can_data(&FCC_PACK3_DATA, rx_data).await,

        FDCAN_FetPack_t::FDCAN_ID => decode_can_data(&FET_DATA, rx_data).await,

        FDCAN_RelPackMtr_t::FDCAN_ID => decode_can_data(&RELAY_MOTOR_PACK, rx_data).await,
        FDCAN_RelPackCap_t::FDCAN_ID => decode_can_data(&REL_CAP_PACK, rx_data).await,
        FDCAN_RelPackFc_t::FDCAN_ID => decode_can_data(&REL_FC_PACK, rx_data).await,

        ECOCAN_H2Pack1_t::FDCAN_ID => decode_can_data(&H2_PACK1_DATA, rx_data).await,
        ECOCAN_H2Pack2_t::FDCAN_ID => decode_can_data(&H2_PACK2_DATA, rx_data).await,

        FDCAN_BOOSTPack1_t::FDCAN_ID => decode_can_data(&BOOST_PACK1_DATA, rx_data).await,
        FDCAN_BOOSTPack2_t::FDCAN_ID => decode_can_data(&BOOST_PACK2_DATA, rx_data).await,
        FDCAN_BOOSTPack3_t::FDCAN_ID => decode_can_data(&BOOST_PACK3_DATA, rx_data).await,

        _ => {
            info!("Non-Relevant ID: {:016b}", id);
            Ok(())
        }
    }
}

/// Decodes a byte array into a CAN package
async fn decode_can_data<T: Decode<()>>(
    package: &Mutex<ThreadModeRawMutex, T>,
    rx_data: &[u8],
) -> Result<(), DecodeError> {
    let bincode_config = bincode::config::standard()
        .with_big_endian()
        .with_fixed_int_encoding();
    // Decode received package bytes into the desired package struct and update can package
    let mut p = package.lock().await;
    *p = bincode::decode_from_slice(&rx_data, bincode_config)?.0;
    Ok(())
}

/// Encodes a CAN package into a byte array
async fn encode_can_package<T: Encode + Clone>(
    package: &Mutex<ThreadModeRawMutex, T>,
    mut tx_data: &mut [u8],
) -> Result<usize, EncodeError> {
    let bincode_config = bincode::config::standard()
        .with_big_endian()
        .with_fixed_int_encoding();
    let p = package.lock().await;
    bincode::encode_into_slice(p.clone(), &mut tx_data, bincode_config)
}
