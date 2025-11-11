//! Contains structs for CAN packages
//! ### CAN Package Information
//! A CAN package is setup like this:
//! ```rust
//! #[allow(non_camel_case_types)]
//! #[derive(bincode::Encode, bincode::Decode, PartialEq, Clone, Debug, Default)]
//! #[repr(C)]
//! pub struct FDCAN_PACKAGE_NAME {
//!     // Package Data
//! }
//! impl FDCANPack for FDCAN_FetPack_t {
//!    const FDCAN_BYTES: FDCANLength = BYTE_LENGTH; // set this to the size of the package in bytes
//!    const FDCAN_ID: u32 = CAN_ID;    // the ID of the CAN package
//! }
//! ```
//! `#[allow(non_camel_case_types)]` allows non-camel-case names for FDCAN packages
//!
//! `#[derive(bincode::Encode, bincode::Decode)]` makes the
//! package able to be encoded to and decoded from bytes.
//!
//! `#[derive(Clone)]` allows the package to be copied.
//!
//! `#[derive(Debug)]` allows the package to be formatted in log messages.
//!
//! `#[derive(Default)]` allows the package to be inialized with default variables (usually 0).
//!
//! `#[repr(C)]` Make Rust use the same memory layout for this struct as C to ensure compatility.
//! For more information: [https://doc.rust-lang.org/nomicon/other-reprs.html](https://doc.rust-lang.org/nomicon/other-reprs.html)

/// Bit Definitions for FET State
#[allow(non_camel_case_types)]
#[repr(u8)]
pub enum FetBit {
    ALL_FET_OFF = 0x00,
    FUELCELL_FET = 0x01,
    CAP_FET = 0x02,
    RES_FET = 0x04,
    OUT_FET = 0x08,
}

/// FET States
#[allow(non_camel_case_types)]
#[repr(u8)]
pub enum FetState {
    FET_STBY = FetBit::ALL_FET_OFF as u8,
    FET_CHRGE = FetBit::FUELCELL_FET as u8 | FetBit::CAP_FET as u8 | FetBit::RES_FET as u8,
    FET_RUN = FetBit::FUELCELL_FET as u8
        | FetBit::CAP_FET as u8
        | FetBit::RES_FET as u8
        | FetBit::OUT_FET as u8,
}

/// Bit Definitions for REL Board State
#[allow(non_camel_case_types)]
#[repr(u8)]
pub enum RelayBit {
    ALL_RELAY_OFF = 0x00,
    CAP_RELAY = 0x01,
    RES_RELAY = 0x02,
    DSCHRGE_RELAY = 0x04,
    MTR_RELAY = 0x08,
}

/// Relay Board State
#[allow(non_camel_case_types)]
#[repr(u8)]
pub enum RelayState {
    RELAY_STBY = RelayBit::ALL_RELAY_OFF as u8,
    RELAY_STRTP = RelayBit::RES_RELAY as u8 | RelayBit::DSCHRGE_RELAY as u8,
    RELAY_CHRGE = RelayBit::RES_RELAY as u8,
    RELAY_RUN =
        RelayBit::CAP_RELAY as u8 | RelayBit::DSCHRGE_RELAY as u8 | RelayBit::MTR_RELAY as u8,
}
/// Relay State ID
pub const FDCAN_RELSTATE_ID: u16 = 0x018;

/// The length of the package in bytes, can be up to 64 bytes.
///
/// pub structs must be a certain size for FDCAN to transfer
/// The following package sizes (in bytes) are 0, 1, 2, 3, 4, 5, 6,
/// 7, 8, 12, 16, 20, 24, 32, 48, 64.
#[allow(non_camel_case_types)]
pub enum FDCANLength {
    BYTES_0 = 0,
    BYTES_1 = 1,
    BYTES_2 = 2,
    BYTES_3 = 3,
    BYTES_4 = 4,
    BYTES_5 = 5,
    BYTES_6 = 6,
    BYTES_7 = 7,
    BYTES_8 = 8,
    BYTES_12 = 12,
    BYTES_16 = 16,
    BYTES_20 = 20,
    BYTES_24 = 24,
    BYTES_32 = 32,
    BYTES_48 = 48,
    BYTES_64 = 64,
}

