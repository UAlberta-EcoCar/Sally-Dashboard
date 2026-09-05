use bincode::config::Configuration;

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
pub trait FDCANPack {
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

const BINCODE_CONFIG: Configuration<bincode::config::LittleEndian, bincode::config::Fixint> =
    bincode::config::standard()
        .with_little_endian()
        .with_fixed_int_encoding();

fn main() {
    let pack = FDCAN_RelPackMtr_t {
        mtr_volt: 0x12345678,
        mtr_curr: 0x01020304,
    };
    let mut tx_data = [0; 64];

    let byte_array_length = bincode::encode_into_slice(pack.clone(), &mut tx_data, BINCODE_CONFIG);
    println!("{:?}", pack);
    println!("{:#04x?}", &tx_data[..byte_array_length.unwrap() as usize]);
}
