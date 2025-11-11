//! Module for CAN communication
//!
//! Handles the reception and transmission of CAN messages
//!
//! This requires a lot of global mutable data. See this
//! [article](https://blog.theembeddedrustacean.com/sharing-data-among-tasks-in-rust-embassy-synchronization-primitives#heading-the-list-of-primitives)
//! on shared data in Embassy.

use bincode::{Decode, error::DecodeError};
use defmt::*;
use embassy_stm32::can::frame::FdFrame;
use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, mutex::Mutex};
use embedded_can::Id;

use crate::eco_can::{
    ECOCAN_H2Pack1_t, ECOCAN_H2Pack2_t, ECOCAN_RelPackChrg_t, FDCAN_BOOSTPack1_t,
    FDCAN_BOOSTPack2_t, FDCAN_BOOSTPack3_t, FDCAN_FccPack1_t, FDCAN_FccPack2_t, FDCAN_FccPack3_t,
    FDCAN_FetPack_t, FDCAN_RelPackCap_t, FDCAN_RelPackFc_t, FDCAN_RelPackMtr_t, FDCAN_RelPackNrg_t,
    FDCANPack,
};

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

pub static REL_PACK_CHRG_DATA: Mutex<ThreadModeRawMutex, ECOCAN_RelPackChrg_t> =
    Mutex::new(ECOCAN_RelPackChrg_t {
        fc_coloumbs: 0,
        cap_coloumbs: 0,
    });
pub static REL_PACK_NRG_DATA: Mutex<ThreadModeRawMutex, FDCAN_RelPackNrg_t> =
    Mutex::new(FDCAN_RelPackNrg_t {
        fc_joules: 0,
        cap_joules: 0,
    });
pub static REL_PACK_MTR_DATA: Mutex<ThreadModeRawMutex, FDCAN_RelPackMtr_t> =
    Mutex::new(FDCAN_RelPackMtr_t {
        mtr_volt: 0,
        mtr_curr: 0,
    });
pub static REL_PACK_CAP_DATA: Mutex<ThreadModeRawMutex, FDCAN_RelPackCap_t> =
    Mutex::new(FDCAN_RelPackCap_t {
        cap_volt: 0,
        cap_curr: 0,
    });
pub static REL_PACK_FC_DATA: Mutex<ThreadModeRawMutex, FDCAN_RelPackFc_t> =
    Mutex::new(FDCAN_RelPackFc_t {
        fc_volt: 0,
        fc_curr: 0,
    });

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
        FDCAN_FccPack1_t::FDCAN_ID => decode_frame(&FCC_PACK1_DATA, data).await,
        FDCAN_FccPack2_t::FDCAN_ID => decode_frame(&FCC_PACK2_DATA, data).await,
        FDCAN_FccPack3_t::FDCAN_ID => decode_frame(&FCC_PACK3_DATA, data).await,

        FDCAN_FetPack_t::FDCAN_ID => decode_frame(&FET_DATA, data).await,

        ECOCAN_RelPackChrg_t::FDCAN_ID => decode_frame(&REL_PACK_CHRG_DATA, data).await,
        FDCAN_RelPackNrg_t::FDCAN_ID => decode_frame(&REL_PACK_NRG_DATA, data).await,
        FDCAN_RelPackMtr_t::FDCAN_ID => decode_frame(&REL_PACK_MTR_DATA, data).await,
        FDCAN_RelPackCap_t::FDCAN_ID => decode_frame(&REL_PACK_CAP_DATA, data).await,
        FDCAN_RelPackFc_t::FDCAN_ID => decode_frame(&REL_PACK_FC_DATA, data).await,

        ECOCAN_H2Pack1_t::FDCAN_ID => decode_frame(&H2_PACK1_DATA, data).await,
        ECOCAN_H2Pack2_t::FDCAN_ID => decode_frame(&H2_PACK2_DATA, data).await,

        FDCAN_BOOSTPack1_t::FDCAN_ID => decode_frame(&BOOST_PACK1_DATA, data).await,
        FDCAN_BOOSTPack2_t::FDCAN_ID => decode_frame(&BOOST_PACK2_DATA, data).await,
        FDCAN_BOOSTPack3_t::FDCAN_ID => decode_frame(&BOOST_PACK3_DATA, data).await,

        _ => {
            info!("ID {} doesn't match existing CAN packages", id);
            Err(DecodeError::Other("ID doesn't match existing CAN packages"))
        }
    }
}

async fn decode_frame<T: Decode<()>>(
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