/// Prerequisite trait for FDCAN Packages
///
/// Sets the ID and number of bytes for a CAN package.
/// Note that associated constants do not increase the size of a struct's memory.
pub trait FDCANPack: bincode::enc::Encode + Clone {
    /// The length of the package in bytes, can be up to 64 bytes.
    ///
    /// pub structs must be a certain size for FDCAN to transfer
    /// The following package sizes (in bytes) are 0, 1, 2, 3, 4, 5, 6,
    /// 7, 8, 12, 16, 20, 24, 32, 48, 64.
    const FDCAN_BYTES: FDCANLength;
    /// 12 bit ID
    ///
    /// Reserved IDs up to 0x01F
    ///
    /// 0x010 = 0b00000010000
    ///
    /// 0x01F = 0b00000011111
    ///
    /// To receive all can filter ids within
    /// this range you must set the mask to
    /// 0x7F0 = 0b11111110000
    ///
    /// because you care that the bits \[10:4\]
    /// of the can id are exactly the same as
    /// bits \[10:4\] in 0x010/0x01F but the last four bits \[3:0\] can be 0 or 1
    /// The same logic will be applied henceforth
    const FDCAN_ID: u32;
}

// Highest priority CAN messages
// ranging from 0x000 to 0x00F
// All boards must accept these
// messages
/// 1 indicates tripped alarm
pub const FDCAN_H2ALARM_ID: u16 = 0x001;
/// 1 indicates led on
pub const FDCAN_SYNCLED_ID: u16 = 0x00F;

#[allow(non_camel_case_types)]
#[derive(bincode::Encode, bincode::Decode, PartialEq, Clone, Debug, Default)]
#[repr(C)]
pub struct FDCAN_FetPack_t {
    pub fet_config: u32,
    pub input_volt: u32,
    pub cap_volt: u32,
    pub cap_curr: u32,
    pub res_curr: u32,
    pub out_curr: u32,
}
impl FDCANPack for FDCAN_FetPack_t {
    const FDCAN_BYTES: FDCANLength = FDCANLength::BYTES_24;
    const FDCAN_ID: u32 = 0x010;
}

#[allow(non_camel_case_types)]
#[derive(bincode::Encode, bincode::Decode, PartialEq, Clone, Debug, Default)]
#[repr(C)]
pub struct ECOCAN_RelPackChrg_t {
    pub fc_coloumbs: i32,
    pub cap_coloumbs: i32,
}
impl FDCANPack for ECOCAN_RelPackChrg_t {
    const FDCAN_BYTES: FDCANLength = FDCANLength::BYTES_8;
    const FDCAN_ID: u32 = 0x013;
}

#[allow(non_camel_case_types)]
#[derive(bincode::Encode, bincode::Decode, PartialEq, Clone, Debug, Default)]
#[repr(C)]
pub struct FDCAN_RelPackNrg_t {
    pub fc_joules: i32,
    pub cap_joules: i32,
}
impl FDCANPack for FDCAN_RelPackNrg_t {
    const FDCAN_BYTES: FDCANLength = FDCANLength::BYTES_8;
    const FDCAN_ID: u32 = 0x014;
}

#[allow(non_camel_case_types)]
#[derive(bincode::Encode, bincode::Decode, PartialEq, Clone, Debug, Default)]
#[repr(C)]
pub struct FDCAN_RelPackMtr_t {
    pub mtr_volt: u32,
    pub mtr_curr: u32,
}
impl FDCANPack for FDCAN_RelPackMtr_t {
    const FDCAN_BYTES: FDCANLength = FDCANLength::BYTES_8;
    const FDCAN_ID: u32 = 0x015;
}

#[allow(non_camel_case_types)]
#[derive(bincode::Encode, bincode::Decode, PartialEq, Clone, Debug, Default)]
#[repr(C)]
pub struct FDCAN_RelPackCap_t {
    pub cap_volt: u32,
    pub cap_curr: i32,
}
impl FDCANPack for FDCAN_RelPackCap_t {
    const FDCAN_BYTES: FDCANLength = FDCANLength::BYTES_8;
    const FDCAN_ID: u32 = 0x016;
}

#[allow(non_camel_case_types)]
#[derive(bincode::Encode, bincode::Decode, PartialEq, Clone, Debug, Default)]
#[repr(C)]
pub struct FDCAN_RelPackFc_t {
    pub fc_volt: u32,
    pub fc_curr: u32,
}
impl FDCANPack for FDCAN_RelPackFc_t {
    const FDCAN_BYTES: FDCANLength = FDCANLength::BYTES_8;
    const FDCAN_ID: u32 = 0x017;
}

#[allow(non_camel_case_types)]
#[derive(bincode::Encode, bincode::Decode, PartialEq, Clone, Debug, Default)]
#[repr(C)]
pub struct FDCAN_FccPack1_t {
    pub fc_temp: i32,
    pub fc_press: u32,
}
impl FDCANPack for FDCAN_FccPack1_t {
    const FDCAN_BYTES: FDCANLength = FDCANLength::BYTES_8;
    const FDCAN_ID: u32 = 0x020;
}

