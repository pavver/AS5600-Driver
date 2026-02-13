/// Standard I2C address for the AS5600 (fixed by manufacturer).
pub const DEFAULT_ADDR: u8 = 0x36;

/// Register map for the AS5600 according to ams datasheet.
///
/// Registers are mostly 12-bit values spread across two 8-bit registers (HI/LO).
pub mod regs {
    /// Zero setting multi-cycle counter.
    ///
    /// Indicates how many times the `BURN_SETTINGS` command has been executed (max 3).
    pub const ZMCO: u8 = 0x00;

    /// Start position (ZPOS) - HI register.
    /// Defines the 0 degree point.
    pub const ZPOS_HI: u8 = 0x01;
    /// Start position (ZPOS) - LO register.
    pub const ZPOS_LO: u8 = 0x02;

    /// Stop position (MPOS) - HI register.
    /// Defines the end point of the measuring range.
    pub const MPOS_HI: u8 = 0x03;
    /// Stop position (MPOS) - LO register.
    pub const MPOS_LO: u8 = 0x04;

    /// Maximum angle (MANG) - HI register.
    /// Defines the full range angle (if ZPOS/MPOS are not set manually).
    pub const MANG_HI: u8 = 0x05;
    /// Maximum angle (MANG) - LO register.
    pub const MANG_LO: u8 = 0x06;

    /// Configuration register - HI byte.
    pub const CONF_HI: u8 = 0x07;
    /// Configuration register - LO byte.
    pub const CONF_LO: u8 = 0x08;

    /// Status register.
    /// Contains magnet detection flags (MH, ML, MD).
    pub const STATUS: u8 = 0x0B;

    /// Raw Angle - HI register.
    /// The direct 12-bit value from the Hall sensors.
    pub const RAW_ANGLE_HI: u8 = 0x0C;
    /// Raw Angle - LO register.
    pub const RAW_ANGLE_LO: u8 = 0x0D;

    /// Angle - HI register.
    /// The 12-bit value after applying Zero Position, Maximum Position and filters.
    pub const ANGLE_HI: u8 = 0x0E;
    /// Angle - LO register.
    pub const ANGLE_LO: u8 = 0x0F;

    /// Automatic Gain Control.
    /// Returns 0..255 indicating the magnetic field stability.
    pub const AGC: u8 = 0x1A;

    /// Magnitude - HI register.
    /// Internal representation of the magnetic field strength.
    pub const MAGNITUDE_HI: u8 = 0x1B;
    /// Magnitude - LO register.
    pub const MAGNITUDE_LO: u8 = 0x1C;

    /// Programming register.
    /// Used for `BURN_SETTINGS` (0x80) and `BURN_ANGLE` (0x40).
    pub const BURN: u8 = 0xFF;
}