#[allow(non_camel_case_types)]
#[derive(bincode::Encode, bincode::Decode, PartialEq, Clone, Debug, Default)]
#[repr(C)]
pub struct FDCAN_FccPack2_t {
    pub fan_rpm1: u32,
    pub fan_rpm2: u32,
}
impl FDCANPack for FDCAN_FccPack2_t {
    const FDCAN_BYTES: FDCANLength = FDCANLength::BYTES_8;
    const FDCAN_ID: u32 = 0x021;
}

#[allow(non_camel_case_types)]
#[derive(bincode::Encode, bincode::Decode, PartialEq, Clone, Debug, Default)]
#[repr(C)]
pub struct FDCAN_FccPack3_t {
    pub bme_temp: u32,
    pub bme_humid: u32,
}
impl FDCANPack for FDCAN_FccPack3_t {
    const FDCAN_BYTES: FDCANLength = FDCANLength::BYTES_8;
    const FDCAN_ID: u32 = 0x022;
}

// Reserved IDs up to 0x03F
// 0x030 = 0b00001000000
// 0x03F = 0b00001001111
// Mask: 0x7F0

#[allow(non_camel_case_types)]
#[derive(bincode::Encode, bincode::Decode, PartialEq, Clone, Debug, Default)]
#[repr(C)]
pub struct ECOCAN_H2Pack1_t {
    pub h2_sense_1: u16,
    pub h2_sense_2: u16,
    pub h2_sense_3: u16,
    pub h2_sense_4: u16,
}
impl FDCANPack for ECOCAN_H2Pack1_t {
    const FDCAN_BYTES: FDCANLength = FDCANLength::BYTES_8;
    const FDCAN_ID: u32 = 0x030;
}

#[allow(non_camel_case_types)]
#[derive(bincode::Encode, bincode::Decode, PartialEq, Clone, Debug, Default)]
#[repr(C)]
pub struct ECOCAN_H2Pack2_t {
    pub bme_temp: u16,
    pub bme_humid: u16,
    pub imon_7v: u16,
    pub imon_12v: u16,
}
impl FDCANPack for ECOCAN_H2Pack2_t {
    const FDCAN_BYTES: FDCANLength = FDCANLength::BYTES_8;
    const FDCAN_ID: u32 = 0x031;
}

#[allow(non_camel_case_types)]
#[derive(bincode::Encode, bincode::Decode, PartialEq, Clone, Debug, Default)]
#[repr(C)]
pub struct ECOCAN_H2_ARM_ALARM_t {
    pub h2_alarm_armed: u8,
}
impl FDCANPack for ECOCAN_H2_ARM_ALARM_t {
    const FDCAN_BYTES: FDCANLength = FDCANLength::BYTES_1;
    const FDCAN_ID: u32 = 0x032;
}

#[allow(non_camel_case_types)]
#[derive(bincode::Encode, bincode::Decode, PartialEq, Clone, Debug, Default)]
#[repr(C)]
pub struct FDCAN_BOOSTPack1_t {
    pub in_curr: u32,
    pub in_volt: u32,
}
impl FDCANPack for FDCAN_BOOSTPack1_t {
    const FDCAN_BYTES: FDCANLength = FDCANLength::BYTES_8;
    const FDCAN_ID: u32 = 0x040;
}

#[allow(non_camel_case_types)]
#[derive(bincode::Encode, bincode::Decode, PartialEq, Clone, Debug, Default)]
#[repr(C)]
pub struct FDCAN_BOOSTPack2_t {
    pub out_curr: u32,
    pub out_volt: u32,
}
impl FDCANPack for FDCAN_BOOSTPack2_t {
    const FDCAN_BYTES: FDCANLength = FDCANLength::BYTES_8;
    const FDCAN_ID: u32 = 0x041;
}

#[allow(non_camel_case_types)]
#[derive(bincode::Encode, bincode::Decode, PartialEq, Clone, Debug, Default)]
#[repr(C)]
pub struct FDCAN_BOOSTPack3_t {
    pub efficiency: u32,
    pub joules: u32,
}
impl FDCANPack for FDCAN_BOOSTPack3_t {
    const FDCAN_BYTES: FDCANLength = FDCANLength::BYTES_8;
    const FDCAN_ID: u32 = 0x042;
}

#[allow(non_camel_case_types)]
#[derive(bincode::Encode, bincode::Decode, PartialEq, Clone, Debug, Default)]
#[repr(C)]
pub struct FDCAN_BATTPack2_t {
    pub out_curr: u16,
    pub out_volt: u16,
}
impl FDCANPack for FDCAN_BATTPack2_t {
    const FDCAN_BYTES: FDCANLength = FDCANLength::BYTES_4;
    const FDCAN_ID: u32 = 0x050;
}
